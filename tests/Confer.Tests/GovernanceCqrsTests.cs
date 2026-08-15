using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Application.Meetings.Governance.AdmitAll;
using Confer.Application.Meetings.Governance.AdmitParticipant;
using Confer.Application.Meetings.Governance.GetPolicy;
using Confer.Application.Meetings.Governance.GetWaitingRoom;
using Confer.Application.Meetings.Governance.RejectParticipant;
using Confer.Application.Meetings.Governance.ToggleWaitingRoom;
using Confer.Application.Meetings.Governance.UpdatePolicy;
using Confer.Domain.Enums;
using Confer.Domain.Meetings;
using Confer.Domain.Sessions;
using Confer.Infrastructure.Persistence;
using FluentAssertions;
using Microsoft.Data.Sqlite;
using Microsoft.EntityFrameworkCore;
using Moq;
using Xunit;

namespace Confer.Tests;

public class GovernanceCqrsTests : IDisposable
{
    private readonly SqliteConnection _connection;
    private readonly ConferDbContext _dbContext;
    private readonly Mock<ISignalingNotifier> _signalingNotifierMock;

    public GovernanceCqrsTests()
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
    public async Task ToggleWaitingRoomCommandHandler_WhenValid_ShouldToggleAndBroadcast()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Room Test", ownerId).Value;
        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();

        var handler = new ToggleWaitingRoomCommandHandler(_dbContext, _signalingNotifierMock.Object);
        var command = new ToggleWaitingRoomCommand(meeting.Id, ownerId, true);

        var result = await handler.HandleAsync(command);

        result.IsSuccess.Should().BeTrue();
        result.Value.Should().BeTrue();

