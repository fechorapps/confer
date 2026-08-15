using Confer.Application.DTOs;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Breakouts.CreateBreakouts;

public record CreateBreakoutsCommand(
    Guid MeetingId,
    Guid ActorId,
    int RoomCount,
    int MaxDurationMinutes = 15,
    List<string>? RoomNames = null,
    List<BreakoutAssignmentDto>? Assignments = null
) : ICommand<Result<List<BreakoutRoomDto>>>;
