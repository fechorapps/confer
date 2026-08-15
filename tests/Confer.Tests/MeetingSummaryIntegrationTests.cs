using System.Net;
using System.Net.Http.Json;
using Confer.Api.Endpoints.Meetings;
using Confer.Application.Meetings.Create;
using Confer.Application.Meetings.Summary;
using Confer.Domain.Meetings;
using Confer.Domain.Sessions;
using Confer.Infrastructure.Persistence;
using FluentAssertions;
using Microsoft.AspNetCore.Mvc.Testing;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.DependencyInjection;
using Xunit;

namespace Confer.Tests;

[Collection("IntegrationTests")]
public class MeetingSummaryIntegrationTests : IClassFixture<WebApplicationFactory<Program>>
{
    private readonly HttpClient _client;
    private readonly WebApplicationFactory<Program> _factory;

    public MeetingSummaryIntegrationTests(WebApplicationFactory<Program> factory)
    {
        _factory = factory;
        _client = factory.CreateClient();
    }

    [Fact]
    public async Task MeetingSummaryLifecycle_GenerateAndRetrieve_ShouldSucceed()
    {
        // 1. Create meeting
        var ownerId = Guid.NewGuid();
        var createRequest = new MeetingsEndpoint.CreateMeetingRequest("Q3 Retrospective", ownerId, 25);
        var createResponse = await _client.PostAsJsonAsync("/api/meetings", createRequest);
        createResponse.StatusCode.Should().Be(HttpStatusCode.Created);
        var created = await createResponse.Content.ReadFromJsonAsync<CreateMeetingResponse>();
        created.Should().NotBeNull();
        var meetingId = created!.Id;

        // 2. Attempt to generate summary before meeting ends -> Should return 409 Conflict
        var prematureRequest = new MeetingsEndpoint.GenerateMeetingSummaryRequest(ownerId);
        var prematureResponse = await _client.PostAsJsonAsync($"/api/meetings/{meetingId}/summary", prematureRequest);
        prematureResponse.StatusCode.Should().Be(HttpStatusCode.Conflict);

        // 3. End meeting and seed chat messages in database
        using (var scope = _factory.Services.CreateScope())
        {
            var db = scope.ServiceProvider.GetRequiredService<ConferDbContext>();
            var meeting = await db.Meetings.Include(m => m.Sessions).FirstOrDefaultAsync(m => m.Id == meetingId);
            meeting.Should().NotBeNull();

            var session = Session.Create(meetingId, "sfu-1");
            var participant = session.AddParticipant(ownerId, "Lead Host", Domain.Enums.ParticipantRole.Host).Value;
            var chat1 = ChatMessage.Create(session.Id, ownerId, "Lead Host", "Decision: Adopt trunk-based development workflow.");
            var chat2 = ChatMessage.Create(session.Id, ownerId, "Lead Host", "TODO: Setup release tags in CI/CD pipeline");

            db.Sessions.Add(session);
            db.Participations.Add(participant);
            db.ChatMessages.AddRange(chat1, chat2);
            meeting!.End(ownerId);

            await db.SaveChangesAsync();
        }

        // 4. Generate summary
        var generateRequest = new MeetingsEndpoint.GenerateMeetingSummaryRequest(ownerId);
        var generateResponse = await _client.PostAsJsonAsync($"/api/meetings/{meetingId}/summary", generateRequest);
        generateResponse.StatusCode.Should().Be(HttpStatusCode.OK);

        var summary = await generateResponse.Content.ReadFromJsonAsync<MeetingSummaryDto>();
        summary.Should().NotBeNull();
        summary!.MeetingId.Should().Be(meetingId);
        summary.Overview.Should().Contain("Q3 Retrospective");
        summary.KeyDecisions.Should().Contain(d => d.Contains("trunk-based", StringComparison.OrdinalIgnoreCase));
        summary.ActionItems.Should().Contain(a => a.Title.Contains("release tags", StringComparison.OrdinalIgnoreCase));

        // 5. Retrieve summary via GET endpoint
        var getResponse = await _client.GetAsync($"/api/meetings/{meetingId}/summary");
        getResponse.StatusCode.Should().Be(HttpStatusCode.OK);

        var retrievedSummary = await getResponse.Content.ReadFromJsonAsync<MeetingSummaryDto>();
        retrievedSummary.Should().NotBeNull();
        retrievedSummary!.Id.Should().Be(summary.Id);
        retrievedSummary.MeetingId.Should().Be(meetingId);
        retrievedSummary.Overview.Should().Be(summary.Overview);
        retrievedSummary.KeyDecisions.Should().BeEquivalentTo(summary.KeyDecisions);
        retrievedSummary.ActionItems.Should().BeEquivalentTo(summary.ActionItems);
    }

    [Fact]
    public async Task GenerateSummary_UnauthorizedUser_ShouldReturnUnauthorized()
    {
        var ownerId = Guid.NewGuid();
        var unauthorizedUser = Guid.NewGuid();

        var createRequest = new MeetingsEndpoint.CreateMeetingRequest("Private Standup", ownerId, 10);
        var createResponse = await _client.PostAsJsonAsync("/api/meetings", createRequest);
        var created = await createResponse.Content.ReadFromJsonAsync<CreateMeetingResponse>();
        var meetingId = created!.Id;

        // End meeting
        using (var scope = _factory.Services.CreateScope())
        {
            var db = scope.ServiceProvider.GetRequiredService<ConferDbContext>();
            var meeting = await db.Meetings.FindAsync(meetingId);
            meeting!.End(ownerId);
            await db.SaveChangesAsync();
        }

        var request = new MeetingsEndpoint.GenerateMeetingSummaryRequest(unauthorizedUser);
        var response = await _client.PostAsJsonAsync($"/api/meetings/{meetingId}/summary", request);
        response.StatusCode.Should().Be(HttpStatusCode.Unauthorized);
    }

    [Fact]
    public async Task GetSummary_WhenMeetingNotFound_ShouldReturnNotFound()
    {
        var nonExistentMeetingId = Guid.NewGuid();
        var response = await _client.GetAsync($"/api/meetings/{nonExistentMeetingId}/summary");
        response.StatusCode.Should().Be(HttpStatusCode.NotFound);
    }
}
