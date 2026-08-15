using System.Security.Cryptography;
using Confer.Shared.Results;

namespace Confer.Domain.Webhooks;

public sealed class WebhookSubscription
{
    public Guid Id { get; private set; }
    public Guid UserId { get; private set; }
    public string TargetUrl { get; private set; } = string.Empty;
    public string Secret { get; private set; } = string.Empty;
    public List<string> SubscribedEvents { get; private set; } = new();
    public bool IsActive { get; private set; }
    public DateTime CreatedAt { get; private set; } = DateTime.UtcNow;

    private WebhookSubscription() { }

    public static Result<WebhookSubscription> Create(
        Guid userId,
        string targetUrl,
        List<string> subscribedEvents,
        string? secret = null,
        Guid? id = null)
    {
        if (userId == Guid.Empty)
            return Result.Failure<WebhookSubscription>(Error.Validation("Webhook.UserIdEmpty", "User ID cannot be empty."));

        if (string.IsNullOrWhiteSpace(targetUrl))
            return Result.Failure<WebhookSubscription>(WebhookErrors.InvalidUrl);

        if (!Uri.TryCreate(targetUrl, UriKind.Absolute, out var uri) ||
            (uri.Scheme != Uri.UriSchemeHttp && uri.Scheme != Uri.UriSchemeHttps))
        {
            return Result.Failure<WebhookSubscription>(WebhookErrors.InvalidUrl);
        }

        if (subscribedEvents is null || subscribedEvents.Count == 0)
            return Result.Failure<WebhookSubscription>(WebhookErrors.InvalidEvents);

        var cleanedEvents = subscribedEvents
            .Where(e => !string.IsNullOrWhiteSpace(e))
            .Select(e => e.Trim())
            .Distinct(StringComparer.OrdinalIgnoreCase)
            .ToList();

        if (cleanedEvents.Count == 0)
            return Result.Failure<WebhookSubscription>(WebhookErrors.InvalidEvents);

        var webhookSecret = !string.IsNullOrWhiteSpace(secret)
            ? secret.Trim()
            : $"whsec_{Convert.ToHexString(RandomNumberGenerator.GetBytes(24)).ToLowerInvariant()}";

        return Result.Success(new WebhookSubscription
        {
            Id = id ?? Guid.NewGuid(),
            UserId = userId,
            TargetUrl = targetUrl.Trim(),
            Secret = webhookSecret,
            SubscribedEvents = cleanedEvents,
            IsActive = true,
            CreatedAt = DateTime.UtcNow
        });
    }

    public void Deactivate()
    {
        IsActive = false;
    }

    public void Activate()
    {
        IsActive = true;
    }

    public bool MatchesEvent(string eventName)
    {
        if (!IsActive || string.IsNullOrWhiteSpace(eventName))
            return false;

        return SubscribedEvents.Contains("*") ||
               SubscribedEvents.Contains(eventName.Trim(), StringComparer.OrdinalIgnoreCase);
    }
}
