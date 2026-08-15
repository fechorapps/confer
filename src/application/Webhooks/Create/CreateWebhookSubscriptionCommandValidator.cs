using FluentValidation;

namespace Confer.Application.Webhooks.Create;

public sealed class CreateWebhookSubscriptionCommandValidator : AbstractValidator<CreateWebhookSubscriptionCommand>
{
    public CreateWebhookSubscriptionCommandValidator()
    {
        RuleFor(x => x.UserId)
            .NotEmpty()
            .WithMessage("User ID is required.");

        RuleFor(x => x.TargetUrl)
            .NotEmpty()
            .WithMessage("Target URL cannot be empty.")
            .Must(url => Uri.TryCreate(url, UriKind.Absolute, out var uri) &&
                         (uri.Scheme == Uri.UriSchemeHttp || uri.Scheme == Uri.UriSchemeHttps))
            .WithMessage("Target URL must be a valid HTTP or HTTPS URL.");

        RuleFor(x => x.SubscribedEvents)
            .NotEmpty()
            .WithMessage("At least one subscribed event is required.")
            .Must(events => events != null && events.Any(e => !string.IsNullOrWhiteSpace(e)))
            .WithMessage("At least one non-empty subscribed event is required.");
    }
}
