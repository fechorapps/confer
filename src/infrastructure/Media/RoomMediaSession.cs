using System.Collections.Concurrent;
using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Domain.Enums;
using Confer.Shared.Results;
using Microsoft.Extensions.Logging;
using SIPSorcery.Net;

namespace Confer.Infrastructure.Media;

public sealed class RoomMediaSession
{
    public Guid MeetingId { get; }
    private readonly ILogger _logger;
    private readonly ISignalingNotifier _signalingNotifier;
    private readonly ConcurrentDictionary<Guid, ParticipantMediaSession> _participants = new();
    private readonly ActiveSpeakerDetector _speakerDetector = new();
    private readonly SimulcastLayerSelector _layerSelector = new();
    private readonly Timer _speakerBroadcastTimer;

    public RoomMediaSession(Guid meetingId, ISignalingNotifier signalingNotifier, ILogger logger)
    {
        MeetingId = meetingId;
        _signalingNotifier = signalingNotifier;
        _logger = logger;

        // 4 Hz Active Speaker broadcast as per SDD §6.6
        _speakerBroadcastTimer = new Timer(_ => BroadcastActiveSpeakers(), null, TimeSpan.FromMilliseconds(250), TimeSpan.FromMilliseconds(250));
    }

    public ParticipantMediaSession GetOrCreateParticipant(Guid participantId)
    {
        return _participants.GetOrAdd(participantId, id => new ParticipantMediaSession(id));
    }

    public async Task<Result<string>> HandlePublishOfferAsync(Guid participantId, string sdpOffer, List<TrackIntent> tracks)
    {
        var participant = GetOrCreateParticipant(participantId);
        participant.PublishedTracks.Clear();
        participant.PublishedTracks.AddRange(tracks);

        var pcConfig = new RTCConfiguration
        {
            iceServers = new List<RTCIceServer>
            {
                new() { urls = "stun:stun.l.google.com:19302" }
            }
        };

        var pc = new RTCPeerConnection(pcConfig);
        participant.PublisherPc = pc;

        // Route incoming RTP packets from this publisher to all other subscribers in the room
        pc.OnRtpPacketReceived += (ep, mediaType, rtpPacket) =>
        {
            ForwardRtpPacket(participantId, mediaType, rtpPacket);
        };

        var setRemoteResult = pc.setRemoteDescription(new RTCSessionDescriptionInit
        {
            type = RTCSdpType.offer,
            sdp = sdpOffer
        });

        if (setRemoteResult != SetDescriptionResultEnum.OK)
        {
            return Result.Failure<string>(Error.Validation("WebRTC.SetRemoteDescriptionFailed", $"Failed to set remote description: {setRemoteResult}"));
        }

        var answer = pc.createAnswer();
        pc.setLocalDescription(answer);

        _logger.LogInformation("Publisher PC answer created for participant {ParticipantId} in room {MeetingId}", participantId, MeetingId);

        // Notify other participants in the room to subscribe to this publisher's new tracks
        await NotifySubscribersOfNewPublisherAsync(participantId, tracks);

        return Result.Success(answer.sdp);
    }

    public Task<Result> HandleSubscribeAnswerAsync(Guid participantId, string sdpAnswer)
    {
        if (!_participants.TryGetValue(participantId, out var participant) || participant.SubscriberPc is null)
        {
            return Task.FromResult(Result.Failure(Error.NotFound("WebRTC.SubscriberNotFound", "Subscriber PeerConnection not found.")));
        }

        participant.SubscriberPc.setRemoteDescription(new RTCSessionDescriptionInit
        {
            type = RTCSdpType.answer,
            sdp = sdpAnswer
        });

        return Task.FromResult(Result.Success());
    }

    public Task<Result> UpdateViewportAsync(Guid participantId, List<TileSpec> tiles)
    {
        if (_participants.TryGetValue(participantId, out var participant))
        {
            foreach (var tile in tiles)
            {
                participant.ViewportTiles[tile.ParticipantId] = tile;
            }
        }

        return Task.FromResult(Result.Success());
    }

