using Confer.Application.Interfaces;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.StartRecording;

public sealed class StartRecordingCommandHandler(
    IConferDbContext dbContext,
    ISignalingNotifier signalingNotifier)
    : ICommandHandler<StartRecordingCommand, Result<StartRecordingResponse>>
{
    public async Task<Result<StartRecordingResponse>> HandleAsync(
        StartRecordingCommand command,
        CancellationToken cancellationToken = default)
    {
        var meeting = await dbContext.Meetings
            .Include(m => m.Recordings)
            .FirstOrDefaultAsync(m => m.Id == command.MeetingId, cancellationToken);

        if (meeting is null)
            return Result.Failure<StartRecordingResponse>(MeetingErrors.NotFound);

        var recordResult = meeting.StartRecording(command.ActorId);
        if (recordResult.IsFailure)
            return Result.Failure<StartRecordingResponse>(recordResult.Error);

        var recording = recordResult.Value;
        dbContext.Recordings.Add(recording);
        await dbContext.SaveChangesAsync(cancellationToken);

        await signalingNotifier.BroadcastRecordingStateAsync(meeting.Id, true, recording.Id);

        return Result.Success(new StartRecordingResponse(
            recording.Id,
            meeting.Id,
            recording.Status.ToString().ToLowerInvariant(),
            recording.StartedAt
        ));
    }
}
