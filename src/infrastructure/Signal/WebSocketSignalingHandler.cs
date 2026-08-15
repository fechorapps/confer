using System.Collections.Concurrent;
using System.Net.WebSockets;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Application.Meetings.Governance.AdmitAll;
using Confer.Application.Meetings.Governance.AdmitParticipant;
using Confer.Application.Meetings.Governance.RejectParticipant;
using Confer.Application.Meetings.Governance.ToggleWaitingRoom;
using Confer.Application.Meetings.Governance.UpdatePolicy;
using Confer.Application.Meetings.Stream.StartLiveStream;
using Confer.Application.Meetings.Stream.StopLiveStream;
using Confer.Domain.Enums;
using Confer.Domain.Sessions;
using Confer.Infrastructure.AI;
using Confer.Infrastructure.Observability;
using Confer.Shared.Application.Interfaces;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

namespace Confer.Infrastructure.Signal;

public sealed class WebSocketSignalingHandler : ISignalingNotifier
{
    private readonly ILogger<WebSocketSignalingHandler> _logger;
    private readonly IServiceProvider _serviceProvider;
    private readonly IConferAiCompanionService _aiService;
    private readonly ConcurrentDictionary<Guid, ConcurrentDictionary<Guid, WebSocket>> _roomSockets = new();
    private readonly ConcurrentDictionary<Guid, RoomTokenClaims> _socketClaims = new();
    private readonly ConcurrentDictionary<Guid, List<ChatMessage>> _roomChatHistory = new();
    private readonly ConcurrentDictionary<Guid, List<CaptionChunkDto>> _roomCaptionHistory = new();

