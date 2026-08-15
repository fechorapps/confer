using System.Security.Cryptography;
using System.Text;
using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Microsoft.Extensions.Configuration;

namespace Confer.Infrastructure.Media;

/// <summary>
/// Builds the ICE server list (STUN + optional TURN) from configuration, so the same set of
/// servers is handed to both the browser/native clients (via JoinMeetingResponse) and the
/// server-side SIPSorcery peer connections. Supports two TURN auth modes:
///   - Static long-term credentials (Ice:TurnUsername / Ice:TurnCredential) — matches the
///     docker-compose coturn setup (`--lt-cred-mech --user=...`).
///   - Ephemeral REST API credentials (Ice:TurnSharedSecret) — matches the Helm chart's coturn
///     StatefulSet (`--use-auth-secret --static-auth-secret=...`), per the standard coturn
///     REST API convention: username = expiry unix timestamp, password = base64(HMAC-SHA1(secret, username)).
/// </summary>
public sealed class ConfigurationIceServerProvider(IConfiguration configuration) : IIceServerProvider
{
    private static readonly string[] DefaultStunUrls =
    [
        "stun:stun.l.google.com:19302",
        "stun:stun1.l.google.com:19302"
    ];

    public List<IceServerConfig> GetIceServers()
    {
        var servers = new List<IceServerConfig>();

        var stunUrls = configuration.GetSection("Ice:StunUrls").Get<string[]>() ?? DefaultStunUrls;
        if (stunUrls.Length > 0)
        {
            servers.Add(new IceServerConfig(stunUrls));
        }

        var turnUrl = configuration["Ice:TurnUrl"];
        if (string.IsNullOrWhiteSpace(turnUrl))
        {
            return servers;
        }

        var sharedSecret = configuration["Ice:TurnSharedSecret"];
        if (!string.IsNullOrWhiteSpace(sharedSecret))
        {
            var (username, credential) = GenerateRestCredentials(sharedSecret, configuration["Ice:TurnCredentialTtlSeconds"]);
            servers.Add(new IceServerConfig([turnUrl], username, credential));
            return servers;
        }

        var staticUsername = configuration["Ice:TurnUsername"];
        var staticCredential = configuration["Ice:TurnCredential"];
        if (!string.IsNullOrWhiteSpace(staticUsername) && !string.IsNullOrWhiteSpace(staticCredential))
        {
            servers.Add(new IceServerConfig([turnUrl], staticUsername, staticCredential));
        }

        return servers;
    }

    private static (string Username, string Credential) GenerateRestCredentials(string sharedSecret, string? ttlSecondsRaw)
    {
        var ttlSeconds = long.TryParse(ttlSecondsRaw, out var parsedTtl) ? parsedTtl : 86_400; // coturn REST API default: 24h
        var expiry = DateTimeOffset.UtcNow.ToUnixTimeSeconds() + ttlSeconds;
        var username = expiry.ToString();

        using var hmac = new HMACSHA1(Encoding.UTF8.GetBytes(sharedSecret));
        var credential = Convert.ToBase64String(hmac.ComputeHash(Encoding.UTF8.GetBytes(username)));

        return (username, credential);
    }
}
