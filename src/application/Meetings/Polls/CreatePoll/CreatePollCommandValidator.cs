using FluentValidation;

namespace Confer.Application.Meetings.Polls.CreatePoll;

public sealed class CreatePollCommandValidator : AbstractValidator<CreatePollCommand>
{
    public CreatePollCommandValidator()
    {
        RuleFor(x => x.MeetingId)
            .NotEmpty().WithMessage("Meeting ID cannot be empty.");

        RuleFor(x => x.CreatorId)
            .NotEmpty().WithMessage("Creator ID cannot be empty.");

        RuleFor(x => x.Question)
            .NotEmpty().WithMessage("Poll question cannot be empty.")
            .MaximumLength(500).WithMessage("Poll question cannot exceed 500 characters.");

        RuleFor(x => x.Options)
            .NotNull().WithMessage("Poll options cannot be null.")
            .Must(o => o != null && o.Where(s => !string.IsNullOrWhiteSpace(s)).Distinct(StringComparer.OrdinalIgnoreCase).Count() >= 2)
            .WithMessage("A poll must have at least 2 distinct non-empty options.")
            .Must(o => o == null || o.Count <= 10)
            .WithMessage("A poll cannot have more than 10 options.");
    }
}
