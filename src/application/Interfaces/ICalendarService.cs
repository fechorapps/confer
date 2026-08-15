using Confer.Domain.Identity;
using Confer.Domain.Meetings;

namespace Confer.Application.Interfaces;

public interface ICalendarService
{
    string GenerateMeetingIcs(
        Meeting meeting,
        User? organizer = null,
        string? baseUrl = null,
        int durationMinutes = 60);

    string GenerateIcs(
        string title,
        string joinCode,
        DateTime startTime,
        DateTime endTime,
        string? organizerName = null,
        string? organizerEmail = null,
        string? baseUrl = null,
        Guid? meetingId = null);
}
