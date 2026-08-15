using System.Text.Json.Serialization;

namespace Confer.Application.DTOs;

public record MeetingPolicyDto(
    [property: JsonPropertyName("allow_screen_share")] bool AllowScreenShare = true,
    [property: JsonPropertyName("allow_chat")] bool AllowChat = true,
    [property: JsonPropertyName("allow_unmute_self")] bool AllowUnmuteSelf = true,
    [property: JsonPropertyName("mute_on_entry")] bool MuteOnEntry = false,
    [property: JsonPropertyName("allow_rename")] bool AllowRename = true
);

public record WaitingRoomParticipantDto(
    [property: JsonPropertyName("participant_id")] Guid ParticipantId,
    [property: JsonPropertyName("user_id")] Guid UserId,
    [property: JsonPropertyName("display_name")] string DisplayName,
    [property: JsonPropertyName("joined_at")] DateTime JoinedAt,
    [property: JsonPropertyName("client_info")] string? ClientInfo = null
);

public record WaitingRoomStatusDto(
    [property: JsonPropertyName("meeting_id")] Guid MeetingId,
    [property: JsonPropertyName("is_waiting_room_enabled")] bool IsWaitingRoomEnabled,
    [property: JsonPropertyName("waiting_participants")] List<WaitingRoomParticipantDto> WaitingParticipants
);

public record MeetingPolicyResponse(
    [property: JsonPropertyName("meeting_id")] Guid MeetingId,
    [property: JsonPropertyName("is_waiting_room_enabled")] bool IsWaitingRoomEnabled,
    [property: JsonPropertyName("is_watermark_enabled")] bool IsWatermarkEnabled,
    [property: JsonPropertyName("policy")] MeetingPolicyDto Policy
);
