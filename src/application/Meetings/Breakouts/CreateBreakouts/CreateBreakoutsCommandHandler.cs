using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using FluentValidation;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.Breakouts.CreateBreakouts;

public sealed class CreateBreakoutsCommandHandler(
    IConferDbContext dbContext,
    IValidator<CreateBreakoutsCommand> validator,
    ISignalingNotifier signalingNotifier)
    : ICommandHandler<CreateBreakoutsCommand, Result<List<BreakoutRoomDto>>>
{
    public async Task<Result<List<BreakoutRoomDto>>> HandleAsync(
        CreateBreakoutsCommand command,
        CancellationToken cancellationToken = default)
    {
        var validation = await validator.ValidateAsync(command, cancellationToken);
        if (!validation.IsValid)
        {
            var firstError = validation.Errors.First();
            return Result.Failure<List<BreakoutRoomDto>>(Error.Validation(firstError.PropertyName, firstError.ErrorMessage));
        }

        var meeting = await dbContext.Meetings
            .FirstOrDefaultAsync(m => m.Id == command.MeetingId, cancellationToken);

        if (meeting is null)
            return Result.Failure<List<BreakoutRoomDto>>(MeetingErrors.NotFound);

        if (meeting.EndedAt.HasValue)
            return Result.Failure<List<BreakoutRoomDto>>(MeetingErrors.Ended);

        if (meeting.OwnerId != command.ActorId)
            return Result.Failure<List<BreakoutRoomDto>>(BreakoutErrors.Unauthorized);

        var createdRooms = new List<BreakoutRoom>();
        for (int i = 0; i < command.RoomCount; i++)
        {
            var name = (command.RoomNames != null && i < command.RoomNames.Count && !string.IsNullOrWhiteSpace(command.RoomNames[i]))
                ? command.RoomNames[i].Trim()
                : $"Breakout Room {i + 1}";

            var roomResult = BreakoutRoom.Create(meeting.Id, name, i + 1, command.MaxDurationMinutes);
            if (roomResult.IsFailure)
                return Result.Failure<List<BreakoutRoomDto>>(roomResult.Error);

            createdRooms.Add(roomResult.Value);
            dbContext.BreakoutRooms.Add(roomResult.Value);
        }

        await dbContext.SaveChangesAsync(cancellationToken);

        var roomDtos = createdRooms.Select(r => new BreakoutRoomDto(
            r.Id,
            r.MeetingId,
            r.SessionId,
            r.Name,
            r.Index,
            r.CreatedAt,
            r.EndsAt,
            r.MaxDurationMinutes
        )).ToList();

        // Broadcast assignments / invitations
        if (command.Assignments != null && command.Assignments.Count > 0)
        {
            foreach (var assignment in command.Assignments)
            {
                var targetRoom = roomDtos.FirstOrDefault(r => r.Id == assignment.BreakoutRoomId)
                    ?? roomDtos.FirstOrDefault(r => string.Equals(r.Name, assignment.RoomName, StringComparison.OrdinalIgnoreCase))
                    ?? roomDtos.First();

                await signalingNotifier.BroadcastBreakoutInviteAsync(
                    meeting.Id,
                    assignment.ParticipantId,
                    new BreakoutInviteDto(targetRoom.Id, targetRoom.Name, targetRoom.SessionId)
                );
            }
        }
        else
        {
            foreach (var room in roomDtos)
            {
                await signalingNotifier.BroadcastBreakoutInviteAsync(
                    meeting.Id,
                    Guid.Empty,
                    new BreakoutInviteDto(room.Id, room.Name, room.SessionId)
                );
            }
        }

        return Result.Success(roomDtos);
    }
}
