using Confer.Domain.Identity;
using Confer.Domain.Meetings;
using Confer.Domain.Sessions;
using Confer.Domain.Webhooks;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Interfaces;

public interface IConferDbContext
{
    DbSet<Meeting> Meetings { get; }
    DbSet<Session> Sessions { get; }
    DbSet<Participation> Participations { get; }
    DbSet<ChatMessage> ChatMessages { get; }
    DbSet<User> Users { get; }
    DbSet<MeetingRecording> Recordings { get; }
    DbSet<Poll> Polls { get; }
    DbSet<PollOption> PollOptions { get; }
    DbSet<PollVote> PollVotes { get; }
    DbSet<BreakoutRoom> BreakoutRooms { get; }
    DbSet<MeetingSummary> MeetingSummaries { get; }
    DbSet<WebhookSubscription> WebhookSubscriptions { get; }

    Task<int> SaveChangesAsync(CancellationToken cancellationToken = default);
}
