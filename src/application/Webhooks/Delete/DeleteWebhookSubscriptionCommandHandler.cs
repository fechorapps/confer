using Confer.Application.Interfaces;
using Confer.Domain.Webhooks;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Webhooks.Delete;

public sealed class DeleteWebhookSubscriptionCommandHandler(IConferDbContext dbContext)
    : ICommandHandler<DeleteWebhookSubscriptionCommand, Result>
{
    public async Task<Result> HandleAsync(
        DeleteWebhookSubscriptionCommand command,
        CancellationToken cancellationToken = default)
    {
        var subscription = await dbContext.WebhookSubscriptions
            .FirstOrDefaultAsync(w => w.Id == command.SubscriptionId, cancellationToken);

        if (subscription is null)
        {
            return Result.Failure(WebhookErrors.SubscriptionNotFound);
        }

        if (command.UserId.HasValue && command.UserId.Value != Guid.Empty && subscription.UserId != command.UserId.Value)
        {
            return Result.Failure(WebhookErrors.Unauthorized);
        }

        dbContext.WebhookSubscriptions.Remove(subscription);
        await dbContext.SaveChangesAsync(cancellationToken);

        return Result.Success();
    }
}
