using System.Text.Json.Serialization;

namespace Confer.Application.DTOs;

public record BreakoutRoomDto(
    [property: JsonPropertyName("id")] Guid Id,
    [property: JsonPropertyName("meeting_id")] Guid MeetingId,
    [property: JsonPropertyName("session_id")] Guid SessionId,
    [property: JsonPropertyName("name")] string Name,
    [property: JsonPropertyName("index")] int Index,
    [property: JsonPropertyName("created_at")] DateTime CreatedAt,
    [property: JsonPropertyName("ends_at")] DateTime? EndsAt,
    [property: JsonPropertyName("max_duration_minutes")] int MaxDurationMinutes
);

public record BreakoutAssignmentDto(
    [property: JsonPropertyName("participant_id")] Guid ParticipantId,
    [property: JsonPropertyName("breakout_room_id")] Guid BreakoutRoomId,
    [property: JsonPropertyName("room_name")] string RoomName
);

public record BreakoutInviteDto(
    [property: JsonPropertyName("breakout_room_id")] Guid BreakoutRoomId,
    [property: JsonPropertyName("room_name")] string RoomName,
    [property: JsonPropertyName("session_id")] Guid SessionId
);
