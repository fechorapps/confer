using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Join;

public record JoinMeetingCommand(
    Guid MeetingId,
    Guid UserId,
    string DisplayName,
    string? ClientInfo = null
) : ICommand<Result<JoinMeetingResponse>>;
