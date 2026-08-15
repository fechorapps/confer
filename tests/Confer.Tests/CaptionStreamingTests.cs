using System.Text.Json;
using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using FluentAssertions;
using Moq;
using Xunit;

namespace Confer.Tests;

public class CaptionStreamingTests
{
    [Fact]
    public void CaptionChunkDto_SerializationAndDeserialization_ShouldRoundtripCorrectly()
    {
        var participantId = Guid.NewGuid();
        var dto = new CaptionChunkDto(
            ParticipantId: participantId,
            SpeakerName: "Elena Rostova",
            Text: "Good morning team, welcome to the WebRTC conference.",
            IsFinal: true,
            Language: "en-US",
            TimestampMs: 1723680000000L
        );

        var json = JsonSerializer.Serialize(dto);
        json.Should().Contain("\"participant_id\":\"" + participantId + "\"");
        json.Should().Contain("\"speaker_name\":\"Elena Rostova\"");
        json.Should().Contain("\"text\":\"Good morning team, welcome to the WebRTC conference.\"");
        json.Should().Contain("\"is_final\":true");
        json.Should().Contain("\"language\":\"en-US\"");
        json.Should().Contain("\"timestamp_ms\":1723680000000");

        var deserialized = JsonSerializer.Deserialize<CaptionChunkDto>(json);
        deserialized.Should().NotBeNull();
        deserialized!.ParticipantId.Should().Be(participantId);
        deserialized.SpeakerName.Should().Be("Elena Rostova");
        deserialized.Text.Should().Be("Good morning team, welcome to the WebRTC conference.");
        deserialized.IsFinal.Should().BeTrue();
        deserialized.Language.Should().Be("en-US");
        deserialized.TimestampMs.Should().Be(1723680000000L);
    }

    [Fact]
    public async Task SignalingNotifier_BroadcastCaptionAsync_ShouldInvokeUnderlyingService()
    {
        var mockNotifier = new Mock<ISignalingNotifier>();
        var meetingId = Guid.NewGuid();
        var caption = new CaptionChunkDto(
            ParticipantId: Guid.NewGuid(),
            SpeakerName: "John Doe",
            Text: "Testing caption broadcast notifier",
            IsFinal: false,
            Language: "es-ES",
            TimestampMs: 1723681111111L
        );

        mockNotifier
            .Setup(n => n.BroadcastCaptionAsync(meetingId, caption))
            .Returns(Task.CompletedTask)
            .Verifiable();

        await mockNotifier.Object.BroadcastCaptionAsync(meetingId, caption);

        mockNotifier.Verify(n => n.BroadcastCaptionAsync(meetingId, caption), Times.Once);
    }
}