        _signalingNotifierMock.Verify(s => s.BroadcastWaitingRoomUpdateAsync(meeting.Id, true, 0), Times.Once);
    }

    [Fact]
    public async Task AdmitParticipantCommandHandler_WhenValid_ShouldAdmitAndNotify()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Waiting Room", ownerId, isWaitingRoomEnabled: true).Value;
        var session = Session.Create(meeting.Id);
        meeting.Sessions.Add(session);

        var participantUserId = Guid.NewGuid();
        var p = session.AddParticipant(participantUserId, "Alice", ParticipantRole.Participant, status: ParticipationStatus.InWaitingRoom).Value;

        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();

        var handler = new AdmitParticipantCommandHandler(_dbContext, _signalingNotifierMock.Object);
        var command = new AdmitParticipantCommand(meeting.Id, ownerId, p.Id);

        var result = await handler.HandleAsync(command);

        result.IsSuccess.Should().BeTrue();
        p.Status.Should().Be(ParticipationStatus.Admitted);

        _signalingNotifierMock.Verify(s => s.BroadcastParticipantAdmittedAsync(meeting.Id, p.Id), Times.Once);
        _signalingNotifierMock.Verify(s => s.BroadcastWaitingRoomUpdateAsync(meeting.Id, true, 0), Times.Once);
    }

    [Fact]
    public async Task AdmitAllCommandHandler_WhenValid_ShouldAdmitAllAndNotify()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Waiting Room", ownerId, isWaitingRoomEnabled: true).Value;
        var session = Session.Create(meeting.Id);
        meeting.Sessions.Add(session);

        var p1 = session.AddParticipant(Guid.NewGuid(), "Alice", ParticipantRole.Participant, status: ParticipationStatus.InWaitingRoom).Value;
        var p2 = session.AddParticipant(Guid.NewGuid(), "Bob", ParticipantRole.Participant, status: ParticipationStatus.InWaitingRoom).Value;

        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();

        var handler = new AdmitAllCommandHandler(_dbContext, _signalingNotifierMock.Object);
        var command = new AdmitAllCommand(meeting.Id, ownerId);

        var result = await handler.HandleAsync(command);

        result.IsSuccess.Should().BeTrue();
        result.Value.Should().Be(2);

        _signalingNotifierMock.Verify(s => s.BroadcastParticipantAdmittedAsync(meeting.Id, p1.Id), Times.Once);
        _signalingNotifierMock.Verify(s => s.BroadcastParticipantAdmittedAsync(meeting.Id, p2.Id), Times.Once);
        _signalingNotifierMock.Verify(s => s.BroadcastWaitingRoomUpdateAsync(meeting.Id, true, 0), Times.Once);
    }

    [Fact]
    public async Task RejectParticipantCommandHandler_WhenValid_ShouldRejectAndNotify()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Waiting Room", ownerId, isWaitingRoomEnabled: true).Value;
        var session = Session.Create(meeting.Id);
        meeting.Sessions.Add(session);

        var p = session.AddParticipant(Guid.NewGuid(), "Eve", ParticipantRole.Participant, status: ParticipationStatus.InWaitingRoom).Value;

        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();

        var handler = new RejectParticipantCommandHandler(_dbContext, _signalingNotifierMock.Object);
        var command = new RejectParticipantCommand(meeting.Id, ownerId, p.Id);

        var result = await handler.HandleAsync(command);

        result.IsSuccess.Should().BeTrue();
        p.Status.Should().Be(ParticipationStatus.Rejected);

        _signalingNotifierMock.Verify(s => s.BroadcastLeftAsync(meeting.Id, p.Id, "rejected_from_waiting_room"), Times.Once);
        _signalingNotifierMock.Verify(s => s.BroadcastWaitingRoomUpdateAsync(meeting.Id, true, 0), Times.Once);
    }

    [Fact]
    public async Task GetWaitingRoomQueryHandler_ShouldReturnWaitingParticipants()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Waiting Room", ownerId, isWaitingRoomEnabled: true).Value;
        var session = Session.Create(meeting.Id);
        meeting.Sessions.Add(session);

        var p1 = session.AddParticipant(Guid.NewGuid(), "Alice", ParticipantRole.Participant, clientInfo: "Desktop Linux", status: ParticipationStatus.InWaitingRoom).Value;
        var p2 = session.AddParticipant(Guid.NewGuid(), "Bob", ParticipantRole.Participant, clientInfo: "Android App", status: ParticipationStatus.InWaitingRoom).Value;
        var p3 = session.AddParticipant(Guid.NewGuid(), "Charlie", ParticipantRole.Participant, clientInfo: "Web", status: ParticipationStatus.Admitted).Value;

        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();

        var handler = new GetWaitingRoomQueryHandler(_dbContext);
        var query = new GetWaitingRoomQuery(meeting.Id, ownerId);

        var result = await handler.HandleAsync(query);

        result.IsSuccess.Should().BeTrue();
        result.Value.MeetingId.Should().Be(meeting.Id);
        result.Value.IsWaitingRoomEnabled.Should().BeTrue();
        result.Value.WaitingParticipants.Should().HaveCount(2);
        result.Value.WaitingParticipants.Select(p => p.DisplayName).Should().Contain(new[] { "Alice", "Bob" });
    }

    [Fact]
    public async Task UpdateMeetingPolicyCommandHandler_ShouldUpdatePoliciesAndBroadcast()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Policy Meeting", ownerId).Value;
        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();

        var handler = new UpdateMeetingPolicyCommandHandler(_dbContext, _signalingNotifierMock.Object);
        var newPolicy = new MeetingPolicyDto(
            AllowScreenShare: false,
            AllowChat: false,
            AllowUnmuteSelf: false,
            MuteOnEntry: true,
            AllowRename: false);

        var command = new UpdateMeetingPolicyCommand(meeting.Id, ownerId, newPolicy, IsWatermarkEnabled: true, IsWaitingRoomEnabled: true);

        var result = await handler.HandleAsync(command);

        result.IsSuccess.Should().BeTrue();
        result.Value.AllowScreenShare.Should().BeFalse();
        result.Value.AllowChat.Should().BeFalse();
        result.Value.MuteOnEntry.Should().BeTrue();

        _signalingNotifierMock.Verify(s => s.BroadcastPolicyChangedAsync(meeting.Id, newPolicy, true, true), Times.Once);
    }

    [Fact]
    public async Task GetMeetingPolicyQueryHandler_ShouldReturnCurrentPolicies()
    {
        var ownerId = Guid.NewGuid();
        var customPolicy = new MeetingPolicy(
            allowScreenShare: true,
            allowChat: false,
            allowUnmuteSelf: true,
            muteOnEntry: true,
            allowRename: false);

        var meeting = Meeting.Create("Policy Meeting", ownerId, isWaitingRoomEnabled: true, isWatermarkEnabled: true, policy: customPolicy).Value;
        _dbContext.Meetings.Add(meeting);
        await _dbContext.SaveChangesAsync();

        var handler = new GetMeetingPolicyQueryHandler(_dbContext);
        var query = new GetMeetingPolicyQuery(meeting.Id);

        var result = await handler.HandleAsync(query);

        result.IsSuccess.Should().BeTrue();
        result.Value.MeetingId.Should().Be(meeting.Id);
        result.Value.IsWaitingRoomEnabled.Should().BeTrue();
        result.Value.IsWatermarkEnabled.Should().BeTrue();
        result.Value.Policy.AllowChat.Should().BeFalse();
        result.Value.Policy.MuteOnEntry.Should().BeTrue();
        result.Value.Policy.AllowRename.Should().BeFalse();
    }
}
