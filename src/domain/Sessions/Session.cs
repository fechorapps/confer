using Confer.Domain.Enums;
using Confer.Domain.Meetings;
using Confer.Shared.Results;

namespace Confer.Domain.Sessions;

public sealed class Session
{
    public Guid Id { get; private set; }
    public Guid MeetingId { get; private set; }
    public Meeting Meeting { get; private set; } = null!;
    public string SfuNodeId { get; private set; } = string.Empty;
    public DateTime StartedAt { get; private set; } = DateTime.UtcNow;
    public DateTime? EndedAt { get; private set; }

    public ICollection<Participation> Participations { get; private set; } = new List<Participation>();
    public ICollection<ChatMessage> ChatMessages { get; private set; } = new List<ChatMessage>();

    private Session() { }

    public static Session Create(Guid meetingId, string sfuNodeId = "sfu-local-1")
    {
        return new Session
        {
            Id = Guid.NewGuid(),
            MeetingId = meetingId,
            SfuNodeId = sfuNodeId,
            StartedAt = DateTime.UtcNow
        };
    }

    public Result<Participation> AddParticipant(Guid userId, string displayName, ParticipantRole role, string? clientInfo = null)
    {
        var existing = Participations.FirstOrDefault(p => p.UserId == userId && p.LeftAt == null);
        if (existing is not null)
            return Result.Success(existing);

        var participation = Participation.Create(Id, userId, displayName, role, clientInfo);
        Participations.Add(participation);
        return Result.Success(participation);
    }

    public Result EndParticipant(Guid userId, LeaveReason reason)
    {
        var participation = Participations.FirstOrDefault(p => p.UserId == userId && p.LeftAt == null);
        if (participation is null)
            return Result.Failure(SessionErrors.ParticipantNotFound);

        participation.Leave(reason);
        return Result.Success();
    }

    public Result End()
    {
        EndedAt = DateTime.UtcNow;
        foreach (var p in Participations.Where(p => p.LeftAt == null))
        {
            p.Leave(LeaveReason.MeetingEnded);
        }
        return Result.Success();
    }
}
