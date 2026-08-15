namespace Confer.Application.Webhooks;

public record WebhookSubscriptionDto(
    Guid Id,
    Guid UserId,
    string TargetUrl,
    List<string> SubscribedEvents,
    bool IsActive,
    DateTime CreatedAt,
    string? Secret = null
);
