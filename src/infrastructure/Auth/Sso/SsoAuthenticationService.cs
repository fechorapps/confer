using System.Security.Cryptography;
using System.Text;
using Confer.Application.Auth.Sso;
using Confer.Application.Interfaces;
using Confer.Domain.Auth;
using Confer.Domain.Identity;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Logging;

namespace Confer.Infrastructure.Auth.Sso;

public class SsoAuthenticationService : ISsoAuthenticationService
{
    private readonly IConferDbContext _dbContext;
    private readonly ITokenProvider _tokenProvider;
    private readonly ILogger<SsoAuthenticationService> _logger;
    private readonly IConfiguration _configuration;

    private static readonly List<SsoProviderDto> AvailableProviders = new()
    {
        new(IdentityProviderType.Google, "google", "Google Workspace", "https://auth.confer.local/icons/google.svg", true),
        new(IdentityProviderType.MicrosoftEntra, "microsoft", "Microsoft Entra ID (Azure AD)", "https://auth.confer.local/icons/microsoft.svg", true),
        new(IdentityProviderType.Okta, "okta", "Okta Enterprise SSO", "https://auth.confer.local/icons/okta.svg", true),
        new(IdentityProviderType.Keycloak, "keycloak", "Keycloak Identity Provider", "https://auth.confer.local/icons/keycloak.svg", true),
        new(IdentityProviderType.Saml2, "saml2", "Enterprise SAML 2.0", "https://auth.confer.local/icons/saml.svg", true)
    };

    public SsoAuthenticationService(
        IConferDbContext dbContext,
        ITokenProvider tokenProvider,
        ILogger<SsoAuthenticationService> logger,
        IConfiguration configuration)
    {
        _dbContext = dbContext;
        _tokenProvider = tokenProvider;
        _logger = logger;
        _configuration = configuration;
    }

    public IReadOnlyList<SsoProviderDto> GetAvailableProviders() => AvailableProviders.AsReadOnly();

    public Task<Result<SsoAuthorizeResultDto>> GenerateAuthorizeUrlAsync(string providerName, string? redirectUri, string? domainHint, CancellationToken ct = default)
    {
        var provider = AvailableProviders.FirstOrDefault(p => p.Name.Equals(providerName, StringComparison.OrdinalIgnoreCase));
        if (provider == null || !provider.IsEnabled)
        {
            return Task.FromResult(Result.Failure<SsoAuthorizeResultDto>(
                Error.NotFound("Sso.ProviderNotFound", $"Identity provider '{providerName}' is not configured or disabled.")));
        }

        // Generate PKCE code verifier and challenge
        var codeVerifier = GenerateSecureRandomString(32);
        var codeChallenge = Base64UrlEncode(SHA256.HashData(Encoding.ASCII.GetBytes(codeVerifier)));
        var state = GenerateSecureRandomString(16);

        var callbackUrl = redirectUri ?? "https://meet.confer.local/api/auth/sso/callback";
        var authUrl = provider.ProviderType switch
        {
            IdentityProviderType.Google =>
                $"https://accounts.google.com/o/oauth2/v2/auth?client_id=confer-google-client-id&response_type=code&scope=openid%20email%20profile&redirect_uri={Uri.EscapeDataString(callbackUrl)}&state={state}&code_challenge={codeChallenge}&code_challenge_method=S256{(string.IsNullOrEmpty(domainHint) ? "" : $"&hd={domainHint}")}",

            IdentityProviderType.MicrosoftEntra =>
                $"https://login.microsoftonline.com/common/oauth2/v2.0/authorize?client_id=confer-ms-client-id&response_type=code&scope=openid%20email%20profile&redirect_uri={Uri.EscapeDataString(callbackUrl)}&state={state}&code_challenge={codeChallenge}&code_challenge_method=S256",

            IdentityProviderType.Okta =>
                $"https://auth.okta.com/oauth2/v1/authorize?client_id=confer-okta-client&response_type=code&scope=openid%20email%20profile&redirect_uri={Uri.EscapeDataString(callbackUrl)}&state={state}&code_challenge={codeChallenge}&code_challenge_method=S256",

            _ =>
                $"https://auth.confer.local/oauth2/authorize?provider={provider.Name}&response_type=code&scope=openid%20email%20profile&redirect_uri={Uri.EscapeDataString(callbackUrl)}&state={state}&code_challenge={codeChallenge}&code_challenge_method=S256"
        };

        return Task.FromResult(Result.Success(new SsoAuthorizeResultDto(authUrl, state, codeVerifier)));
    }

    public async Task<Result<SsoCallbackResultDto>> HandleCallbackAsync(SsoCallbackCommand command, CancellationToken ct = default)
    {
        var provider = AvailableProviders.FirstOrDefault(p => p.Name.Equals(command.ProviderName, StringComparison.OrdinalIgnoreCase));
        if (provider == null)
        {
            return Result.Failure<SsoCallbackResultDto>(Error.NotFound("Sso.ProviderNotFound", "Provider not found."));
        }

        // Mock/Simulate standard OIDC id_token claim resolution for tests & dev
        var externalSub = $"sso_{command.ProviderName}_{command.Code.GetHashCode():X8}";
        var email = $"{command.ProviderName}_user_{Math.Abs(command.Code.GetHashCode()) % 1000}@enterprise.local";
        var displayName = $"{provider.DisplayName} User";

        // Find or provision user in DB
        var user = await _dbContext.Users.FirstOrDefaultAsync(u => u.Email == email || u.IdpSubject == externalSub, ct);
        var isNewUser = false;

        if (user == null)
        {
            var userResult = User.Create(email, displayName, null, externalSub);
            if (userResult.IsFailure)
            {
                return Result.Failure<SsoCallbackResultDto>(userResult.Error);
            }

            user = userResult.Value;
            _dbContext.Users.Add(user);
            await _dbContext.SaveChangesAsync(ct);
            isNewUser = true;
            _logger.LogInformation("Provisioned new enterprise SSO user {Email} via provider {Provider}", email, provider.Name);
        }

        var token = _tokenProvider.GenerateAuthToken(user.Id, user.Email, user.DisplayName);

        return Result.Success(new SsoCallbackResultDto(
            user.Id,
            user.Email,
            user.DisplayName,
            token,
            86400,
            isNewUser
        ));
    }

    private static string GenerateSecureRandomString(int bytesCount)
    {
        var bytes = new byte[bytesCount];
        RandomNumberGenerator.Fill(bytes);
        return Base64UrlEncode(bytes);
    }

    private static string Base64UrlEncode(byte[] bytes)
    {
        return Convert.ToBase64String(bytes)
            .TrimEnd('=')
            .Replace('+', '-')
            .Replace('/', '_');
    }
}
