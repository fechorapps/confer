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
    }
}
