using System.Collections.Concurrent;
using System.Net.WebSockets;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Domain.Enums;
using Confer.Shared.Application.Interfaces;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

namespace Confer.Infrastructure.Signal;

public sealed class WebSocketSignalingHandler : ISignalingNotifier
{
    private readonly ILogger<WebSocketSignalingHandler> _logger;
    private readonly IServiceProvider _serviceProvider;
    private readonly ConcurrentDictionary<Guid, ConcurrentDictionary<Guid, WebSocket>> _roomSockets = new();
    private readonly ConcurrentDictionary<Guid, RoomTokenClaims> _socketClaims = new();

    public WebSocketSignalingHandler(
        ILogger<WebSocketSignalingHandler> logger,
        IServiceProvider serviceProvider)
    {
        _logger = logger;
        _serviceProvider = serviceProvider;
    }

    public async Task HandleWebSocketAsync(HttpContext context, WebSocket webSocket, string? token)
    {
        using var scope = _serviceProvider.CreateScope();
        var tokenProvider = scope.ServiceProvider.GetRequiredService<ITokenProvider>();
        var presenceService = scope.ServiceProvider.GetRequiredService<IPresenceService>();
        var sfuManager = scope.ServiceProvider.GetRequiredService<ISfuRoomManager>();

        if (string.IsNullOrWhiteSpace(token))
        {
            await webSocket.CloseAsync(WebSocketCloseStatus.PolicyViolation, "Token missing", CancellationToken.None);
            return;
        }

        var validation = tokenProvider.ValidateRoomToken(token);
        if (validation.IsFailure)
        {
            await webSocket.CloseAsync(WebSocketCloseStatus.PolicyViolation, "Invalid token", CancellationToken.None);
            return;
        }

        var claims = validation.Value;
        var meetingId = claims.MeetingId;
        var participantId = claims.ParticipantId;

        var sockets = _roomSockets.GetOrAdd(meetingId, _ => new ConcurrentDictionary<Guid, WebSocket>());
        sockets[participantId] = webSocket;
        _socketClaims[participantId] = claims;

        var participantState = new ParticipantStateDto(
            participantId,
            claims.UserId,
            claims.DisplayName,
            claims.Role.ToString().ToLowerInvariant()
        );

        await presenceService.SetParticipantOnlineAsync(meetingId, participantState);

        // Send Joined confirmation with current roster
        var roster = await presenceService.GetRosterAsync(meetingId);
        var joinedMsg = new JsonObject
        {
            ["type"] = "joined",
            ["participant_id"] = participantId.ToString(),
            ["meeting_id"] = meetingId.ToString(),
            ["room_title"] = "Confer Meeting",
            ["role"] = claims.Role.ToString().ToLowerInvariant(),
            ["roster"] = JsonSerializer.SerializeToNode(roster)
        };
        await SendJsonAsync(webSocket, joinedMsg);

        // Broadcast to others in the room
        await BroadcastJoinedAsync(meetingId, participantState);

        _logger.LogInformation("Participant {ParticipantId} ({DisplayName}) joined room {MeetingId} over WebSocket",
            participantId, claims.DisplayName, meetingId);

        var buffer = new byte[1024 * 64];
        try
        {
            while (webSocket.State == WebSocketState.Open)
            {
                var result = await webSocket.ReceiveAsync(new ArraySegment<byte>(buffer), CancellationToken.None);
                if (result.MessageType == WebSocketMessageType.Close)
                {
                    break;
                }

                if (result.MessageType == WebSocketMessageType.Text)
                {
                    var text = Encoding.UTF8.GetString(buffer, 0, result.Count);
                    await ProcessClientMessageAsync(claims, text, webSocket, sfuManager, presenceService);
                }
            }
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "WebSocket error for participant {ParticipantId}", participantId);
        }
        finally
        {
            sockets.TryRemove(participantId, out _);
            _socketClaims.TryRemove(participantId, out _);
            await presenceService.SetParticipantOfflineAsync(meetingId, participantId);
            await sfuManager.RemoveParticipantAsync(meetingId, participantId);
            await BroadcastLeftAsync(meetingId, participantId, "disconnected");

            if (webSocket.State == WebSocketState.Open || webSocket.State == WebSocketState.CloseReceived)
            {
                try
                {
                    await webSocket.CloseAsync(WebSocketCloseStatus.NormalClosure, "Closed", CancellationToken.None);
                }
                catch { }
            }
        }
    }

    private async Task ProcessClientMessageAsync(
        RoomTokenClaims claims,
        string jsonText,
        WebSocket webSocket,
        ISfuRoomManager sfuManager,
        IPresenceService presenceService)
    {
        try
        {
            var node = JsonNode.Parse(jsonText);
            if (node == null) return;

            var type = node["type"]?.GetValue<string>()?.ToLowerInvariant();

            switch (type)
            {
                case "ping":
                    var seq = node["seq"]?.GetValue<long>() ?? 0;
                    await SendJsonAsync(webSocket, new JsonObject { ["type"] = "pong", ["seq"] = seq });
                    break;

                case "publish":
                    var sdpOffer = node["sdp"]?.GetValue<string>() ?? string.Empty;
                    var tracks = JsonSerializer.Deserialize<List<TrackIntent>>(node["tracks"]?.ToJsonString() ?? "[]") ?? new();
                    var pubResult = await sfuManager.HandlePublishOfferAsync(claims.MeetingId, claims.ParticipantId, sdpOffer, tracks);
                    if (pubResult.IsSuccess)
                    {
                        await SendJsonAsync(webSocket, new JsonObject { ["type"] = "publish_ok", ["sdp"] = pubResult.Value });
                    }
                    break;

                case "subscribe_answer":
                    var sdpAnswer = node["sdp"]?.GetValue<string>() ?? string.Empty;
                    await sfuManager.HandleSubscribeAnswerAsync(claims.MeetingId, claims.ParticipantId, sdpAnswer);
                    break;

                case "update_viewport":
                    var tiles = JsonSerializer.Deserialize<List<TileSpec>>(node["tiles"]?.ToJsonString() ?? "[]") ?? new();
                    await sfuManager.UpdateViewportAsync(claims.MeetingId, claims.ParticipantId, tiles);
                    break;

                case "set_mute":
                    var kindStr = node["kind"]?.GetValue<string>()?.ToLowerInvariant();
                    var isMuted = node["muted"]?.GetValue<bool>() ?? false;
                    var kind = kindStr == "video" ? MediaKind.Video : MediaKind.Audio;
                    await sfuManager.SetMuteAsync(claims.MeetingId, claims.ParticipantId, kind, isMuted);
                    await BroadcastMuteChangedAsync(claims.MeetingId, claims.ParticipantId, kindStr ?? "audio", isMuted);
                    break;

                case "chat":
                    var body = node["body"]?.GetValue<string>() ?? string.Empty;
                    if (!string.IsNullOrWhiteSpace(body))
                    {
                        await BroadcastChatAsync(claims.MeetingId, Guid.NewGuid(), claims.ParticipantId, claims.DisplayName, body, DateTime.UtcNow);
                    }
                    break;

                case "reaction":
                    var emoji = node["emoji"]?.GetValue<string>() ?? "👍";
                    await BroadcastReactionAsync(claims.MeetingId, claims.ParticipantId, claims.DisplayName, emoji);
                    break;

                case "host_action":
                    if (claims.Role == ParticipantRole.Host || claims.Role == ParticipantRole.CoHost)
                    {
                        var action = node["action"]?.GetValue<string>()?.ToLowerInvariant();
                        var targetIdStr = node["target_participant_id"]?.GetValue<string>();
                        if (Guid.TryParse(targetIdStr, out var targetId))
                        {
                            if (action == "mute")
                            {
                                await sfuManager.SetMuteAsync(claims.MeetingId, targetId, MediaKind.Audio, true);
                                await BroadcastMuteChangedAsync(claims.MeetingId, targetId, "audio", true);
                            }
                            else if (action == "kick")
                            {
                                if (_roomSockets.TryGetValue(claims.MeetingId, out var sockets) && sockets.TryGetValue(targetId, out var targetSocket))
                                {
                                    await targetSocket.CloseAsync(WebSocketCloseStatus.NormalClosure, "Kicked by host", CancellationToken.None);
                                }
                            }
                        }
                    }
                    break;
            }
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "Error processing client message");
        }
    }

    public Task BroadcastJoinedAsync(Guid meetingId, ParticipantStateDto participant) =>
        BroadcastToRoomAsync(meetingId, new JsonObject
        {
            ["type"] = "participant_joined",
            ["participant"] = JsonSerializer.SerializeToNode(participant)
        }, excludeParticipantId: participant.ParticipantId);

    public Task BroadcastLeftAsync(Guid meetingId, Guid participantId, string reason) =>
        BroadcastToRoomAsync(meetingId, new JsonObject
        {
            ["type"] = "participant_left",
            ["participant_id"] = participantId.ToString(),
            ["reason"] = reason
        });

    public Task BroadcastMuteChangedAsync(Guid meetingId, Guid participantId, string kind, bool isMuted) =>
        BroadcastToRoomAsync(meetingId, new JsonObject
        {
            ["type"] = "participant_mute_changed",
            ["participant_id"] = participantId.ToString(),
            ["kind"] = kind,
            ["muted"] = isMuted
        });

    public Task BroadcastActiveSpeakersAsync(Guid meetingId, List<SpeakerInfoDto> speakers) =>
        BroadcastToRoomAsync(meetingId, new JsonObject
        {
            ["type"] = "active_speakers",
            ["ranked"] = JsonSerializer.SerializeToNode(speakers)
        });

    public Task BroadcastChatAsync(Guid meetingId, Guid messageId, Guid fromId, string fromName, string body, DateTime sentAt) =>
        BroadcastToRoomAsync(meetingId, new JsonObject
        {
            ["type"] = "chat",
            ["id"] = messageId.ToString(),
            ["from_id"] = fromId.ToString(),
            ["from_name"] = fromName,
            ["body"] = body,
            ["sent_at"] = sentAt.ToString("o")
        });

    public Task BroadcastReactionAsync(Guid meetingId, Guid fromId, string fromName, string emoji) =>
        BroadcastToRoomAsync(meetingId, new JsonObject
        {
            ["type"] = "reaction",
            ["from_id"] = fromId.ToString(),
            ["from_name"] = fromName,
            ["emoji"] = emoji
        });

    public Task BroadcastMeetingLockedAsync(Guid meetingId, bool isLocked) =>
        BroadcastToRoomAsync(meetingId, new JsonObject
        {
            ["type"] = "meeting_locked",
            ["is_locked"] = isLocked
        });

    public Task BroadcastMeetingEndedAsync(Guid meetingId, string reason) =>
        BroadcastToRoomAsync(meetingId, new JsonObject
        {
            ["type"] = "meeting_ended",
            ["reason"] = reason
        });

    public Task BroadcastRecordingStateAsync(Guid meetingId, bool isRecording, Guid? recordingId = null) =>
        BroadcastToRoomAsync(meetingId, new JsonObject
        {
            ["type"] = "recording_state_changed",
            ["is_recording"] = isRecording,
            ["recording_id"] = recordingId?.ToString()
        });

    public Task SendSubscribeOfferAsync(Guid meetingId, Guid participantId, string sdpOffer, List<TrackMappingDto> mappings)
    {
        if (_roomSockets.TryGetValue(meetingId, out var sockets) && sockets.TryGetValue(participantId, out var socket))
        {
            var msg = new JsonObject
            {
                ["type"] = "subscribe_offer",
                ["sdp"] = sdpOffer,
                ["mapping"] = JsonSerializer.SerializeToNode(mappings)
            };
            return SendJsonAsync(socket, msg);
        }
        return Task.CompletedTask;
    }

    private async Task BroadcastToRoomAsync(Guid meetingId, JsonObject json, Guid? excludeParticipantId = null)
    {
        if (_roomSockets.TryGetValue(meetingId, out var sockets))
        {
            var tasks = sockets
                .Where(kvp => !excludeParticipantId.HasValue || kvp.Key != excludeParticipantId.Value)
                .Select(kvp => SendJsonAsync(kvp.Value, json));

            await Task.WhenAll(tasks);
        }
    }

    private static async Task SendJsonAsync(WebSocket socket, JsonObject json)
    {
        if (socket.State == WebSocketState.Open)
        {
            var bytes = Encoding.UTF8.GetBytes(json.ToJsonString());
            await socket.SendAsync(new ArraySegment<byte>(bytes), WebSocketMessageType.Text, true, CancellationToken.None);
        }
    }
}
