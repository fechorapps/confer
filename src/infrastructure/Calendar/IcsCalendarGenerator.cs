using System.Globalization;
using System.Text;
using Confer.Application.Interfaces;
using Confer.Domain.Identity;
using Confer.Domain.Meetings;

namespace Confer.Infrastructure.Calendar;

public sealed class IcsCalendarGenerator : ICalendarService
{
    public string GenerateMeetingIcs(
        Meeting meeting,
        User? organizer = null,
        string? baseUrl = null,
        int durationMinutes = 60)
    {
        var startTime = meeting.ScheduledStart ?? meeting.CreatedAt;
        var endTime = meeting.EndedAt ?? startTime.AddMinutes(durationMinutes);
        var organizerName = organizer?.DisplayName;
        var organizerEmail = organizer?.Email;

        return GenerateIcs(
            meeting.Title,
            meeting.JoinCode,
            startTime,
            endTime,
            organizerName,
            organizerEmail,
            baseUrl,
            meeting.Id);
    }

    public string GenerateIcs(
        string title,
        string joinCode,
        DateTime startTime,
        DateTime endTime,
        string? organizerName = null,
        string? organizerEmail = null,
        string? baseUrl = null,
        Guid? meetingId = null)
    {
        var uid = meetingId.HasValue && meetingId.Value != Guid.Empty
            ? $"{meetingId.Value}@confer.local"
            : $"{Guid.NewGuid()}@confer.local";

        var hostUrl = string.IsNullOrWhiteSpace(baseUrl) ? "https://confer.app" : baseUrl.TrimEnd('/');
        var joinUrl = $"{hostUrl}/join/{joinCode}";

        var dtStamp = DateTime.UtcNow.ToString("yyyyMMdd'T'HHmmss'Z'", CultureInfo.InvariantCulture);
        var dtStart = startTime.ToUniversalTime().ToString("yyyyMMdd'T'HHmmss'Z'", CultureInfo.InvariantCulture);
        var dtEnd = endTime.ToUniversalTime().ToString("yyyyMMdd'T'HHmmss'Z'", CultureInfo.InvariantCulture);

        var descriptionBuilder = new StringBuilder();
        descriptionBuilder.Append($"Confer Video Meeting: {title}\n\n");
        descriptionBuilder.Append($"Join Confer Meeting:\n{joinUrl}\n\n");
        descriptionBuilder.Append($"Meeting Join Code: {joinCode}");

        var escapedTitle = EscapeText(title);
        var escapedDescription = EscapeText(descriptionBuilder.ToString());
        var escapedLocation = EscapeText($"Confer Meeting (Code: {joinCode})");

        var sb = new StringBuilder();
        sb.Append("BEGIN:VCALENDAR\r\n");
        sb.Append("VERSION:2.0\r\n");
        sb.Append("PRODID:-//Confer Inc//Confer Video Meetings//EN\r\n");
        sb.Append("CALSCALE:GREGORIAN\r\n");
        sb.Append("METHOD:REQUEST\r\n");
        sb.Append("BEGIN:VEVENT\r\n");
        sb.Append($"UID:{uid}\r\n");
        sb.Append($"DTSTAMP:{dtStamp}\r\n");
        sb.Append($"DTSTART:{dtStart}\r\n");
        sb.Append($"DTEND:{dtEnd}\r\n");
        sb.Append($"SUMMARY:{escapedTitle}\r\n");
        sb.Append($"DESCRIPTION:{escapedDescription}\r\n");
        sb.Append($"LOCATION:{escapedLocation}\r\n");
        sb.Append($"URL:{joinUrl}\r\n");

        if (!string.IsNullOrWhiteSpace(organizerEmail) && !string.IsNullOrWhiteSpace(organizerName))
        {
            sb.Append($"ORGANIZER;CN={EscapeText(organizerName)}:mailto:{organizerEmail.Trim()}\r\n");
        }
        else if (!string.IsNullOrWhiteSpace(organizerEmail))
        {
            sb.Append($"ORGANIZER:mailto:{organizerEmail.Trim()}\r\n");
        }
        else if (!string.IsNullOrWhiteSpace(organizerName))
        {
            sb.Append($"ORGANIZER;CN={EscapeText(organizerName)}:mailto:host@confer.local\r\n");
        }
        else
        {
            sb.Append("ORGANIZER;CN=Confer Host:mailto:host@confer.local\r\n");
        }

        sb.Append("STATUS:CONFIRMED\r\n");
        sb.Append("SEQUENCE:0\r\n");
        sb.Append("BEGIN:VALARM\r\n");
        sb.Append("TRIGGER:-PT15M\r\n");
        sb.Append("ACTION:DISPLAY\r\n");
        sb.Append($"DESCRIPTION:Reminder: {escapedTitle}\r\n");
        sb.Append("END:VALARM\r\n");
        sb.Append("END:VEVENT\r\n");
        sb.Append("END:VCALENDAR\r\n");

        return sb.ToString();
    }

    private static string EscapeText(string? input)
    {
        if (string.IsNullOrEmpty(input))
            return string.Empty;

        return input
            .Replace("\\", "\\\\")
            .Replace(";", "\\;")
            .Replace(",", "\\,")
            .Replace("\r\n", "\\n")
            .Replace("\n", "\\n")
            .Replace("\r", "\\n");
    }
}
