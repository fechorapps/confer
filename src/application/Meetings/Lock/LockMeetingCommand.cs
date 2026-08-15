using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Lock;

public record LockMeetingCommand(
    Guid MeetingId,
    Guid ActorId,
    bool Lock
) : ICommand<Result>;
