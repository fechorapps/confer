using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using System.Text;
using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Domain.Enums;
using Confer.Domain.Identity;
using Confer.Shared.Results;
using Microsoft.Extensions.Configuration;
using Microsoft.IdentityModel.Tokens;

namespace Confer.Infrastructure.Security;

public sealed class JwtTokenProvider(IConfiguration configuration) : ITokenProvider
{
    private readonly string _secret = configuration["Jwt:Secret"] ?? "confer-super-secret-key-that-is-at-least-256-bits-long-for-hmac-sha256!";
    private readonly string _issuer = configuration["Jwt:Issuer"] ?? "confer-api";
    private readonly string _audience = configuration["Jwt:Audience"] ?? "confer-client";

    public string GenerateAuthToken(Guid userId, string email, string displayName)
    {
        var tokenHandler = new JwtSecurityTokenHandler();
        var key = Encoding.UTF8.GetBytes(_secret);

        var tokenDescriptor = new SecurityTokenDescriptor
        {
            Subject = new ClaimsIdentity(
            [
                new Claim(JwtRegisteredClaimNames.Sub, userId.ToString()),
                new Claim(JwtRegisteredClaimNames.Email, email),
                new Claim("name", displayName),
                new Claim("type", "auth")
            ]),
            Expires = DateTime.UtcNow.AddDays(7),
            Issuer = _issuer,
            Audience = _audience,
            SigningCredentials = new SigningCredentials(new SymmetricSecurityKey(key), SecurityAlgorithms.HmacSha256Signature)
        };

        var token = tokenHandler.CreateToken(tokenDescriptor);
        return tokenHandler.WriteToken(token);
    }

    public Result<(Guid UserId, string Email, string DisplayName)> ValidateAuthToken(string token)
    {
        if (string.IsNullOrWhiteSpace(token))
            return Result.Failure<(Guid, string, string)>(AuthErrors.MissingToken);

        var tokenHandler = new JwtSecurityTokenHandler();
        var key = Encoding.UTF8.GetBytes(_secret);

        try
        {
            var principal = tokenHandler.ValidateToken(token, new TokenValidationParameters
            {
                ValidateIssuerSigningKey = true,
                IssuerSigningKey = new SymmetricSecurityKey(key),
                ValidateIssuer = true,
                ValidIssuer = _issuer,
                ValidateAudience = true,
                ValidAudience = _audience,
                ClockSkew = TimeSpan.FromSeconds(30)
            }, out var validatedToken);

            var sub = principal.FindFirst(ClaimTypes.NameIdentifier)?.Value ?? principal.FindFirst(JwtRegisteredClaimNames.Sub)?.Value;
            var email = principal.FindFirst(ClaimTypes.Email)?.Value ?? principal.FindFirst(JwtRegisteredClaimNames.Email)?.Value;
            var name = principal.FindFirst("name")?.Value ?? "User";

            if (Guid.TryParse(sub, out var userId) && !string.IsNullOrWhiteSpace(email))
            {
                return Result.Success((userId, email, name));
            }

            return Result.Failure<(Guid, string, string)>(AuthErrors.InvalidToken);
        }
        catch
        {
            return Result.Failure<(Guid, string, string)>(AuthErrors.InvalidToken);
        }
    }

    public string GenerateRoomToken(
        Guid meetingId,
        Guid userId,
        Guid participantId,
        string displayName,
        ParticipantRole role,
        string sfuNodeId = "sfu-local-1")
    {
        var tokenHandler = new JwtSecurityTokenHandler();
        var key = Encoding.UTF8.GetBytes(_secret);

        var tokenDescriptor = new SecurityTokenDescriptor
        {
            Subject = new ClaimsIdentity(
            [
                new Claim("meeting_id", meetingId.ToString()),
                new Claim("user_id", userId.ToString()),
                new Claim("participant_id", participantId.ToString()),
                new Claim("name", displayName),
                new Claim("role", role.ToString().ToLowerInvariant()),
                new Claim("sfu_node", sfuNodeId),
                new Claim("type", "room")
            ]),
            Expires = DateTime.UtcNow.AddSeconds(120), // 120s TTL as per SDD §9.1
            Issuer = _issuer,
            Audience = _audience,
            SigningCredentials = new SigningCredentials(new SymmetricSecurityKey(key), SecurityAlgorithms.HmacSha256Signature)
        };

        var token = tokenHandler.CreateToken(tokenDescriptor);
        return tokenHandler.WriteToken(token);
    }

    public Result<RoomTokenClaims> ValidateRoomToken(string token)
    {
        if (string.IsNullOrWhiteSpace(token))
            return Result.Failure<RoomTokenClaims>(AuthErrors.MissingToken);

        var tokenHandler = new JwtSecurityTokenHandler();
        var key = Encoding.UTF8.GetBytes(_secret);

        try
        {
            var principal = tokenHandler.ValidateToken(token, new TokenValidationParameters
            {
                ValidateIssuerSigningKey = true,
                IssuerSigningKey = new SymmetricSecurityKey(key),
                ValidateIssuer = true,
                ValidIssuer = _issuer,
                ValidateAudience = true,
                ValidAudience = _audience,
                ClockSkew = TimeSpan.FromSeconds(30)
            }, out var validatedToken);

            var meetingIdStr = principal.FindFirst("meeting_id")?.Value;
            var userIdStr = principal.FindFirst("user_id")?.Value;
            var participantIdStr = principal.FindFirst("participant_id")?.Value;
            var displayName = principal.FindFirst("name")?.Value
                ?? principal.FindFirst(System.Security.Claims.ClaimTypes.Name)?.Value
                ?? "Participant";
            var roleStr = principal.FindFirst("role")?.Value
                ?? principal.FindFirst(System.Security.Claims.ClaimTypes.Role)?.Value
                ?? "participant";
            var sfuNode = principal.FindFirst("sfu_node")?.Value ?? "sfu-local-1";

            if (Guid.TryParse(meetingIdStr, out var meetingId) &&
                Guid.TryParse(userIdStr, out var userId) &&
                Guid.TryParse(participantIdStr, out var participantId))
            {
                var role = Enum.TryParse<ParticipantRole>(roleStr, true, out var parsedRole)
                    ? parsedRole
                    : ParticipantRole.Participant;

                return Result.Success(new RoomTokenClaims(
                    meetingId,
                    userId,
                    participantId,
                    displayName,
                    role,
                    sfuNode,
                    validatedToken.ValidTo
                ));
            }

            return Result.Failure<RoomTokenClaims>(AuthErrors.InvalidToken);
        }
        catch
        {
            return Result.Failure<RoomTokenClaims>(AuthErrors.InvalidToken);
        }
    }
}
