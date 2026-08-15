using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.GetRecordings;

public record GetMeetingRecordingsQuery(
    Guid MeetingId
) : IQuery<Result<List<MeetingRecordingDto>>>;
