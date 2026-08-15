namespace Confer.Application.Webhooks;

public record WebhookPayloadDto(
    string Event,
    DateTime Timestamp,
    object? Data,
    string? Id = null
);
