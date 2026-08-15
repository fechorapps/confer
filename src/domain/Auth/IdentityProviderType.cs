namespace Confer.Domain.Auth;

public enum IdentityProviderType
{
    Google = 1,
    MicrosoftEntra = 2,
    Okta = 3,
    Keycloak = 4,
    CustomOidc = 5,
    Saml2 = 6
}
