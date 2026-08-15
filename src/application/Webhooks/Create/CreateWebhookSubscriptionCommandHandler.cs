using Confer.Application.Interfaces;
using Confer.Domain.Webhooks;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using FluentValidation;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Webhooks.Create;

public sealed class CreateWebhookSubscriptionCommandHandler(
    IConferDbContext dbContext,
    IValidator<CreateWebhookSubscriptionCommand> validator)
    : ICommandHandler<CreateWebhookSubscriptionCommand, Result<WebhookSubscriptionDto>>
{
    public async Task<Result<WebhookSubscriptionDto>> HandleAsync(
        CreateWebhookSubscriptionCommand command,
        CancellationToken cancellationToken = default)
    {
        var validation = await validator.ValidateAsync(command, cancellationToken);
        if (!validation.IsValid)
        {
            var firstError = validation.Errors.First();
            return Result.Failure<WebhookSubscriptionDto>(Error.Validation(firstError.PropertyName, firstError.ErrorMessage));
        }

        var normalizedUrl = command.TargetUrl.Trim();
        var duplicateExists = await dbContext.WebhookSubscriptions
            .AnyAsync(w => w.UserId == command.UserId && w.TargetUrl == normalizedUrl, cancellationToken);

        if (duplicateExists)
        {
            return Result.Failure<WebhookSubscriptionDto>(WebhookErrors.DuplicateUrl);
        }

        var webhookResult = WebhookSubscription.Create(
            command.UserId,
            normalizedUrl,
            command.SubscribedEvents,
            command.Secret);

        if (webhookResult.IsFailure)
        {
            return Result.Failure<WebhookSubscriptionDto>(webhookResult.Error);
        }

        var webhook = webhookResult.Value;
        dbContext.WebhookSubscriptions.Add(webhook);
        await dbContext.SaveChangesAsync(cancellationToken);

        return Result.Success(new WebhookSubscriptionDto(
            webhook.Id,
            webhook.UserId,
            webhook.TargetUrl,
            webhook.SubscribedEvents,
            webhook.IsActive,
            webhook.CreatedAt,
            webhook.Secret
        ));
    }
}
