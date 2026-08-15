using FluentValidation;

namespace Confer.Application.Meetings.Polls.SubmitPollVote;

public sealed class SubmitPollVoteCommandValidator : AbstractValidator<SubmitPollVoteCommand>
{
    public SubmitPollVoteCommandValidator()
    {
        RuleFor(x => x.MeetingId).NotEmpty().WithMessage("Meeting ID cannot be empty.");
        RuleFor(x => x.PollId).NotEmpty().WithMessage("Poll ID cannot be empty.");
        RuleFor(x => x.VoterId).NotEmpty().WithMessage("Voter ID cannot be empty.");
        RuleFor(x => x.OptionIds).NotNull().NotEmpty().WithMessage("At least one option must be selected.");
    }
}