    public WebSocketSignalingHandler(
        ILogger<WebSocketSignalingHandler> logger,
        IServiceProvider serviceProvider,
        IConferAiCompanionService aiService)
    {
        _logger = logger;
        _serviceProvider = serviceProvider;
        _aiService = aiService;
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

        // Register with the SFU room and pick up any tracks already being published, so a
        // participant who joins after others are already on camera doesn't have to wait for a
        // re-publish to see/hear them.
        await sfuManager.JoinRoomAsync(meetingId, participantId);

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

                case "ice_candidate":
                    var candidateStr = node["candidate"]?.GetValue<string>();
                    if (!string.IsNullOrWhiteSpace(candidateStr))
                    {
                        var target = node["target"]?.GetValue<string>()?.ToLowerInvariant() == "subscribe" ? "subscribe" : "publish";
                        var candidateDto = new IceCandidateDto(
                            candidateStr,
                            node["sdp_mid"]?.GetValue<string>(),
                            node["sdp_mline_index"]?.GetValue<ushort>(),
                            node["username_fragment"]?.GetValue<string>());
                        await sfuManager.HandleIceCandidateAsync(claims.MeetingId, claims.ParticipantId, target, candidateDto);
                    }
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
                case "chat_message":
                case "chat_send":
                    var body = node["body"]?.GetValue<string>() ?? node["message"]?.GetValue<string>() ?? node["content"]?.GetValue<string>() ?? string.Empty;
                    if (!string.IsNullOrWhiteSpace(body))
                    {
                        var msgId = Guid.NewGuid();
                        var now = DateTime.UtcNow;
                        await BroadcastChatAsync(claims.MeetingId, msgId, claims.ParticipantId, claims.DisplayName, body, now);

                        // Record in room history
                        var chatHistory = _roomChatHistory.GetOrAdd(claims.MeetingId, _ => new List<ChatMessage>());
                        lock (chatHistory)
                        {
                            chatHistory.Add(ChatMessage.Create(claims.MeetingId, claims.ParticipantId, claims.DisplayName, body));
                            if (chatHistory.Count > 100) chatHistory.RemoveAt(0);
                        }

                        // Check if message is addressed to Confer AI Copilot (@confer-ai or /ai)
                        if (_aiService.IsAiCommand(body))
                        {
                            var captionHistory = _roomCaptionHistory.GetOrAdd(claims.MeetingId, _ => new List<CaptionChunkDto>());
                            List<CaptionChunkDto> captionsSnapshot;
                            List<ChatMessage> chatSnapshot;
                            lock (captionHistory) { captionsSnapshot = captionHistory.ToList(); }
                            lock (chatHistory) { chatSnapshot = chatHistory.ToList(); }

                            var aiResponse = await _aiService.ProcessAiQueryAsync(
                                claims.MeetingId,
                                body,
                                claims.DisplayName,
                                chatSnapshot,
                                captionsSnapshot);

                            // Send AI Copilot reply to room
                            await Task.Delay(250); // slight natural cadence
                            await BroadcastChatAsync(
                                claims.MeetingId,
                                Guid.NewGuid(),
                                Guid.Empty,
                                "Confer AI Copilot",
                                aiResponse,
                                DateTime.UtcNow);
                        }
                    }
                    break;

                case "reaction":
                    var emoji = node["emoji"]?.GetValue<string>() ?? "👍";
                    await BroadcastReactionAsync(claims.MeetingId, claims.ParticipantId, claims.DisplayName, emoji);
                    break;

                case "admit_participant":
                    if (claims.Role == ParticipantRole.Host || claims.Role == ParticipantRole.CoHost)
                    {
                        var targetIdStr = node["target_participant_id"]?.GetValue<string>() ?? node["participant_id"]?.GetValue<string>();
                        if (Guid.TryParse(targetIdStr, out var targetId))
                        {
                            using var scope = _serviceProvider.CreateScope();
                            var dispatcher = scope.ServiceProvider.GetRequiredService<ICqrsDispatcher>();
                            await dispatcher.SendAsync(new AdmitParticipantCommand(claims.MeetingId, claims.UserId, targetId));
                        }
                    }
                    break;

                case "admit_all":
                    if (claims.Role == ParticipantRole.Host || claims.Role == ParticipantRole.CoHost)
                    {
                        using var scope = _serviceProvider.CreateScope();
                        var dispatcher = scope.ServiceProvider.GetRequiredService<ICqrsDispatcher>();
                        await dispatcher.SendAsync(new AdmitAllCommand(claims.MeetingId, claims.UserId));
                    }
                    break;

                case "reject_participant":
                    if (claims.Role == ParticipantRole.Host || claims.Role == ParticipantRole.CoHost)
                    {
                        var targetIdStr = node["target_participant_id"]?.GetValue<string>() ?? node["participant_id"]?.GetValue<string>();
                        if (Guid.TryParse(targetIdStr, out var targetId))
                        {
                            using var scope = _serviceProvider.CreateScope();
                            var dispatcher = scope.ServiceProvider.GetRequiredService<ICqrsDispatcher>();
                            await dispatcher.SendAsync(new RejectParticipantCommand(claims.MeetingId, claims.UserId, targetId));
                            if (_roomSockets.TryGetValue(claims.MeetingId, out var sockets) && sockets.TryGetValue(targetId, out var targetSocket))
                            {
                                await targetSocket.CloseAsync(WebSocketCloseStatus.NormalClosure, "Rejected from waiting room", CancellationToken.None);
                            }
                        }
                    }
                    break;

                case "update_policy":
                    if (claims.Role == ParticipantRole.Host || claims.Role == ParticipantRole.CoHost)
                    {
                        var policyNode = node["policy"];
                        var allowScreenShare = policyNode?["allow_screen_share"]?.GetValue<bool>() ?? true;
                        var allowChat = policyNode?["allow_chat"]?.GetValue<bool>() ?? true;
                        var allowUnmuteSelf = policyNode?["allow_unmute_self"]?.GetValue<bool>() ?? true;
                        var muteOnEntry = policyNode?["mute_on_entry"]?.GetValue<bool>() ?? false;
                        var allowRename = policyNode?["allow_rename"]?.GetValue<bool>() ?? true;
                        var isWatermark = node["is_watermark_enabled"]?.GetValue<bool>();
                        var isWaitingRoom = node["is_waiting_room_enabled"]?.GetValue<bool>();

                        var policyDto = new MeetingPolicyDto(allowScreenShare, allowChat, allowUnmuteSelf, muteOnEntry, allowRename);
                        using var scope = _serviceProvider.CreateScope();
                        var dispatcher = scope.ServiceProvider.GetRequiredService<ICqrsDispatcher>();
                        await dispatcher.SendAsync(new UpdateMeetingPolicyCommand(claims.MeetingId, claims.UserId, policyDto, isWatermark, isWaitingRoom));
                    }
                    break;

                case "toggle_waiting_room":
                    if (claims.Role == ParticipantRole.Host || claims.Role == ParticipantRole.CoHost)
                    {
                        var enabled = node["enabled"]?.GetValue<bool>() ?? true;
                        using var scope = _serviceProvider.CreateScope();
                        var dispatcher = scope.ServiceProvider.GetRequiredService<ICqrsDispatcher>();
                        await dispatcher.SendAsync(new ToggleWaitingRoomCommand(claims.MeetingId, claims.UserId, enabled));
                    }
                    break;

                case "host_action":
                    if (claims.Role == ParticipantRole.Host || claims.Role == ParticipantRole.CoHost)
                    {
                        var action = node["action"]?.GetValue<string>()?.ToLowerInvariant();
                        var targetIdStr = node["target_participant_id"]?.GetValue<string>() ?? node["participant_id"]?.GetValue<string>();
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
                            else if (action == "admit")
                            {
                                using var scope = _serviceProvider.CreateScope();
                                var dispatcher = scope.ServiceProvider.GetRequiredService<ICqrsDispatcher>();
                                await dispatcher.SendAsync(new AdmitParticipantCommand(claims.MeetingId, claims.UserId, targetId));
                            }
                            else if (action == "reject")
                            {
                                using var scope = _serviceProvider.CreateScope();
                                var dispatcher = scope.ServiceProvider.GetRequiredService<ICqrsDispatcher>();
                                await dispatcher.SendAsync(new RejectParticipantCommand(claims.MeetingId, claims.UserId, targetId));
                                if (_roomSockets.TryGetValue(claims.MeetingId, out var sockets) && sockets.TryGetValue(targetId, out var targetSocket))
                                {
                                    await targetSocket.CloseAsync(WebSocketCloseStatus.NormalClosure, "Rejected by host", CancellationToken.None);
                                }
                            }
                        }
                        else if (action == "admit_all")
                        {
                            using var scope = _serviceProvider.CreateScope();
                            var dispatcher = scope.ServiceProvider.GetRequiredService<ICqrsDispatcher>();
                            await dispatcher.SendAsync(new AdmitAllCommand(claims.MeetingId, claims.UserId));
                        }
                    }
                    break;

