using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.Stream.GetLiveStreamStatus;

public sealed class GetLiveStreamStatusQueryHandler(
    IConferDbContext dbContext)
    : IQueryHandler<GetLiveStreamStatusQuery, Result<LiveStreamResponse>>
{
    public async Task<Result<LiveStreamResponse>> HandleAsync(
        GetLiveStreamStatusQuery query,
        CancellationToken cancellationToken = default)
    {
        var meeting = await dbContext.Meetings
            .FirstOrDefaultAsync(m => m.Id == query.MeetingId, cancellationToken);

        if (meeting is null)
            return Result.Failure<LiveStreamResponse>(MeetingErrors.NotFound);

        var config = meeting.LiveStreamConfig;
        return Result.Success(new LiveStreamResponse(
            meeting.Id,
            config.IsStreamingEnabled,
            config.RtmpUrl,
            config.Status.ToString().ToLowerInvariant(),
            config.StartedAt
        ));
    }
}
