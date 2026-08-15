using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Application.Meetings.Breakouts.CloseBreakouts;
using Confer.Application.Meetings.Breakouts.CreateBreakouts;
using Confer.Domain.Meetings;
using Confer.Infrastructure.Persistence;
using FluentAssertions;
using Microsoft.Data.Sqlite;
using Microsoft.EntityFrameworkCore;
using Moq;
using Xunit;

namespace Confer.Tests;

public class BreakoutCqrsTests : IDisposable
{
    private readonly SqliteConnection _connection;
    private readonly ConferDbContext _dbContext;
    private readonly Mock<ISignalingNotifier> _signalingNotifierMock;

    public BreakoutCqrsTests()
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
    public async Task CreateBreakoutsCommandHandler_WhenValid_ShouldCreateRoomsAndBroadcastInvites()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Breakout Meeting", ownerId).Value;
        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var validator = new CreateBreakoutsCommandValidator();
        var handler = new CreateBreakoutsCommandHandler(_dbContext, validator, _signalingNotifierMock.Object);
        var command = new CreateBreakoutsCommand(meeting.Id, ownerId, 3, 10, new List<string> { "Room Alpha", "Room Beta", "Room Gamma" });

        var result = await handler.HandleAsync(command);

        result.IsSuccess.Should().BeTrue();
        result.Value.Should().HaveCount(3);
        result.Value[0].Name.Should().Be("Room Alpha");
        result.Value[1].Name.Should().Be("Room Beta");
        result.Value[2].Name.Should().Be("Room Gamma");

        _signalingNotifierMock.Verify(
            s => s.BroadcastBreakoutInviteAsync(meeting.Id, Guid.Empty, It.IsAny<BreakoutInviteDto>()),
            Times.Exactly(3));
    }

    [Fact]
    public async Task CreateBreakoutsCommandHandler_WithAssignments_ShouldBroadcastToParticipants()
    {
        var ownerId = Guid.NewGuid();
        var participantId = Guid.NewGuid();
        var meeting = Meeting.Create("Breakout Meeting", ownerId).Value;
        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var assignments = new List<BreakoutAssignmentDto>
        {
            new(participantId, Guid.Empty, "Room Alpha")
        };

        var validator = new CreateBreakoutsCommandValidator();
        var handler = new CreateBreakoutsCommandHandler(_dbContext, validator, _signalingNotifierMock.Object);
        var command = new CreateBreakoutsCommand(meeting.Id, ownerId, 2, 15, new List<string> { "Room Alpha", "Room Beta" }, assignments);

        var result = await handler.HandleAsync(command);

        result.IsSuccess.Should().BeTrue();
        result.Value.Should().HaveCount(2);

        _signalingNotifierMock.Verify(
            s => s.BroadcastBreakoutInviteAsync(meeting.Id, participantId, It.Is<BreakoutInviteDto>(i => i.RoomName == "Room Alpha")),
            Times.Once);
    }

    [Fact]
    public async Task CreateBreakoutsCommandHandler_WhenNotOwner_ShouldFailWithUnauthorized()
    {
        var ownerId = Guid.NewGuid();
        var strangerId = Guid.NewGuid();
        var meeting = Meeting.Create("Breakout Meeting", ownerId).Value;
        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var validator = new CreateBreakoutsCommandValidator();
        var handler = new CreateBreakoutsCommandHandler(_dbContext, validator, _signalingNotifierMock.Object);
        var command = new CreateBreakoutsCommand(meeting.Id, strangerId, 2);

        var result = await handler.HandleAsync(command);

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(BreakoutErrors.Unauthorized);
    }

    [Fact]
    public async Task CloseBreakoutsCommandHandler_WhenValid_ShouldCloseActiveRooms()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Breakout Meeting", ownerId).Value;
        _dbContext.Meetings.Add(meeting);

        var room1 = BreakoutRoom.Create(meeting.Id, "Room 1", 1, 15).Value;
        var room2 = BreakoutRoom.Create(meeting.Id, "Room 2", 2, 15).Value;
        _dbContext.BreakoutRooms.AddRange(room1, room2);
        await _dbContext.SaveChangesAsync();
        _dbContext.ChangeTracker.Clear();

        var handler = new CloseBreakoutsCommandHandler(_dbContext);
        var command = new CloseBreakoutsCommand(meeting.Id, ownerId);

        var result = await handler.HandleAsync(command);

        result.IsSuccess.Should().BeTrue();

        var rooms = await _dbContext.BreakoutRooms.Where(b => b.MeetingId == meeting.Id).ToListAsync();
        rooms.Should().AllSatisfy(r => r.EndsAt.Should().NotBeNull());
    }
}
