using Confer.Shared.Results;

namespace Confer.Domain.Webhooks;

public static class WebhookErrors
{
    public static readonly Error InvalidUrl = Error.Validation("Webhook.InvalidUrl", "The target URL is invalid. It must be an absolute HTTP or HTTPS URL.");
    public static readonly Error SubscriptionNotFound = Error.NotFound("Webhook.SubscriptionNotFound", "The webhook subscription was not found.");
    public static readonly Error DuplicateUrl = Error.Conflict("Webhook.DuplicateUrl", "A webhook subscription already exists for this URL.");
    public static readonly Error InvalidEvents = Error.Validation("Webhook.InvalidEvents", "At least one subscribed event is required.");
    public static readonly Error Unauthorized = Error.Unauthorized("Webhook.Unauthorized", "You are not authorized to manage this webhook subscription.");
}
