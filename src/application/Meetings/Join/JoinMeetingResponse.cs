using Confer.Application.DTOs;

namespace Confer.Application.Meetings.Join;

public record JoinMeetingResponse(
    Guid MeetingId,
    string Title,
    Guid ParticipantId,
    string Role,
    bool IsLocked,
    string WsUrl,
    string RoomToken,
    List<IceServerConfig> IceServers
);
