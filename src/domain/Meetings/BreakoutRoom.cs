using Confer.Shared.Results;

namespace Confer.Domain.Meetings;

public sealed class BreakoutRoom
{
    public Guid Id { get; private set; }
    public Guid MeetingId { get; private set; }
    public Guid SessionId { get; private set; }
    public string Name { get; private set; } = string.Empty;
    public int Index { get; private set; }
    public DateTime CreatedAt { get; private set; } = DateTime.UtcNow;
    public DateTime? EndsAt { get; private set; }
    public int MaxDurationMinutes { get; private set; }

    private BreakoutRoom() { }

    public static Result<BreakoutRoom> Create(
        Guid meetingId,
        string name,
        int index,
        int maxDurationMinutes = 15,
        Guid? sessionId = null)
    {
        if (meetingId == Guid.Empty)
            return Result.Failure<BreakoutRoom>(Error.Validation("Breakout.MeetingIdEmpty", "Meeting ID cannot be empty."));

        if (string.IsNullOrWhiteSpace(name))
            return Result.Failure<BreakoutRoom>(Error.Validation("Breakout.NameEmpty", "Breakout room name cannot be empty."));

        if (maxDurationMinutes is < 1 or > 180)
            return Result.Failure<BreakoutRoom>(BreakoutErrors.InvalidDuration);

        return Result.Success(new BreakoutRoom
        {
            Id = Guid.NewGuid(),
            MeetingId = meetingId,
            SessionId = sessionId ?? Guid.NewGuid(),
            Name = name.Trim(),
            Index = index,
            CreatedAt = DateTime.UtcNow,
            MaxDurationMinutes = maxDurationMinutes
        });
    }

    public void Close()
    {
        EndsAt = DateTime.UtcNow;
    }
}
