using System.Net;
using System.Net.Http.Headers;
using System.Net.Http.Json;
using Confer.Api.Endpoints.Auth;
using Confer.Api.Endpoints.Meetings;
using Confer.Application.DTOs;
using FluentAssertions;
using Microsoft.AspNetCore.Mvc.Testing;
using Xunit;

namespace Confer.Tests;

[Collection("IntegrationTests")]
public class LiveStreamIntegrationTests : IClassFixture<WebApplicationFactory<Program>>
{
    private readonly HttpClient _client;

    public LiveStreamIntegrationTests(WebApplicationFactory<Program> factory)
    {
        _client = factory.CreateClient();
    }

    private async Task<AuthEndpoint.DevLoginResponse> DevLoginAsync(string email, string displayName)
    {
        var response = await _client.PostAsJsonAsync("/api/auth/dev-login", new AuthEndpoint.DevLoginRequest(email, displayName));
        response.StatusCode.Should().Be(HttpStatusCode.OK);
        var auth = await response.Content.ReadFromJsonAsync<AuthEndpoint.DevLoginResponse>();
        auth.Should().NotBeNull();
        return auth!;
    }

    private void AuthorizeAs(AuthEndpoint.DevLoginResponse actor) =>
        _client.DefaultRequestHeaders.Authorization = new AuthenticationHeaderValue("Bearer", actor.Token);

    [Fact]
    public async Task LiveStreamLifecycle_StartStopAndQuery_ShouldSucceed()
    {
        // 1. Log in the owner and create the meeting
        var owner = await DevLoginAsync("livestream_owner@confer.local", "LiveStream Owner");
        var ownerId = owner.UserId;
        var createRequest = new MeetingsEndpoint.CreateMeetingRequest("Global Developers Conference", ownerId, 100);
        var createResponse = await _client.PostAsJsonAsync("/api/meetings", createRequest);
        createResponse.StatusCode.Should().Be(HttpStatusCode.Created);
        var created = await createResponse.Content.ReadFromJsonAsync<MeetingsEndpoint.CreateMeetingResponse>();
        created.Should().NotBeNull();
        var meetingId = created!.Id;

        // 2. Query initial live stream status -> should be idle
        var initialStatusResp = await _client.GetAsync($"/api/meetings/{meetingId}/stream");
        initialStatusResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var initialStatus = await initialStatusResp.Content.ReadFromJsonAsync<LiveStreamResponse>();
        initialStatus.Should().NotBeNull();
        initialStatus!.IsStreamingEnabled.Should().BeFalse();
        initialStatus.Status.Should().Be("idle");

        // 3. Start live stream as owner
        var rtmpUrl = "rtmp://live.youtube.com/app";
        var streamKey = "live_test_stream_key_999";
        var startRequest = new MeetingsEndpoint.StartLiveStreamRequest(ownerId, rtmpUrl, streamKey);
        AuthorizeAs(owner);
        var startResponse = await _client.PostAsJsonAsync($"/api/meetings/{meetingId}/stream/start", startRequest);
        startResponse.StatusCode.Should().Be(HttpStatusCode.OK);
        var startData = await startResponse.Content.ReadFromJsonAsync<LiveStreamResponse>();
        startData.Should().NotBeNull();
        startData!.MeetingId.Should().Be(meetingId);
        startData.IsStreamingEnabled.Should().BeTrue();
        startData.Status.Should().Be("live");
        startData.RtmpUrl.Should().Be(rtmpUrl);
        startData.StartedAt.Should().NotBeNull();

        // 4. Attempt duplicate start -> should fail with 409 Conflict
        var duplicateStart = await _client.PostAsJsonAsync($"/api/meetings/{meetingId}/stream/start", startRequest);
        duplicateStart.StatusCode.Should().Be(HttpStatusCode.Conflict);

        // 5. Query active live stream status -> should be live
        var liveStatusResp = await _client.GetAsync($"/api/meetings/{meetingId}/stream");
        liveStatusResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var liveStatus = await liveStatusResp.Content.ReadFromJsonAsync<LiveStreamResponse>();
        liveStatus.Should().NotBeNull();
        liveStatus!.IsStreamingEnabled.Should().BeTrue();
        liveStatus.Status.Should().Be("live");
        liveStatus.RtmpUrl.Should().Be(rtmpUrl);

        // 6. Stop live stream as owner
        var stopRequest = new MeetingsEndpoint.StopLiveStreamRequest(ownerId);
        var stopResponse = await _client.PostAsJsonAsync($"/api/meetings/{meetingId}/stream/stop", stopRequest);
        stopResponse.StatusCode.Should().Be(HttpStatusCode.OK);
        var stopData = await stopResponse.Content.ReadFromJsonAsync<LiveStreamResponse>();
        stopData.Should().NotBeNull();
        stopData!.MeetingId.Should().Be(meetingId);
        stopData.IsStreamingEnabled.Should().BeFalse();
        stopData.Status.Should().Be("idle");

        // 7. Attempt duplicate stop -> should fail with 409 Conflict
        var duplicateStop = await _client.PostAsJsonAsync($"/api/meetings/{meetingId}/stream/stop", stopRequest);
        duplicateStop.StatusCode.Should().Be(HttpStatusCode.Conflict);

        // 8. Re-query live stream status -> should be idle
        var finalStatusResp = await _client.GetAsync($"/api/meetings/{meetingId}/stream");
        finalStatusResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var finalStatus = await finalStatusResp.Content.ReadFromJsonAsync<LiveStreamResponse>();
        finalStatus.Should().NotBeNull();
        finalStatus!.IsStreamingEnabled.Should().BeFalse();
        finalStatus.Status.Should().Be("idle");
    }