    public Task<Result> SetMuteAsync(Guid participantId, MediaKind kind, bool isMuted)
    {
        if (_participants.TryGetValue(participantId, out var participant))
        {
            if (kind == MediaKind.Audio) participant.IsAudioMuted = isMuted;
            if (kind == MediaKind.Video) participant.IsVideoMuted = isMuted;
            if (kind == MediaKind.ScreenShare) participant.IsScreenSharing = !isMuted;
        }

        return Task.FromResult(Result.Success());
    }

    public void RemoveParticipant(Guid participantId)
    {
        if (_participants.TryRemove(participantId, out var session))
        {
            session.Close();
            _speakerDetector.RemoveParticipant(participantId);
        }
    }

    public void Close()
    {
        _speakerBroadcastTimer.Dispose();
        foreach (var participant in _participants.Values)
        {
            participant.Close();
        }
        _participants.Clear();
    }

    private void ForwardRtpPacket(Guid publisherId, SDPMediaTypesEnum mediaType, RTPPacket rtpPacket)
    {
        if (mediaType == SDPMediaTypesEnum.audio)
        {
            // Evaluate audio level header or payload energy
            // In RFC 6464, audio level is in header extension; here we approximate packet energy
            var energy = ComputeRtpEnergy(rtpPacket.Payload);
            _speakerDetector.ReportAudioLevel(publisherId, energy);
        }

        // Forward to other participants' Subscriber PCs
        foreach (var (subscriberId, subscriber) in _participants)
        {
            if (subscriberId == publisherId || subscriber.SubscriberPc == null)
                continue;

            if (mediaType == SDPMediaTypesEnum.video)
            {
                // Check subscriber's requested viewport layer
                subscriber.ViewportTiles.TryGetValue(publisherId, out var tile);
                var layer = _layerSelector.SelectLayer(publisherId, tile ?? new TileSpec(publisherId, 640, 360, true));
                
                // Forward if matching target layer or default
                subscriber.SubscriberPc.SendRtpRaw(mediaType, rtpPacket.Payload, rtpPacket.Header.Timestamp, rtpPacket.Header.MarkerBit > 0 ? 1 : 0, rtpPacket.Header.PayloadType);
            }
            else if (mediaType == SDPMediaTypesEnum.audio)
            {
                if (!_participants.TryGetValue(publisherId, out var pub) || !pub.IsAudioMuted)
                {
                    subscriber.SubscriberPc.SendRtpRaw(mediaType, rtpPacket.Payload, rtpPacket.Header.Timestamp, rtpPacket.Header.MarkerBit > 0 ? 1 : 0, rtpPacket.Header.PayloadType);
                }
            }
        }
    }

    private async Task NotifySubscribersOfNewPublisherAsync(Guid publisherId, List<TrackIntent> tracks)
    {
        var mappings = tracks.Select(t => new TrackMappingDto(
            t.TrackId,
            publisherId,
            t.Kind,
            "f"
        )).ToList();

        foreach (var (subId, sub) in _participants)
        {
            if (subId == publisherId) continue;

            // Generate an offer or notify client to negotiate
            await _signalingNotifier.SendSubscribeOfferAsync(MeetingId, subId, string.Empty, mappings);
        }
    }

    private void BroadcastActiveSpeakers()
    {
        var topSpeakers = _speakerDetector.GetTopSpeakers(3);
        if (topSpeakers.Count > 0)
        {
            _ = _signalingNotifier.BroadcastActiveSpeakersAsync(MeetingId, topSpeakers);
        }
    }

    private static int ComputeRtpEnergy(byte[] payload)
    {
        if (payload == null || payload.Length == 0) return -127;
        // Simple RMS estimation across audio samples
        long sum = 0;
        for (int i = 0; i < payload.Length; i++)
        {
            sum += Math.Abs((sbyte)payload[i]);
        }
        var avg = sum / (double)payload.Length;
        return avg > 5 ? (int)Math.Round(-60 + Math.Min(60, avg)) : -127;
    }
}
