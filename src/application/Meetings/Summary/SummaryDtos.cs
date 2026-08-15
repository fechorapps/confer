using System.Text.Json.Serialization;

namespace Confer.Application.Meetings.Summary;

public record ActionItemDto(
    [property: JsonPropertyName("title")] string Title,
    [property: JsonPropertyName("assignee")] string Assignee,
    [property: JsonPropertyName("status")] string Status = "pending"
);

public record MeetingSummaryDto(
    [property: JsonPropertyName("id")] Guid Id,
    [property: JsonPropertyName("meeting_id")] Guid MeetingId,
    [property: JsonPropertyName("overview")] string Overview,
    [property: JsonPropertyName("key_decisions")] List<string> KeyDecisions,
    [property: JsonPropertyName("action_items")] List<ActionItemDto> ActionItems,
    [property: JsonPropertyName("generated_at")] DateTime GeneratedAt,
    [property: JsonPropertyName("duration_minutes")] int DurationMinutes,
    [property: JsonPropertyName("participant_count")] int ParticipantCount
);
