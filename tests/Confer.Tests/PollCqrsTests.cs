using Confer.Application.Interfaces;
using Confer.Application.Meetings.Polls.ClosePoll;
using Confer.Application.Meetings.Polls.CreatePoll;
using Confer.Application.Meetings.Polls.GetPolls;
using Confer.Application.Meetings.Polls.SubmitPollVote;
using Confer.Domain.Meetings;
using Confer.Infrastructure.Persistence;
using FluentAssertions;
using Microsoft.Data.Sqlite;
using Microsoft.EntityFrameworkCore;
using Moq;
using Xunit;

namespace Confer.Tests;

public class PollCqrsTests : IDisposable
{
    private readonly SqliteConnection _connection;
    private readonly ConferDbContext _dbContext;
    private readonly Mock<ISignalingNotifier> _signalingNotifierMock;

    public PollCqrsTests()
    {
        _connection = new SqliteConnection("DataSource=:memory:");
        _connection.Open();

        var options = new DbContextOptionsBuilder<ConferDbContext>()
            .UseSqlite(_connection)
            .Options;

        _dbContext = new ConferDbContext(options);
        _dbContext.Database.EnsureCreated();

        _signalingNotifierMock = new Mock<ISignalingNotifier>();
    }

    public void Dispose()
    {
        _dbContext.Dispose();
        _connection.Close();
        _connection.Dispose();
    }

    [Fact]
    public async Task CreatePollCommandHandler_WhenValid_ShouldCreateAndBroadcast()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Poll Meeting", ownerId).Value;
        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var validator = new CreatePollCommandValidator();
        var handler = new CreatePollCommandHandler(_dbContext, validator, _signalingNotifierMock.Object);
        var command = new CreatePollCommand(
            meeting.Id,
            ownerId,
            "What framework should we use?",
            new List<string> { "React", "Vue", "Svelte" },
            IsAnonymous: false,
            IsMultiChoice: false
        );

        var result = await handler.HandleAsync(command);

        result.IsSuccess.Should().BeTrue();
        result.Value.MeetingId.Should().Be(meeting.Id);
        result.Value.Question.Should().Be("What framework should we use?");
        result.Value.Options.Should().HaveCount(3);
        result.Value.IsActive.Should().BeTrue();

        _signalingNotifierMock.Verify(
            s => s.BroadcastPollCreatedAsync(meeting.Id, It.Is<Confer.Application.DTOs.PollDto>(p => p.Id == result.Value.Id)),
            Times.Once);
    }

    [Fact]
    public async Task CreatePollCommandHandler_WhenMeetingNotFound_ShouldFail()
    {
        var validator = new CreatePollCommandValidator();
        var handler = new CreatePollCommandHandler(_dbContext, validator, _signalingNotifierMock.Object);
        var command = new CreatePollCommand(Guid.NewGuid(), Guid.NewGuid(), "Question?", new List<string> { "A", "B" });

        var result = await handler.HandleAsync(command);

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.NotFound);
    }

    [Fact]
    public async Task SubmitPollVoteCommandHandler_WhenValid_ShouldVoteAndBroadcast()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Vote Meeting", ownerId).Value;
        _dbContext.Meetings.Add(meeting);

        var poll = Poll.Create(meeting.Id, ownerId, "Vote Topic", new List<string> { "Option 1", "Option 2" }).Value;
        _dbContext.Polls.Add(poll);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var optionId = poll.Options.First().Id;
        var voterId = Guid.NewGuid();

        var validator = new SubmitPollVoteCommandValidator();
        var handler = new SubmitPollVoteCommandHandler(_dbContext, validator, _signalingNotifierMock.Object);
        var command = new SubmitPollVoteCommand(meeting.Id, poll.Id, voterId, new List<Guid> { optionId });

        var result = await handler.HandleAsync(command);

        result.IsSuccess.Should().BeTrue();
        result.Value.TotalVotes.Should().Be(1);
        result.Value.Options.First(o => o.Id == optionId).VoteCount.Should().Be(1);

        _signalingNotifierMock.Verify(
            s => s.BroadcastPollUpdatedAsync(meeting.Id, It.Is<Confer.Application.DTOs.PollDto>(p => p.Id == poll.Id)),
            Times.Once);
    }

    [Fact]
    public async Task SubmitPollVoteCommandHandler_WhenAlreadyVoted_ShouldFail()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Vote Meeting", ownerId).Value;
        _dbContext.Meetings.Add(meeting);

        var poll = Poll.Create(meeting.Id, ownerId, "Vote Topic", new List<string> { "Option 1", "Option 2" }).Value;
        var optionId = poll.Options.First().Id;
        var voterId = Guid.NewGuid();
        poll.Vote(voterId, new List<Guid> { optionId });
        _dbContext.Polls.Add(poll);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var validator = new SubmitPollVoteCommandValidator();
        var handler = new SubmitPollVoteCommandHandler(_dbContext, validator, _signalingNotifierMock.Object);
        var command = new SubmitPollVoteCommand(meeting.Id, poll.Id, voterId, new List<Guid> { optionId });

        var result = await handler.HandleAsync(command);

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(PollErrors.AlreadyVoted);
    }

    [Fact]
    public async Task ClosePollCommandHandler_WhenValid_ShouldCloseAndBroadcast()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Close Meeting", ownerId).Value;
        _dbContext.Meetings.Add(meeting);

        var poll = Poll.Create(meeting.Id, ownerId, "Close Topic", new List<string> { "Option 1", "Option 2" }).Value;
        _dbContext.Polls.Add(poll);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var handler = new ClosePollCommandHandler(_dbContext, _signalingNotifierMock.Object);
        var command = new ClosePollCommand(meeting.Id, poll.Id, ownerId);

        var result = await handler.HandleAsync(command);

        result.IsSuccess.Should().BeTrue();
        result.Value.IsActive.Should().BeFalse();

        _signalingNotifierMock.Verify(
            s => s.BroadcastPollClosedAsync(meeting.Id, poll.Id),
            Times.Once);
    }

    [Fact]
    public async Task GetPollsQueryHandler_ShouldReturnPollsList()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("List Meeting", ownerId).Value;
        _dbContext.Meetings.Add(meeting);

        var poll1 = Poll.Create(meeting.Id, ownerId, "Question 1", new List<string> { "A", "B" }).Value;
        var poll2 = Poll.Create(meeting.Id, ownerId, "Question 2", new List<string> { "X", "Y" }).Value;
        _dbContext.Polls.AddRange(poll1, poll2);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var handler = new GetPollsQueryHandler(_dbContext);
        var query = new GetPollsQuery(meeting.Id);

        var result = await handler.HandleAsync(query);

        result.IsSuccess.Should().BeTrue();
        result.Value.Should().HaveCount(2);
        result.Value.Select(p => p.Question).Should().Contain(new[] { "Question 1", "Question 2" });
    }
}
