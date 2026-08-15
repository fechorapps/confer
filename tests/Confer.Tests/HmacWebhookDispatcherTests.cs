using System.Net;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using Confer.Application.Webhooks;
using Confer.Domain.Webhooks;
using Confer.Infrastructure.Persistence;
using Confer.Infrastructure.Webhooks;
using FluentAssertions;
using Microsoft.Data.Sqlite;
using Microsoft.EntityFrameworkCore;
using Moq;
using Moq.Protected;
using Xunit;

namespace Confer.Tests;

public class HmacWebhookDispatcherTests : IDisposable
{
    private readonly SqliteConnection _connection;
    private readonly ConferDbContext _dbContext;

    public HmacWebhookDispatcherTests()
    {
        _connection = new SqliteConnection("DataSource=:memory:");
        _connection.Open();

        var options = new DbContextOptionsBuilder<ConferDbContext>()
            .UseSqlite(_connection)
            .Options;

        _dbContext = new ConferDbContext(options);
        _dbContext.Database.EnsureCreated();
    }

    public void Dispose()
    {
        _dbContext.Dispose();
        _connection.Close();
        _connection.Dispose();
    }

    [Fact]
    public void ComputeSignature_ShouldGenerateDeterministicHmacSha256()
    {
        var secret = "secret_key_testing_12345";
        var payloadJson = "{\"event\":\"meeting.created\",\"data\":{\"id\":\"123\"}}";
        long timestamp = 1700000000;

        using var httpClient = new HttpClient();
        var dispatcher = new HmacWebhookDispatcher(httpClient, _dbContext);

        var signature = dispatcher.ComputeSignature(payloadJson, secret, timestamp);

        // Compute expected HMAC manually
        using var hmac = new HMACSHA256(Encoding.UTF8.GetBytes(secret));
        var expectedBytes = hmac.ComputeHash(Encoding.UTF8.GetBytes($"{timestamp}.{payloadJson}"));
        var expectedSignature = Convert.ToHexString(expectedBytes).ToLowerInvariant();

        signature.Should().Be(expectedSignature);
    }

    [Fact]
    public void VerifySignature_WithValidSignature_ShouldReturnTrue()
    {
        var secret = "my_webhook_secret_key";
        var payloadJson = "{\"event\":\"participant.joined\",\"timestamp\":\"2026-08-15T00:00:00Z\"}";
        var currentEpoch = DateTimeOffset.UtcNow.ToUnixTimeSeconds();

        using var hmac = new HMACSHA256(Encoding.UTF8.GetBytes(secret));
        var hash = hmac.ComputeHash(Encoding.UTF8.GetBytes($"{currentEpoch}.{payloadJson}"));
        var hexSignature = Convert.ToHexString(hash).ToLowerInvariant();
        var signatureHeader = $"t={currentEpoch},v1={hexSignature}";

        var isValid = HmacWebhookDispatcher.VerifySignature(payloadJson, secret, signatureHeader, toleranceSeconds: 300);

        isValid.Should().BeTrue();
    }

    [Fact]
    public void VerifySignature_WithTamperedPayload_ShouldReturnFalse()
    {
        var secret = "my_webhook_secret_key";
        var payloadJson = "{\"event\":\"participant.joined\",\"timestamp\":\"2026-08-15T00:00:00Z\"}";
        var tamperedPayload = "{\"event\":\"participant.joined\",\"timestamp\":\"2026-08-15T00:00:01Z\"}";
        var currentEpoch = DateTimeOffset.UtcNow.ToUnixTimeSeconds();

        using var hmac = new HMACSHA256(Encoding.UTF8.GetBytes(secret));
        var hash = hmac.ComputeHash(Encoding.UTF8.GetBytes($"{currentEpoch}.{payloadJson}"));
        var hexSignature = Convert.ToHexString(hash).ToLowerInvariant();
        var signatureHeader = $"t={currentEpoch},v1={hexSignature}";

        var isValid = HmacWebhookDispatcher.VerifySignature(tamperedPayload, secret, signatureHeader, toleranceSeconds: 300);

        isValid.Should().BeFalse();
    }

