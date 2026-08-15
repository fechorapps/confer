namespace Confer.Application.Meetings.GetRecordings;

public record MeetingRecordingDto(
    Guid Id,
    Guid MeetingId,
    Guid InitiatedBy,
    DateTime StartedAt,
    DateTime? StoppedAt,
    long DurationSeconds,
    long FileSizeBytes,
    string StoragePath,
    string Status
);
