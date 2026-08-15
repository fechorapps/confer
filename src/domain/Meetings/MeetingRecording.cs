using Confer.Domain.Enums;
using Confer.Shared.Results;

namespace Confer.Domain.Meetings;

public sealed class MeetingRecording
{
    public Guid Id { get; private set; }
    public Guid MeetingId { get; private set; }
    public Guid InitiatedBy { get; private set; }
    public DateTime StartedAt { get; private set; }
    public DateTime? StoppedAt { get; private set; }
    public long DurationSeconds { get; private set; }
    public long FileSizeBytes { get; private set; }
    public string StoragePath { get; private set; } = string.Empty;
    public string StorageProvider { get; private set; } = "local";
    public RecordingStatus Status { get; private set; } = RecordingStatus.Recording;

    private MeetingRecording() { }

    public static MeetingRecording Create(Guid meetingId, Guid initiatedBy, string storageProvider = "local")
    {
        return new MeetingRecording
        {
            Id = Guid.NewGuid(),
            MeetingId = meetingId,
            InitiatedBy = initiatedBy,
            StartedAt = DateTime.UtcNow,
            StorageProvider = storageProvider,
            Status = RecordingStatus.Recording
        };
    }

    public void Complete(string storagePath, long fileSizeBytes)
    {
        StoppedAt = DateTime.UtcNow;
        DurationSeconds = (long)Math.Max(0, (StoppedAt.Value - StartedAt).TotalSeconds);
        StoragePath = storagePath;
        FileSizeBytes = fileSizeBytes;
        Status = RecordingStatus.Completed;
    }

    public void Fail(string reason)
    {
        StoppedAt = DateTime.UtcNow;
        DurationSeconds = (long)Math.Max(0, (StoppedAt.Value - StartedAt).TotalSeconds);
        Status = RecordingStatus.Failed;
    }
}
