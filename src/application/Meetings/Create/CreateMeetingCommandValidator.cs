using FluentValidation;

namespace Confer.Application.Meetings.Create;

public sealed class CreateMeetingCommandValidator : AbstractValidator<CreateMeetingCommand>
{
    public CreateMeetingCommandValidator()
    {
        RuleFor(x => x.Title)
            .NotEmpty().WithMessage("Meeting title is required.")
            .MaximumLength(200).WithMessage("Meeting title must not exceed 200 characters.");

        RuleFor(x => x.OwnerId)
            .NotEmpty().WithMessage("Owner ID is required.");

        RuleFor(x => x.MaxParticipants)
            .InclusiveBetween(2, 200).WithMessage("Max participants must be between 2 and 200.");
    }
}
