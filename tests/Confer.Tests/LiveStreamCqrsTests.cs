using Confer.Application.Interfaces;
using Confer.Application.Meetings.Stream.GetLiveStreamStatus;
using Confer.Application.Meetings.Stream.StartLiveStream;
using Confer.Application.Meetings.Stream.StopLiveStream;
using Confer.Domain.Meetings;
using Confer.Infrastructure.Persistence;
using FluentAssertions;
using Microsoft.Data.Sqlite;
using Microsoft.EntityFrameworkCore;
using Moq;
using Xunit;

namespace Confer.Tests;

public class LiveStreamCqrsTests : IDisposable
{
    private readonly SqliteConnection _connection;
    private readonly ConferDbContext _dbContext;
    private readonly Mock<ISignalingNotifier> _signalingNotifierMock;

    public LiveStreamCqrsTests()
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
    public async Task StartLiveStreamCommandHandler_WhenValid_ShouldStartAndBroadcast()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Keynote Broadcast", ownerId).Value;
        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var handler = new StartLiveStreamCommandHandler(_dbContext, _signalingNotifierMock.Object);
        var command = new StartLiveStreamCommand(meeting.Id, ownerId, "rtmp://live.youtube.com/app", "stream_key_xyz");

        var result = await handler.HandleAsync(command);

        result.IsSuccess.Should().BeTrue();
        result.Value.MeetingId.Should().Be(meeting.Id);
        result.Value.IsStreamingEnabled.Should().BeTrue();
        result.Value.Status.Should().Be("live");
        result.Value.RtmpUrl.Should().Be("rtmp://live.youtube.com/app");
        result.Value.StartedAt.Should().NotBeNull();

        _signalingNotifierMock.Verify(
            s => s.BroadcastStreamStatusAsync(meeting.Id, true, "rtmp://live.youtube.com/app", "live", It.IsAny<DateTime?>()),
            Times.Once);

        // Verify persisted state
        var dbMeeting = await _dbContext.Meetings.FindAsync(meeting.Id);
        dbMeeting.Should().NotBeNull();
        dbMeeting!.LiveStreamConfig.IsStreamingEnabled.Should().BeTrue();
        dbMeeting.LiveStreamConfig.RtmpUrl.Should().Be("rtmp://live.youtube.com/app");
    }

    [Fact]
    public async Task StartLiveStreamCommandHandler_WhenMeetingNotFound_ShouldFail()
    {
        var handler = new StartLiveStreamCommandHandler(_dbContext, _signalingNotifierMock.Object);
        var command = new StartLiveStreamCommand(Guid.NewGuid(), Guid.NewGuid(), "rtmp://live.youtube.com/app", "stream_key_xyz");

        var result = await handler.HandleAsync(command);

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.NotFound);
    }

    [Fact]
    public async Task StopLiveStreamCommandHandler_WhenLive_ShouldStopAndBroadcast()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Keynote Broadcast", ownerId).Value;
        meeting.StartLiveStream(ownerId, "rtmp://live.youtube.com/app", "stream_key_xyz");
        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var handler = new StopLiveStreamCommandHandler(_dbContext, _signalingNotifierMock.Object);
        var command = new StopLiveStreamCommand(meeting.Id, ownerId);

        var result = await handler.HandleAsync(command);

        result.IsSuccess.Should().BeTrue();
        result.Value.MeetingId.Should().Be(meeting.Id);
        result.Value.IsStreamingEnabled.Should().BeFalse();
        result.Value.Status.Should().Be("idle");

        _signalingNotifierMock.Verify(
            s => s.BroadcastStreamStatusAsync(meeting.Id, false, It.IsAny<string>(), "idle", null),
            Times.Once);

        // Verify persisted state
        var dbMeeting = await _dbContext.Meetings.FindAsync(meeting.Id);
        dbMeeting.Should().NotBeNull();
        dbMeeting!.LiveStreamConfig.IsStreamingEnabled.Should().BeFalse();
    }

    [Fact]
    public async Task StopLiveStreamCommandHandler_WhenNotStreaming_ShouldFail()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Keynote Broadcast", ownerId).Value;
        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var handler = new StopLiveStreamCommandHandler(_dbContext, _signalingNotifierMock.Object);
        var command = new StopLiveStreamCommand(meeting.Id, ownerId);

        var result = await handler.HandleAsync(command);

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.NotStreaming);
    }

    [Fact]
    public async Task GetLiveStreamStatusQueryHandler_ShouldReturnStatus()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Keynote Broadcast", ownerId).Value;
        meeting.StartLiveStream(ownerId, "rtmp://live.youtube.com/app", "stream_key_xyz");
        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var handler = new GetLiveStreamStatusQueryHandler(_dbContext);
        var query = new GetLiveStreamStatusQuery(meeting.Id);

        var result = await handler.HandleAsync(query);

        result.IsSuccess.Should().BeTrue();
        result.Value.MeetingId.Should().Be(meeting.Id);
        result.Value.IsStreamingEnabled.Should().BeTrue();
        result.Value.Status.Should().Be("live");
        result.Value.RtmpUrl.Should().Be("rtmp://live.youtube.com/app");
    }
}
