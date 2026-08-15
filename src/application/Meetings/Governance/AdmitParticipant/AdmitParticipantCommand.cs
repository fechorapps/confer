using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Governance.AdmitParticipant;

public record AdmitParticipantCommand(
    Guid MeetingId,
    Guid ActorId,
    Guid ParticipantId
) : ICommand<Result>;
