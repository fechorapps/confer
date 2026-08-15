using Confer.Application.Interfaces;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.GetByCode;

public sealed class GetMeetingByCodeQueryHandler(IConferDbContext dbContext)
    : IQueryHandler<GetMeetingByCodeQuery, Result<MeetingResponse>>
{
    public async Task<Result<MeetingResponse>> HandleAsync(
        GetMeetingByCodeQuery query,
        CancellationToken cancellationToken = default)
    {
        if (string.IsNullOrWhiteSpace(query.Code))
            return Result.Failure<MeetingResponse>(MeetingErrors.InvalidJoinCode);

        var normalizedCode = query.Code.Trim().ToLowerInvariant();

        // Search by JoinCode or parse as Guid Id
        Meeting? meeting;
        if (Guid.TryParse(normalizedCode, out var meetingId))
        {
            meeting = await dbContext.Meetings.FirstOrDefaultAsync(m => m.Id == meetingId, cancellationToken);
        }
        else
        {
            meeting = await dbContext.Meetings.FirstOrDefaultAsync(m => m.JoinCode == normalizedCode, cancellationToken);
        }

        if (meeting is null)
            return Result.Failure<MeetingResponse>(MeetingErrors.NotFound);

        return Result.Success(new MeetingResponse(
            meeting.Id,
            meeting.JoinCode,
            meeting.Title,
            meeting.OwnerId,
            meeting.MaxParticipants,
            meeting.IsLocked,
            meeting.EndedAt != null,
            meeting.CreatedAt,
            meeting.IsRecording
        ));
    }
}