                case "start_stream":
                    if (claims.Role == ParticipantRole.Host || claims.Role == ParticipantRole.CoHost)
                    {
                        var rtmpUrl = node["rtmp_url"]?.GetValue<string>() ?? string.Empty;
                        var streamKey = node["stream_key"]?.GetValue<string>() ?? string.Empty;
                        using var scope = _serviceProvider.CreateScope();
                        var dispatcher = scope.ServiceProvider.GetRequiredService<ICqrsDispatcher>();
                        await dispatcher.SendAsync(new StartLiveStreamCommand(claims.MeetingId, claims.UserId, rtmpUrl, streamKey));
                    }
                    break;

                case "stop_stream":
                    if (claims.Role == ParticipantRole.Host || claims.Role == ParticipantRole.CoHost)
                    {
                        using var scope = _serviceProvider.CreateScope();
                        var dispatcher = scope.ServiceProvider.GetRequiredService<ICqrsDispatcher>();
                        await dispatcher.SendAsync(new StopLiveStreamCommand(claims.MeetingId, claims.UserId));
                    }
                    break;

                case "caption_chunk":
                case "send_caption":
                    var captionText = node["text"]?.GetValue<string>() ?? string.Empty;
                    if (!string.IsNullOrWhiteSpace(captionText))
                    {
                        var isFinal = node["is_final"]?.GetValue<bool>() ?? false;
                        var language = node["language"]?.GetValue<string>() ?? "en-US";
                        var timestampMs = node["timestamp_ms"]?.GetValue<long>() ?? DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();

                        var captionChunk = new CaptionChunkDto(
                            claims.ParticipantId,
                            claims.DisplayName,
                            captionText,
                            isFinal,
                            language,
                            timestampMs
                        );
                        await BroadcastCaptionAsync(claims.MeetingId, captionChunk);

                        if (isFinal)
                        {
                            var captionHistory = _roomCaptionHistory.GetOrAdd(claims.MeetingId, _ => new List<CaptionChunkDto>());
                            lock (captionHistory)
                            {
                                captionHistory.Add(captionChunk);
                                if (captionHistory.Count > 200) captionHistory.RemoveAt(0);
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

    public Task SendIceCandidateAsync(Guid meetingId, Guid participantId, string target, IceCandidateDto candidate)
    {
        if (_roomSockets.TryGetValue(meetingId, out var sockets) && sockets.TryGetValue(participantId, out var socket))
        {
            var msg = new JsonObject
            {
                ["type"] = "ice_candidate",
                ["target"] = target,
                ["candidate"] = candidate.Candidate,
                ["sdp_mid"] = candidate.SdpMid,
                ["sdp_mline_index"] = candidate.SdpMLineIndex,
                ["username_fragment"] = candidate.UsernameFragment
            };
            return SendJsonAsync(socket, msg);
        }
        return Task.CompletedTask;
    }

    public Task BroadcastPollCreatedAsync(Guid meetingId, PollDto poll) =>
        BroadcastToRoomAsync(meetingId, new JsonObject
        {
            ["type"] = "poll_created",
            ["poll"] = JsonSerializer.SerializeToNode(poll)
        });

    public Task BroadcastPollUpdatedAsync(Guid meetingId, PollDto poll) =>
        BroadcastToRoomAsync(meetingId, new JsonObject
        {
            ["type"] = "poll_updated",
            ["poll"] = JsonSerializer.SerializeToNode(poll)
        });

    public Task BroadcastPollClosedAsync(Guid meetingId, Guid pollId) =>
        BroadcastToRoomAsync(meetingId, new JsonObject
        {
            ["type"] = "poll_closed",
            ["poll_id"] = pollId.ToString()
        });

    public Task BroadcastBreakoutInviteAsync(Guid meetingId, Guid participantId, BreakoutInviteDto invite)
    {
        if (participantId == Guid.Empty)
        {
            return BroadcastToRoomAsync(meetingId, new JsonObject
            {
                ["type"] = "breakout_invite",
                ["breakout_room_id"] = invite.BreakoutRoomId.ToString(),
                ["room_name"] = invite.RoomName,
                ["session_id"] = invite.SessionId.ToString()
            });
        }

        if (_roomSockets.TryGetValue(meetingId, out var sockets) && sockets.TryGetValue(participantId, out var socket))
        {
            var msg = new JsonObject
            {
                ["type"] = "breakout_invite",
                ["breakout_room_id"] = invite.BreakoutRoomId.ToString(),
                ["room_name"] = invite.RoomName,
                ["session_id"] = invite.SessionId.ToString()
            };
            return SendJsonAsync(socket, msg);
        }
        return Task.CompletedTask;
    }

    public Task BroadcastWaitingRoomUpdateAsync(Guid meetingId, bool isWaitingRoomEnabled, int waitingCount) =>
        BroadcastToRoomAsync(meetingId, new JsonObject
        {
            ["type"] = "waiting_room_update",
            ["is_waiting_room_enabled"] = isWaitingRoomEnabled,
            ["waiting_count"] = waitingCount
        });

    public Task BroadcastPolicyChangedAsync(Guid meetingId, MeetingPolicyDto policy, bool isWatermarkEnabled = false, bool isWaitingRoomEnabled = false) =>
        BroadcastToRoomAsync(meetingId, new JsonObject
        {
            ["type"] = "policy_changed",
            ["policy"] = JsonSerializer.SerializeToNode(policy),
            ["is_watermark_enabled"] = isWatermarkEnabled,
            ["is_waiting_room_enabled"] = isWaitingRoomEnabled
        });

    public Task BroadcastParticipantAdmittedAsync(Guid meetingId, Guid participantId) =>
        BroadcastToRoomAsync(meetingId, new JsonObject
        {
            ["type"] = "participant_admitted",
            ["participant_id"] = participantId.ToString()
        });

    public Task BroadcastStreamStatusAsync(Guid meetingId, bool isStreaming, string rtmpUrl, string status, DateTime? startedAt = null) =>
        BroadcastToRoomAsync(meetingId, new JsonObject
        {
            ["type"] = "live_stream_status",
            ["is_streaming"] = isStreaming,
            ["rtmp_url"] = rtmpUrl,
            ["status"] = status,
            ["started_at"] = startedAt?.ToString("o")
        });

    public Task BroadcastCaptionAsync(Guid meetingId, CaptionChunkDto caption) =>
        BroadcastToRoomAsync(meetingId, new JsonObject
        {
            ["type"] = "caption_broadcast",
            ["participant_id"] = caption.ParticipantId.ToString(),
            ["speaker_name"] = caption.SpeakerName,
            ["text"] = caption.Text,
            ["is_final"] = caption.IsFinal,
            ["language"] = caption.Language,
            ["timestamp_ms"] = caption.TimestampMs,
            ["caption"] = JsonSerializer.SerializeToNode(caption)
        });

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
