using Confer.Application.Webhooks.Create;
using Confer.Application.Webhooks.Delete;
using Confer.Application.Webhooks.List;
using Confer.Domain.Webhooks;
using Confer.Infrastructure.Persistence;
using FluentAssertions;
using Microsoft.Data.Sqlite;
using Microsoft.EntityFrameworkCore;
using Xunit;

namespace Confer.Tests;

public class WebhookCqrsTests : IDisposable
{
    private readonly SqliteConnection _connection;
    private readonly ConferDbContext _dbContext;
    private readonly CreateWebhookSubscriptionCommandValidator _createValidator;

    public WebhookCqrsTests()
    {
        _connection = new SqliteConnection("DataSource=:memory:");
        _connection.Open();

        var options = new DbContextOptionsBuilder<ConferDbContext>()
            .UseSqlite(_connection)
            .Options;

        _dbContext = new ConferDbContext(options);
        _dbContext.Database.EnsureCreated();

        _createValidator = new CreateWebhookSubscriptionCommandValidator();
    }

    public void Dispose()
    {
        _dbContext.Dispose();
        _connection.Close();
        _connection.Dispose();
    }

    [Fact]
    public async Task CreateWebhookSubscription_WhenValid_ShouldPersistAndReturnDto()
    {
        var handler = new CreateWebhookSubscriptionCommandHandler(_dbContext, _createValidator);
        var userId = Guid.NewGuid();
        var command = new CreateWebhookSubscriptionCommand(
            userId,
            "https://hooks.slack.com/services/T00/B00/XXXX",
            new List<string> { "meeting.created", "recording.stopped" },
            "custom_secret_123"
        );

        var result = await handler.HandleAsync(command);

        result.IsSuccess.Should().BeTrue();
        result.Value.Id.Should().NotBeEmpty();
        result.Value.UserId.Should().Be(userId);
        result.Value.TargetUrl.Should().Be("https://hooks.slack.com/services/T00/B00/XXXX");
        result.Value.SubscribedEvents.Should().Contain("meeting.created");
        result.Value.Secret.Should().Be("custom_secret_123");

        var persisted = await _dbContext.WebhookSubscriptions.FirstOrDefaultAsync(w => w.Id == result.Value.Id);
        persisted.Should().NotBeNull();
        persisted!.TargetUrl.Should().Be("https://hooks.slack.com/services/T00/B00/XXXX");
    }

    [Fact]
    public async Task CreateWebhookSubscription_WhenDuplicateUrlForSameUser_ShouldFail()
    {
        var handler = new CreateWebhookSubscriptionCommandHandler(_dbContext, _createValidator);
        var userId = Guid.NewGuid();
        var url = "https://example.com/duplicate-test";

        var command = new CreateWebhookSubscriptionCommand(
            userId,
            url,
            new List<string> { "meeting.created" }
        );

        var firstResult = await handler.HandleAsync(command);
        firstResult.IsSuccess.Should().BeTrue();

        var secondResult = await handler.HandleAsync(command);
        secondResult.IsFailure.Should().BeTrue();
        secondResult.Error.Should().Be(WebhookErrors.DuplicateUrl);
    }

    [Fact]
    public async Task CreateWebhookSubscription_WhenInvalidUrl_ShouldFailValidation()
    {
        var handler = new CreateWebhookSubscriptionCommandHandler(_dbContext, _createValidator);
        var command = new CreateWebhookSubscriptionCommand(
            Guid.NewGuid(),
            "invalid-url-schema",
            new List<string> { "meeting.created" }
        );

        var result = await handler.HandleAsync(command);

        result.IsFailure.Should().BeTrue();
        result.Error.Code.Should().Be("TargetUrl");
    }

    [Fact]
    public async Task ListWebhookSubscriptions_ShouldFilterByUserId()
    {
        var user1 = Guid.NewGuid();
        var user2 = Guid.NewGuid();

        _dbContext.WebhookSubscriptions.AddRange(
            WebhookSubscription.Create(user1, "https://user1.com/hook1", new List<string> { "meeting.created" }).Value,
            WebhookSubscription.Create(user1, "https://user1.com/hook2", new List<string> { "meeting.ended" }).Value,
            WebhookSubscription.Create(user2, "https://user2.com/hook3", new List<string> { "*" }).Value
        );
        await _dbContext.SaveChangesAsync();

        var handler = new ListWebhookSubscriptionsQueryHandler(_dbContext);

        // Filter by user1
        var user1Result = await handler.HandleAsync(new ListWebhookSubscriptionsQuery(user1));
        user1Result.IsSuccess.Should().BeTrue();
        user1Result.Value.Should().HaveCount(2);
        user1Result.Value.Select(x => x.TargetUrl).Should().Contain("https://user1.com/hook1");

        // Filter by user2
        var user2Result = await handler.HandleAsync(new ListWebhookSubscriptionsQuery(user2));
        user2Result.IsSuccess.Should().BeTrue();
        user2Result.Value.Should().HaveCount(1);
        user2Result.Value[0].TargetUrl.Should().Be("https://user2.com/hook3");

        // Unfiltered (null)
        var allResult = await handler.HandleAsync(new ListWebhookSubscriptionsQuery(null));
        allResult.IsSuccess.Should().BeTrue();
        allResult.Value.Should().HaveCount(3);
    }

    [Fact]
    public async Task DeleteWebhookSubscription_WhenFound_ShouldDelete()
    {
        var userId = Guid.NewGuid();
        var sub = WebhookSubscription.Create(userId, "https://delete.com/hook", new List<string> { "meeting.created" }).Value;
        _dbContext.WebhookSubscriptions.Add(sub);
        await _dbContext.SaveChangesAsync();

        var handler = new DeleteWebhookSubscriptionCommandHandler(_dbContext);
        var result = await handler.HandleAsync(new DeleteWebhookSubscriptionCommand(sub.Id, userId));

        result.IsSuccess.Should().BeTrue();

        var exists = await _dbContext.WebhookSubscriptions.AnyAsync(w => w.Id == sub.Id);
        exists.Should().BeFalse();
    }

    [Fact]
    public async Task DeleteWebhookSubscription_WhenNotFound_ShouldReturnError()
    {
        var handler = new DeleteWebhookSubscriptionCommandHandler(_dbContext);
        var result = await handler.HandleAsync(new DeleteWebhookSubscriptionCommand(Guid.NewGuid()));

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(WebhookErrors.SubscriptionNotFound);
    }

    [Fact]
    public async Task DeleteWebhookSubscription_WhenUnauthorizedUser_ShouldReturnUnauthorized()
    {
        var ownerId = Guid.NewGuid();
        var sub = WebhookSubscription.Create(ownerId, "https://auth.com/hook", new List<string> { "meeting.created" }).Value;
        _dbContext.WebhookSubscriptions.Add(sub);
        await _dbContext.SaveChangesAsync();

        var otherUser = Guid.NewGuid();
        var handler = new DeleteWebhookSubscriptionCommandHandler(_dbContext);
        var result = await handler.HandleAsync(new DeleteWebhookSubscriptionCommand(sub.Id, otherUser));

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(WebhookErrors.Unauthorized);
    }
}
