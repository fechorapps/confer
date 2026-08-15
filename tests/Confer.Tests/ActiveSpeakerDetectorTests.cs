using Confer.Infrastructure.Media;
using FluentAssertions;
using Xunit;

namespace Confer.Tests;

public class ActiveSpeakerDetectorTests
{
    [Fact]
    public void GetTopSpeakers_WhenMultipleSpeakersReportAudio_ShouldRankByLoudness()
    {
        var detector = new ActiveSpeakerDetector();
        var speaker1 = Guid.NewGuid();
        var speaker2 = Guid.NewGuid();
        var speaker3 = Guid.NewGuid();
        var quietSpeaker = Guid.NewGuid();

        detector.ReportAudioLevel(speaker1, -10); // Loudest
        detector.ReportAudioLevel(speaker2, -25); // Medium
        detector.ReportAudioLevel(speaker3, -35); // Soft
        detector.ReportAudioLevel(quietSpeaker, -90); // Below threshold

        var topSpeakers = detector.GetTopSpeakers(3);

        topSpeakers.Should().HaveCount(3);
        topSpeakers[0].ParticipantId.Should().Be(speaker1);
        topSpeakers[1].ParticipantId.Should().Be(speaker2);
        topSpeakers[2].ParticipantId.Should().Be(speaker3);
    }

    [Fact]
    public void GetTopSpeakers_WhenParticipantRemoved_ShouldNotAppearInRankings()
    {
        var detector = new ActiveSpeakerDetector();
        var speaker1 = Guid.NewGuid();

        detector.ReportAudioLevel(speaker1, -15);
        detector.RemoveParticipant(speaker1);

        var topSpeakers = detector.GetTopSpeakers(3);
        topSpeakers.Should().BeEmpty();
    }
}
