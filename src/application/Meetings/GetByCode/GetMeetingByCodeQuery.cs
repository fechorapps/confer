using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.GetByCode;

public record GetMeetingByCodeQuery(string Code) : IQuery<Result<MeetingResponse>>;
