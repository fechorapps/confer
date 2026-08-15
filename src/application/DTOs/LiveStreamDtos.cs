using System.Text.Json.Serialization;

namespace Confer.Application.DTOs;

public record LiveStreamResponse(
    [property: JsonPropertyName("meeting_id")] Guid MeetingId,
    [property: JsonPropertyName("is_streaming_enabled")] bool IsStreamingEnabled,
    [property: JsonPropertyName("rtmp_url")] string RtmpUrl,
    [property: JsonPropertyName("status")] string Status,
    [property: JsonPropertyName("started_at")] DateTime? StartedAt
);
