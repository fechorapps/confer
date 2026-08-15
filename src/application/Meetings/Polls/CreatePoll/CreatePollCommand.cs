using Confer.Application.DTOs;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Polls.CreatePoll;

public record CreatePollCommand(
    Guid MeetingId,
    Guid CreatorId,
    string Question,
    List<string> Options,
    bool IsAnonymous = false,
    bool IsMultiChoice = false
) : ICommand<Result<PollDto>>;
