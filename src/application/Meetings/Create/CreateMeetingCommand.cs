using Confer.Application.DTOs;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Create;

public record CreateMeetingCommand(
    string Title,
    Guid OwnerId,
    int MaxParticipants = 50,
    string? CustomJoinCode = null,
    bool IsWaitingRoomEnabled = false,
    bool IsWatermarkEnabled = false,
    MeetingPolicyDto? Policy = null
) : ICommand<Result<CreateMeetingResponse>>;
