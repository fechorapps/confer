using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Summary;

public record GetMeetingSummaryQuery(
    Guid MeetingId
) : IQuery<Result<MeetingSummaryDto>>;
