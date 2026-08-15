using Confer.Application.Interfaces;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.Summary;

public sealed class GetMeetingSummaryQueryHandler(IConferDbContext dbContext)
    : IQueryHandler<GetMeetingSummaryQuery, Result<MeetingSummaryDto>>
{
    public async Task<Result<MeetingSummaryDto>> HandleAsync(
        GetMeetingSummaryQuery query,
        CancellationToken cancellationToken = default)
    {
        var meetingExists = await dbContext.Meetings.AnyAsync(m => m.Id == query.MeetingId, cancellationToken);
        if (!meetingExists)
            return Result.Failure<MeetingSummaryDto>(MeetingErrors.NotFound);

        var summary = await dbContext.MeetingSummaries
            .FirstOrDefaultAsync(s => s.MeetingId == query.MeetingId, cancellationToken);

        if (summary is null)
            return Result.Failure<MeetingSummaryDto>(MeetingErrors.SummaryNotFound);

        return Result.Success(new MeetingSummaryDto(
            summary.Id,
            summary.MeetingId,
            summary.Overview,
            summary.KeyDecisions,
            summary.ActionItems.Select(a => new ActionItemDto(a.Title, a.Assignee, a.Status)).ToList(),
            summary.GeneratedAt,
            summary.DurationMinutes,
            summary.ParticipantCount
        ));
    }
}
