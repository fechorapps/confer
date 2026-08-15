using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Webhooks.List;

public record ListWebhookSubscriptionsQuery(
    Guid? UserId = null
) : IQuery<Result<List<WebhookSubscriptionDto>>>;
