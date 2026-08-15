using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.Polls.ClosePoll;

public sealed class ClosePollCommandHandler(
    IConferDbContext dbContext,
    ISignalingNotifier signalingNotifier)
    : ICommandHandler<ClosePollCommand, Result<PollDto>>
{
    public async Task<Result<PollDto>> HandleAsync(
        ClosePollCommand command,
        CancellationToken cancellationToken = default)
    {
        var meeting = await dbContext.Meetings
            .FirstOrDefaultAsync(m => m.Id == command.MeetingId, cancellationToken);

        if (meeting is null)
            return Result.Failure<PollDto>(MeetingErrors.NotFound);

        var poll = await dbContext.Polls
            .Include(p => p.Options)
            .Include(p => p.Votes)
            .FirstOrDefaultAsync(p => p.Id == command.PollId && p.MeetingId == command.MeetingId, cancellationToken);

        if (poll is null)
            return Result.Failure<PollDto>(PollErrors.NotFound);

        var closeResult = poll.Close(command.ActorId, meeting.OwnerId);
        if (closeResult.IsFailure)
            return Result.Failure<PollDto>(closeResult.Error);

        await dbContext.SaveChangesAsync(cancellationToken);

        var pollDto = new PollDto(
            poll.Id,
            poll.MeetingId,
            poll.CreatorId,
            poll.Question,
            poll.IsAnonymous,
            poll.IsMultiChoice,
            poll.IsActive,
            poll.CreatedAt,
            poll.ClosedAt,
            poll.Options.OrderBy(o => o.Index).Select(o => new PollOptionDto(
                o.Id,
                o.PollId,
                o.Text,
                o.Index,
                o.VoteCount
            )).ToList(),
            poll.Votes.Count
        );

        await signalingNotifier.BroadcastPollClosedAsync(poll.MeetingId, poll.Id);

        return Result.Success(pollDto);
    }
}
