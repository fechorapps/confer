using Confer.Domain.Enums;

namespace Confer.Domain.Sessions;

public sealed class Participation
{
    public Guid Id { get; private set; }
    public Guid SessionId { get; private set; }
    public Guid UserId { get; private set; }
    public string DisplayName { get; private set; } = string.Empty;
    public ParticipantRole Role { get; private set; } = ParticipantRole.Participant;
    public ParticipationStatus Status { get; private set; } = ParticipationStatus.Admitted;
    public DateTime JoinedAt { get; private set; } = DateTime.UtcNow;
    public DateTime? AdmittedAt { get; private set; }
    public DateTime? LeftAt { get; private set; }
    public LeaveReason? LeaveReason { get; private set; }
    public string? ClientInfo { get; private set; }

    private Participation() { }

    public static Participation Create(
        Guid sessionId,
        Guid userId,
        string displayName,
        ParticipantRole role = ParticipantRole.Participant,
        string? clientInfo = null,
        ParticipationStatus status = ParticipationStatus.Admitted)
    {
        return new Participation
        {
            Id = Guid.NewGuid(),
            SessionId = sessionId,
            UserId = userId,
            DisplayName = displayName,
            Role = role,
            Status = status,
            JoinedAt = DateTime.UtcNow,
            AdmittedAt = status == ParticipationStatus.Admitted ? DateTime.UtcNow : null,
            ClientInfo = clientInfo
        };
    }

    public void Admit()
    {
        Status = ParticipationStatus.Admitted;
        AdmittedAt = DateTime.UtcNow;
    }

    public void Reject()
    {
        Status = ParticipationStatus.Rejected;
        LeftAt = DateTime.UtcNow;
        LeaveReason = Enums.LeaveReason.Kicked;
    }

    public void Leave(LeaveReason reason)
    {
        LeftAt = DateTime.UtcNow;
        LeaveReason = reason;
    }
}
