using Confer.Shared.Results;

namespace Confer.Domain.Identity;

public static class AuthErrors
{
    public static readonly Error UserNotFound = Error.NotFound("User.NotFound", "The requested user was not found.");
    public static readonly Error InvalidToken = Error.Unauthorized("Auth.InvalidToken", "The authentication or room token is invalid or expired.");
    public static readonly Error MissingToken = Error.Unauthorized("Auth.MissingToken", "The required authentication token was not provided.");
    public static readonly Error Unauthorized = Error.Unauthorized("Auth.Unauthorized", "You are not authorized to perform this operation.");
}
