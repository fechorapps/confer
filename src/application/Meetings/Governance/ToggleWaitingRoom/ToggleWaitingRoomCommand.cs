using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Governance.ToggleWaitingRoom;

public record ToggleWaitingRoomCommand(
    Guid MeetingId,
    Guid ActorId,
    bool Enabled
) : ICommand<Result<bool>>;
