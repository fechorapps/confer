using Confer.Application.Interfaces;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.Lock;

public sealed class LockMeetingCommandHandler(
    IConferDbContext dbContext,
    ISignalingNotifier signalingNotifier)
    : ICommandHandler<LockMeetingCommand, Result>
{
    public async Task<Result> HandleAsync(
        LockMeetingCommand command,
        CancellationToken cancellationToken = default)
    {
        var meeting = await dbContext.Meetings
            .FirstOrDefaultAsync(m => m.Id == command.MeetingId, cancellationToken);

        if (meeting is null)
            return Result.Failure(MeetingErrors.NotFound);

        var lockResult = command.Lock ? meeting.Lock(command.ActorId) : meeting.Unlock(command.ActorId);
        if (lockResult.IsFailure)
            return lockResult;

        await dbContext.SaveChangesAsync(cancellationToken);
        await signalingNotifier.BroadcastMeetingLockedAsync(meeting.Id, meeting.IsLocked);

        return Result.Success();
    }
}
