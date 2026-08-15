using Confer.Application.Interfaces;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.GetRecordings;

public sealed class GetMeetingRecordingsQueryHandler(IConferDbContext dbContext)
    : IQueryHandler<GetMeetingRecordingsQuery, Result<List<MeetingRecordingDto>>>
{
    public async Task<Result<List<MeetingRecordingDto>>> HandleAsync(
        GetMeetingRecordingsQuery query,
        CancellationToken cancellationToken = default)
    {
        var meetingExists = await dbContext.Meetings.AnyAsync(m => m.Id == query.MeetingId, cancellationToken);
        if (!meetingExists)
            return Result.Failure<List<MeetingRecordingDto>>(MeetingErrors.NotFound);

        var recordings = await dbContext.Recordings
            .Where(r => r.MeetingId == query.MeetingId)
            .OrderByDescending(r => r.StartedAt)
            .Select(r => new MeetingRecordingDto(
                r.Id,
                r.MeetingId,
                r.InitiatedBy,
                r.StartedAt,
                r.StoppedAt,
                r.DurationSeconds,
                r.FileSizeBytes,
                r.StoragePath,
                r.Status.ToString().ToLowerInvariant()
            ))
            .ToListAsync(cancellationToken);

        return Result.Success(recordings);
    }
}
