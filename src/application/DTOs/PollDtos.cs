using System.Text.Json.Serialization;

namespace Confer.Application.DTOs;

public record PollOptionDto(
    [property: JsonPropertyName("id")] Guid Id,
    [property: JsonPropertyName("poll_id")] Guid PollId,
    [property: JsonPropertyName("text")] string Text,
    [property: JsonPropertyName("index")] int Index,
    [property: JsonPropertyName("vote_count")] int VoteCount
);

public record PollDto(
    [property: JsonPropertyName("id")] Guid Id,
    [property: JsonPropertyName("meeting_id")] Guid MeetingId,
    [property: JsonPropertyName("creator_id")] Guid CreatorId,
    [property: JsonPropertyName("question")] string Question,
    [property: JsonPropertyName("is_anonymous")] bool IsAnonymous,
    [property: JsonPropertyName("is_multi_choice")] bool IsMultiChoice,
    [property: JsonPropertyName("is_active")] bool IsActive,
    [property: JsonPropertyName("created_at")] DateTime CreatedAt,
    [property: JsonPropertyName("closed_at")] DateTime? ClosedAt,
    [property: JsonPropertyName("options")] List<PollOptionDto> Options,
    [property: JsonPropertyName("total_votes")] int TotalVotes
);
