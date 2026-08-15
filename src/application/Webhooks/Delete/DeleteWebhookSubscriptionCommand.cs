using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Webhooks.Delete;

public record DeleteWebhookSubscriptionCommand(
    Guid SubscriptionId,
    Guid? UserId = null
) : ICommand<Result>;
