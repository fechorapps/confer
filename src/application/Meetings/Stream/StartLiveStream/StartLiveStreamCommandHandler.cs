using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.Stream.StartLiveStream;

public sealed class StartLiveStreamCommandHandler(
    IConferDbContext dbContext,
    ISignalingNotifier signalingNotifier)
    : ICommandHandler<StartLiveStreamCommand, Result<LiveStreamResponse>>
{
    public async Task<Result<LiveStreamResponse>> HandleAsync(
        StartLiveStreamCommand command,
        CancellationToken cancellationToken = default)
    {
        var meeting = await dbContext.Meetings
            .FirstOrDefaultAsync(m => m.Id == command.MeetingId, cancellationToken);

        if (meeting is null)
            return Result.Failure<LiveStreamResponse>(MeetingErrors.NotFound);

        var streamResult = meeting.StartLiveStream(command.ActorId, command.RtmpUrl, command.StreamKey);
        if (streamResult.IsFailure)
            return Result.Failure<LiveStreamResponse>(streamResult.Error);

        var config = streamResult.Value;
        await dbContext.SaveChangesAsync(cancellationToken);

        await signalingNotifier.BroadcastStreamStatusAsync(
            meeting.Id,
            config.IsStreamingEnabled,
            config.RtmpUrl,
            config.Status.ToString().ToLowerInvariant(),
            config.StartedAt);

        return Result.Success(new LiveStreamResponse(
            meeting.Id,
            config.IsStreamingEnabled,
            config.RtmpUrl,
            config.Status.ToString().ToLowerInvariant(),
            config.StartedAt
        ));
    }
}
