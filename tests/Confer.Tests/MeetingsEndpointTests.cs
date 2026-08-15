using System.Net;
using System.Net.Http.Json;
using Confer.Api.Endpoints.Auth;
using Confer.Api.Endpoints.Meetings;
using Confer.Application.Meetings.Create;
using Confer.Application.Meetings.GetByCode;
using Confer.Application.Meetings.Join;
using FluentAssertions;
using Microsoft.AspNetCore.Mvc.Testing;
using Xunit;

namespace Confer.Tests;

[Collection("IntegrationTests")]
public class MeetingsEndpointTests : IClassFixture<WebApplicationFactory<Program>>
{
    private readonly HttpClient _client;

    public MeetingsEndpointTests(WebApplicationFactory<Program> factory)
    {
        _client = factory.CreateClient();
    }

    [Fact]
    public async Task HealthCheck_ShouldReturnOk()
    {
        var response = await _client.GetAsync("/v1/health");
        response.StatusCode.Should().Be(HttpStatusCode.OK);
    }

    [Fact]
    public async Task DevLogin_ShouldReturnToken()
    {
        var request = new AuthEndpoint.DevLoginRequest("test@confer.local", "Test User");
        var response = await _client.PostAsJsonAsync("/api/auth/dev-login", request);

        response.StatusCode.Should().Be(HttpStatusCode.OK);
        var body = await response.Content.ReadFromJsonAsync<AuthEndpoint.DevLoginResponse>();

        body.Should().NotBeNull();
        body!.Email.Should().Be("test@confer.local");
        body.Token.Should().NotBeNullOrWhiteSpace();
    }

    [Fact]
    public async Task CreateAndJoinMeeting_ShouldSucceed()
    {
        var ownerId = Guid.NewGuid();
        var createRequest = new MeetingsEndpoint.CreateMeetingRequest("Sprint Planning", ownerId, 25);
        var createResponse = await _client.PostAsJsonAsync("/api/meetings", createRequest);

        createResponse.StatusCode.Should().Be(HttpStatusCode.Created);
        var createdMeeting = await createResponse.Content.ReadFromJsonAsync<CreateMeetingResponse>();
        createdMeeting.Should().NotBeNull();
        createdMeeting!.Title.Should().Be("Sprint Planning");
        createdMeeting.JoinCode.Should().NotBeNullOrWhiteSpace();

        // Query meeting by join code
        var getResponse = await _client.GetAsync($"/api/meetings/{createdMeeting.JoinCode}");
        getResponse.StatusCode.Should().Be(HttpStatusCode.OK);
        var meetingDto = await getResponse.Content.ReadFromJsonAsync<MeetingResponse>();
        meetingDto.Should().NotBeNull();
        meetingDto!.Id.Should().Be(createdMeeting.Id);

        // Join meeting
        var joinRequest = new MeetingsEndpoint.JoinMeetingRequest(ownerId, "Host User", "Confer Desktop Linux");
        var joinResponse = await _client.PostAsJsonAsync($"/api/meetings/{createdMeeting.Id}/join", joinRequest);

        joinResponse.StatusCode.Should().Be(HttpStatusCode.OK);
        var joinData = await joinResponse.Content.ReadFromJsonAsync<JoinMeetingResponse>();
        joinData.Should().NotBeNull();
        joinData!.Role.Should().Be("host");
        joinData.RoomToken.Should().NotBeNullOrWhiteSpace();
        joinData.WsUrl.Should().Be("/v1/signal");
    }
}
