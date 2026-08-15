using Confer.Application.DTOs;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Polls.GetPolls;

public record GetPollsQuery(Guid MeetingId) : IQuery<Result<List<PollDto>>>;
