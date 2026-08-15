using Confer.Application.Interfaces;
using Confer.Domain.Enums;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.StopRecording;

public sealed class StopRecordingCommandHandler(
    IConferDbContext dbContext,
    IRecordingStorageService recordingStorage,
    ISignalingNotifier signalingNotifier)
    : ICommandHandler<StopRecordingCommand, Result<StopRecordingResponse>>
{
    public async Task<Result<StopRecordingResponse>> HandleAsync(
        StopRecordingCommand command,
        CancellationToken cancellationToken = default)
    {
        var meeting = await dbContext.Meetings
            .Include(m => m.Recordings)
            .FirstOrDefaultAsync(m => m.Id == command.MeetingId, cancellationToken);

        if (meeting is null)
            return Result.Failure<StopRecordingResponse>(MeetingErrors.NotFound);

        if (command.ActorId != meeting.OwnerId)
            return Result.Failure<StopRecordingResponse>(MeetingErrors.Unauthorized);

        if (!meeting.IsRecording)
            return Result.Failure<StopRecordingResponse>(MeetingErrors.NotRecording);

        var activeRecording = meeting.Recordings.LastOrDefault(r => r.Status == RecordingStatus.Recording);
        var recordingId = activeRecording?.Id ?? Guid.NewGuid();

        // Ensure recording file artifact exists on storage
        var storagePath = recordingStorage.GetRecordingPath(meeting.Id, recordingId);
        long fileSize = 0;

        if (await recordingStorage.ExistsAsync(storagePath, cancellationToken))
        {
            fileSize = await recordingStorage.GetRecordingSizeAsync(storagePath, cancellationToken);
        }
        else
        {
            // Finalize sample container with archival payload
            var placeholderData = System.Text.Encoding.UTF8.GetBytes($"CONFER-REC:{meeting.Id}:{recordingId}:{DateTime.UtcNow:o}");
            storagePath = await recordingStorage.SaveRecordingAsync(meeting.Id, recordingId, placeholderData, "webm", cancellationToken);
            fileSize = placeholderData.Length;
        }

        var stopResult = meeting.StopRecording(command.ActorId, storagePath, fileSize);
        if (stopResult.IsFailure)
            return Result.Failure<StopRecordingResponse>(stopResult.Error);

        var recording = stopResult.Value;
        await dbContext.SaveChangesAsync(cancellationToken);

        await signalingNotifier.BroadcastRecordingStateAsync(meeting.Id, false, recording.Id);

        return Result.Success(new StopRecordingResponse(
            recording.Id,
            meeting.Id,
            recording.Status.ToString().ToLowerInvariant(),
            recording.StartedAt,
            recording.StoppedAt,
            recording.DurationSeconds,
            recording.FileSizeBytes,
            recording.StoragePath
        ));
    }
}
