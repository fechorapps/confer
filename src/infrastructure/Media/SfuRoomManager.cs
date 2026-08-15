using System.Collections.Concurrent;
using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Domain.Enums;
using Confer.Shared.Results;
using Microsoft.Extensions.Logging;
using SIPSorcery.Net;

namespace Confer.Infrastructure.Media;

public sealed class SfuRoomManager(
    ISignalingNotifier signalingNotifier,
    IIceServerProvider iceServerProvider,
    ILogger<SfuRoomManager> logger,
    ILoggerFactory loggerFactory) : ISfuRoomManager
{
    private readonly ConcurrentDictionary<Guid, RoomMediaSession> _rooms = new();

    public Task<Result> CreateOrGetRoomAsync(Guid meetingId)
    {
        _rooms.GetOrAdd(meetingId, id =>
            new RoomMediaSession(id, signalingNotifier, BuildIceServers(), loggerFactory.CreateLogger<RoomMediaSession>()));
        return Task.FromResult(Result.Success());
    }

    public async Task<Result> JoinRoomAsync(Guid meetingId, Guid participantId)
    {
        var room = GetOrCreateRoom(meetingId);
        return await room.JoinRoomAsync(participantId);
    }

    public async Task<Result<string>> HandlePublishOfferAsync(
        Guid meetingId,
        Guid participantId,
        string sdpOffer,
        List<TrackIntent> tracks)
    {
        var room = GetOrCreateRoom(meetingId);
        return await room.HandlePublishOfferAsync(participantId, sdpOffer, tracks);
    }

    public async Task<Result> HandleSubscribeAnswerAsync(Guid meetingId, Guid participantId, string sdpAnswer)
    {
        if (!_rooms.TryGetValue(meetingId, out var room))
            return Result.Failure(Error.NotFound("SFU.RoomNotFound", "Room not found."));

        return await room.HandleSubscribeAnswerAsync(participantId, sdpAnswer);
    }

    public async Task<Result> HandleIceCandidateAsync(Guid meetingId, Guid participantId, string target, IceCandidateDto candidate)
    {
        if (!_rooms.TryGetValue(meetingId, out var room))
            return Result.Failure(Error.NotFound("SFU.RoomNotFound", "Room not found."));

        var candidateInit = new RTCIceCandidateInit
        {
            candidate = candidate.Candidate,
            sdpMid = candidate.SdpMid,
            sdpMLineIndex = candidate.SdpMLineIndex ?? 0,
            usernameFragment = candidate.UsernameFragment
        };

        return await room.HandleIceCandidateAsync(participantId, target, candidateInit);
    }

    public async Task<Result> UpdateViewportAsync(Guid meetingId, Guid participantId, List<TileSpec> tiles)
    {
        if (_rooms.TryGetValue(meetingId, out var room))
        {
            return await room.UpdateViewportAsync(participantId, tiles);
        }
        return Result.Success();
    }

    public async Task<Result> SetMuteAsync(Guid meetingId, Guid participantId, MediaKind kind, bool isMuted)
    {
        if (_rooms.TryGetValue(meetingId, out var room))
        {
            return await room.SetMuteAsync(participantId, kind, isMuted);
        }
        return Result.Success();
    }

    public Task<Result> RemoveParticipantAsync(Guid meetingId, Guid participantId)
    {
        if (_rooms.TryGetValue(meetingId, out var room))
        {
            room.RemoveParticipant(participantId);
        }
        return Task.FromResult(Result.Success());
    }

    public Task<Result> CloseRoomAsync(Guid meetingId)
    {
        if (_rooms.TryRemove(meetingId, out var room))
        {
            room.Close();
        }
        return Task.FromResult(Result.Success());
    }

    private RoomMediaSession GetOrCreateRoom(Guid meetingId) =>
        _rooms.GetOrAdd(meetingId, id =>
            new RoomMediaSession(id, signalingNotifier, BuildIceServers(), loggerFactory.CreateLogger<RoomMediaSession>()));

    private List<RTCIceServer> BuildIceServers()
    {
        var servers = new List<RTCIceServer>();
        foreach (var server in iceServerProvider.GetIceServers())
        {
            foreach (var url in server.Urls)
            {
                servers.Add(new RTCIceServer
                {
                    urls = url,
                    username = server.Username,
                    credential = server.Credential
                });
            }
        }

        if (servers.Count == 0)
        {
            logger.LogWarning("No ICE servers configured for room media sessions; WebRTC connectivity will likely fail behind NAT.");
        }

        return servers;
    }
}
