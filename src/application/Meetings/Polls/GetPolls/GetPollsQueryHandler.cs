using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.Polls.GetPolls;

public sealed class GetPollsQueryHandler(IConferDbContext dbContext)
    : IQueryHandler<GetPollsQuery, Result<List<PollDto>>>
{
    public async Task<Result<List<PollDto>>> HandleAsync(
        GetPollsQuery query,
        CancellationToken cancellationToken = default)
    {
        var meetingExists = await dbContext.Meetings
            .AnyAsync(m => m.Id == query.MeetingId, cancellationToken);

        if (!meetingExists)
            return Result.Failure<List<PollDto>>(MeetingErrors.NotFound);

        var polls = await dbContext.Polls
            .Include(p => p.Options)
            .Include(p => p.Votes)
            .Where(p => p.MeetingId == query.MeetingId)
            .OrderBy(p => p.CreatedAt)
            .ToListAsync(cancellationToken);

        var dtoList = polls.Select(p => new PollDto(
            p.Id,
            p.MeetingId,
            p.CreatorId,
            p.Question,
            p.IsAnonymous,
            p.IsMultiChoice,
            p.IsActive,
            p.CreatedAt,
            p.ClosedAt,
            p.Options.OrderBy(o => o.Index).Select(o => new PollOptionDto(
                o.Id,
                o.PollId,
                o.Text,
                o.Index,
                o.VoteCount
            )).ToList(),
            p.Votes.Count
        )).ToList();

        return Result.Success(dtoList);
    }
}