    [Fact]
    public void VerifySignature_WithExpiredTimestamp_ShouldReturnFalse()
    {
        var secret = "my_webhook_secret_key";
        var payloadJson = "{\"event\":\"meeting.created\"}";
        var oldEpoch = DateTimeOffset.UtcNow.ToUnixTimeSeconds() - 600; // 10 minutes ago

        using var hmac = new HMACSHA256(Encoding.UTF8.GetBytes(secret));
        var hash = hmac.ComputeHash(Encoding.UTF8.GetBytes($"{oldEpoch}.{payloadJson}"));
        var hexSignature = Convert.ToHexString(hash).ToLowerInvariant();
        var signatureHeader = $"t={oldEpoch},v1={hexSignature}";

        var isValid = HmacWebhookDispatcher.VerifySignature(payloadJson, secret, signatureHeader, toleranceSeconds: 300);

        isValid.Should().BeFalse();
    }

    [Fact]
    public async Task DispatchAsync_ShouldSendPostWithSignatureHeader()
    {
        var handlerMock = new Mock<HttpMessageHandler>();
        HttpRequestMessage? capturedRequest = null;
        string? capturedBody = null;

        handlerMock
            .Protected()
            .Setup<Task<HttpResponseMessage>>(
                "SendAsync",
                ItExpr.IsAny<HttpRequestMessage>(),
                ItExpr.IsAny<CancellationToken>()
            )
            .Callback<HttpRequestMessage, CancellationToken>((req, _) =>
            {
                capturedRequest = req;
                capturedBody = req.Content?.ReadAsStringAsync().GetAwaiter().GetResult();
            })
            .ReturnsAsync(new HttpResponseMessage(HttpStatusCode.OK)
            {
                Content = new StringContent("{\"status\":\"ok\"}")
            });

        var httpClient = new HttpClient(handlerMock.Object);
        var dispatcher = new HmacWebhookDispatcher(httpClient, _dbContext);

        var targetUrl = "https://endpoint.example.com/webhooks";
        var secret = "webhook_secret_key_abc";
        var payload = new WebhookPayloadDto("meeting.created", DateTime.UtcNow, new { MeetingId = Guid.NewGuid() });

        var result = await dispatcher.DispatchAsync(targetUrl, secret, payload);

        result.IsSuccess.Should().BeTrue();
        capturedRequest.Should().NotBeNull();
        capturedRequest!.Method.Should().Be(HttpMethod.Post);
        capturedRequest.RequestUri.Should().Be(new Uri(targetUrl));
        capturedRequest.Headers.Contains("X-Confer-Signature").Should().BeTrue();

        var sigHeader = capturedRequest.Headers.GetValues("X-Confer-Signature").FirstOrDefault();
        sigHeader.Should().NotBeNull();
        sigHeader!.Should().MatchRegex(@"^t=\d+,v1=[a-f0-9]{64}$");

        capturedBody.Should().NotBeNull();
        HmacWebhookDispatcher.VerifySignature(capturedBody!, secret, sigHeader!).Should().BeTrue();
    }

    [Fact]
    public async Task DispatchToSubscribersAsync_ShouldOnlyDispatchToMatchingSubscribers()
    {
        var userId = Guid.NewGuid();
        var sub1 = WebhookSubscription.Create(userId, "https://sub1.com/hook", new List<string> { "meeting.created" }).Value;
        var sub2 = WebhookSubscription.Create(userId, "https://sub2.com/hook", new List<string> { "recording.started" }).Value;
        var sub3 = WebhookSubscription.Create(userId, "https://sub3.com/hook", new List<string> { "*" }).Value;

        _dbContext.WebhookSubscriptions.AddRange(sub1, sub2, sub3);
        await _dbContext.SaveChangesAsync();

        var handlerMock = new Mock<HttpMessageHandler>();
        var sentUrls = new List<string>();

        handlerMock
            .Protected()
            .Setup<Task<HttpResponseMessage>>(
                "SendAsync",
                ItExpr.IsAny<HttpRequestMessage>(),
                ItExpr.IsAny<CancellationToken>()
            )
            .Callback<HttpRequestMessage, CancellationToken>((req, _) => sentUrls.Add(req.RequestUri!.ToString()))
            .ReturnsAsync(new HttpResponseMessage(HttpStatusCode.OK));

        var httpClient = new HttpClient(handlerMock.Object);
        var dispatcher = new HmacWebhookDispatcher(httpClient, _dbContext);

        var count = await dispatcher.DispatchToSubscribersAsync("meeting.created", new { Title = "Test Meeting" });

        count.Should().Be(2); // sub1 (meeting.created) and sub3 (*)
        sentUrls.Should().Contain("https://sub1.com/hook");
        sentUrls.Should().Contain("https://sub3.com/hook");
        sentUrls.Should().NotContain("https://sub2.com/hook");
    }
}
