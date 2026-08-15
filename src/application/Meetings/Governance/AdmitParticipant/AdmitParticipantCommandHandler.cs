using Confer.Application.Interfaces;
using Confer.Domain.Enums;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.Governance.AdmitParticipant;

public sealed class AdmitParticipantCommandHandler(
    IConferDbContext dbContext,
    ISignalingNotifier signalingNotifier)
    : ICommandHandler<AdmitParticipantCommand, Result>
{
    public async Task<Result> HandleAsync(
        AdmitParticipantCommand command,
        CancellationToken cancellationToken = default)
    {
        var meeting = await dbContext.Meetings
            .Include(m => m.Sessions)
            .ThenInclude(s => s.Participations)
            .FirstOrDefaultAsync(m => m.Id == command.MeetingId, cancellationToken);

        if (meeting is null)
            return Result.Failure(MeetingErrors.NotFound);

        var admitResult = meeting.AdmitParticipant(command.ActorId, command.ParticipantId);
        if (admitResult.IsFailure)
            return Result.Failure(admitResult.Error);

        await dbContext.SaveChangesAsync(cancellationToken);

        await signalingNotifier.BroadcastParticipantAdmittedAsync(meeting.Id, command.ParticipantId);

        var activeSession = meeting.Sessions.FirstOrDefault(s => s.EndedAt == null);
        var waitingCount = activeSession?.Participations
            .Count(p => p.Status == ParticipationStatus.InWaitingRoom && p.LeftAt == null) ?? 0;

        await signalingNotifier.BroadcastWaitingRoomUpdateAsync(meeting.Id, meeting.IsWaitingRoomEnabled, waitingCount);

        return Result.Success();
    }
}
