using Confer.Application.Interfaces;
using Confer.Domain.Enums;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.Governance.ToggleWaitingRoom;

public sealed class ToggleWaitingRoomCommandHandler(
    IConferDbContext dbContext,
    ISignalingNotifier signalingNotifier)
    : ICommandHandler<ToggleWaitingRoomCommand, Result<bool>>
{
    public async Task<Result<bool>> HandleAsync(
        ToggleWaitingRoomCommand command,
        CancellationToken cancellationToken = default)
    {
        var meeting = await dbContext.Meetings
            .Include(m => m.Sessions)
            .ThenInclude(s => s.Participations)
            .FirstOrDefaultAsync(m => m.Id == command.MeetingId, cancellationToken);

        if (meeting is null)
            return Result.Failure<bool>(MeetingErrors.NotFound);

        var toggleResult = meeting.ToggleWaitingRoom(command.ActorId, command.Enabled);
        if (toggleResult.IsFailure)
            return Result.Failure<bool>(toggleResult.Error);

        await dbContext.SaveChangesAsync(cancellationToken);

        var activeSession = meeting.Sessions.FirstOrDefault(s => s.EndedAt == null);
        var waitingCount = activeSession?.Participations
            .Count(p => p.Status == ParticipationStatus.InWaitingRoom && p.LeftAt == null) ?? 0;

        await signalingNotifier.BroadcastWaitingRoomUpdateAsync(meeting.Id, meeting.IsWaitingRoomEnabled, waitingCount);

        return Result.Success(meeting.IsWaitingRoomEnabled);
    }
}
