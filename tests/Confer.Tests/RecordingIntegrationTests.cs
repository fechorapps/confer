using System.Net;
using System.Net.Http.Json;
using Confer.Api.Endpoints.Meetings;
using Confer.Application.Meetings.Create;
using Confer.Application.Meetings.GetRecordings;
using Confer.Application.Meetings.StartRecording;
using Confer.Application.Meetings.StopRecording;
using FluentAssertions;
using Microsoft.AspNetCore.Mvc.Testing;
using Xunit;

namespace Confer.Tests;

[Collection("IntegrationTests")]
public class RecordingIntegrationTests : IClassFixture<WebApplicationFactory<Program>>
{
    private readonly HttpClient _client;

    public RecordingIntegrationTests(WebApplicationFactory<Program> factory)
    {
        _client = factory.CreateClient();
    }

    [Fact]
    public async Task RecordingLifecycle_StartStopAndQuery_ShouldSucceed()
    {
        // 1. Create meeting
        var ownerId = Guid.NewGuid();
        var createRequest = new MeetingsEndpoint.CreateMeetingRequest("All Hands Call", ownerId, 50);
        var createResponse = await _client.PostAsJsonAsync("/api/meetings", createRequest);
        createResponse.StatusCode.Should().Be(HttpStatusCode.Created);
        var created = await createResponse.Content.ReadFromJsonAsync<CreateMeetingResponse>();
        created.Should().NotBeNull();
        var meetingId = created!.Id;

        // 2. Start recording as owner
        var startRequest = new MeetingsEndpoint.StartRecordingRequest(ownerId);
        var startResponse = await _client.PostAsJsonAsync($"/api/meetings/{meetingId}/recording/start", startRequest);
        startResponse.StatusCode.Should().Be(HttpStatusCode.OK);
        var startData = await startResponse.Content.ReadFromJsonAsync<StartRecordingResponse>();
        startData.Should().NotBeNull();
        startData!.MeetingId.Should().Be(meetingId);
        startData.Status.Should().Be("recording");

        // 3. Attempt to start recording again -> should fail with Conflict (409)
        var duplicateStart = await _client.PostAsJsonAsync($"/api/meetings/{meetingId}/recording/start", startRequest);
        duplicateStart.StatusCode.Should().Be(HttpStatusCode.Conflict);

        // 4. Query recordings list
        var listResponse = await _client.GetAsync($"/api/meetings/{meetingId}/recordings");
        listResponse.StatusCode.Should().Be(HttpStatusCode.OK);
        var list = await listResponse.Content.ReadFromJsonAsync<List<MeetingRecordingDto>>();
        list.Should().NotBeNull();
        list.Should().HaveCount(1);
        list![0].Status.Should().Be("recording");

        // 5. Stop recording as owner
        var stopRequest = new MeetingsEndpoint.StopRecordingRequest(ownerId);
        var stopResponse = await _client.PostAsJsonAsync($"/api/meetings/{meetingId}/recording/stop", stopRequest);
        stopResponse.StatusCode.Should().Be(HttpStatusCode.OK);
        var stopData = await stopResponse.Content.ReadFromJsonAsync<StopRecordingResponse>();
        stopData.Should().NotBeNull();
        stopData!.MeetingId.Should().Be(meetingId);
        stopData.Status.Should().Be("completed");
        stopData.StoragePath.Should().NotBeNullOrWhiteSpace();

        // 6. Attempt to stop recording again -> should fail with Conflict (409)
        var duplicateStop = await _client.PostAsJsonAsync($"/api/meetings/{meetingId}/recording/stop", stopRequest);
        duplicateStop.StatusCode.Should().Be(HttpStatusCode.Conflict);

        // 7. Re-query recordings list -> should be completed
        var finalListResponse = await _client.GetAsync($"/api/meetings/{meetingId}/recordings");
        finalListResponse.StatusCode.Should().Be(HttpStatusCode.OK);
        var finalList = await finalListResponse.Content.ReadFromJsonAsync<List<MeetingRecordingDto>>();
        finalList.Should().NotBeNull();
        finalList.Should().HaveCount(1);
        finalList![0].Status.Should().Be("completed");
    }

    [Fact]
    public async Task StartRecording_ByUnauthorizedUser_ShouldReturnForbiddenOrUnauthorized()
    {
        var ownerId = Guid.NewGuid();
        var nonOwnerId = Guid.NewGuid();

        var createRequest = new MeetingsEndpoint.CreateMeetingRequest("Private Exec Meeting", ownerId, 10);
        var createResponse = await _client.PostAsJsonAsync("/api/meetings", createRequest);
        var created = await createResponse.Content.ReadFromJsonAsync<CreateMeetingResponse>();
        var meetingId = created!.Id;

        var unauthorizedRequest = new MeetingsEndpoint.StartRecordingRequest(nonOwnerId);
        var startResponse = await _client.PostAsJsonAsync($"/api/meetings/{meetingId}/recording/start", unauthorizedRequest);

        startResponse.StatusCode.Should().Be(HttpStatusCode.Unauthorized);
    }
}
