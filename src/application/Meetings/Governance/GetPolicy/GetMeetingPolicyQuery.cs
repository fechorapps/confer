using Confer.Application.DTOs;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Governance.GetPolicy;

public record GetMeetingPolicyQuery(
    Guid MeetingId
) : IQuery<Result<MeetingPolicyResponse>>;
