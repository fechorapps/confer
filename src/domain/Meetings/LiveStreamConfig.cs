using Confer.Domain.Enums;

namespace Confer.Domain.Meetings;

public sealed record LiveStreamConfig
{
    public bool IsStreamingEnabled { get; init; } = false;
    public string RtmpUrl { get; init; } = string.Empty;
    public string StreamKey { get; init; } = string.Empty;
    public LiveStreamStatus Status { get; init; } = LiveStreamStatus.Idle;
    public DateTime? StartedAt { get; init; }

    public LiveStreamConfig() { }

    public LiveStreamConfig(
        bool isStreamingEnabled,
        string rtmpUrl,
        string streamKey,
        LiveStreamStatus status = LiveStreamStatus.Idle,
        DateTime? startedAt = null)
    {
        IsStreamingEnabled = isStreamingEnabled;
        RtmpUrl = rtmpUrl;
        StreamKey = streamKey;
        Status = status;
        StartedAt = startedAt;
    }

    public static LiveStreamConfig Default => new();
}
