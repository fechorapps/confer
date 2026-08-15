using FluentValidation;

namespace Confer.Application.Meetings.Summary;

public sealed class GenerateMeetingSummaryCommandValidator : AbstractValidator<GenerateMeetingSummaryCommand>
{
    public GenerateMeetingSummaryCommandValidator()
    {
        RuleFor(x => x.MeetingId)
            .NotEmpty().WithMessage("Meeting ID cannot be empty.");

        RuleFor(x => x.ActorId)
            .NotEmpty().WithMessage("Actor ID cannot be empty.");
    }
}
