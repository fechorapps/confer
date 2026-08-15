using Confer.Domain.Enums;

namespace Confer.Domain.Sessions;

public sealed class Participation
{
    public Guid Id { get; private set; }
    public Guid SessionId { get; private set; }
    public Guid UserId { get; private set; }
    public string DisplayName { get; private set; } = string.Empty;
    public ParticipantRole Role { get; private set; } = ParticipantRole.Participant;
    public DateTime JoinedAt { get; private set; } = DateTime.UtcNow;
    public DateTime? LeftAt { get; private set; }
    public LeaveReason? LeaveReason { get; private set; }
    public string? ClientInfo { get; private set; }

    private Participation() { }

    public static Participation Create(
        Guid sessionId,
        Guid userId,
        string displayName,
        ParticipantRole role = ParticipantRole.Participant,
        string? clientInfo = null)
    {
        return new Participation
        {
            Id = Guid.NewGuid(),
            SessionId = sessionId,
            UserId = userId,
            DisplayName = displayName,
            Role = role,
            JoinedAt = DateTime.UtcNow,
            ClientInfo = clientInfo
        };
    }

    public void Leave(LeaveReason reason)
    {
        LeftAt = DateTime.UtcNow;
        LeaveReason = reason;
    }
}
