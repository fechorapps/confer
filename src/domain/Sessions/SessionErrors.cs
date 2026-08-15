using Confer.Shared.Results;

namespace Confer.Domain.Sessions;

public static class SessionErrors
{
    public static readonly Error NotFound = Error.NotFound("Session.NotFound", "The meeting session was not found.");
    public static readonly Error ParticipantNotFound = Error.NotFound("Participation.NotFound", "The participant was not found in this session.");
    public static readonly Error ParticipantAlreadyJoined = Error.Conflict("Participation.AlreadyJoined", "The participant is already active in this session.");
}
