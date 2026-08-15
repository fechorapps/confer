using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Breakouts.CloseBreakouts;

public record CloseBreakoutsCommand(
    Guid MeetingId,
    Guid ActorId
) : ICommand<Result>;
