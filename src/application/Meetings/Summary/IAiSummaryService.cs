using Confer.Domain.Meetings;
using Confer.Domain.Sessions;

namespace Confer.Application.Meetings.Summary;

public interface IAiSummaryService
{
    Task<AiSummaryResult> GenerateSummaryAsync(
        Meeting meeting,
        IReadOnlyList<ChatMessage> chatMessages,
        IReadOnlyList<Participation> participations,
        CancellationToken cancellationToken = default);
}

public record AiSummaryResult(
    string Overview,
    List<string> KeyDecisions,
    List<ActionItemDto> ActionItems,
    int DurationMinutes,
    int ParticipantCount
);
