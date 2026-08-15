namespace Confer.Application.Meetings.Create;

public record CreateMeetingResponse(
    Guid Id,
    string JoinCode,
    string Title,
    Guid OwnerId,
    int MaxParticipants,
    DateTime CreatedAt
);
