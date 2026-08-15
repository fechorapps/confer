using Confer.Application.Interfaces;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.Governance.AdmitAll;

public sealed class AdmitAllCommandHandler(
    IConferDbContext dbContext,
    ISignalingNotifier signalingNotifier)
    : ICommandHandler<AdmitAllCommand, Result<int>>
{
    public async Task<Result<int>> HandleAsync(
        AdmitAllCommand command,
        CancellationToken cancellationToken = default)
    {
        var meeting = await dbContext.Meetings
            .Include(m => m.Sessions)
            .ThenInclude(s => s.Participations)
            .FirstOrDefaultAsync(m => m.Id == command.MeetingId, cancellationToken);

        if (meeting is null)
            return Result.Failure<int>(MeetingErrors.NotFound);

        var admitResult = meeting.AdmitAll(command.ActorId);
        if (admitResult.IsFailure)
            return Result.Failure<int>(admitResult.Error);

        var admittedList = admitResult.Value;
        await dbContext.SaveChangesAsync(cancellationToken);

        foreach (var participant in admittedList)
        {
            await signalingNotifier.BroadcastParticipantAdmittedAsync(meeting.Id, participant.Id);
        }

        await signalingNotifier.BroadcastWaitingRoomUpdateAsync(meeting.Id, meeting.IsWaitingRoomEnabled, 0);

        return Result.Success(admittedList.Count);
    }
}
