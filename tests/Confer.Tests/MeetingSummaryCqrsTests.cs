using Confer.Application.Meetings.Summary;
using Confer.Domain.Meetings;
using Confer.Domain.Sessions;
using Confer.Infrastructure.Persistence;
using FluentAssertions;
using Microsoft.Data.Sqlite;
using Microsoft.EntityFrameworkCore;
using Moq;
using Xunit;

namespace Confer.Tests;

public class MeetingSummaryCqrsTests : IDisposable
{
    private readonly SqliteConnection _connection;
    private readonly ConferDbContext _dbContext;
    private readonly Mock<IAiSummaryService> _aiSummaryServiceMock;

    public MeetingSummaryCqrsTests()
    {
        _connection = new SqliteConnection("DataSource=:memory:");
        _connection.Open();

        var options = new DbContextOptionsBuilder<ConferDbContext>()
            .UseSqlite(_connection)
            .Options;

        _dbContext = new ConferDbContext(options);
        _dbContext.Database.EnsureCreated();

        _aiSummaryServiceMock = new Mock<IAiSummaryService>();
    }

    public void Dispose()
    {
        _dbContext.Dispose();
        _connection.Close();
        _connection.Dispose();
    }

    [Fact]
    public async Task GenerateMeetingSummaryCommandHandler_WhenMeetingEnded_ShouldGenerateAndSaveSummary()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Sprint Retro", ownerId).Value;
        meeting.End(ownerId);
        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var expectedResult = new AiSummaryResult(
            "Retro meeting discussed velocity improvements.",
            new List<string> { "Adopt two-week sprint cycle" },
            new List<ActionItemDto> { new("Refactor test fixtures", "Alice", "pending") },
            DurationMinutes: 45,
            ParticipantCount: 3
        );

        _aiSummaryServiceMock
            .Setup(s => s.GenerateSummaryAsync(It.IsAny<Meeting>(), It.IsAny<IReadOnlyList<ChatMessage>>(), It.IsAny<IReadOnlyList<Participation>>(), It.IsAny<CancellationToken>()))
            .ReturnsAsync(expectedResult);

        var handler = new GenerateMeetingSummaryCommandHandler(_dbContext, _aiSummaryServiceMock.Object);
        var command = new GenerateMeetingSummaryCommand(meeting.Id, ownerId);

        var result = await handler.HandleAsync(command);

        result.IsSuccess.Should().BeTrue();
        result.Value.MeetingId.Should().Be(meeting.Id);
        result.Value.Overview.Should().Be(expectedResult.Overview);
        result.Value.KeyDecisions.Should().HaveCount(1);
        result.Value.ActionItems.Should().HaveCount(1);
        result.Value.DurationMinutes.Should().Be(45);
        result.Value.ParticipantCount.Should().Be(3);

