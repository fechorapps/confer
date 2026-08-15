using FluentValidation;

namespace Confer.Application.Meetings.Breakouts.CreateBreakouts;

public sealed class CreateBreakoutsCommandValidator : AbstractValidator<CreateBreakoutsCommand>
{
    public CreateBreakoutsCommandValidator()
    {
        RuleFor(x => x.MeetingId).NotEmpty().WithMessage("Meeting ID cannot be empty.");
        RuleFor(x => x.ActorId).NotEmpty().WithMessage("Actor ID cannot be empty.");
        RuleFor(x => x.RoomCount).InclusiveBetween(1, 20).WithMessage("Room count must be between 1 and 20.");
        RuleFor(x => x.MaxDurationMinutes).InclusiveBetween(1, 180).WithMessage("Max duration must be between 1 and 180 minutes.");
    }
}
