using Confer.Shared.Results;

namespace Confer.Domain.Meetings;

public static class MeetingErrors
{
    public static readonly Error NotFound = Error.NotFound("Meeting.NotFound", "The meeting was not found.");
    public static readonly Error Locked = Error.Forbidden("Meeting.Locked", "The meeting is currently locked by the host.");
    public static readonly Error Full = Error.Conflict("Meeting.Full", "The meeting has reached its maximum participant capacity.");
    public static readonly Error Ended = Error.Conflict("Meeting.Ended", "The meeting has already ended.");
    public static readonly Error Unauthorized = Error.Unauthorized("Meeting.Unauthorized", "You are not authorized to perform this meeting action.");
    public static readonly Error InvalidJoinCode = Error.Validation("Meeting.InvalidJoinCode", "The meeting join code is invalid.");
    public static readonly Error AlreadyRecording = Error.Conflict("Meeting.AlreadyRecording", "The meeting is already being recorded.");
    public static readonly Error NotRecording = Error.Conflict("Meeting.NotRecording", "The meeting is not currently being recorded.");
    public static readonly Error RecordingNotFound = Error.NotFound("Meeting.RecordingNotFound", "The meeting recording was not found.");
    public static readonly Error ParticipantNotInWaitingRoom = Error.Conflict("Meeting.ParticipantNotInWaitingRoom", "The participant is not in the waiting room.");
    public static readonly Error WaitingRoomDisabled = Error.Conflict("Meeting.WaitingRoomDisabled", "Waiting room is not enabled for this meeting.");
    public static readonly Error InvalidPolicy = Error.Validation("Meeting.InvalidPolicy", "Meeting policy cannot be null or invalid.");
    public static readonly Error SummaryNotFound = Error.NotFound("Meeting.SummaryNotFound", "The meeting summary was not found.");
    public static readonly Error MeetingNotEnded = Error.Conflict("Meeting.MeetingNotEnded", "Meeting must be ended before generating a summary.");
    public static readonly Error AlreadyGenerating = Error.Conflict("Meeting.AlreadyGenerating", "A summary is already being generated or exists for this meeting.");
}
