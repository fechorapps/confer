using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Webhooks.Create;

public record CreateWebhookSubscriptionCommand(
    Guid UserId,
    string TargetUrl,
    List<string> SubscribedEvents,
    string? Secret = null
) : ICommand<Result<WebhookSubscriptionDto>>;
