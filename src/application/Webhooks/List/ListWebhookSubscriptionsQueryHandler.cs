using Confer.Application.Interfaces;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Webhooks.List;

public sealed class ListWebhookSubscriptionsQueryHandler(IConferDbContext dbContext)
    : IQueryHandler<ListWebhookSubscriptionsQuery, Result<List<WebhookSubscriptionDto>>>
{
    public async Task<Result<List<WebhookSubscriptionDto>>> HandleAsync(
        ListWebhookSubscriptionsQuery query,
        CancellationToken cancellationToken = default)
    {
        var queryable = dbContext.WebhookSubscriptions.AsQueryable();

        if (query.UserId.HasValue && query.UserId.Value != Guid.Empty)
        {
            queryable = queryable.Where(w => w.UserId == query.UserId.Value);
        }

        var list = await queryable
            .OrderByDescending(w => w.CreatedAt)
            .Select(w => new WebhookSubscriptionDto(
                w.Id,
                w.UserId,
                w.TargetUrl,
                w.SubscribedEvents,
                w.IsActive,
                w.CreatedAt,
                w.Secret
            ))
            .ToListAsync(cancellationToken);

        return Result.Success(list);
    }
}
