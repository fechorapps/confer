using System.Collections.Concurrent;
using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Domain.Enums;
using Confer.Shared.Results;
using Microsoft.Extensions.Logging;
using SIPSorcery.Net;
using SIPSorceryMedia.Abstractions;

namespace Confer.Infrastructure.Media;

public sealed class RoomMediaSession
{
    public Guid MeetingId { get; }
    private readonly ILogger _logger;
    private readonly ISignalingNotifier _signalingNotifier;
    private readonly List<RTCIceServer> _iceServers;
    private readonly ConcurrentDictionary<Guid, ParticipantMediaSession> _participants = new();
    private readonly ActiveSpeakerDetector _speakerDetector = new();
    private readonly Timer _speakerBroadcastTimer;

    // Fixed relay formats: this SFU forwards raw RTP payloads unchanged (no transcoding), so
    // every publisher is expected to negotiate VP8 video / Opus audio — matching what the
    // bundled web/desktop clients offer. A publisher offering a different codec still gets a
    // publisher-side answer (SIPSorcery negotiates whatever the offer proposes), but subscriber
    // fan-out is always VP8/Opus, so mixed-codec rooms are not supported yet.
    private const int Vp8PayloadTypeId = 96;
    private const int OpusPayloadTypeId = 111;

    public RoomMediaSession(Guid meetingId, ISignalingNotifier signalingNotifier, List<RTCIceServer> iceServers, ILogger logger)
    {
        MeetingId = meetingId;
        _signalingNotifier = signalingNotifier;
        _iceServers = iceServers;
        _logger = logger;

        // 4 Hz Active Speaker broadcast as per SDD §6.6
        _speakerBroadcastTimer = new Timer(_ => BroadcastActiveSpeakers(), null, TimeSpan.FromMilliseconds(250), TimeSpan.FromMilliseconds(250));
    }

    public ParticipantMediaSession GetOrCreateParticipant(Guid participantId)
    {
        return _participants.GetOrAdd(participantId, id => new ParticipantMediaSession(id));
    }

    /// <summary>
    /// Registers a participant as soon as they connect and offers them any tracks already
    /// published by other participants in the room, so joining late doesn't require the
    /// existing publishers to re-publish before the newcomer can see/hear them.
    /// </summary>
    public async Task<Result> JoinRoomAsync(Guid participantId)
    {
        GetOrCreateParticipant(participantId);

        var existingTracks = _participants
            .Where(kvp => kvp.Key != participantId && kvp.Value.PublishedTracks.Count > 0)
            .SelectMany(kvp => kvp.Value.PublishedTracks.Select(track => (PublisherId: kvp.Key, Track: track)))
            .ToList();

        if (existingTracks.Count > 0)
        {
            await OfferTracksToSubscriberAsync(participantId, existingTracks);
        }

        return Result.Success();
    }

    public async Task<Result<string>> HandlePublishOfferAsync(Guid participantId, string sdpOffer, List<TrackIntent> tracks)
    {
        var participant = GetOrCreateParticipant(participantId);
        participant.PublishedTracks.Clear();
        participant.PublishedTracks.AddRange(tracks);

        var isNewPublisherPc = participant.PublisherPc is null;
        var pc = participant.PublisherPc ??= CreatePeerConnection(participantId, "publish");

        if (isNewPublisherPc)
        {
            // The publisher PC never sends anything back, but it still has to declare local
            // (receive-only) capabilities for audio/video before answering — otherwise SIPSorcery
            // has nothing to offer for those m-lines and answers "inactive" instead of "recvonly",
            // which tells the browser's sender to stop, and no media is ever actually received.
            pc.addTrack(new MediaStreamTrack(
                new AudioFormat(AudioCodecsEnum.OPUS, OpusPayloadTypeId, 48000, 2, string.Empty),
                MediaStreamStatusEnum.RecvOnly));
            pc.addTrack(new MediaStreamTrack(
                new VideoFormat(VideoCodecsEnum.VP8, Vp8PayloadTypeId, 90000, string.Empty),
                MediaStreamStatusEnum.RecvOnly));
        }

        var setRemoteResult = pc.setRemoteDescription(new RTCSessionDescriptionInit
        {
            type = RTCSdpType.offer,
            sdp = sdpOffer
        });

        if (setRemoteResult != SetDescriptionResultEnum.OK)
        {
            return Result.Failure<string>(Error.Validation("WebRTC.SetRemoteDescriptionFailed", $"Failed to set remote description: {setRemoteResult}"));
        }

        participant.AudioLevelExtensionId = ResolveAudioLevelExtensionId(pc);

        var answer = pc.createAnswer();
        await pc.setLocalDescription(answer);

        _logger.LogInformation("Publisher PC answer created for participant {ParticipantId} in room {MeetingId}", participantId, MeetingId);

        // Notify other participants in the room to subscribe to this publisher's new tracks
        var newTracks = tracks.Select(t => (PublisherId: participantId, Track: t)).ToList();
        foreach (var subscriberId in _participants.Keys.Where(id => id != participantId))
        {
            await OfferTracksToSubscriberAsync(subscriberId, newTracks);
        }

        return Result.Success(answer.sdp);
    }

