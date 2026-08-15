using Confer.Application.Meetings.Summary;
using Confer.Domain.Meetings;
using Confer.Domain.Sessions;
using FluentAssertions;
using Xunit;

namespace Confer.Tests;

public class SmartAiSummaryServiceTests
{
    private readonly SmartAiSummaryService _service = new();

    [Fact]
    public async Task GenerateSummaryAsync_WithTranscripts_ShouldExtractActionItemsAndDecisions()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Sprint Planning & Architecture", ownerId).Value;
        meeting.End(ownerId);

        var sessionId = Guid.NewGuid();
        var userAlice = Guid.NewGuid();
        var userBob = Guid.NewGuid();
        var userCharlie = Guid.NewGuid();

        var participations = new List<Participation>
        {
            Participation.Create(sessionId, ownerId, "Alice (Host)"),
            Participation.Create(sessionId, userBob, "Bob Smith"),
            Participation.Create(sessionId, userCharlie, "Charlie Brown")
        };

        var chatMessages = new List<ChatMessage>
        {
            ChatMessage.Create(sessionId, ownerId, "Alice", "Welcome everyone to sprint planning!"),
            ChatMessage.Create(sessionId, userBob, "Bob", "Decision: We will use PostgreSQL for metadata persistence."),
            ChatMessage.Create(sessionId, userCharlie, "Charlie", "We agreed that Redis is optimal for presence."),
            ChatMessage.Create(sessionId, ownerId, "Alice", "TODO: Update OpenAPI specifications by end of day"),
            ChatMessage.Create(sessionId, userBob, "Bob", "@Charlie please benchmark WebSocket throughput"),
            ChatMessage.Create(sessionId, userCharlie, "Charlie", "I will create the integration test suite"),
            ChatMessage.Create(sessionId, ownerId, "Alice", "Assigned to Bob: Configure Coturn TURN server [x]")
        };

        var result = await _service.GenerateSummaryAsync(meeting, chatMessages, participations);

        result.Should().NotBeNull();
        result.ParticipantCount.Should().Be(3);
        result.DurationMinutes.Should().BeGreaterThanOrEqualTo(1);

        // Verify Key Decisions
        result.KeyDecisions.Should().HaveCountGreaterThanOrEqualTo(2);
        result.KeyDecisions.Should().Contain(d => d.Contains("PostgreSQL", StringComparison.OrdinalIgnoreCase));
        result.KeyDecisions.Should().Contain(d => d.Contains("Redis", StringComparison.OrdinalIgnoreCase));

        // Verify Action Items
        result.ActionItems.Should().HaveCountGreaterThanOrEqualTo(4);

        var openApiItem = result.ActionItems.FirstOrDefault(a => a.Title.Contains("OpenAPI", StringComparison.OrdinalIgnoreCase));
        openApiItem.Should().NotBeNull();
        openApiItem!.Assignee.Should().Be("Alice");

        var benchmarkItem = result.ActionItems.FirstOrDefault(a => a.Title.Contains("benchmark", StringComparison.OrdinalIgnoreCase));
        benchmarkItem.Should().NotBeNull();
        benchmarkItem!.Assignee.Should().Be("Charlie");

        var testItem = result.ActionItems.FirstOrDefault(a => a.Title.Contains("integration test", StringComparison.OrdinalIgnoreCase));
        testItem.Should().NotBeNull();
        testItem!.Assignee.Should().Be("Charlie");

        var coturnItem = result.ActionItems.FirstOrDefault(a => a.Title.Contains("Coturn", StringComparison.OrdinalIgnoreCase));
        coturnItem.Should().NotBeNull();
        coturnItem!.Assignee.Should().Be("Bob");
        coturnItem.Status.Should().Be("completed");

        // Verify Overview
        result.Overview.Should().Contain("Sprint Planning & Architecture");
        result.Overview.Should().Contain("participant");
        result.Overview.Should().Contain("Alice");
        result.Overview.Should().Contain("Bob Smith");
    }

    [Fact]
    public async Task GenerateSummaryAsync_WithEmptyChat_ShouldGenerateGracefulOverview()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Quick Standup", ownerId).Value;
        meeting.End(ownerId);

        var sessionId = Guid.NewGuid();
        var participations = new List<Participation>
        {
            Participation.Create(sessionId, ownerId, "Alice (Host)")
        };

        var result = await _service.GenerateSummaryAsync(meeting, new List<ChatMessage>(), participations);

        result.Should().NotBeNull();
        result.ParticipantCount.Should().Be(1);
        result.DurationMinutes.Should().BeGreaterThanOrEqualTo(1);
        result.KeyDecisions.Should().BeEmpty();
        result.ActionItems.Should().BeEmpty();
        result.Overview.Should().Contain("Quick Standup");
        result.Overview.Should().Contain("No in-meeting chat messages were logged");
    }
}
