using Confer.Shared.Results;

namespace Confer.Domain.Meetings;

public static class PollErrors
{
    public static readonly Error NotFound = Error.NotFound("Poll.NotFound", "The requested poll was not found.");
    public static readonly Error Closed = Error.Validation("Poll.Closed", "The poll is already closed.");
    public static readonly Error Unauthorized = Error.Unauthorized("Poll.Unauthorized", "You are not authorized to perform this poll action.");
    public static readonly Error InvalidOptions = Error.Validation("Poll.InvalidOptions", "A poll must have at least two distinct valid options.");
    public static readonly Error OptionNotFound = Error.NotFound("Poll.OptionNotFound", "One or more selected poll options were not found.");
    public static readonly Error AlreadyVoted = Error.Conflict("Poll.AlreadyVoted", "Participant has already voted in this poll.");
    public static readonly Error SingleChoiceOnly = Error.Validation("Poll.SingleChoiceOnly", "Multiple choice is not permitted for this poll.");
}
