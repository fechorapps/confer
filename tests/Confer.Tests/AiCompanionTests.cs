using Confer.Application.DTOs;
using Confer.Domain.Sessions;
using Confer.Infrastructure.AI;
using Xunit;

namespace Confer.Tests;

public sealed class AiCompanionTests
{
    private readonly ConferAiCompanionService _aiService = new();

    [Theory]
    [InlineData("/ai summarize", true)]
    [InlineData("@confer-ai que se ha dicho", true)]
    [InlineData("@ai recap", true)]
    [InlineData("/ai action-items", true)]
    [InlineData("Hello everyone", false)]
    [InlineData("Let's discuss the architecture", false)]
    public void IsAiCommand_ShouldDetectValidTriggerPrefixes(string message, bool expected)
    {
        var result = _aiService.IsAiCommand(message);
        Assert.Equal(expected, result);
    }

    [Fact]
    public async Task ProcessAiQueryAsync_WithSummarizeCommand_ShouldReturnLiveRecap()
    {
        var meetingId = Guid.NewGuid();
        var chatHistory = new List<ChatMessage>
        {
            ChatMessage.Create(meetingId, Guid.NewGuid(), "Alice", "We should ship by Friday."),
            ChatMessage.Create(meetingId, Guid.NewGuid(), "Bob", "Agreed, the test suite is green.")
        };
        var captionHistory = new List<CaptionChunkDto>
        {
            new(Guid.NewGuid(), "Alice", "The WebRTC video latency is under 50 milliseconds.", true, "en-US", 1000),
            new(Guid.NewGuid(), "Bob", "All unit and integration tests are passing.", true, "en-US", 2000)
        };

        var response = await _aiService.ProcessAiQueryAsync(meetingId, "/ai summarize", "Alice", chatHistory, captionHistory);

        Assert.NotNull(response);
        Assert.Contains("Live AI Meeting Recap", response);
        Assert.Contains("Alice", response);
        Assert.Contains("Bob", response);
    }

    [Fact]
    public async Task ProcessAiQueryAsync_WithActionItemsCommand_ShouldExtractActionItems()
    {
        var meetingId = Guid.NewGuid();
        var chatHistory = new List<ChatMessage>
        {
            ChatMessage.Create(meetingId, Guid.NewGuid(), "Alice", "todo: deploy the new helm chart to production")
        };
        var captionHistory = new List<CaptionChunkDto>
        {
            new(Guid.NewGuid(), "Bob", "I will review the pull request by 5pm today.", true, "en-US", 3000)
        };

        var response = await _aiService.ProcessAiQueryAsync(meetingId, "/ai action-items", "Bob", chatHistory, captionHistory);

        Assert.NotNull(response);
        Assert.Contains("Detected Action Items", response);
        Assert.Contains("Bob", response);
    }

    [Fact]
    public async Task ProcessAiQueryAsync_WithHelpCommand_ShouldReturnAvailableCommands()
    {
        var meetingId = Guid.NewGuid();
        var response = await _aiService.ProcessAiQueryAsync(meetingId, "/ai help", "Alice", new(), new());

        Assert.NotNull(response);
        Assert.Contains("Confer AI Copilot", response);
        Assert.Contains("/ai summarize", response);
        Assert.Contains("/ai action-items", response);
    }
}