        // Verify summary was saved in DB
        _dbContext.ChangeTracker.Clear();
        var dbSummary = await _dbContext.MeetingSummaries.FirstOrDefaultAsync(s => s.MeetingId == meeting.Id);
        dbSummary.Should().NotBeNull();
        dbSummary!.Overview.Should().Be(expectedResult.Overview);
        dbSummary.KeyDecisions.Should().ContainSingle().Which.Should().Be("Adopt two-week sprint cycle");
        dbSummary.ActionItems.Should().ContainSingle().Which.Title.Should().Be("Refactor test fixtures");
    }

    [Fact]
    public async Task GenerateMeetingSummaryCommandHandler_WhenMeetingNotEnded_ShouldFail()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Active Meeting", ownerId).Value;
        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var handler = new GenerateMeetingSummaryCommandHandler(_dbContext, _aiSummaryServiceMock.Object);
        var command = new GenerateMeetingSummaryCommand(meeting.Id, ownerId);

        var result = await handler.HandleAsync(command);

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.MeetingNotEnded);
    }

    [Fact]
    public async Task GenerateMeetingSummaryCommandHandler_WhenMeetingNotFound_ShouldFail()
    {
        var handler = new GenerateMeetingSummaryCommandHandler(_dbContext, _aiSummaryServiceMock.Object);
        var command = new GenerateMeetingSummaryCommand(Guid.NewGuid(), Guid.NewGuid());

        var result = await handler.HandleAsync(command);

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.NotFound);
    }

    [Fact]
    public async Task GenerateMeetingSummaryCommandHandler_WhenUnauthorizedActor_ShouldFail()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Confidential Board Sync", ownerId).Value;
        meeting.End(ownerId);
        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var nonParticipantId = Guid.NewGuid();
        var handler = new GenerateMeetingSummaryCommandHandler(_dbContext, _aiSummaryServiceMock.Object);
        var command = new GenerateMeetingSummaryCommand(meeting.Id, nonParticipantId);

        var result = await handler.HandleAsync(command);

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.Unauthorized);
    }

    [Fact]
    public async Task GenerateMeetingSummaryCommandHandler_WhenSummaryAlreadyExists_ShouldReturnExistingUnlessForced()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("All Hands Call", ownerId).Value;
        meeting.End(ownerId);
        meeting.AddSummary("Initial overview", new List<string>(), new List<ActionItem>(), 20, 2);
        _dbContext.Meetings.Add(meeting);
        _dbContext.MeetingSummaries.Add(meeting.Summary!);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var handler = new GenerateMeetingSummaryCommandHandler(_dbContext, _aiSummaryServiceMock.Object);
        var command = new GenerateMeetingSummaryCommand(meeting.Id, ownerId, ForceRegenerate: false);

        var result = await handler.HandleAsync(command);

        result.IsSuccess.Should().BeTrue();
        result.Value.Overview.Should().Be("Initial overview");

        // Verify AI service was not invoked when cached
        _aiSummaryServiceMock.Verify(
            s => s.GenerateSummaryAsync(It.IsAny<Meeting>(), It.IsAny<IReadOnlyList<ChatMessage>>(), It.IsAny<IReadOnlyList<Participation>>(), It.IsAny<CancellationToken>()),
            Times.Never);
    }

    [Fact]
    public async Task GetMeetingSummaryQueryHandler_WhenSummaryExists_ShouldReturnSummaryDto()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Roadmap Sync", ownerId).Value;
        meeting.End(ownerId);
        var summary = meeting.AddSummary(
            "Roadmap overview details",
            new List<string> { "Launch beta in November" },
            new List<ActionItem> { new("Security audit", "SecOps", "pending") },
            50,
            8).Value;

        _dbContext.Meetings.Add(meeting);
        _dbContext.MeetingSummaries.Add(summary);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var handler = new GetMeetingSummaryQueryHandler(_dbContext);
        var query = new GetMeetingSummaryQuery(meeting.Id);

        var result = await handler.HandleAsync(query);

        result.IsSuccess.Should().BeTrue();
        result.Value.MeetingId.Should().Be(meeting.Id);
        result.Value.Overview.Should().Be("Roadmap overview details");
        result.Value.KeyDecisions.Should().ContainSingle().Which.Should().Be("Launch beta in November");
        result.Value.ActionItems.Should().ContainSingle().Which.Title.Should().Be("Security audit");
        result.Value.DurationMinutes.Should().Be(50);
        result.Value.ParticipantCount.Should().Be(8);
    }

    [Fact]
    public async Task GetMeetingSummaryQueryHandler_WhenMeetingNotFound_ShouldReturnNotFound()
    {
        var handler = new GetMeetingSummaryQueryHandler(_dbContext);
        var query = new GetMeetingSummaryQuery(Guid.NewGuid());

        var result = await handler.HandleAsync(query);

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.NotFound);
    }

    [Fact]
    public async Task GetMeetingSummaryQueryHandler_WhenSummaryNotFound_ShouldReturnSummaryNotFound()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Empty Meeting", ownerId).Value;
        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var handler = new GetMeetingSummaryQueryHandler(_dbContext);
        var query = new GetMeetingSummaryQuery(meeting.Id);

        var result = await handler.HandleAsync(query);

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.SummaryNotFound);
    }

    [Fact]
    public void GenerateMeetingSummaryCommandValidator_ValidationRules()
    {
        var validator = new GenerateMeetingSummaryCommandValidator();

        validator.Validate(new GenerateMeetingSummaryCommand(Guid.Empty, Guid.NewGuid())).IsValid.Should().BeFalse();
        validator.Validate(new GenerateMeetingSummaryCommand(Guid.NewGuid(), Guid.Empty)).IsValid.Should().BeFalse();
        validator.Validate(new GenerateMeetingSummaryCommand(Guid.NewGuid(), Guid.NewGuid())).IsValid.Should().BeTrue();
    }
}
