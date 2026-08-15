using Confer.Application.Interfaces;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.Breakouts.CloseBreakouts;

public sealed class CloseBreakoutsCommandHandler(IConferDbContext dbContext)
    : ICommandHandler<CloseBreakoutsCommand, Result>
{
    public async Task<Result> HandleAsync(
        CloseBreakoutsCommand command,
        CancellationToken cancellationToken = default)
    {
        var meeting = await dbContext.Meetings
            .FirstOrDefaultAsync(m => m.Id == command.MeetingId, cancellationToken);

        if (meeting is null)
            return Result.Failure(MeetingErrors.NotFound);

        if (meeting.OwnerId != command.ActorId)
            return Result.Failure(BreakoutErrors.Unauthorized);

        var activeRooms = await dbContext.BreakoutRooms
            .Where(b => b.MeetingId == command.MeetingId && b.EndsAt == null)
            .ToListAsync(cancellationToken);

        foreach (var room in activeRooms)
        {
            room.Close();
        }

        await dbContext.SaveChangesAsync(cancellationToken);
        return Result.Success();
    }
}
