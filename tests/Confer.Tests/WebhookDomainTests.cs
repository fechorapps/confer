using Confer.Domain.Webhooks;
using FluentAssertions;
using Xunit;

namespace Confer.Tests;

public class WebhookDomainTests
{
    [Fact]
    public void Create_WithValidParameters_ShouldSucceed()
    {
        var userId = Guid.NewGuid();
        var targetUrl = "https://api.mycompany.com/webhooks/confer";
        var events = new List<string> { "meeting.created", "meeting.ended" };

        var result = WebhookSubscription.Create(userId, targetUrl, events);

        result.IsSuccess.Should().BeTrue();
        var subscription = result.Value;
        subscription.Id.Should().NotBeEmpty();
        subscription.UserId.Should().Be(userId);
        subscription.TargetUrl.Should().Be(targetUrl);
        subscription.Secret.Should().StartWith("whsec_");
        subscription.SubscribedEvents.Should().BeEquivalentTo(events);
        subscription.IsActive.Should().BeTrue();
        subscription.CreatedAt.Should().BeCloseTo(DateTime.UtcNow, TimeSpan.FromSeconds(2));
    }

    [Fact]
    public void Create_WithCustomSecret_ShouldUseProvidedSecret()
    {
        var userId = Guid.NewGuid();
        var targetUrl = "https://webhook.site/test-uuid";
        var events = new List<string> { "*" };
        var customSecret = "custom_secret_key_123456";

        var result = WebhookSubscription.Create(userId, targetUrl, events, customSecret);

        result.IsSuccess.Should().BeTrue();
        result.Value.Secret.Should().Be(customSecret);
    }

    [Theory]
    [InlineData("")]
    [InlineData(" ")]
    [InlineData("ftp://invalid.com")]
    [InlineData("not-a-url")]
    [InlineData("http://")]
    public void Create_WithInvalidUrl_ShouldFail(string invalidUrl)
    {
        var userId = Guid.NewGuid();
        var events = new List<string> { "meeting.created" };

        var result = WebhookSubscription.Create(userId, invalidUrl, events);

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(WebhookErrors.InvalidUrl);
    }

    [Fact]
    public void Create_WithEmptyUserId_ShouldFail()
    {
        var events = new List<string> { "meeting.created" };
        var result = WebhookSubscription.Create(Guid.Empty, "https://example.com/webhook", events);

        result.IsFailure.Should().BeTrue();
        result.Error.Code.Should().Be("Webhook.UserIdEmpty");
    }

    [Fact]
    public void Create_WithNullOrEmptyEvents_ShouldFail()
    {
        var userId = Guid.NewGuid();
        var url = "https://example.com/webhook";

        var resultNull = WebhookSubscription.Create(userId, url, null!);
        resultNull.IsFailure.Should().BeTrue();
        resultNull.Error.Should().Be(WebhookErrors.InvalidEvents);

        var resultEmpty = WebhookSubscription.Create(userId, url, new List<string>());
        resultEmpty.IsFailure.Should().BeTrue();
        resultEmpty.Error.Should().Be(WebhookErrors.InvalidEvents);

        var resultWhitespace = WebhookSubscription.Create(userId, url, new List<string> { "   ", "" });
        resultWhitespace.IsFailure.Should().BeTrue();
        resultWhitespace.Error.Should().Be(WebhookErrors.InvalidEvents);
    }

    [Fact]
    public void DeactivateAndActivate_ShouldChangeIsActiveState()
    {
        var subscription = WebhookSubscription.Create(
            Guid.NewGuid(),
            "https://example.com/hook",
            new List<string> { "meeting.started" }
        ).Value;

        subscription.IsActive.Should().BeTrue();

        subscription.Deactivate();
        subscription.IsActive.Should().BeFalse();
        subscription.MatchesEvent("meeting.started").Should().BeFalse();

        subscription.Activate();
        subscription.IsActive.Should().BeTrue();
        subscription.MatchesEvent("meeting.started").Should().BeTrue();
    }

    [Fact]
    public void MatchesEvent_WithWildcard_ShouldMatchAnyEvent()
    {
        var subscription = WebhookSubscription.Create(
            Guid.NewGuid(),
            "https://example.com/hook",
            new List<string> { "*" }
        ).Value;

        subscription.MatchesEvent("meeting.created").Should().BeTrue();
        subscription.MatchesEvent("recording.stopped").Should().BeTrue();
        subscription.MatchesEvent("poll.created").Should().BeTrue();
    }

    [Fact]
    public void MatchesEvent_WithSpecificEvents_ShouldMatchOnlySubscribed()
    {
        var subscription = WebhookSubscription.Create(
            Guid.NewGuid(),
            "https://example.com/hook",
            new List<string> { "meeting.created", "MEETING.ENDED" }
        ).Value;

        subscription.MatchesEvent("meeting.created").Should().BeTrue();
        subscription.MatchesEvent("meeting.ended").Should().BeTrue();
        subscription.MatchesEvent("participant.joined").Should().BeFalse();
    }
}