    [Fact]
    public async Task StartLiveStream_ByUnauthorizedUser_ShouldReturnUnauthorized()
    {
        var ownerId = Guid.NewGuid();
        var nonOwnerId = Guid.NewGuid();

        var createRequest = new MeetingsEndpoint.CreateMeetingRequest("Private Keynote", ownerId, 25);
        var createResponse = await _client.PostAsJsonAsync("/api/meetings", createRequest);
        var created = await createResponse.Content.ReadFromJsonAsync<MeetingsEndpoint.CreateMeetingResponse>();
        var meetingId = created!.Id;

        var unauthorizedRequest = new MeetingsEndpoint.StartLiveStreamRequest(nonOwnerId, "rtmp://live.twitch.tv/app", "key123");
        var startResponse = await _client.PostAsJsonAsync($"/api/meetings/{meetingId}/stream/start", unauthorizedRequest);

        startResponse.StatusCode.Should().Be(HttpStatusCode.Unauthorized);
    }

    [Fact]
    public async Task StopLiveStream_ByUnauthorizedUser_ShouldReturnUnauthorized()
    {
        var ownerId = Guid.NewGuid();
        var nonOwnerId = Guid.NewGuid();

        var createRequest = new MeetingsEndpoint.CreateMeetingRequest("Private Keynote", ownerId, 25);
        var createResponse = await _client.PostAsJsonAsync("/api/meetings", createRequest);
        var created = await createResponse.Content.ReadFromJsonAsync<MeetingsEndpoint.CreateMeetingResponse>();
        var meetingId = created!.Id;

        var startRequest = new MeetingsEndpoint.StartLiveStreamRequest(ownerId, "rtmp://live.twitch.tv/app", "key123");
        await _client.PostAsJsonAsync($"/api/meetings/{meetingId}/stream/start", startRequest);

        var unauthorizedStop = new MeetingsEndpoint.StopLiveStreamRequest(nonOwnerId);
        var stopResponse = await _client.PostAsJsonAsync($"/api/meetings/{meetingId}/stream/stop", unauthorizedStop);

        stopResponse.StatusCode.Should().Be(HttpStatusCode.Unauthorized);
    }

    [Fact]
    public async Task StartLiveStream_ByAuthenticatedUserImpersonatingAnotherActorId_ShouldReturnForbidden()
    {
        // A valid, authenticated caller (attacker) must not be able to moderate a meeting by
        // simply putting someone else's ActorId in the request body.
        var owner = await DevLoginAsync("livestream_impersonated_owner@confer.local", "Real Owner");
        var attacker = await DevLoginAsync("livestream_attacker@confer.local", "Attacker");

        var createRequest = new MeetingsEndpoint.CreateMeetingRequest("Private Keynote", owner.UserId, 25);
        AuthorizeAs(owner);
        var createResponse = await _client.PostAsJsonAsync("/api/meetings", createRequest);
        var created = await createResponse.Content.ReadFromJsonAsync<MeetingsEndpoint.CreateMeetingResponse>();
        var meetingId = created!.Id;

        // Attacker is authenticated as themselves, but claims to be the owner in the body.
        AuthorizeAs(attacker);
        var impersonatingRequest = new MeetingsEndpoint.StartLiveStreamRequest(owner.UserId, "rtmp://live.twitch.tv/app", "key123");
        var startResponse = await _client.PostAsJsonAsync($"/api/meetings/{meetingId}/stream/start", impersonatingRequest);

        startResponse.StatusCode.Should().Be(HttpStatusCode.Forbidden);
    }

    [Fact]
    public async Task GetLiveStreamStatus_WhenMeetingNotFound_ShouldReturnNotFound()
    {
        var missingId = Guid.NewGuid();
        var response = await _client.GetAsync($"/api/meetings/{missingId}/stream");
        response.StatusCode.Should().Be(HttpStatusCode.NotFound);
    }
}
