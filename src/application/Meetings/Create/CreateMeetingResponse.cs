using Confer.Application.DTOs;

namespace Confer.Application.Meetings.Create;

public record CreateMeetingResponse(
    Guid Id,
    string JoinCode,
    string Title,
    Guid OwnerId,
    int MaxParticipants,
    DateTime CreatedAt,
    bool IsWaitingRoomEnabled = false,
    bool IsWatermarkEnabled = false,
    MeetingPolicyDto? Policy = null
);
