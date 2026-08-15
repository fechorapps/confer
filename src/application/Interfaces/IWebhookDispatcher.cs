using Confer.Application.Webhooks;
using Confer.Shared.Results;

namespace Confer.Application.Interfaces;

public interface IWebhookDispatcher
{
    string ComputeSignature(string payloadJson, string secret, long timestamp);

    Task<Result> DispatchAsync(
        string targetUrl,
        string secret,
        WebhookPayloadDto payload,
        CancellationToken cancellationToken = default);

    Task<int> DispatchToSubscribersAsync(
        string eventType,
        object? data,
        CancellationToken cancellationToken = default);
}
