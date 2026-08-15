using Microsoft.AspNetCore.Http;
using Confer.Shared.Results;

namespace Confer.Shared.Api;

public abstract class BaseModule
{
    protected static IResult OkOrFailure<T>(Result<T> result) =>
        result.IsSuccess
            ? TypedResults.Ok(result.Value)
            : HandleFailure(result.Error);

    protected static IResult CreatedOrFailure<T>(Result<T> result, string uri) =>
        result.IsSuccess
            ? TypedResults.Created(uri, result.Value)
            : HandleFailure(result.Error);

    protected static IResult NoContentOrFailure(Result result) =>
        result.IsSuccess
            ? TypedResults.NoContent()
            : HandleFailure(result.Error);

    private static IResult HandleFailure(Error error) =>
        error.Type switch
        {
            ErrorType.NotFound => TypedResults.NotFound(new { error.Code, error.Description }),
            ErrorType.Validation => TypedResults.BadRequest(new { error.Code, error.Description }),
            ErrorType.Conflict => TypedResults.Conflict(new { error.Code, error.Description }),
            ErrorType.Unauthorized => TypedResults.Unauthorized(),
            ErrorType.Forbidden => TypedResults.Forbid(),
            _ => TypedResults.BadRequest(new { error.Code, error.Description })
        };
}
