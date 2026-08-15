using Confer.Domain.Enums;
using Confer.Domain.Meetings;
using FluentAssertions;
using Xunit;

namespace Confer.Tests;

public class LiveStreamDomainTests
{
    [Fact]
    public void CreateMeeting_ShouldHaveDefaultLiveStreamConfig()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Team Sync", ownerId).Value;

        meeting.LiveStreamConfig.Should().NotBeNull();
        meeting.LiveStreamConfig.IsStreamingEnabled.Should().BeFalse();
        meeting.LiveStreamConfig.Status.Should().Be(LiveStreamStatus.Idle);
        meeting.LiveStreamConfig.RtmpUrl.Should().BeEmpty();
        meeting.LiveStreamConfig.StreamKey.Should().BeEmpty();
        meeting.LiveStreamConfig.StartedAt.Should().BeNull();
        meeting.IsLiveStreaming.Should().BeFalse();
    }

    [Fact]
    public void StartLiveStream_ByOwner_WithValidConfig_ShouldSucceed()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Keynote Presentation", ownerId).Value;
        var rtmpUrl = "rtmp://live.youtube.com/app";
        var streamKey = "live_stream_key_abc123";

        var result = meeting.StartLiveStream(ownerId, rtmpUrl, streamKey);

        result.IsSuccess.Should().BeTrue();
        meeting.IsLiveStreaming.Should().BeTrue();
        meeting.LiveStreamConfig.IsStreamingEnabled.Should().BeTrue();
        meeting.LiveStreamConfig.Status.Should().Be(LiveStreamStatus.Live);
        meeting.LiveStreamConfig.RtmpUrl.Should().Be(rtmpUrl);
        meeting.LiveStreamConfig.StreamKey.Should().Be(streamKey);
        meeting.LiveStreamConfig.StartedAt.Should().NotBeNull();
    }

    [Fact]
    public void StartLiveStream_ByNonOwner_ShouldFailWithUnauthorized()
    {
        var ownerId = Guid.NewGuid();
        var nonOwnerId = Guid.NewGuid();
        var meeting = Meeting.Create("Keynote Presentation", ownerId).Value;

        var result = meeting.StartLiveStream(nonOwnerId, "rtmp://live.youtube.com/app", "key123");

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.Unauthorized);
        meeting.IsLiveStreaming.Should().BeFalse();
    }

    [Theory]
    [InlineData("", "key123")]
    [InlineData("   ", "key123")]
    [InlineData(null, "key123")]
    public void StartLiveStream_WithEmptyRtmpUrl_ShouldFail(string? rtmpUrl, string streamKey)
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Keynote", ownerId).Value;

        var result = meeting.StartLiveStream(ownerId, rtmpUrl!, streamKey);

        result.IsFailure.Should().BeTrue();
        result.Error.Code.Should().Be("LiveStream.InvalidRtmpUrl");
    }

    [Theory]
    [InlineData("rtmp://live.twitch.tv/app", "")]
    [InlineData("rtmp://live.twitch.tv/app", "   ")]
    [InlineData("rtmp://live.twitch.tv/app", null)]
    public void StartLiveStream_WithEmptyStreamKey_ShouldFail(string rtmpUrl, string? streamKey)
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Keynote", ownerId).Value;

        var result = meeting.StartLiveStream(ownerId, rtmpUrl, streamKey!);

        result.IsFailure.Should().BeTrue();
        result.Error.Code.Should().Be("LiveStream.InvalidStreamKey");
    }

    [Fact]
    public void StartLiveStream_WhenAlreadyLive_ShouldFailWithConflict()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Townhall", ownerId).Value;
        meeting.StartLiveStream(ownerId, "rtmp://live.twitch.tv/app", "key123");

        var result = meeting.StartLiveStream(ownerId, "rtmp://live.twitch.tv/app", "key456");

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.AlreadyStreaming);
    }

    [Fact]
    public void StartLiveStream_OnEndedMeeting_ShouldFailWithEnded()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Townhall", ownerId).Value;
        meeting.End(ownerId);

        var result = meeting.StartLiveStream(ownerId, "rtmp://live.twitch.tv/app", "key123");

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.Ended);
    }

    [Fact]
    public void StopLiveStream_ByOwner_WhenLive_ShouldSucceed()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Townhall", ownerId).Value;
        meeting.StartLiveStream(ownerId, "rtmp://live.twitch.tv/app", "key123");

        var result = meeting.StopLiveStream(ownerId);

        result.IsSuccess.Should().BeTrue();
        meeting.IsLiveStreaming.Should().BeFalse();
        meeting.LiveStreamConfig.IsStreamingEnabled.Should().BeFalse();
        meeting.LiveStreamConfig.Status.Should().Be(LiveStreamStatus.Idle);
        meeting.LiveStreamConfig.StartedAt.Should().BeNull();
    }

    [Fact]
    public void StopLiveStream_ByNonOwner_ShouldFailWithUnauthorized()
    {
        var ownerId = Guid.NewGuid();
        var nonOwnerId = Guid.NewGuid();
        var meeting = Meeting.Create("Townhall", ownerId).Value;
        meeting.StartLiveStream(ownerId, "rtmp://live.twitch.tv/app", "key123");

        var result = meeting.StopLiveStream(nonOwnerId);

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.Unauthorized);
        meeting.IsLiveStreaming.Should().BeTrue();
    }

    [Fact]
    public void StopLiveStream_WhenNotStreaming_ShouldFailWithNotStreaming()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Townhall", ownerId).Value;

        var result = meeting.StopLiveStream(ownerId);

        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(MeetingErrors.NotStreaming);
    }

    [Fact]
    public void EndMeeting_WhenLiveStreaming_ShouldResetStreamStatusToIdle()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Townhall", ownerId).Value;
        meeting.StartLiveStream(ownerId, "rtmp://live.twitch.tv/app", "key123");
        meeting.IsLiveStreaming.Should().BeTrue();

        var endResult = meeting.End(ownerId);

        endResult.IsSuccess.Should().BeTrue();
        meeting.IsLiveStreaming.Should().BeFalse();
        meeting.LiveStreamConfig.Status.Should().Be(LiveStreamStatus.Idle);
        meeting.LiveStreamConfig.IsStreamingEnabled.Should().BeFalse();
        meeting.LiveStreamConfig.StartedAt.Should().BeNull();
    }
}
