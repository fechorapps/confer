namespace Confer.Application.Meetings.StopRecording;

public record StopRecordingResponse(
    Guid RecordingId,
    Guid MeetingId,
    string Status,
    DateTime StartedAt,
    DateTime? StoppedAt,
    long DurationSeconds,
    long FileSizeBytes,
    string StoragePath
);
