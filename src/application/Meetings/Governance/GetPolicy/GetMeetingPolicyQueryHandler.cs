using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.Governance.GetPolicy;

public sealed class GetMeetingPolicyQueryHandler(
    IConferDbContext dbContext)
    : IQueryHandler<GetMeetingPolicyQuery, Result<MeetingPolicyResponse>>
{
    public async Task<Result<MeetingPolicyResponse>> HandleAsync(
        GetMeetingPolicyQuery query,
        CancellationToken cancellationToken = default)
    {
        var meeting = await dbContext.Meetings
            .FirstOrDefaultAsync(m => m.Id == query.MeetingId, cancellationToken);

        if (meeting is null)
            return Result.Failure<MeetingPolicyResponse>(MeetingErrors.NotFound);

        var policyDto = new MeetingPolicyDto(
            meeting.Policy.AllowScreenShare,
            meeting.Policy.AllowChat,
            meeting.Policy.AllowUnmuteSelf,
            meeting.Policy.MuteOnEntry,
            meeting.Policy.AllowRename
        );

        return Result.Success(new MeetingPolicyResponse(
            meeting.Id,
            meeting.IsWaitingRoomEnabled,
            meeting.IsWatermarkEnabled,
            policyDto
        ));
    }
}
