using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Domain.Enums;
using Confer.Domain.Meetings;
using Confer.Domain.Sessions;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.Join;

public sealed class JoinMeetingCommandHandler(
    IConferDbContext dbContext,
    ITokenProvider tokenProvider,
    IPresenceService presenceService,
    ISfuRoomManager sfuManager)
    : ICommandHandler<JoinMeetingCommand, Result<JoinMeetingResponse>>
{
    public async Task<Result<JoinMeetingResponse>> HandleAsync(
        JoinMeetingCommand command,
        CancellationToken cancellationToken = default)
    {
        var meeting = await dbContext.Meetings
            .Include(m => m.Sessions)
            .FirstOrDefaultAsync(m => m.Id == command.MeetingId, cancellationToken);

        if (meeting is null)
            return Result.Failure<JoinMeetingResponse>(MeetingErrors.NotFound);

        var currentActiveCount = await presenceService.GetActiveCountAsync(meeting.Id);
        var canJoin = meeting.CanJoin(currentActiveCount);
        if (canJoin.IsFailure)
            return Result.Failure<JoinMeetingResponse>(canJoin.Error);

        var session = meeting.Sessions.FirstOrDefault(s => s.EndedAt == null);
        if (session is null)
        {
            session = Session.Create(meeting.Id, "sfu-local-1");
            dbContext.Sessions.Add(session);
        }

        var role = meeting.OwnerId == command.UserId ? ParticipantRole.Host : ParticipantRole.Participant;
        var participationResult = session.AddParticipant(command.UserId, command.DisplayName, role, command.ClientInfo);
        if (participationResult.IsFailure)
            return Result.Failure<JoinMeetingResponse>(participationResult.Error);

        dbContext.Participations.Add(participationResult.Value);
        await dbContext.SaveChangesAsync(cancellationToken);

        // Ensure SFU room media session is instantiated
        await sfuManager.CreateOrGetRoomAsync(meeting.Id);

        var participantId = participationResult.Value.Id;
        var roomToken = tokenProvider.GenerateRoomToken(
            meeting.Id,
            command.UserId,
            participantId,
            command.DisplayName,
            role,
            session.SfuNodeId
        );

        var iceServers = new List<IceServerConfig>
        {
            new(["stun:stun.l.google.com:19302", "stun:stun1.l.google.com:19302"])
        };

        return Result.Success(new JoinMeetingResponse(
            meeting.Id,
            meeting.Title,
            participantId,
            role.ToString().ToLowerInvariant(),
            meeting.IsLocked,
            "/v1/signal",
            roomToken,
            iceServers
        ));
    }
}
