using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using FluentValidation;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.Polls.CreatePoll;

public sealed class CreatePollCommandHandler(
    IConferDbContext dbContext,
    IValidator<CreatePollCommand> validator,
    ISignalingNotifier signalingNotifier)
    : ICommandHandler<CreatePollCommand, Result<PollDto>>
{
    public async Task<Result<PollDto>> HandleAsync(
        CreatePollCommand command,
        CancellationToken cancellationToken = default)
    {
        var validation = await validator.ValidateAsync(command, cancellationToken);
        if (!validation.IsValid)
        {
            var firstError = validation.Errors.First();
            return Result.Failure<PollDto>(Error.Validation(firstError.PropertyName, firstError.ErrorMessage));
        }

        var meeting = await dbContext.Meetings
            .FirstOrDefaultAsync(m => m.Id == command.MeetingId, cancellationToken);

        if (meeting is null)
            return Result.Failure<PollDto>(MeetingErrors.NotFound);

        if (meeting.EndedAt.HasValue)
            return Result.Failure<PollDto>(MeetingErrors.Ended);

        var pollResult = Poll.Create(
            command.MeetingId,
            command.CreatorId,
            command.Question,
            command.Options,
            command.IsAnonymous,
            command.IsMultiChoice
        );

        if (pollResult.IsFailure)
            return Result.Failure<PollDto>(pollResult.Error);

        var poll = pollResult.Value;
        dbContext.Polls.Add(poll);
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
            0
        );

        await signalingNotifier.BroadcastPollCreatedAsync(poll.MeetingId, pollDto);

        return Result.Success(pollDto);
    }
}
