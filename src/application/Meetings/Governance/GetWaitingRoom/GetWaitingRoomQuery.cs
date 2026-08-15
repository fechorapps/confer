using Confer.Application.DTOs;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Governance.GetWaitingRoom;

public record GetWaitingRoomQuery(
    Guid MeetingId,
    Guid? ActorId = null
) : IQuery<Result<WaitingRoomStatusDto>>;
