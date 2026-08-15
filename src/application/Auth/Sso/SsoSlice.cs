using Confer.Domain.Auth;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using FluentValidation;

namespace Confer.Application.Auth.Sso;

public sealed record SsoProviderDto(
    IdentityProviderType ProviderType,
    string Name,
    string DisplayName,
    string IconUrl,
    bool IsEnabled
);

public sealed record SsoAuthorizeQuery(
    string ProviderName,
    string? RedirectUri,
    string? DomainHint
) : IQuery<Result<SsoAuthorizeResultDto>>;

public sealed record SsoAuthorizeResultDto(
    string AuthorizationUrl,
    string State,
    string CodeVerifier
);

public sealed class SsoAuthorizeQueryValidator : AbstractValidator<SsoAuthorizeQuery>
{
    public SsoAuthorizeQueryValidator()
    {
        RuleFor(x => x.ProviderName)
            .NotEmpty()
            .WithMessage("Provider name is required.");
    }
}

public sealed record SsoCallbackCommand(
    string ProviderName,
    string Code,
    string State,
    string? CodeVerifier
) : ICommand<Result<SsoCallbackResultDto>>;

public sealed record SsoCallbackResultDto(
    Guid UserId,
    string Email,
    string DisplayName,
    string AccessToken,
    int ExpiresInSeconds,
    bool IsNewUser
);

public sealed class SsoCallbackCommandValidator : AbstractValidator<SsoCallbackCommand>
{
    public SsoCallbackCommandValidator()
    {
        RuleFor(x => x.ProviderName).NotEmpty();
        RuleFor(x => x.Code).NotEmpty();
        RuleFor(x => x.State).NotEmpty();
    }
}

public interface ISsoAuthenticationService
{
    IReadOnlyList<SsoProviderDto> GetAvailableProviders();
    Task<Result<SsoAuthorizeResultDto>> GenerateAuthorizeUrlAsync(string providerName, string? redirectUri, string? domainHint, CancellationToken ct = default);
    Task<Result<SsoCallbackResultDto>> HandleCallbackAsync(SsoCallbackCommand command, CancellationToken ct = default);
}

public sealed class SsoAuthorizeQueryHandler : IQueryHandler<SsoAuthorizeQuery, Result<SsoAuthorizeResultDto>>
{
    private readonly ISsoAuthenticationService _ssoService;

    public SsoAuthorizeQueryHandler(ISsoAuthenticationService ssoService)
    {
        _ssoService = ssoService;
    }

    public Task<Result<SsoAuthorizeResultDto>> HandleAsync(SsoAuthorizeQuery request, CancellationToken cancellationToken = default)
    {
        return _ssoService.GenerateAuthorizeUrlAsync(request.ProviderName, request.RedirectUri, request.DomainHint, cancellationToken);
    }
}

public sealed class SsoCallbackCommandHandler : ICommandHandler<SsoCallbackCommand, Result<SsoCallbackResultDto>>
{
    private readonly ISsoAuthenticationService _ssoService;

    public SsoCallbackCommandHandler(ISsoAuthenticationService ssoService)
    {
        _ssoService = ssoService;
    }

    public Task<Result<SsoCallbackResultDto>> HandleAsync(SsoCallbackCommand request, CancellationToken cancellationToken = default)
    {
        return _ssoService.HandleCallbackAsync(request, cancellationToken);
    }
}
