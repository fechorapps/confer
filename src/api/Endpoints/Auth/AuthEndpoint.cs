using Confer.Application.Auth.Sso;
using Confer.Application.Interfaces;
using Confer.Domain.Identity;
using Confer.Shared.Api;
using Confer.Shared.Application.Interfaces;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.Routing;
using Microsoft.EntityFrameworkCore;

namespace Confer.Api.Endpoints.Auth;

public sealed class AuthEndpoint : BaseModule, IEndpoint
{
    public void MapEndpoint(IEndpointRouteBuilder app)
    {
        var group = app.MapGroup("/api/auth")
            .WithTags("Auth");

        group.MapPost("/dev-login", DevLogin).WithName("DevLogin");
        group.MapGet("/me", GetMe).WithName("GetMe");

        // Enterprise SSO Endpoints
        group.MapGet("/sso/providers", GetSsoProviders).WithName("GetSsoProviders");
        group.MapGet("/sso/{provider}/authorize", SsoAuthorize).WithName("SsoAuthorize");
        group.MapPost("/sso/{provider}/callback", SsoCallback).WithName("SsoCallback");
    }

    public record DevLoginRequest(string Email, string DisplayName);
    public record DevLoginResponse(Guid UserId, string Email, string DisplayName, string Token);

    private static async Task<IResult> DevLogin(
        [FromBody] DevLoginRequest request,
        [FromServices] IConferDbContext dbContext,
        [FromServices] ITokenProvider tokenProvider,
        CancellationToken ct)
    {
        var email = request.Email.Trim().ToLowerInvariant();
        var user = await dbContext.Users.FirstOrDefaultAsync(u => u.Email == email, ct);

        if (user is null)
        {
            var userResult = User.Create(email, request.DisplayName);
            if (userResult.IsFailure)
                return TypedResults.BadRequest(new { userResult.Error.Code, userResult.Error.Description });

            user = userResult.Value;
            dbContext.Users.Add(user);
            await dbContext.SaveChangesAsync(ct);
        }

        var token = tokenProvider.GenerateAuthToken(user.Id, user.Email, user.DisplayName);
        return TypedResults.Ok(new DevLoginResponse(user.Id, user.Email, user.DisplayName, token));
    }

    private static IResult GetMe(
        [FromHeader(Name = "Authorization")] string? authHeader,
        [FromServices] ITokenProvider tokenProvider)
    {
        if (string.IsNullOrWhiteSpace(authHeader) || !authHeader.StartsWith("Bearer "))
            return TypedResults.Unauthorized();

        var token = authHeader["Bearer ".Length..].Trim();
        var validation = tokenProvider.ValidateAuthToken(token);

        if (validation.IsFailure)
            return TypedResults.Unauthorized();

        var (userId, email, name) = validation.Value;
        return TypedResults.Ok(new { UserId = userId, Email = email, DisplayName = name });
    }

    private static IResult GetSsoProviders(
        [FromServices] ISsoAuthenticationService ssoService)
    {
        var providers = ssoService.GetAvailableProviders();
        return TypedResults.Ok(providers);
    }

    private static async Task<IResult> SsoAuthorize(
        [FromRoute] string provider,
        [FromQuery] string? redirectUri,
        [FromQuery] string? domainHint,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var result = await dispatcher.QueryAsync(new SsoAuthorizeQuery(provider, redirectUri, domainHint), ct);
        return result.IsSuccess
            ? TypedResults.Ok(result.Value)
            : TypedResults.BadRequest(new { result.Error.Code, result.Error.Description });
    }

    public record SsoCallbackBody(string Code, string State, string? CodeVerifier);

    private static async Task<IResult> SsoCallback(
        [FromRoute] string provider,
        [FromBody] SsoCallbackBody body,
        [FromServices] ICqrsDispatcher dispatcher,
        CancellationToken ct)
    {
        var result = await dispatcher.SendAsync(new SsoCallbackCommand(provider, body.Code, body.State, body.CodeVerifier), ct);
        return result.IsSuccess
            ? TypedResults.Ok(result.Value)
            : TypedResults.BadRequest(new { result.Error.Code, result.Error.Description });
    }
}
