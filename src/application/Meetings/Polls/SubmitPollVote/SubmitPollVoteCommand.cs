using Confer.Application.DTOs;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Polls.SubmitPollVote;

public record SubmitPollVoteCommand(
    Guid MeetingId,
    Guid PollId,
    Guid VoterId,
    List<Guid> OptionIds
) : ICommand<Result<PollDto>>;
