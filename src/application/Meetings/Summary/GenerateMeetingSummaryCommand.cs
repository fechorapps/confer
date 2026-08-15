using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Summary;

public record GenerateMeetingSummaryCommand(
    Guid MeetingId,
    Guid ActorId,
    bool ForceRegenerate = false
) : ICommand<Result<MeetingSummaryDto>>;
