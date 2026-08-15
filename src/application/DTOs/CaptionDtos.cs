using System.Text.Json.Serialization;

namespace Confer.Application.DTOs;

public record CaptionChunkDto(
    [property: JsonPropertyName("participant_id")] Guid ParticipantId,
    [property: JsonPropertyName("speaker_name")] string SpeakerName,
    [property: JsonPropertyName("text")] string Text,
    [property: JsonPropertyName("is_final")] bool IsFinal,
    [property: JsonPropertyName("language")] string Language,
    [property: JsonPropertyName("timestamp_ms")] long TimestampMs
);
