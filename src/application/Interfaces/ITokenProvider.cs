using Confer.Application.DTOs;
using Confer.Domain.Enums;
using Confer.Shared.Results;

namespace Confer.Application.Interfaces;

public interface ITokenProvider
{
    string GenerateAuthToken(Guid userId, string email, string displayName);
    Result<(Guid UserId, string Email, string DisplayName)> ValidateAuthToken(string token);

    string GenerateRoomToken(
        Guid meetingId,
        Guid userId,
        Guid participantId,
        string displayName,
        ParticipantRole role,
        string sfuNodeId = "sfu-local-1");

    Result<RoomTokenClaims> ValidateRoomToken(string token);
}
