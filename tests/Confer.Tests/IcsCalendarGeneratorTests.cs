using Confer.Domain.Identity;
using Confer.Domain.Meetings;
using Confer.Infrastructure.Calendar;
using FluentAssertions;
using Xunit;

namespace Confer.Tests;

public class IcsCalendarGeneratorTests
{
    private readonly IcsCalendarGenerator _generator = new();

    [Fact]
    public void GenerateIcs_ShouldProduceValidRfc5545Content()
    {
        var startTime = new DateTime(2026, 8, 20, 15, 0, 0, DateTimeKind.Utc);
        var endTime = new DateTime(2026, 8, 20, 16, 0, 0, DateTimeKind.Utc);
        var meetingId = Guid.NewGuid();

        var ics = _generator.GenerateIcs(
            title: "Q3 Engineering Sync",
            joinCode: "sync-q3",
            startTime: startTime,
            endTime: endTime,
            organizerName: "Alice Lead",
            organizerEmail: "alice@confer.local",
            baseUrl: "https://confer.app",
            meetingId: meetingId
        );

        ics.Should().StartWith("BEGIN:VCALENDAR\r\n");
        ics.Should().EndWith("END:VCALENDAR\r\n");
        ics.Should().Contain("VERSION:2.0\r\n");
        ics.Should().Contain("PRODID:-//Confer Inc//Confer Video Meetings//EN\r\n");
        ics.Should().Contain("BEGIN:VEVENT\r\n");
        ics.Should().Contain($"UID:{meetingId}@confer.local\r\n");
        ics.Should().Contain("DTSTART:20260820T150000Z\r\n");
        ics.Should().Contain("DTEND:20260820T160000Z\r\n");
        ics.Should().Contain("SUMMARY:Q3 Engineering Sync\r\n");
        ics.Should().Contain("URL:https://confer.app/join/sync-q3\r\n");
        ics.Should().Contain("LOCATION:Confer Meeting (Code: sync-q3)\r\n");
        ics.Should().Contain("ORGANIZER;CN=Alice Lead:mailto:alice@confer.local\r\n");
        ics.Should().Contain("STATUS:CONFIRMED\r\n");
        ics.Should().Contain("BEGIN:VALARM\r\n");
        ics.Should().Contain("TRIGGER:-PT15M\r\n");
        ics.Should().Contain("ACTION:DISPLAY\r\n");
        ics.Should().Contain("DESCRIPTION:Reminder: Q3 Engineering Sync\r\n");
        ics.Should().Contain("END:VALARM\r\n");
        ics.Should().Contain("END:VEVENT\r\n");

        // Verify direct join link and code in description
        ics.Should().Contain("https://confer.app/join/sync-q3");
        ics.Should().Contain("sync-q3");
    }

    [Fact]
    public void GenerateMeetingIcs_WithMeetingAndOrganizerEntities_ShouldFormatCorrectly()
    {
        var ownerId = Guid.NewGuid();
        var meeting = Meeting.Create("Architecture Review", ownerId, 25, "arch-123").Value;
        var organizer = User.CreateWithId(ownerId, "lead@confer.local", "Tech Lead");

        var ics = _generator.GenerateMeetingIcs(meeting, organizer, "http://localhost:5000", durationMinutes: 45);

        ics.Should().Contain($"UID:{meeting.Id}@confer.local\r\n");
        ics.Should().Contain("SUMMARY:Architecture Review\r\n");
        ics.Should().Contain("ORGANIZER;CN=Tech Lead:mailto:lead@confer.local\r\n");
        ics.Should().Contain("http://localhost:5000/join/arch-123");
        ics.Should().Contain("arch-123");
        ics.Should().Contain("BEGIN:VALARM\r\n");
        ics.Should().Contain("TRIGGER:-PT15M\r\n");
    }

    [Fact]
    public void GenerateIcs_WithSpecialCharacters_ShouldProperlyEscapePerRfc5545()
    {
        var title = "Sprint Review, Planning; Demo & Retrospective\\Wrap-up";
        var ics = _generator.GenerateIcs(
            title: title,
            joinCode: "demo-code",
            startTime: DateTime.UtcNow,
            endTime: DateTime.UtcNow.AddHours(1)
        );

        // Commas, semicolons, and backslashes should be escaped
        ics.Should().Contain("SUMMARY:Sprint Review\\, Planning\\; Demo & Retrospective\\\\Wrap-up\r\n");
        ics.Should().Contain("Reminder: Sprint Review\\, Planning\\; Demo & Retrospective\\\\Wrap-up\r\n");
    }

    [Fact]
    public void GenerateIcs_AllLinesShouldEndWithCrLf()
    {
        var ics = _generator.GenerateIcs(
            title: "Line Endings Test",
            joinCode: "line-test",
            startTime: DateTime.UtcNow,
            endTime: DateTime.UtcNow.AddHours(1)
        );

        var lines = ics.Split(new[] { "\r\n" }, StringSplitOptions.None);
        lines.Length.Should().BeGreaterThan(10);
        // Ensure no standalone '\n' without '\r'
        var normalized = ics.Replace("\r\n", "");
        normalized.Should().NotContain("\n");
        normalized.Should().NotContain("\r");
    }
}
