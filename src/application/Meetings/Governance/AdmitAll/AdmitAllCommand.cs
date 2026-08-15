using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Governance.AdmitAll;

public record AdmitAllCommand(
    Guid MeetingId,
    Guid ActorId
) : ICommand<Result<int>>;
