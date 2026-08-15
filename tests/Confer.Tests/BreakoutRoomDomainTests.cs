using Confer.Domain.Meetings;
using FluentAssertions;
using Xunit;

namespace Confer.Tests;

public class BreakoutRoomDomainTests
{
    [Fact]
    public void CreateBreakoutRoom_WithValidParameters_ShouldSucceed()
    {
        var meetingId = Guid.NewGuid();
        var result = BreakoutRoom.Create(meetingId, "Discussion Room A", 1, 20);

        result.IsSuccess.Should().BeTrue();
        result.Value.MeetingId.Should().Be(meetingId);
        result.Value.Name.Should().Be("Discussion Room A");
        result.Value.Index.Should().Be(1);
        result.Value.MaxDurationMinutes.Should().Be(20);
        result.Value.EndsAt.Should().BeNull();
    }

    [Theory]
    [InlineData("")]
    [InlineData("   ")]
    public void CreateBreakoutRoom_WithEmptyName_ShouldFail(string name)
    {
        var result = BreakoutRoom.Create(Guid.NewGuid(), name, 1, 15);
        result.IsFailure.Should().BeTrue();
    }

    [Theory]
    [InlineData(0)]
    [InlineData(-5)]
    [InlineData(200)]
    public void CreateBreakoutRoom_WithInvalidDuration_ShouldFail(int duration)
    {
        var result = BreakoutRoom.Create(Guid.NewGuid(), "Room", 1, duration);
        result.IsFailure.Should().BeTrue();
        result.Error.Should().Be(BreakoutErrors.InvalidDuration);
    }

    [Fact]
    public void CloseBreakoutRoom_ShouldSetEndsAt()
    {
        var room = BreakoutRoom.Create(Guid.NewGuid(), "Room", 1, 15).Value;
        room.Close();

        room.EndsAt.Should().NotBeNull();
    }
}
