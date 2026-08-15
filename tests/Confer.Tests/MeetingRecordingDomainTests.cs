using Confer.Domain.Enums;
using Confer.Domain.Meetings;
using FluentAssertions;
using Xunit;

namespace Confer.Tests;

public class MeetingRecordingDomainTests
{
    [Fact]
    public void StartRecording_AsOwner_ShouldSucceedAndSetState()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Team Sync", ownerId).Value;

        var result = meeting.StartRecording(ownerId);

        result.IsSuccess.Should().BeTrue();
        result.Value.Should().NotBeNull();
        result.Value.MeetingId.Should().Be(meeting.Id);
        result.Value.InitiatedBy.Should().Be(ownerId);
        result.Value.Status.Should().Be(RecordingStatus.Recording);
        meeting.IsRecording.Should().BeTrue();
        meeting.RecordingStartedAt.Should().NotBeNull();
        meeting.Recordings.Should().ContainSingle();
    }

    [Fact]
    public void StartRecording_WhenAlreadyRecording_ShouldFailWithAlreadyRecording()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Team Sync", ownerId).Value;

        var first = meeting.StartRecording(ownerId);
        first.IsSuccess.Should().BeTrue();

        var second = meeting.StartRecording(ownerId);
        second.IsFailure.Should().BeTrue();
        second.Error.Should().Be(MeetingErrors.AlreadyRecording);
    }

    [Fact]
    public void StartRecording_ByNonOwner_ShouldFailWithUnauthorized()
    {
        var ownerId = Guid.NewGuid();
        var nonOwnerId = Guid.NewGuid();
        var meeting = Meeting.Create("Team Sync", ownerId).Value;

        var result = meeting.StartRecording(nonOwnerId);

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.Unauthorized);
        meeting.IsRecording.Should().BeFalse();
    }

    [Fact]
    public void StartRecording_OnEndedMeeting_ShouldFailWithEnded()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Team Sync", ownerId).Value;
        meeting.End(ownerId);

        var result = meeting.StartRecording(ownerId);

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.Ended);
    }

    [Fact]
    public void StopRecording_WhenRecording_ShouldCompleteActiveRecording()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Architecture Review", ownerId).Value;
        meeting.StartRecording(ownerId);

        var stopResult = meeting.StopRecording(ownerId, "/var/recordings/test.webm", 1048576);

        stopResult.IsSuccess.Should().BeTrue();
        stopResult.Value.Status.Should().Be(RecordingStatus.Completed);
        stopResult.Value.StoragePath.Should().Be("/var/recordings/test.webm");
        stopResult.Value.FileSizeBytes.Should().Be(1048576);
        stopResult.Value.StoppedAt.Should().NotBeNull();
        meeting.IsRecording.Should().BeFalse();
    }

    [Fact]
    public void StopRecording_WhenNotRecording_ShouldFailWithNotRecording()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Architecture Review", ownerId).Value;

        var stopResult = meeting.StopRecording(ownerId);

        stopResult.IsFailure.Should().BeTrue();
        stopResult.Error.Should().Be(MeetingErrors.NotRecording);
    }

    [Fact]
    public void StopRecording_ByNonOwner_ShouldFailWithUnauthorized()
    {
        var ownerId = Guid.NewGuid();
        var nonOwnerId = Guid.NewGuid();
        var meeting = Meeting.Create("Architecture Review", ownerId).Value;
        meeting.StartRecording(ownerId);

        var stopResult = meeting.StopRecording(nonOwnerId);

        stopResult.IsFailure.Should().BeTrue();
        stopResult.Error.Should().Be(MeetingErrors.Unauthorized);
        meeting.IsRecording.Should().BeTrue();
    }

    [Fact]
    public void EndingMeeting_WhileRecording_ShouldAutoStopRecording()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Live Demo", ownerId).Value;
        meeting.StartRecording(ownerId);

        var endResult = meeting.End(ownerId);

        endResult.IsSuccess.Should().BeTrue();
        meeting.IsRecording.Should().BeFalse();
        meeting.EndedAt.Should().NotBeNull();
        meeting.Recordings.First().Status.Should().Be(RecordingStatus.Completed);
    }
}
