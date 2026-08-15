using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Governance.RejectParticipant;

public record RejectParticipantCommand(
    Guid MeetingId,
    Guid ActorId,
    Guid ParticipantId
) : ICommand<Result>;
