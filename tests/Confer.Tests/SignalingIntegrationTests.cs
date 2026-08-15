using System.Net;
using System.Net.Http.Json;
using System.Net.WebSockets;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using Confer.Api.Endpoints.Auth;
using Confer.Api.Endpoints.Meetings;
using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Domain.Enums;
using Confer.Infrastructure.Signal;
using FluentAssertions;
using Microsoft.AspNetCore.Mvc.Testing;
using Microsoft.Extensions.DependencyInjection;
using Xunit;

namespace Confer.Tests;

[Collection("IntegrationTests")]
public class SignalingIntegrationTests : IClassFixture<WebApplicationFactory<Program>>
{
    private readonly WebApplicationFactory<Program> _factory;
    private readonly HttpClient _httpClient;

    public SignalingIntegrationTests(WebApplicationFactory<Program> factory)
    {
        _factory = factory;
        _httpClient = factory.CreateClient();
    }

    [Fact]
    public async Task EndToEnd_Signaling_Handshake_And_PubSub_Flow_ShouldSucceed()
    {
        // 1. Dev-Login Host User
        var hostLoginReq = new AuthEndpoint.DevLoginRequest("host@confer.local", "Host User (Dev)");
        var hostLoginResp = await _httpClient.PostAsJsonAsync("/api/auth/dev-login", hostLoginReq);
        hostLoginResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var hostAuth = await hostLoginResp.Content.ReadFromJsonAsync<AuthEndpoint.DevLoginResponse>();
        hostAuth.Should().NotBeNull();
        hostAuth!.UserId.Should().NotBeEmpty();

        // 2. Dev-Login Participant User (Alice)
        var aliceLoginReq = new AuthEndpoint.DevLoginRequest("alice@confer.local", "Alice (Dev)");
        var aliceLoginResp = await _httpClient.PostAsJsonAsync("/api/auth/dev-login", aliceLoginReq);
        aliceLoginResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var aliceAuth = await aliceLoginResp.Content.ReadFromJsonAsync<AuthEndpoint.DevLoginResponse>();
        aliceAuth.Should().NotBeNull();
        aliceAuth!.UserId.Should().NotBeEmpty();

        // 3. Create Meeting by Host
        var createMeetingReq = new MeetingsEndpoint.CreateMeetingRequest("All Hands Architecture Review", hostAuth.UserId, 50);
        var createResp = await _httpClient.PostAsJsonAsync("/api/meetings", createMeetingReq);
        createResp.StatusCode.Should().Be(HttpStatusCode.Created);
        var createdMeeting = await createResp.Content.ReadFromJsonAsync<MeetingsEndpoint.CreateMeetingResponse>();
        createdMeeting.Should().NotBeNull();
        createdMeeting!.Id.Should().NotBeEmpty();

        // 4. Host Joins Meeting to get Room Token
        var hostJoinReq = new MeetingsEndpoint.JoinMeetingRequest(hostAuth.UserId, hostAuth.DisplayName, "Confer Desktop Linux (Rust)");
        var hostJoinResp = await _httpClient.PostAsJsonAsync($"/api/meetings/{createdMeeting.Id}/join", hostJoinReq);
        hostJoinResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var hostJoinData = await hostJoinResp.Content.ReadFromJsonAsync<MeetingsEndpoint.JoinMeetingResponse>();
        hostJoinData.Should().NotBeNull();
        hostJoinData!.Role.Should().Be("host");
        hostJoinData.RoomToken.Should().NotBeNullOrWhiteSpace();

        // 5. Alice Joins Meeting to get Room Token
        var aliceJoinReq = new MeetingsEndpoint.JoinMeetingRequest(aliceAuth.UserId, aliceAuth.DisplayName, "Confer Desktop Linux (Rust)");
        var aliceJoinResp = await _httpClient.PostAsJsonAsync($"/api/meetings/{createdMeeting.Id}/join", aliceJoinReq);
        aliceJoinResp.StatusCode.Should().Be(HttpStatusCode.OK);
        var aliceJoinData = await aliceJoinResp.Content.ReadFromJsonAsync<MeetingsEndpoint.JoinMeetingResponse>();
        aliceJoinData.Should().NotBeNull();
        aliceJoinData!.Role.Should().Be("participant");
        aliceJoinData.RoomToken.Should().NotBeNullOrWhiteSpace();

        // 6. Connect Host WebSocket with confer.v1 Subprotocol
        var wsClientHost = _factory.Server.CreateWebSocketClient();
        wsClientHost.SubProtocols.Add("confer.v1");
        var hostWsUri = new Uri(_factory.Server.BaseAddress, $"/v1/signal?token={hostJoinData.RoomToken}");
        using var hostWs = await wsClientHost.ConnectAsync(hostWsUri, CancellationToken.None);
        hostWs.State.Should().Be(WebSocketState.Open);

        // Host receives initial "joined" message
        var hostJoinedJson = await ReceiveJsonAsync(hostWs);
        hostJoinedJson["type"]?.GetValue<string>().Should().Be("joined");
        hostJoinedJson["participant_id"]?.GetValue<string>().Should().Be(hostJoinData.ParticipantId.ToString());
        hostJoinedJson["meeting_id"]?.GetValue<string>().Should().Be(createdMeeting.Id.ToString());
        hostJoinedJson["role"]?.GetValue<string>().Should().Be("host");

        // 7. Connect Alice WebSocket with confer.v1 Subprotocol
        var wsClientAlice = _factory.Server.CreateWebSocketClient();
        wsClientAlice.SubProtocols.Add("confer.v1");
        var aliceWsUri = new Uri(_factory.Server.BaseAddress, $"/v1/signal?token={aliceJoinData.RoomToken}");
        using var aliceWs = await wsClientAlice.ConnectAsync(aliceWsUri, CancellationToken.None);
        aliceWs.State.Should().Be(WebSocketState.Open);

        // Alice receives "joined" message with current roster containing Host and Alice
        var aliceJoinedJson = await ReceiveJsonAsync(aliceWs);
        aliceJoinedJson["type"]?.GetValue<string>().Should().Be("joined");
        aliceJoinedJson["participant_id"]?.GetValue<string>().Should().Be(aliceJoinData.ParticipantId.ToString());
        aliceJoinedJson["role"]?.GetValue<string>().Should().Be("participant");

        // Host receives "participant_joined" broadcast for Alice
        var hostReceivedAliceJoined = await ReceiveJsonAsync(hostWs);
        hostReceivedAliceJoined["type"]?.GetValue<string>().Should().Be("participant_joined");
        var joinedPart = hostReceivedAliceJoined["participant"];
        joinedPart.Should().NotBeNull();
        joinedPart!["participant_id"]?.GetValue<string>().Should().Be(aliceJoinData.ParticipantId.ToString());
        joinedPart["display_name"]?.GetValue<string>().Should().Be("Alice (Dev)");

        // 8. Test Ping / Pong Handshake
        var pingSeq = 1337L;
        await SendJsonAsync(hostWs, new JsonObject
        {
            ["type"] = "ping",
            ["seq"] = pingSeq
        });
        var pongJson = await ReceiveJsonAsync(hostWs);
        pongJson["type"]?.GetValue<string>().Should().Be("pong");
        pongJson["seq"]?.GetValue<long>().Should().Be(pingSeq);

        // 9. Test set_mute Pub/Sub Broadcast
        await SendJsonAsync(aliceWs, new JsonObject
        {
            ["type"] = "set_mute",
            ["kind"] = "audio",
            ["muted"] = true
        });

        // Both Host and Alice receive participant_mute_changed broadcast
        var hostMuteBroadcast = await ReceiveMessageOfTypeAsync(hostWs, "participant_mute_changed");
        hostMuteBroadcast["participant_id"]?.GetValue<string>().Should().Be(aliceJoinData.ParticipantId.ToString());
        hostMuteBroadcast["kind"]?.GetValue<string>().Should().Be("audio");
        hostMuteBroadcast["muted"]?.GetValue<bool>().Should().BeTrue();

        var aliceMuteBroadcast = await ReceiveMessageOfTypeAsync(aliceWs, "participant_mute_changed");
        aliceMuteBroadcast["participant_id"]?.GetValue<string>().Should().Be(aliceJoinData.ParticipantId.ToString());
        aliceMuteBroadcast["kind"]?.GetValue<string>().Should().Be("audio");
        aliceMuteBroadcast["muted"]?.GetValue<bool>().Should().BeTrue();

        // 10. Test Chat Pub/Sub Broadcast
        await SendJsonAsync(aliceWs, new JsonObject
        {
            ["type"] = "chat",
            ["body"] = "Hello from Alice! Testing real-time messaging."
        });

        var hostChatBroadcast = await ReceiveMessageOfTypeAsync(hostWs, "chat");
        hostChatBroadcast["from_id"]?.GetValue<string>().Should().Be(aliceJoinData.ParticipantId.ToString());
        hostChatBroadcast["from_name"]?.GetValue<string>().Should().Be("Alice (Dev)");
        hostChatBroadcast["body"]?.GetValue<string>().Should().Be("Hello from Alice! Testing real-time messaging.");
        hostChatBroadcast["id"].Should().NotBeNull();
        hostChatBroadcast["sent_at"].Should().NotBeNull();

        var aliceChatBroadcast = await ReceiveMessageOfTypeAsync(aliceWs, "chat");
        aliceChatBroadcast["from_name"]?.GetValue<string>().Should().Be("Alice (Dev)");
        aliceChatBroadcast["body"]?.GetValue<string>().Should().Be("Hello from Alice! Testing real-time messaging.");

        // 11. Test Emoji Reaction Pub/Sub Broadcast
        await SendJsonAsync(hostWs, new JsonObject
        {
            ["type"] = "reaction",
            ["emoji"] = "🔥"
        });

        var hostReactionBroadcast = await ReceiveMessageOfTypeAsync(hostWs, "reaction");
        hostReactionBroadcast["emoji"]?.GetValue<string>().Should().Be("🔥");
        hostReactionBroadcast["from_name"]?.GetValue<string>().Should().Be("Host User (Dev)");

        var aliceReactionBroadcast = await ReceiveMessageOfTypeAsync(aliceWs, "reaction");
        aliceReactionBroadcast["emoji"]?.GetValue<string>().Should().Be("🔥");
        aliceReactionBroadcast["from_name"]?.GetValue<string>().Should().Be("Host User (Dev)");

        // 12. Test Active Speaker Detection & Broadcast via Signaling Notifier
        var signalingNotifier = _factory.Services.GetRequiredService<ISignalingNotifier>();
        var rankedSpeakers = new List<SpeakerInfoDto>
        {
            new(aliceJoinData.ParticipantId, -12),
            new(hostJoinData.ParticipantId, -30)
        };
        await signalingNotifier.BroadcastActiveSpeakersAsync(createdMeeting.Id, rankedSpeakers);

        var hostSpeakerMsg = await ReceiveMessageOfTypeAsync(hostWs, "active_speakers");
        var hostRanked = hostSpeakerMsg["ranked"]?.AsArray();
        hostRanked.Should().NotBeNull();
        hostRanked!.Count.Should().Be(2);

        var aliceSpeakerMsg = await ReceiveMessageOfTypeAsync(aliceWs, "active_speakers");
        var aliceRanked = aliceSpeakerMsg["ranked"]?.AsArray();
        aliceRanked.Should().NotBeNull();
        aliceRanked!.Count.Should().Be(2);

        // 13. Test Alice Disconnect -> Host Receives participant_left
        await aliceWs.CloseAsync(WebSocketCloseStatus.NormalClosure, "Leaving meeting", CancellationToken.None);

        var hostLeftMsg = await ReceiveMessageOfTypeAsync(hostWs, "participant_left");
        hostLeftMsg["participant_id"]?.GetValue<string>().Should().Be(aliceJoinData.ParticipantId.ToString());
    }

    [Fact]
    public async Task Signaling_InvalidToken_ShouldCloseWithPolicyViolation()
    {
        var wsClient = _factory.Server.CreateWebSocketClient();
        wsClient.SubProtocols.Add("confer.v1");
        var wsUri = new Uri(_factory.Server.BaseAddress, "/v1/signal?token=invalid.jwt.token");
        
        using var ws = await wsClient.ConnectAsync(wsUri, CancellationToken.None);
        var buffer = new byte[1024];
        var result = await ws.ReceiveAsync(new ArraySegment<byte>(buffer), CancellationToken.None);
        result.MessageType.Should().Be(WebSocketMessageType.Close);
        result.CloseStatus.Should().Be(WebSocketCloseStatus.PolicyViolation);
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
