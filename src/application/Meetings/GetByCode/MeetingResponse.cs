namespace Confer.Application.Meetings.GetByCode;

public record MeetingResponse(
    Guid Id,
    string JoinCode,
    string Title,
    Guid OwnerId,
    int MaxParticipants,
    bool IsLocked,
    bool HasEnded,
    DateTime CreatedAt,
    bool IsRecording = false
);
