using Confer.Shared.Results;

namespace Confer.Domain.Meetings;

public sealed class ActionItem
{
    public string Title { get; private set; } = string.Empty;
    public string Assignee { get; private set; } = string.Empty;
    public string Status { get; private set; } = "pending";

    private ActionItem() { }

    public ActionItem(string title, string assignee, string status = "pending")
    {
        Title = title?.Trim() ?? string.Empty;
        Assignee = assignee?.Trim() ?? string.Empty;
        Status = string.IsNullOrWhiteSpace(status) ? "pending" : status.Trim();
    }
}

public sealed class MeetingSummary
{
    public Guid Id { get; private set; }
    public Guid MeetingId { get; private set; }
    public string Overview { get; private set; } = string.Empty;
    public List<string> KeyDecisions { get; private set; } = new();
    public List<ActionItem> ActionItems { get; private set; } = new();
    public DateTime GeneratedAt { get; private set; } = DateTime.UtcNow;
    public int DurationMinutes { get; private set; }
    public int ParticipantCount { get; private set; }

    private MeetingSummary() { }

    public static Result<MeetingSummary> Create(
        Guid meetingId,
        string overview,
        List<string>? keyDecisions = null,
        List<ActionItem>? actionItems = null,
        int durationMinutes = 0,
        int participantCount = 0)
    {
        if (meetingId == Guid.Empty)
            return Result.Failure<MeetingSummary>(Error.Validation("MeetingSummary.MeetingIdEmpty", "Meeting ID cannot be empty."));

        if (string.IsNullOrWhiteSpace(overview))
            return Result.Failure<MeetingSummary>(Error.Validation("MeetingSummary.OverviewEmpty", "Overview cannot be empty."));

        var summary = new MeetingSummary
        {
            Id = Guid.NewGuid(),
            MeetingId = meetingId,
            Overview = overview.Trim(),
            KeyDecisions = keyDecisions?
                .Where(d => !string.IsNullOrWhiteSpace(d))
                .Select(d => d.Trim())
                .ToList() ?? new List<string>(),
            ActionItems = actionItems?
                .Where(a => !string.IsNullOrWhiteSpace(a.Title))
                .ToList() ?? new List<ActionItem>(),
            GeneratedAt = DateTime.UtcNow,
            DurationMinutes = Math.Max(0, durationMinutes),
            ParticipantCount = Math.Max(0, participantCount)
        };

        return Result.Success(summary);
    }
}
