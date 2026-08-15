using Confer.Domain.Enums;
using Confer.Domain.Meetings;
using Confer.Domain.Sessions;
using FluentAssertions;
using Xunit;

namespace Confer.Tests;

public class WaitingRoomAndPolicyDomainTests
{
    [Fact]
    public void CreateMeeting_WithDefaultGovernanceSettings_ShouldHaveExpectedDefaults()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("All Hands", ownerId).Value;

        meeting.IsWaitingRoomEnabled.Should().BeFalse();
        meeting.IsWatermarkEnabled.Should().BeFalse();
        meeting.Policy.Should().NotBeNull();
        meeting.Policy.AllowScreenShare.Should().BeTrue();
        meeting.Policy.AllowChat.Should().BeTrue();
        meeting.Policy.AllowUnmuteSelf.Should().BeTrue();
        meeting.Policy.MuteOnEntry.Should().BeFalse();
        meeting.Policy.AllowRename.Should().BeTrue();
    }

    [Fact]
    public void CreateMeeting_WithCustomGovernanceSettings_ShouldApplySettings()
    {
        var ownerId = Guid.NewGuid();
        var customPolicy = new MeetingPolicy(
            allowScreenShare: false,
            allowChat: false,
            allowUnmuteSelf: false,
            muteOnEntry: true,
            allowRename: false);

        var meeting = Meeting.Create(
            "Secured Meeting",
            ownerId,
            maxParticipants: 30,
            isWaitingRoomEnabled: true,
            isWatermarkEnabled: true,
            policy: customPolicy).Value;

        meeting.IsWaitingRoomEnabled.Should().BeTrue();
        meeting.IsWatermarkEnabled.Should().BeTrue();
        meeting.Policy.AllowScreenShare.Should().BeFalse();
        meeting.Policy.AllowChat.Should().BeFalse();
        meeting.Policy.AllowUnmuteSelf.Should().BeFalse();
        meeting.Policy.MuteOnEntry.Should().BeTrue();
        meeting.Policy.AllowRename.Should().BeFalse();
    }

    [Fact]
    public void ToggleWaitingRoom_ByHost_ShouldSucceed()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Room Test", ownerId).Value;

        var result = meeting.ToggleWaitingRoom(ownerId, true);
        result.IsSuccess.Should().BeTrue();
        meeting.IsWaitingRoomEnabled.Should().BeTrue();

        var toggleOff = meeting.ToggleWaitingRoom(ownerId, false);
        toggleOff.IsSuccess.Should().BeTrue();
        meeting.IsWaitingRoomEnabled.Should().BeFalse();
    }

    [Fact]
    public void ToggleWaitingRoom_ByNonHost_ShouldFailWithUnauthorized()
    {
        var ownerId = Guid.NewGuid();
        var nonHostId = Guid.NewGuid();
        var meeting = Meeting.Create("Room Test", ownerId).Value;

        var result = meeting.ToggleWaitingRoom(nonHostId, true);
        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.Unauthorized);
        meeting.IsWaitingRoomEnabled.Should().BeFalse();
    }

    [Fact]
    public void ToggleWatermark_ByHost_ShouldSucceed()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Watermark Test", ownerId).Value;

        var result = meeting.ToggleWatermark(ownerId, true);
        result.IsSuccess.Should().BeTrue();
        meeting.IsWatermarkEnabled.Should().BeTrue();

        var toggleOff = meeting.ToggleWatermark(ownerId, false);
        toggleOff.IsSuccess.Should().BeTrue();
        meeting.IsWatermarkEnabled.Should().BeFalse();
    }

    [Fact]
    public void ToggleWatermark_ByNonHost_ShouldFailWithUnauthorized()
    {
        var ownerId = Guid.NewGuid();
        var nonHostId = Guid.NewGuid();
        var meeting = Meeting.Create("Watermark Test", ownerId).Value;

        var result = meeting.ToggleWatermark(nonHostId, true);
        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.Unauthorized);
    }

    [Fact]
    public void UpdatePolicy_ByHost_ShouldSucceed()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Policy Test", ownerId).Value;
        var updatedPolicy = new MeetingPolicy(
            allowScreenShare: true,
            allowChat: false,
            allowUnmuteSelf: true,
            muteOnEntry: true,
            allowRename: false);

        var result = meeting.UpdatePolicy(ownerId, updatedPolicy);
        result.IsSuccess.Should().BeTrue();
        meeting.Policy.AllowChat.Should().BeFalse();
        meeting.Policy.MuteOnEntry.Should().BeTrue();
        meeting.Policy.AllowRename.Should().BeFalse();
    }

    [Fact]
    public void UpdatePolicy_ByNonHost_ShouldFailWithUnauthorized()
    {
        var ownerId = Guid.NewGuid();
        var nonHostId = Guid.NewGuid();
        var meeting = Meeting.Create("Policy Test", ownerId).Value;

        var result = meeting.UpdatePolicy(nonHostId, new MeetingPolicy());
        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.Unauthorized);
    }

    [Fact]
    public void AdmitParticipant_WhenInWaitingRoom_ShouldTransitionToAdmitted()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Waiting Room", ownerId, isWaitingRoomEnabled: true).Value;
        var session = Session.Create(meeting.Id);
        meeting.Sessions.Add(session);

        var participantUserId = Guid.NewGuid();
        var addResult = session.AddParticipant(participantUserId, "Alice", ParticipantRole.Participant, status: ParticipationStatus.InWaitingRoom);
        addResult.IsSuccess.Should().BeTrue();
        var participant = addResult.Value;
        participant.Status.Should().Be(ParticipationStatus.InWaitingRoom);

        var admitResult = meeting.AdmitParticipant(ownerId, participant.Id);
        admitResult.IsSuccess.Should().BeTrue();
        participant.Status.Should().Be(ParticipationStatus.Admitted);
        participant.AdmittedAt.Should().NotBeNull();
    }

    [Fact]
    public void AdmitParticipant_WhenAlreadyAdmitted_ShouldFail()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Waiting Room", ownerId).Value;
        var session = Session.Create(meeting.Id);
        meeting.Sessions.Add(session);

        var participantUserId = Guid.NewGuid();
        var participant = session.AddParticipant(participantUserId, "Bob", ParticipantRole.Participant, status: ParticipationStatus.Admitted).Value;

        var result = meeting.AdmitParticipant(ownerId, participant.Id);
        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.ParticipantNotInWaitingRoom);
    }

    [Fact]
    public void AdmitAll_ShouldAdmitAllWaitingParticipants()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Waiting Room", ownerId, isWaitingRoomEnabled: true).Value;
        var session = Session.Create(meeting.Id);
        meeting.Sessions.Add(session);

        var p1 = session.AddParticipant(Guid.NewGuid(), "Alice", ParticipantRole.Participant, status: ParticipationStatus.InWaitingRoom).Value;
        var p2 = session.AddParticipant(Guid.NewGuid(), "Bob", ParticipantRole.Participant, status: ParticipationStatus.InWaitingRoom).Value;
        var p3 = session.AddParticipant(Guid.NewGuid(), "Charlie", ParticipantRole.Participant, status: ParticipationStatus.Admitted).Value;

        var admitAllResult = meeting.AdmitAll(ownerId);
        admitAllResult.IsSuccess.Should().BeTrue();
        admitAllResult.Value.Should().HaveCount(2);

        p1.Status.Should().Be(ParticipationStatus.Admitted);
        p2.Status.Should().Be(ParticipationStatus.Admitted);
        p3.Status.Should().Be(ParticipationStatus.Admitted);
    }

    [Fact]
    public void RejectParticipant_WhenInWaitingRoom_ShouldTransitionToRejectedAndKicked()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Waiting Room", ownerId, isWaitingRoomEnabled: true).Value;
        var session = Session.Create(meeting.Id);
        meeting.Sessions.Add(session);

        var participantUserId = Guid.NewGuid();
        var participant = session.AddParticipant(participantUserId, "Dave", ParticipantRole.Participant, status: ParticipationStatus.InWaitingRoom).Value;

        var rejectResult = meeting.RejectParticipant(ownerId, participant.Id);
        rejectResult.IsSuccess.Should().BeTrue();
        participant.Status.Should().Be(ParticipationStatus.Rejected);
        participant.LeftAt.Should().NotBeNull();
        participant.LeaveReason.Should().Be(LeaveReason.Kicked);
    }
}