    public Task<Result> HandleSubscribeAnswerAsync(Guid participantId, string sdpAnswer)
    {
        if (!_participants.TryGetValue(participantId, out var participant) || participant.SubscriberPc is null)
        {
            return Task.FromResult(Result.Failure(Error.NotFound("WebRTC.SubscriberNotFound", "Subscriber PeerConnection not found.")));
        }

        var result = participant.SubscriberPc.setRemoteDescription(new RTCSessionDescriptionInit
        {
            type = RTCSdpType.answer,
            sdp = sdpAnswer
        });

        return Task.FromResult(result == SetDescriptionResultEnum.OK
            ? Result.Success()
            : Result.Failure(Error.Validation("WebRTC.SetRemoteDescriptionFailed", $"Failed to set subscriber remote description: {result}")));
    }

    public Task<Result> HandleIceCandidateAsync(Guid participantId, string target, RTCIceCandidateInit candidate)
    {
        if (!_participants.TryGetValue(participantId, out var participant))
        {
            return Task.FromResult(Result.Failure(Error.NotFound("WebRTC.ParticipantNotFound", "Participant not found.")));
        }

        var pc = target == "subscribe" ? participant.SubscriberPc : participant.PublisherPc;
        if (pc is null)
        {
            // Candidates can arrive before the corresponding offer/answer exchange completes;
            // trickle ICE on an as-yet-nonexistent PC is a client timing issue, not a server error.
            return Task.FromResult(Result.Success());
        }

        pc.addIceCandidate(candidate);
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

    /// <summary>
    /// Adds the given (publisher, track) pairs to the subscriber's SubscriberPc — creating it
    /// if this is their first subscription — and renegotiates with a single offer covering all
    /// of them. Renegotiations for the same subscriber are serialized so two publishers
    /// appearing close together don't race two concurrent createOffer() calls against a signaling
    /// state that can only have one offer outstanding at a time.
    /// </summary>
    private async Task OfferTracksToSubscriberAsync(Guid subscriberId, List<(Guid PublisherId, TrackIntent Track)> newTracks)
    {
        if (newTracks.Count == 0) return;
        if (!_participants.TryGetValue(subscriberId, out var subscriber)) return;

        await subscriber.RenegotiationLock.WaitAsync();
        try
        {
            var pc = subscriber.SubscriberPc ??= CreatePeerConnection(subscriberId, "subscribe");

            var mappings = new List<TrackMappingDto>();

            foreach (var (publisherId, track) in newTracks)
            {
                if (track.Kind == "audio")
                {
                    if (subscriber.SubscribedAudioStreams.ContainsKey(publisherId)) continue;

                    var mediaTrack = new MediaStreamTrack(
                        new AudioFormat(AudioCodecsEnum.OPUS, OpusPayloadTypeId, 48000, 2, string.Empty),
                        MediaStreamStatusEnum.SendOnly);
                    pc.addTrack(mediaTrack);

                    var stream = pc.AudioStreamList.LastOrDefault(s => ReferenceEquals(s.LocalTrack, mediaTrack));
                    if (stream is not null)
                    {
                        subscriber.SubscribedAudioStreams[publisherId] = stream;
                    }
                }
                else
                {
                    if (subscriber.SubscribedVideoStreams.ContainsKey(publisherId)) continue;

                    var mediaTrack = new MediaStreamTrack(
                        new VideoFormat(VideoCodecsEnum.VP8, Vp8PayloadTypeId, 90000, string.Empty),
                        MediaStreamStatusEnum.SendOnly);
                    pc.addTrack(mediaTrack);

                    var stream = pc.VideoStreamList.LastOrDefault(s => ReferenceEquals(s.LocalTrack, mediaTrack));
                    if (stream is not null)
                    {
                        subscriber.SubscribedVideoStreams[publisherId] = stream;
                    }
                }

                mappings.Add(new TrackMappingDto(track.TrackId, publisherId, track.Kind, "f"));
            }

            if (mappings.Count == 0) return;

            var offer = pc.createOffer();
            await pc.setLocalDescription(offer);

            await _signalingNotifier.SendSubscribeOfferAsync(MeetingId, subscriberId, offer.sdp, mappings);
        }
        finally
        {
            subscriber.RenegotiationLock.Release();
        }
    }

    private RTCPeerConnection CreatePeerConnection(Guid participantId, string role)
    {
        var pcConfig = new RTCConfiguration { iceServers = _iceServers };
        var pc = new RTCPeerConnection(pcConfig);

        pc.onicecandidate += candidate => ForwardLocalIceCandidate(participantId, role, candidate);
        pc.oniceconnectionstatechange += state =>
            _logger.LogDebug("{Role} PC for participant {ParticipantId} ICE state: {State}", role, participantId, state);

        if (role == "publish")
        {
            pc.OnRtpPacketReceived += (_, mediaType, rtpPacket) => ForwardRtpPacket(participantId, mediaType, rtpPacket);
        }

        return pc;
    }

    private void ForwardLocalIceCandidate(Guid participantId, string target, RTCIceCandidate? candidate)
    {
        if (candidate is null) return;

        // SIPSorcery's RTCIceCandidate.candidate holds just the SDP candidate-attribute value
        // (no "candidate:" prefix), but browsers require the full attribute line per the
        // RTCIceCandidateInit spec when it's passed to addIceCandidate() client-side.
        var candidateLine = candidate.candidate.StartsWith("candidate:", StringComparison.Ordinal)
            ? candidate.candidate
            : $"candidate:{candidate.candidate}";

        var dto = new IceCandidateDto(candidateLine, candidate.sdpMid, candidate.sdpMLineIndex);
        _ = _signalingNotifier.SendIceCandidateAsync(MeetingId, participantId, target, dto);
    }

    private void ForwardRtpPacket(Guid publisherId, SDPMediaTypesEnum mediaType, RTPPacket rtpPacket)
    {
        if (!_participants.TryGetValue(publisherId, out var publisher)) return;

        if (mediaType == SDPMediaTypesEnum.audio)
        {
            var levelDbov = ExtractAudioLevel(publisher, rtpPacket.Header);
            if (levelDbov.HasValue)
            {
                _speakerDetector.ReportAudioLevel(publisherId, levelDbov.Value);
            }

            if (publisher.IsAudioMuted) return;
        }
        else if (mediaType == SDPMediaTypesEnum.video && publisher.IsVideoMuted)
        {
            return;
        }

        var markerBit = rtpPacket.Header.MarkerBit > 0 ? 1 : 0;

        foreach (var (subscriberId, subscriber) in _participants)
        {
            if (subscriberId == publisherId) continue;

            if (mediaType == SDPMediaTypesEnum.video)
            {
                // Skip tiles the subscriber's current layout has marked as not visible — real
                // bitrate-tiered simulcast (SimulcastLayerSelector) needs the publisher to send
                // multiple encodings, which the clients don't do yet; this is the one bandwidth
                // saving we can correctly make with a single relayed encoding.
                if (subscriber.ViewportTiles.TryGetValue(publisherId, out var tile) && !tile.Visible)
                    continue;

                if (subscriber.SubscribedVideoStreams.TryGetValue(publisherId, out var videoStream))
                {
                    var payloadTypeId = videoStream.GetSendingFormat().ID;
                    videoStream.SendRtpRaw(rtpPacket.Payload, rtpPacket.Header.Timestamp, markerBit, payloadTypeId);
                }
            }
            else if (mediaType == SDPMediaTypesEnum.audio)
            {
                if (subscriber.SubscribedAudioStreams.TryGetValue(publisherId, out var audioStream))
                {
                    var payloadTypeId = audioStream.GetSendingFormat().ID;
                    audioStream.SendRtpRaw(rtpPacket.Payload, rtpPacket.Header.Timestamp, markerBit, payloadTypeId);
                }
            }
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

    /// <summary>
    /// Finds the RTP header extension ID this publisher's browser negotiated for RFC 6464
    /// audio level (urn:ietf:params:rtp-hdrext:ssrc-audio-level), so incoming packets can be
    /// matched against it. Returns null if the offer didn't include the extension.
    /// </summary>
    private static int? ResolveAudioLevelExtensionId(RTCPeerConnection pc)
    {
        var remoteTrack = pc.AudioStream?.RemoteTrack;
        if (remoteTrack?.HeaderExtensions is null) return null;

        foreach (var (id, extension) in remoteTrack.HeaderExtensions)
        {
            if (extension.Uri == AudioLevelExtension.RTP_HEADER_EXTENSION_URI)
            {
                return id;
            }
        }

        return null;
    }

    /// <summary>
    /// Reads the RFC 6464 audio level from the packet's header extensions and converts it to
    /// the -127 (silence) .. 0 (loudest) dBov scale ActiveSpeakerDetector expects. Returns null
    /// when the publisher didn't negotiate the extension (energy can't be recovered from an
    /// Opus-encoded payload — treating it as PCM, as this code used to, produces noise).
    /// </summary>
    private static int? ExtractAudioLevel(ParticipantMediaSession publisher, RTPHeader header)
    {
        if (publisher.AudioLevelExtensionId is not { } extensionId) return null;

        foreach (var extension in header.GetHeaderExtensions())
        {
            if (extension.Id != extensionId) continue;

            var level = new AudioLevelExtension.AudioLevel(extension.Data);
            // RFC 6464: Level is 0 (loudest) .. 127 (silence), i.e. -dBov.
            return -Math.Clamp((int)level.Level, 0, 127);
        }

        return null;
    }
}
