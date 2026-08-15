using Confer.Domain.Meetings;
using FluentAssertions;
using Xunit;

namespace Confer.Tests;

public class MeetingSummaryDomainTests
{
    [Fact]
    public void Create_WhenValidParameters_ShouldReturnSuccess()
    {
        var meetingId = Guid.NewGuid();
        var overview = "Product review meeting to discuss roadmap and priorities.";
        var decisions = new List<string> { "Adopt new WebRTC SFU engine", "Target Q4 release" };
        var actionItems = new List<ActionItem>
        {
            new("Update API documentation", "Alice", "pending"),
            new("Benchmark VP8 simulcast layer", "Bob", "in_progress")
        };

        var result = MeetingSummary.Create(
            meetingId,
            overview,
            decisions,
            actionItems,
            durationMinutes: 45,
            participantCount: 5);

        result.IsSuccess.Should().BeTrue();
        result.Value.Should().NotBeNull();
        result.Value.MeetingId.Should().Be(meetingId);
        result.Value.Overview.Should().Be(overview);
        result.Value.KeyDecisions.Should().HaveCount(2);
        result.Value.ActionItems.Should().HaveCount(2);
        result.Value.DurationMinutes.Should().Be(45);
        result.Value.ParticipantCount.Should().Be(5);
        result.Value.GeneratedAt.Should().BeCloseTo(DateTime.UtcNow, TimeSpan.FromSeconds(5));
    }

    [Fact]
    public void Create_WhenMeetingIdEmpty_ShouldFail()
    {
        var result = MeetingSummary.Create(
            Guid.Empty,
            "Valid overview",
            new List<string>(),
            new List<ActionItem>());

        result.IsFailure.Should().BeTrue();
        result.Error.Code.Should().Be("MeetingSummary.MeetingIdEmpty");
    }

    [Theory]
    [InlineData("")]
    [InlineData("   ")]
    [InlineData(null)]
    public void Create_WhenOverviewEmptyOrWhitespace_ShouldFail(string? invalidOverview)
    {
        var result = MeetingSummary.Create(
            Guid.NewGuid(),
            invalidOverview!,
            new List<string>(),
            new List<ActionItem>());

        result.IsFailure.Should().BeTrue();
        result.Error.Code.Should().Be("MeetingSummary.OverviewEmpty");
    }

    [Fact]
    public void Meeting_AddSummary_WhenMeetingNotEnded_ShouldReturnMeetingNotEndedError()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Active Meeting", ownerId).Value;

        var result = meeting.AddSummary(
            "Overview of active meeting",
            new List<string> { "Decision 1" },
            new List<ActionItem>(),
            durationMinutes: 10,
            participantCount: 2);

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.MeetingNotEnded);
        meeting.Summary.Should().BeNull();
    }

    [Fact]
    public void Meeting_AddSummary_WhenMeetingEnded_ShouldSucceedAndSetSummary()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Completed Meeting", ownerId).Value;
        meeting.End(ownerId);

        var result = meeting.AddSummary(
            "Concluded strategy meeting",
            new List<string> { "Finalized architecture" },
            new List<ActionItem> { new("Deploy services", "Charlie", "pending") },
            durationMinutes: 30,
            participantCount: 4);

        result.IsSuccess.Should().BeTrue();
        meeting.Summary.Should().NotBeNull();
        meeting.Summary!.MeetingId.Should().Be(meeting.Id);
        meeting.Summary.Overview.Should().Be("Concluded strategy meeting");
        meeting.Summary.KeyDecisions.Should().ContainSingle().Which.Should().Be("Finalized architecture");
        meeting.Summary.ActionItems.Should().ContainSingle().Which.Title.Should().Be("Deploy services");
    }

    [Fact]
    public void Meeting_AddSummaryInstance_WhenValid_ShouldSetSummary()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Strategy Sync", ownerId).Value;
        meeting.End(ownerId);

        var summary = MeetingSummary.Create(
            meeting.Id,
            "Direct summary creation",
            new List<string> { "Go with option A" },
            new List<ActionItem> { new("Setup CI/CD", "DevOps") },
            15,
            3).Value;

        var result = meeting.AddSummary(summary);

        result.IsSuccess.Should().BeTrue();
        meeting.Summary.Should().Be(summary);
    }
}
