using Confer.Application.DTOs;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Polls.ClosePoll;

public record ClosePollCommand(
    Guid MeetingId,
    Guid PollId,
    Guid ActorId
) : ICommand<Result<PollDto>>;
