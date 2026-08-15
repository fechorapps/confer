using System.Net.Http.Headers;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using Confer.Application.Interfaces;
using Confer.Application.Webhooks;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Logging;

namespace Confer.Infrastructure.Webhooks;

public sealed class HmacWebhookDispatcher : IWebhookDispatcher
{
    private readonly HttpClient _httpClient;
    private readonly IConferDbContext _dbContext;
    private readonly ILogger<HmacWebhookDispatcher>? _logger;
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNamingPolicy = JsonNamingPolicy.CamelCase,
        WriteIndented = false
    };

    public HmacWebhookDispatcher(
        HttpClient httpClient,
        IConferDbContext dbContext,
        ILogger<HmacWebhookDispatcher>? logger = null)
    {
        _httpClient = httpClient;
        _dbContext = dbContext;
        _logger = logger;
    }

    public string ComputeSignature(string payloadJson, string secret, long timestamp)
    {
        var signedPayload = $"{timestamp}.{payloadJson}";
        using var hmac = new HMACSHA256(Encoding.UTF8.GetBytes(secret));
        var hash = hmac.ComputeHash(Encoding.UTF8.GetBytes(signedPayload));
        return Convert.ToHexString(hash).ToLowerInvariant();
    }

    public static bool VerifySignature(string payloadJson, string secret, string signatureHeader, long toleranceSeconds = 300)
    {
        if (string.IsNullOrWhiteSpace(signatureHeader) || string.IsNullOrWhiteSpace(secret))
            return false;

        var parts = signatureHeader.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries);
        var tPart = parts.FirstOrDefault(p => p.StartsWith("t="));
        var v1Part = parts.FirstOrDefault(p => p.StartsWith("v1="));

        if (tPart is null || v1Part is null)
            return false;

        if (!long.TryParse(tPart[2..], out var timestamp))
            return false;

        var expectedSignature = v1Part[3..];

        if (toleranceSeconds > 0)
        {
            var currentEpoch = DateTimeOffset.UtcNow.ToUnixTimeSeconds();
            if (Math.Abs(currentEpoch - timestamp) > toleranceSeconds)
                return false;
        }

        var signedPayload = $"{timestamp}.{payloadJson}";
        using var hmac = new HMACSHA256(Encoding.UTF8.GetBytes(secret));
        var calculatedHash = hmac.ComputeHash(Encoding.UTF8.GetBytes(signedPayload));
        var calculatedSignature = Convert.ToHexString(calculatedHash).ToLowerInvariant();

        return CryptographicOperations.FixedTimeEquals(
            Encoding.UTF8.GetBytes(calculatedSignature),
            Encoding.UTF8.GetBytes(expectedSignature));
    }

    public async Task<Result> DispatchAsync(
        string targetUrl,
        string secret,
        WebhookPayloadDto payload,
        CancellationToken cancellationToken = default)
    {
        try
        {
            var payloadJson = JsonSerializer.Serialize(payload, JsonOptions);
            var timestamp = DateTimeOffset.UtcNow.ToUnixTimeSeconds();
            var signature = ComputeSignature(payloadJson, secret, timestamp);

            using var request = new HttpRequestMessage(HttpMethod.Post, targetUrl);
            request.Content = new StringContent(payloadJson, Encoding.UTF8, "application/json");
            request.Headers.Add("X-Confer-Signature", $"t={timestamp},v1={signature}");
            request.Headers.UserAgent.Add(new ProductInfoHeaderValue("Confer-Webhooks", "1.0"));

            var response = await _httpClient.SendAsync(request, cancellationToken);
            if (!response.IsSuccessStatusCode)
            {
                var errorBody = await response.Content.ReadAsStringAsync(cancellationToken);
                _logger?.LogWarning("Webhook dispatch to {TargetUrl} failed with status {StatusCode}: {ErrorBody}",
                    targetUrl, response.StatusCode, errorBody);
                return Result.Failure(Error.Failure(
                    "Webhook.DeliveryFailed",
                    $"Webhook delivery to {targetUrl} failed with status code {(int)response.StatusCode}."));
            }

            return Result.Success();
        }
        catch (Exception ex)
        {
            _logger?.LogError(ex, "Exception while dispatching webhook to {TargetUrl}", targetUrl);
            return Result.Failure(Error.Failure("Webhook.DeliveryError", ex.Message));
        }
    }

    public async Task<int> DispatchToSubscribersAsync(
        string eventType,
        object? data,
        CancellationToken cancellationToken = default)
    {
        var subscriptions = await _dbContext.WebhookSubscriptions
            .Where(s => s.IsActive)
            .ToListAsync(cancellationToken);

        var matching = subscriptions
            .Where(s => s.MatchesEvent(eventType))
            .ToList();

        if (matching.Count == 0)
            return 0;

        var payload = new WebhookPayloadDto(
            eventType,
            DateTime.UtcNow,
            data,
            Guid.NewGuid().ToString()
        );

        int deliveredCount = 0;
        foreach (var sub in matching)
        {
            var result = await DispatchAsync(sub.TargetUrl, sub.Secret, payload, cancellationToken);
            if (result.IsSuccess)
            {
                deliveredCount++;
            }
        }

        return deliveredCount;
    }
}
