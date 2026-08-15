using System.Text.Json.Serialization;
using Confer.Domain.Enums;

namespace Confer.Application.DTOs;

public record TrackIntent(
    [property: JsonPropertyName("track_id")] string TrackId,
    [property: JsonPropertyName("kind")] string Kind,
    [property: JsonPropertyName("simulcast")] bool Simulcast = false
);

public record TileSpec(
    [property: JsonPropertyName("participant_id")] Guid ParticipantId,
    [property: JsonPropertyName("width")] int Width,
    [property: JsonPropertyName("height")] int Height,
    [property: JsonPropertyName("visible")] bool Visible = true
);

public record IceServerConfig(
    [property: JsonPropertyName("urls")] string[] Urls,
    [property: JsonPropertyName("username")] string? Username = null,
    [property: JsonPropertyName("credential")] string? Credential = null
);

public record ParticipantStateDto(
    [property: JsonPropertyName("participant_id")] Guid ParticipantId,
    [property: JsonPropertyName("user_id")] Guid UserId,
    [property: JsonPropertyName("display_name")] string DisplayName,
    [property: JsonPropertyName("role")] string Role,
    [property: JsonPropertyName("is_audio_muted")] bool IsAudioMuted = false,
    [property: JsonPropertyName("is_video_muted")] bool IsVideoMuted = false,
    [property: JsonPropertyName("is_screen_sharing")] bool IsScreenSharing = false,
    [property: JsonPropertyName("is_hand_raised")] bool IsHandRaised = false
);

public record SpeakerInfoDto(
    [property: JsonPropertyName("participant_id")] Guid ParticipantId,
    [property: JsonPropertyName("level_dbov")] int LevelDbov
);

public record TrackMappingDto(
    [property: JsonPropertyName("track_id")] string TrackId,
    [property: JsonPropertyName("publisher_id")] Guid PublisherId,
    [property: JsonPropertyName("kind")] string Kind,
    [property: JsonPropertyName("layer")] string Layer
);

public record RoomTokenClaims(
    Guid MeetingId,
    Guid UserId,
    Guid ParticipantId,
    string DisplayName,
    ParticipantRole Role,
    string SfuNodeId,
    DateTime ExpiresAt
);
