using Confer.Application.Webhooks;
using Confer.Application.Webhooks.Create;
using Confer.Application.Webhooks.Delete;
using Confer.Application.Webhooks.List;
using Confer.Shared.Api;
using Confer.Shared.Application.Interfaces;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Routing;

namespace Confer.Api.Endpoints.Webhooks;

public sealed class WebhooksEndpoint : BaseModule, IEndpoint
{
    public void MapEndpoint(IEndpointRouteBuilder app)
    {
        var group = app.MapGroup("/api/webhooks")
            .WithTags("Webhooks");

        group.MapPost("/", CreateWebhookSubscription).WithName("CreateWebhookSubscription");
        group.MapGet("/", ListWebhookSubscriptions).WithName("ListWebhookSubscriptions");
        group.MapDelete("/{id:guid}", DeleteWebhookSubscription).WithName("DeleteWebhookSubscription");
    }

    public record CreateWebhookSubscriptionRequest(
        Guid UserId,
        string TargetUrl,
        List<string> SubscribedEvents,
        string? Secret = null
    );

    private static async Task<IResult> CreateWebhookSubscription(
        [FromBody] CreateWebhookSubscriptionRequest request,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var command = new CreateWebhookSubscriptionCommand(
            request.UserId,
            request.TargetUrl,
            request.SubscribedEvents,
            request.Secret
        );

        var result = await dispatcher.SendAsync(command, ct);
        return CreatedOrFailure(result, result.IsSuccess ? $"/api/webhooks/{result.Value.Id}" : string.Empty);
    }

    private static async Task<IResult> ListWebhookSubscriptions(
        [FromQuery] Guid? userId,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var query = new ListWebhookSubscriptionsQuery(userId);
        var result = await dispatcher.QueryAsync(query, ct);
        return OkOrFailure(result);
    }

    private static async Task<IResult> DeleteWebhookSubscription(
        [FromRoute] Guid id,
        [FromQuery] Guid? userId,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var command = new DeleteWebhookSubscriptionCommand(id, userId);
        var result = await dispatcher.SendAsync(command, ct);
        return NoContentOrFailure(result);
    }
}
