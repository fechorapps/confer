using System.Net;
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
public class GovernanceIntegrationTests : IClassFixture<WebApplicationFactory<Program>>
{
    private readonly WebApplicationFactory<Program> _factory;
    private readonly HttpClient _client;

    public GovernanceIntegrationTests(WebApplicationFactory<Program> factory)
    {
        _factory = factory;
        _client = factory.CreateClient();
    }

    [Fact]
    public async Task Full_Governance_And_WaitingRoom_Lifecycle_ShouldSucceed()
    {
        // 1. Dev-Login Host User
        var hostLoginReq = new AuthEndpoint.DevLoginRequest("gov_host@confer.local", "Governance Host");
        var hostLoginResp = await _client.PostAsJsonAsync("/api/auth/dev-login", hostLoginReq);
        hostLoginResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var hostAuth = await hostLoginResp.Content.ReadFromJsonAsync<AuthEndpoint.DevLoginResponse>();
        hostAuth.Should().NotBeNull();

        // 2. Dev-Login Participant (Alice)
        var aliceLoginReq = new AuthEndpoint.DevLoginRequest("gov_alice@confer.local", "Alice Waiting");
        var aliceLoginResp = await _client.PostAsJsonAsync("/api/auth/dev-login", aliceLoginReq);
        aliceLoginResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var aliceAuth = await aliceLoginResp.Content.ReadFromJsonAsync<AuthEndpoint.DevLoginResponse>();
        aliceAuth.Should().NotBeNull();

        // 3. Dev-Login Participant (Bob)
        var bobLoginReq = new AuthEndpoint.DevLoginRequest("gov_bob@confer.local", "Bob Waiting");
        var bobLoginResp = await _client.PostAsJsonAsync("/api/auth/dev-login", bobLoginReq);
        bobLoginResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var bobAuth = await bobLoginResp.Content.ReadFromJsonAsync<AuthEndpoint.DevLoginResponse>();
        bobAuth.Should().NotBeNull();

        // 4. Create Meeting with Waiting Room and Watermarking Enabled
        var initialPolicy = new MeetingPolicyDto(
            AllowScreenShare: true,
            AllowChat: true,
            AllowUnmuteSelf: false,
            MuteOnEntry: true,
            AllowRename: true);

        var createReq = new MeetingsEndpoint.CreateMeetingRequest(
            "Executive Governance Board",
            hostAuth!.UserId,
            MaxParticipants: 50,
            CustomJoinCode: null,
            IsWaitingRoomEnabled: true,
            IsWatermarkEnabled: true,
            Policy: initialPolicy);

        var createResp = await _client.PostAsJsonAsync("/api/meetings", createReq);
        createResp.StatusCode.Should().Be(HttpStatusCode.Created);
        var createdMeeting = await createResp.Content.ReadFromJsonAsync<MeetingsEndpoint.CreateMeetingResponse>();
        createdMeeting.Should().NotBeNull();
        createdMeeting!.IsWaitingRoomEnabled.Should().BeTrue();
        createdMeeting.IsWatermarkEnabled.Should().BeTrue();

        // 5. Host Joins Meeting
        var hostJoinReq = new MeetingsEndpoint.JoinMeetingRequest(hostAuth.UserId, hostAuth.DisplayName, "Desktop Linux Host");
        var hostJoinResp = await _client.PostAsJsonAsync($"/api/meetings/{createdMeeting.Id}/join", hostJoinReq);
        hostJoinResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var hostJoinData = await hostJoinResp.Content.ReadFromJsonAsync<MeetingsEndpoint.JoinMeetingResponse>();
        hostJoinData.Should().NotBeNull();
        hostJoinData!.Role.Should().Be("host");
        hostJoinData.Status.Should().Be("admitted");
        hostJoinData.IsWaitingRoom.Should().BeFalse();

        // 6. Alice Joins Meeting -> Should be in Waiting Room
        var aliceJoinReq = new MeetingsEndpoint.JoinMeetingRequest(aliceAuth!.UserId, aliceAuth.DisplayName, "Web Client Alice");
        var aliceJoinResp = await _client.PostAsJsonAsync($"/api/meetings/{createdMeeting.Id}/join", aliceJoinReq);
        aliceJoinResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var aliceJoinData = await aliceJoinResp.Content.ReadFromJsonAsync<MeetingsEndpoint.JoinMeetingResponse>();
        aliceJoinData.Should().NotBeNull();
        aliceJoinData!.Role.Should().Be("participant");
        aliceJoinData.Status.Should().Be("inwaitingroom");
        aliceJoinData.IsWaitingRoom.Should().BeTrue();

        // 7. Bob Joins Meeting -> Should also be in Waiting Room
        var bobJoinReq = new MeetingsEndpoint.JoinMeetingRequest(bobAuth!.UserId, bobAuth.DisplayName, "Mobile Client Bob");
        var bobJoinResp = await _client.PostAsJsonAsync($"/api/meetings/{createdMeeting.Id}/join", bobJoinReq);
        bobJoinResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var bobJoinData = await bobJoinResp.Content.ReadFromJsonAsync<MeetingsEndpoint.JoinMeetingResponse>();
        bobJoinData.Should().NotBeNull();
        bobJoinData!.Status.Should().Be("inwaitingroom");

        // 8. Query Waiting Room (GET /api/meetings/{id}/waiting-room)
        var wrResp = await _client.GetAsync($"/api/meetings/{createdMeeting.Id}/waiting-room");
        wrResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var wrStatus = await wrResp.Content.ReadFromJsonAsync<WaitingRoomStatusDto>();
        wrStatus.Should().NotBeNull();
        wrStatus!.IsWaitingRoomEnabled.Should().BeTrue();
        wrStatus.WaitingParticipants.Should().HaveCount(2);
        wrStatus.WaitingParticipants.Select(p => p.DisplayName).Should().Contain(new[] { "Alice Waiting", "Bob Waiting" });

        // 9. Query Policies (GET /api/meetings/{id}/policies)
        var getPoliciesResp = await _client.GetAsync($"/api/meetings/{createdMeeting.Id}/policies");
        getPoliciesResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var policyData = await getPoliciesResp.Content.ReadFromJsonAsync<MeetingPolicyResponse>();
        policyData.Should().NotBeNull();
        policyData!.IsWaitingRoomEnabled.Should().BeTrue();
        policyData.IsWatermarkEnabled.Should().BeTrue();
        policyData.Policy.AllowUnmuteSelf.Should().BeFalse();
        policyData.Policy.MuteOnEntry.Should().BeTrue();

        // 10. Update Policies (POST /api/meetings/{id}/policies)
        var updatedPolicy = new MeetingPolicyDto(
            AllowScreenShare: false,
            AllowChat: false,
            AllowUnmuteSelf: true,
            MuteOnEntry: false,
            AllowRename: false);

        var updatePolicyReq = new MeetingsEndpoint.UpdateMeetingPolicyRequest(
            hostAuth.UserId,
            updatedPolicy,
            IsWatermarkEnabled: true,
            IsWaitingRoomEnabled: true);

        var updatePolicyResp = await _client.PostAsJsonAsync($"/api/meetings/{createdMeeting.Id}/policies", updatePolicyReq);
        updatePolicyResp.StatusCode.Should().Be(HttpStatusCode.OK);

        // 11. Connect Host and Alice over WebSocket
        var wsClientHost = _factory.Server.CreateWebSocketClient();
        wsClientHost.SubProtocols.Add("confer.v1");
        var hostWsUri = new Uri(_factory.Server.BaseAddress, $"/v1/signal?token={hostJoinData.RoomToken}");
        using var hostWs = await wsClientHost.ConnectAsync(hostWsUri, CancellationToken.None);
        hostWs.State.Should().Be(WebSocketState.Open);
        var hostJoinedJson = await ReceiveMessageOfTypeAsync(hostWs, "joined");
        hostJoinedJson["type"]?.GetValue<string>().Should().Be("joined");

        var wsClientAlice = _factory.Server.CreateWebSocketClient();
        wsClientAlice.SubProtocols.Add("confer.v1");
        var aliceWsUri = new Uri(_factory.Server.BaseAddress, $"/v1/signal?token={aliceJoinData.RoomToken}");
        using var aliceWs = await wsClientAlice.ConnectAsync(aliceWsUri, CancellationToken.None);
        aliceWs.State.Should().Be(WebSocketState.Open);
        var aliceJoinedJson = await ReceiveMessageOfTypeAsync(aliceWs, "joined");
        aliceJoinedJson["type"]?.GetValue<string>().Should().Be("joined");

        // 12. Host Admits Alice (POST /api/meetings/{id}/waiting-room/admit)
        var admitAliceReq = new MeetingsEndpoint.AdmitParticipantRequest(hostAuth.UserId, aliceJoinData.ParticipantId);
        var admitAliceResp = await _client.PostAsJsonAsync($"/api/meetings/{createdMeeting.Id}/waiting-room/admit", admitAliceReq);
        admitAliceResp.StatusCode.Should().Be(HttpStatusCode.NoContent);

        // Alice receives participant_admitted broadcast
        var aliceAdmittedEvent = await ReceiveMessageOfTypeAsync(aliceWs, "participant_admitted");
        aliceAdmittedEvent["participant_id"]?.GetValue<string>().Should().Be(aliceJoinData.ParticipantId.ToString());

        // 13. Host Admits All Remaining (POST /api/meetings/{id}/waiting-room/admit-all)
        var admitAllReq = new MeetingsEndpoint.AdmitAllRequest(hostAuth.UserId);
        var admitAllResp = await _client.PostAsJsonAsync($"/api/meetings/{createdMeeting.Id}/waiting-room/admit-all", admitAllReq);
        admitAllResp.StatusCode.Should().Be(HttpStatusCode.OK);

        // Verify waiting room is now empty
        var wrEmptyResp = await _client.GetAsync($"/api/meetings/{createdMeeting.Id}/waiting-room");
        var wrEmptyStatus = await wrEmptyResp.Content.ReadFromJsonAsync<WaitingRoomStatusDto>();
        wrEmptyStatus!.WaitingParticipants.Should().BeEmpty();

        // 14. Host Toggles Waiting Room Off (POST /api/meetings/{id}/waiting-room/toggle)
        var toggleWrReq = new MeetingsEndpoint.ToggleWaitingRoomRequest(hostAuth.UserId, false);
        var toggleWrResp = await _client.PostAsJsonAsync($"/api/meetings/{createdMeeting.Id}/waiting-room/toggle", toggleWrReq);
        toggleWrResp.StatusCode.Should().Be(HttpStatusCode.OK);

        // 15. Dev-Login and Join Charlie (Should be admitted immediately since waiting room is off)
        var charlieLoginReq = new AuthEndpoint.DevLoginRequest("gov_charlie@confer.local", "Charlie Fast");
        var charlieLoginResp = await _client.PostAsJsonAsync("/api/auth/dev-login", charlieLoginReq);
        var charlieAuth = await charlieLoginResp.Content.ReadFromJsonAsync<AuthEndpoint.DevLoginResponse>();

        var charlieJoinReq = new MeetingsEndpoint.JoinMeetingRequest(charlieAuth!.UserId, charlieAuth.DisplayName, "Web Client");
        var charlieJoinResp = await _client.PostAsJsonAsync($"/api/meetings/{createdMeeting.Id}/join", charlieJoinReq);
        charlieJoinResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var charlieJoinData = await charlieJoinResp.Content.ReadFromJsonAsync<MeetingsEndpoint.JoinMeetingResponse>();
        charlieJoinData!.Status.Should().Be("admitted");
        charlieJoinData.IsWaitingRoom.Should().BeFalse();

        // 16. Test WebSocket message handling for update_policy
        await SendJsonAsync(hostWs, new JsonObject
        {
            ["type"] = "update_policy",
            ["policy"] = new JsonObject
            {
                ["allow_screen_share"] = true,
                ["allow_chat"] = true,
                ["allow_unmute_self"] = true,
                ["mute_on_entry"] = false,
                ["allow_rename"] = true
            },
            ["is_watermark_enabled"] = false,
            ["is_waiting_room_enabled"] = false
        });

        var policyBroadcast = await ReceiveMessageOfTypeAsync(aliceWs, "policy_changed");
        policyBroadcast["type"]?.GetValue<string>().Should().Be("policy_changed");
    }

    private static async Task SendJsonAsync(WebSocket socket, JsonObject json)
    {
        var bytes = Encoding.UTF8.GetBytes(json.ToJsonString());
        await socket.SendAsync(new ArraySegment<byte>(bytes), WebSocketMessageType.Text, true, CancellationToken.None);
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
