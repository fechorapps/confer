using System.Net;
using System.Net.Http.Headers;
using System.Net.Http.Json;
using System.Net.WebSockets;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using Confer.Api.Endpoints.Auth;
using Confer.Api.Endpoints.Meetings;
using Confer.Application.DTOs;
using FluentAssertions;
using Microsoft.AspNetCore.Mvc.Testing;
using Xunit;

namespace Confer.Tests;

[Collection("IntegrationTests")]
public class PollsAndBreakoutsIntegrationTests : IClassFixture<WebApplicationFactory<Program>>
{
    private readonly WebApplicationFactory<Program> _factory;
    private readonly HttpClient _client;

    public PollsAndBreakoutsIntegrationTests(WebApplicationFactory<Program> factory)
    {
        _factory = factory;
        _client = factory.CreateClient();
    }

    [Fact]
    public async Task Full_Polls_And_Breakouts_Lifecycle_With_Signaling_ShouldSucceed()
    {
        // 1. Dev-Login Host User
        var hostLoginReq = new AuthEndpoint.DevLoginRequest("host_polls@confer.local", "Host Polls");
        var hostLoginResp = await _client.PostAsJsonAsync("/api/auth/dev-login", hostLoginReq);
        hostLoginResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var hostAuth = await hostLoginResp.Content.ReadFromJsonAsync<AuthEndpoint.DevLoginResponse>();
        hostAuth.Should().NotBeNull();

        // 2. Dev-Login Participant User (Bob)
        var bobLoginReq = new AuthEndpoint.DevLoginRequest("bob_polls@confer.local", "Bob Polls");
        var bobLoginResp = await _client.PostAsJsonAsync("/api/auth/dev-login", bobLoginReq);
        bobLoginResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var bobAuth = await bobLoginResp.Content.ReadFromJsonAsync<AuthEndpoint.DevLoginResponse>();
        bobAuth.Should().NotBeNull();

        // 3. Create Meeting
        var createReq = new MeetingsEndpoint.CreateMeetingRequest("Polls & Breakouts Session", hostAuth!.UserId, 20);
        var createResp = await _client.PostAsJsonAsync("/api/meetings", createReq);
        createResp.StatusCode.Should().Be(HttpStatusCode.Created);
        var createdMeeting = await createResp.Content.ReadFromJsonAsync<MeetingsEndpoint.CreateMeetingResponse>();
        createdMeeting.Should().NotBeNull();

        // 4. Host Joins Meeting
        var hostJoinReq = new MeetingsEndpoint.JoinMeetingRequest(hostAuth.UserId, hostAuth.DisplayName, "Desktop Linux");
        var hostJoinResp = await _client.PostAsJsonAsync($"/api/meetings/{createdMeeting!.Id}/join", hostJoinReq);
        hostJoinResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var hostJoinData = await hostJoinResp.Content.ReadFromJsonAsync<MeetingsEndpoint.JoinMeetingResponse>();
        hostJoinData.Should().NotBeNull();

        // 5. Bob Joins Meeting
        var bobJoinReq = new MeetingsEndpoint.JoinMeetingRequest(bobAuth!.UserId, bobAuth.DisplayName, "Mobile Android");
        var bobJoinResp = await _client.PostAsJsonAsync($"/api/meetings/{createdMeeting.Id}/join", bobJoinReq);
        bobJoinResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var bobJoinData = await bobJoinResp.Content.ReadFromJsonAsync<MeetingsEndpoint.JoinMeetingResponse>();
        bobJoinData.Should().NotBeNull();

        // 6. Connect Host & Bob WebSockets
        var wsClientHost = _factory.Server.CreateWebSocketClient();
        wsClientHost.SubProtocols.Add("confer.v1");
        var hostWsUri = new Uri(_factory.Server.BaseAddress, $"/v1/signal?token={hostJoinData!.RoomToken}");
        using var hostWs = await wsClientHost.ConnectAsync(hostWsUri, CancellationToken.None);
        hostWs.State.Should().Be(WebSocketState.Open);

        var wsClientBob = _factory.Server.CreateWebSocketClient();
        wsClientBob.SubProtocols.Add("confer.v1");
        var bobWsUri = new Uri(_factory.Server.BaseAddress, $"/v1/signal?token={bobJoinData!.RoomToken}");
        using var bobWs = await wsClientBob.ConnectAsync(bobWsUri, CancellationToken.None);
        bobWs.State.Should().Be(WebSocketState.Open);

        // 7. Host Creates a Poll via REST
        var createPollReq = new MeetingsEndpoint.CreatePollRequest(
            hostAuth.UserId,
            "Which feature should we prioritize?",
            new List<string> { "Simulcast VP8", "Live Polls", "Breakout Rooms" },
            IsAnonymous: false,
            IsMultiChoice: false
        );

        var createPollResp = await _client.PostAsJsonAsync($"/api/meetings/{createdMeeting.Id}/polls", createPollReq);
        createPollResp.StatusCode.Should().Be(HttpStatusCode.Created);
        var pollDto = await createPollResp.Content.ReadFromJsonAsync<PollDto>();
        pollDto.Should().NotBeNull();
        pollDto!.Question.Should().Be("Which feature should we prioritize?");
        pollDto.Options.Should().HaveCount(3);

        // Verify WebSocket broadcast of poll_created
        var hostPollCreatedMsg = await ReceiveMessageOfTypeAsync(hostWs, "poll_created");
        hostPollCreatedMsg["poll"]?["id"]?.GetValue<string>().Should().Be(pollDto.Id.ToString());

        var bobPollCreatedMsg = await ReceiveMessageOfTypeAsync(bobWs, "poll_created");
        bobPollCreatedMsg["poll"]?["id"]?.GetValue<string>().Should().Be(pollDto.Id.ToString());

        // 8. Bob Queries Polls via REST
        var getPollsResp = await _client.GetAsync($"/api/meetings/{createdMeeting.Id}/polls");
        getPollsResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var pollsList = await getPollsResp.Content.ReadFromJsonAsync<List<PollDto>>();
        pollsList.Should().NotBeNull();
        pollsList!.Should().HaveCount(1);

        // 9. Bob Submits a Vote via REST
        var chosenOptionId = pollDto.Options[1].Id; // "Live Polls"
        var voteReq = new MeetingsEndpoint.SubmitPollVoteRequest(
            bobJoinData.ParticipantId,
            new List<Guid> { chosenOptionId }
        );

        var voteResp = await _client.PostAsJsonAsync($"/api/meetings/{createdMeeting.Id}/polls/{pollDto.Id}/vote", voteReq);
        voteResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var votedPollDto = await voteResp.Content.ReadFromJsonAsync<PollDto>();
        votedPollDto.Should().NotBeNull();
        votedPollDto!.TotalVotes.Should().Be(1);
        votedPollDto.Options.First(o => o.Id == chosenOptionId).VoteCount.Should().Be(1);

        // Verify WebSocket broadcast of poll_updated
        var hostPollUpdatedMsg = await ReceiveMessageOfTypeAsync(hostWs, "poll_updated");
        hostPollUpdatedMsg["poll"]?["total_votes"]?.GetValue<int>().Should().Be(1);

        var bobPollUpdatedMsg = await ReceiveMessageOfTypeAsync(bobWs, "poll_updated");
        bobPollUpdatedMsg["poll"]?["total_votes"]?.GetValue<int>().Should().Be(1);

        // 10. Host Closes the Poll via REST
        var closePollReq = new MeetingsEndpoint.ClosePollRequest(hostAuth.UserId);
        var closePollResp = await _client.PostAsJsonAsync($"/api/meetings/{createdMeeting.Id}/polls/{pollDto.Id}/close", closePollReq);
        closePollResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var closedPollDto = await closePollResp.Content.ReadFromJsonAsync<PollDto>();
        closedPollDto.Should().NotBeNull();
        closedPollDto!.IsActive.Should().BeFalse();

        // Verify WebSocket broadcast of poll_closed
        var hostPollClosedMsg = await ReceiveMessageOfTypeAsync(hostWs, "poll_closed");
        hostPollClosedMsg["poll_id"]?.GetValue<string>().Should().Be(pollDto.Id.ToString());

        var bobPollClosedMsg = await ReceiveMessageOfTypeAsync(bobWs, "poll_closed");
        bobPollClosedMsg["poll_id"]?.GetValue<string>().Should().Be(pollDto.Id.ToString());

        // 11. Host Creates Breakout Rooms with assignment for Bob (moderation action: requires host's own bearer token)
        _client.DefaultRequestHeaders.Authorization = new AuthenticationHeaderValue("Bearer", hostAuth.Token);
        var createBreakoutsReq = new MeetingsEndpoint.CreateBreakoutsRequest(
            hostAuth.UserId,
            2,
            15,
            new List<string> { "Engineering Breakout", "Design Breakout" },
            new List<BreakoutAssignmentDto>
            {
                new(bobJoinData.ParticipantId, Guid.Empty, "Engineering Breakout")
            }
        );

        var breakoutsResp = await _client.PostAsJsonAsync($"/api/meetings/{createdMeeting.Id}/breakouts", createBreakoutsReq);
        breakoutsResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var breakoutsList = await breakoutsResp.Content.ReadFromJsonAsync<List<BreakoutRoomDto>>();
        breakoutsList.Should().NotBeNull();
        breakoutsList!.Should().HaveCount(2);

        // Verify Bob receives breakout_invite over WebSocket
        var bobBreakoutInviteMsg = await ReceiveMessageOfTypeAsync(bobWs, "breakout_invite");
        bobBreakoutInviteMsg["room_name"]?.GetValue<string>().Should().Be("Engineering Breakout");

        // 12. Host Closes Breakout Rooms via REST
        var closeBreakoutsReq = new MeetingsEndpoint.CloseBreakoutsRequest(hostAuth.UserId);
        var closeBreakoutsResp = await _client.PostAsJsonAsync($"/api/meetings/{createdMeeting.Id}/breakouts/close", closeBreakoutsReq);
        closeBreakoutsResp.StatusCode.Should().Be(HttpStatusCode.NoContent);
    }

    private static async Task<JsonObject> ReceiveMessageOfTypeAsync(WebSocket socket, string expectedType, int timeoutMs = 5000)
    {
        var deadline = DateTime.UtcNow.AddMilliseconds(timeoutMs);
        while (DateTime.UtcNow < deadline)
        {
            var remaining = (int)Math.Max(100, (deadline - DateTime.UtcNow).TotalMilliseconds);
            var msg = await ReceiveJsonAsync(socket, remaining);
            if (msg["type"]?.GetValue<string>() == expectedType)
            {
                return msg;
            }
        }
        throw new TimeoutException($"Did not receive message of type '{expectedType}' within {timeoutMs}ms");
    }

    private static async Task<JsonObject> ReceiveJsonAsync(WebSocket socket, int timeoutMs = 5000)
    {
        using var cts = new CancellationTokenSource(timeoutMs);
        var buffer = new byte[1024 * 64];
        var result = await socket.ReceiveAsync(new ArraySegment<byte>(buffer), cts.Token);
        var text = Encoding.UTF8.GetString(buffer, 0, result.Count);
        var node = JsonNode.Parse(text) as JsonObject;
        node.Should().NotBeNull($"Expected valid JSON object but received: {text}");
        return node!;
    }
}
