using Confer.Application.Interfaces;
using Confer.Domain.Enums;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.Governance.RejectParticipant;

public sealed class RejectParticipantCommandHandler(
    IConferDbContext dbContext,
    ISignalingNotifier signalingNotifier)
    : ICommandHandler<RejectParticipantCommand, Result>
{
    public async Task<Result> HandleAsync(
        RejectParticipantCommand command,
        CancellationToken cancellationToken = default)
    {
        var meeting = await dbContext.Meetings
            .Include(m => m.Sessions)
            .ThenInclude(s => s.Participations)
            .FirstOrDefaultAsync(m => m.Id == command.MeetingId, cancellationToken);

        if (meeting is null)
            return Result.Failure(MeetingErrors.NotFound);

        var rejectResult = meeting.RejectParticipant(command.ActorId, command.ParticipantId);
        if (rejectResult.IsFailure)
            return Result.Failure(rejectResult.Error);

        await dbContext.SaveChangesAsync(cancellationToken);

        await signalingNotifier.BroadcastLeftAsync(meeting.Id, command.ParticipantId, "rejected_from_waiting_room");

        var activeSession = meeting.Sessions.FirstOrDefault(s => s.EndedAt == null);
        var waitingCount = activeSession?.Participations
            .Count(p => p.Status == ParticipationStatus.InWaitingRoom && p.LeftAt == null) ?? 0;

        await signalingNotifier.BroadcastWaitingRoomUpdateAsync(meeting.Id, meeting.IsWaitingRoomEnabled, waitingCount);

        return Result.Success();
    }
}
