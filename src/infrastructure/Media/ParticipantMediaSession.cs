using System.Collections.Concurrent;
using Confer.Application.DTOs;
using Confer.Domain.Enums;
using SIPSorcery.Net;

namespace Confer.Infrastructure.Media;

public sealed class ParticipantMediaSession
{
    public Guid ParticipantId { get; }
    public RTCPeerConnection? PublisherPc { get; set; }
    public RTCPeerConnection? SubscriberPc { get; set; }

    public bool IsAudioMuted { get; set; }
    public bool IsVideoMuted { get; set; }
    public bool IsScreenSharing { get; set; }
    public bool IsHandRaised { get; set; }

    public List<TrackIntent> PublishedTracks { get; } = new();
    public ConcurrentDictionary<Guid, TileSpec> ViewportTiles { get; } = new();

    /// <summary>
    /// The RTP header extension ID this participant's publisher connection negotiated for
    /// RFC 6464 audio level (urn:ietf:params:rtp-hdrext:ssrc-audio-level), resolved once from
    /// the remote SDP. Null if the client didn't offer the extension.
    /// </summary>
    public int? AudioLevelExtensionId { get; set; }

    /// <summary>
    /// On this participant's SubscriberPc, which VideoStream/AudioStream instance carries a
    /// given remote publisher's media. SIPSorcery's RTPSession supports multiple simultaneous
    /// streams per kind (RTPSession.VideoStreamList/AudioStreamList), so RTP forwarded to this
    /// subscriber for publisher X must be sent on X's specific stream — the session-level
    /// SendRtpRaw(mediaType) only ever targets the primary/first stream and silently corrupts
    /// multi-party rooms.
    /// </summary>
    public ConcurrentDictionary<Guid, VideoStream> SubscribedVideoStreams { get; } = new();
    public ConcurrentDictionary<Guid, AudioStream> SubscribedAudioStreams { get; } = new();

    /// <summary>
    /// Serializes renegotiations of this participant's SubscriberPc — SIPSorcery (like all
    /// WebRTC stacks) can only have one offer outstanding at a time, so two publishers
    /// appearing close together must not both call createOffer() concurrently.
    /// </summary>
    public SemaphoreSlim RenegotiationLock { get; } = new(1, 1);

    public ParticipantMediaSession(Guid participantId)
    {
        ParticipantId = participantId;
    }

    public void Close()
    {
        try
        {
            PublisherPc?.Close("Participant left");
            SubscriberPc?.Close("Participant left");
        }
        catch
        {
            // Ignore close errors
        }
        finally
        {
            RenegotiationLock.Dispose();
        }
    }
}
