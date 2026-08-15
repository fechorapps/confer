namespace Confer.Application.Meetings.StartRecording;

public record StartRecordingResponse(
    Guid RecordingId,
    Guid MeetingId,
    string Status,
    DateTime StartedAt
);
