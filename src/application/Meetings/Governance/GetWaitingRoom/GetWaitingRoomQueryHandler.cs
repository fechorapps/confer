using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Domain.Enums;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.Governance.GetWaitingRoom;

public sealed class GetWaitingRoomQueryHandler(
    IConferDbContext dbContext)
    : IQueryHandler<GetWaitingRoomQuery, Result<WaitingRoomStatusDto>>
{
    public async Task<Result<WaitingRoomStatusDto>> HandleAsync(
        GetWaitingRoomQuery query,
        CancellationToken cancellationToken = default)
    {
        var meeting = await dbContext.Meetings
            .Include(m => m.Sessions)
            .ThenInclude(s => s.Participations)
            .FirstOrDefaultAsync(m => m.Id == query.MeetingId, cancellationToken);

        if (meeting is null)
            return Result.Failure<WaitingRoomStatusDto>(MeetingErrors.NotFound);

        var activeSession = meeting.Sessions.FirstOrDefault(s => s.EndedAt == null);
        var waitingParticipants = activeSession?.Participations
            .Where(p => p.Status == ParticipationStatus.InWaitingRoom && p.LeftAt == null)
            .OrderBy(p => p.JoinedAt)
            .Select(p => new WaitingRoomParticipantDto(
                p.Id,
                p.UserId,
                p.DisplayName,
                p.JoinedAt,
                p.ClientInfo))
            .ToList() ?? new List<WaitingRoomParticipantDto>();

        return Result.Success(new WaitingRoomStatusDto(
            meeting.Id,
            meeting.IsWaitingRoomEnabled,
            waitingParticipants
        ));
    }
}
