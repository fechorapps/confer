using Confer.Application.DTOs;

namespace Confer.Application.Interfaces;

public interface IPresenceService
{
    Task SetParticipantOnlineAsync(Guid meetingId, ParticipantStateDto participant);
    Task SetParticipantOfflineAsync(Guid meetingId, Guid participantId);
    Task<List<ParticipantStateDto>> GetRosterAsync(Guid meetingId);
    Task<int> GetActiveCountAsync(Guid meetingId);
    Task SetNodeAffinityAsync(Guid meetingId, string nodeId, TimeSpan ttl);
    Task<string?> GetNodeAffinityAsync(Guid meetingId);
}
