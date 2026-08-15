using Confer.Application.Interfaces;
using Confer.Application.Meetings.GetRecordings;
using Confer.Application.Meetings.StartRecording;
using Confer.Application.Meetings.StopRecording;
using Confer.Domain.Meetings;
using Confer.Infrastructure.Persistence;
using FluentAssertions;
using Microsoft.Data.Sqlite;
using Microsoft.EntityFrameworkCore;
using Moq;
using Xunit;

namespace Confer.Tests;

public class RecordingCqrsTests : IDisposable
{
    private readonly SqliteConnection _connection;
    private readonly ConferDbContext _dbContext;
    private readonly Mock<ISignalingNotifier> _signalingNotifierMock;
    private readonly Mock<IRecordingStorageService> _recordingStorageMock;

    public RecordingCqrsTests()
    {
        _connection = new SqliteConnection("DataSource=:memory:");
        _connection.Open();

        var options = new DbContextOptionsBuilder<ConferDbContext>()
            .UseSqlite(_connection)
            .Options;

        _dbContext = new ConferDbContext(options);
        _dbContext.Database.EnsureCreated();

        _signalingNotifierMock = new Mock<ISignalingNotifier>();
        _recordingStorageMock = new Mock<IRecordingStorageService>();
    }

    public void Dispose()
    {
        _dbContext.Dispose();
        _connection.Close();
        _connection.Dispose();
    }

    [Fact]
    public async Task StartRecordingCommandHandler_WhenValid_ShouldStartAndBroadcast()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Planning", ownerId).Value;
        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var handler = new StartRecordingCommandHandler(_dbContext, _signalingNotifierMock.Object);
        var command = new StartRecordingCommand(meeting.Id, ownerId);

        var result = await handler.HandleAsync(command);

        result.IsSuccess.Should().BeTrue();
        result.Value.MeetingId.Should().Be(meeting.Id);
        result.Value.Status.Should().Be("recording");

        _signalingNotifierMock.Verify(
            s => s.BroadcastRecordingStateAsync(meeting.Id, true, It.IsAny<Guid?>()),
            Times.Once);
    }

    [Fact]
    public async Task StartRecordingCommandHandler_WhenMeetingNotFound_ShouldFail()
    {
        var handler = new StartRecordingCommandHandler(_dbContext, _signalingNotifierMock.Object);
        var command = new StartRecordingCommand(Guid.NewGuid(), Guid.NewGuid());

        var result = await handler.HandleAsync(command);

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.NotFound);
    }

    [Fact]
    public async Task StopRecordingCommandHandler_WhenRecording_ShouldStopAndArchive()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Sprint Retro", ownerId).Value;
        meeting.StartRecording(ownerId);
        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();

        var activeRecording = meeting.Recordings.First();
        var mockPath = $"/var/data/recordings/{meeting.Id}/{activeRecording.Id}.webm";
        _dbContext.ChangeTracker.Clear();

        _recordingStorageMock
            .Setup(s => s.GetRecordingPath(meeting.Id, activeRecording.Id, "webm"))
            .Returns(mockPath);
        _recordingStorageMock
            .Setup(s => s.ExistsAsync(mockPath, It.IsAny<CancellationToken>()))
            .ReturnsAsync(false);
        _recordingStorageMock
            .Setup(s => s.SaveRecordingAsync(meeting.Id, activeRecording.Id, It.IsAny<byte[]>(), "webm", It.IsAny<CancellationToken>()))
            .ReturnsAsync(mockPath);

        var handler = new StopRecordingCommandHandler(_dbContext, _recordingStorageMock.Object, _signalingNotifierMock.Object);
        var command = new StopRecordingCommand(meeting.Id, ownerId);

        var result = await handler.HandleAsync(command);

        result.IsSuccess.Should().BeTrue();
        result.Value.MeetingId.Should().Be(meeting.Id);
        result.Value.Status.Should().Be("completed");
        result.Value.StoragePath.Should().Be(mockPath);

        _signalingNotifierMock.Verify(
            s => s.BroadcastRecordingStateAsync(meeting.Id, false, It.IsAny<Guid?>()),
            Times.Once);
    }

    [Fact]
    public async Task GetMeetingRecordingsQueryHandler_ShouldReturnRecordingsList()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Design Review", ownerId).Value;
        meeting.StartRecording(ownerId);
        meeting.StopRecording(ownerId, "/path/rec1.webm", 2048);
        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var handler = new GetMeetingRecordingsQueryHandler(_dbContext);
        var query = new GetMeetingRecordingsQuery(meeting.Id);

        var result = await handler.HandleAsync(query);

        result.IsSuccess.Should().BeTrue();
        result.Value.Should().HaveCount(1);
        result.Value[0].MeetingId.Should().Be(meeting.Id);
        result.Value[0].Status.Should().Be("completed");
        result.Value[0].StoragePath.Should().Be("/path/rec1.webm");
    }
}
