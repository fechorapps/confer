using Confer.Shared.Results;

namespace Confer.Domain.Meetings;

public static class BreakoutErrors
{
    public static readonly Error NotFound = Error.NotFound("Breakout.NotFound", "The breakout room was not found.");
    public static readonly Error Unauthorized = Error.Unauthorized("Breakout.Unauthorized", "You are not authorized to manage breakout rooms.");
    public static readonly Error InvalidCount = Error.Validation("Breakout.InvalidCount", "Number of breakout rooms must be between 1 and 20.");
    public static readonly Error InvalidDuration = Error.Validation("Breakout.InvalidDuration", "Max duration must be between 1 and 180 minutes.");
}
