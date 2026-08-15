using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using FluentValidation;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.Polls.SubmitPollVote;

public sealed class SubmitPollVoteCommandHandler(
    IConferDbContext dbContext,
    IValidator<SubmitPollVoteCommand> validator,
    ISignalingNotifier signalingNotifier)
    : ICommandHandler<SubmitPollVoteCommand, Result<PollDto>>
{
    public async Task<Result<PollDto>> HandleAsync(
        SubmitPollVoteCommand command,
        CancellationToken cancellationToken = default)
    {
        var validation = await validator.ValidateAsync(command, cancellationToken);
        if (!validation.IsValid)
        {
            var firstError = validation.Errors.First();
            return Result.Failure<PollDto>(Error.Validation(firstError.PropertyName, firstError.ErrorMessage));
        }

        var poll = await dbContext.Polls
            .Include(p => p.Options)
            .Include(p => p.Votes)
            .FirstOrDefaultAsync(p => p.Id == command.PollId && p.MeetingId == command.MeetingId, cancellationToken);

        if (poll is null)
            return Result.Failure<PollDto>(PollErrors.NotFound);

        var voteResult = poll.Vote(command.VoterId, command.OptionIds);
        if (voteResult.IsFailure)
            return Result.Failure<PollDto>(voteResult.Error);

        dbContext.PollVotes.AddRange(voteResult.Value);
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

        await signalingNotifier.BroadcastPollUpdatedAsync(poll.MeetingId, pollDto);

        return Result.Success(pollDto);
    }
}
