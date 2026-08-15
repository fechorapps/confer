using Confer.Application.DTOs;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Governance.UpdatePolicy;

public record UpdateMeetingPolicyCommand(
    Guid MeetingId,
    Guid ActorId,
    MeetingPolicyDto Policy,
    bool? IsWatermarkEnabled = null,
    bool? IsWaitingRoomEnabled = null
) : ICommand<Result<MeetingPolicyDto>>;
