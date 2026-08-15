using Confer.Application.Interfaces;
using Confer.Domain.Identity;
using Confer.Domain.Meetings;
using Confer.Domain.Sessions;
using Microsoft.EntityFrameworkCore;

namespace Confer.Infrastructure.Persistence;

public sealed class ConferDbContext(DbContextOptions<ConferDbContext> options)
    : DbContext(options), IConferDbContext
{
    public DbSet<Meeting> Meetings => Set<Meeting>();
    public DbSet<Session> Sessions => Set<Session>();
    public DbSet<Participation> Participations => Set<Participation>();
    public DbSet<ChatMessage> ChatMessages => Set<ChatMessage>();
    public DbSet<User> Users => Set<User>();
    public DbSet<MeetingRecording> Recordings => Set<MeetingRecording>();
    public DbSet<Poll> Polls => Set<Poll>();
    public DbSet<PollOption> PollOptions => Set<PollOption>();
    public DbSet<PollVote> PollVotes => Set<PollVote>();
    public DbSet<BreakoutRoom> BreakoutRooms => Set<BreakoutRoom>();

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        base.OnModelCreating(modelBuilder);

        modelBuilder.Entity<User>(builder =>
        {
            builder.ToTable("users");
            builder.HasKey(u => u.Id);
            builder.Property(u => u.Email).IsRequired().HasMaxLength(255);
            builder.HasIndex(u => u.Email).IsUnique();
            builder.Property(u => u.DisplayName).IsRequired().HasMaxLength(150);
            builder.Property(u => u.IdpSubject).HasMaxLength(255);
            builder.HasIndex(u => u.IdpSubject).IsUnique().HasFilter("\"IdpSubject\" IS NOT NULL");
        });

        modelBuilder.Entity<Meeting>(builder =>
        {
            builder.ToTable("meetings");
            builder.HasKey(m => m.Id);
            builder.Property(m => m.JoinCode).IsRequired().HasMaxLength(32);
            builder.HasIndex(m => m.JoinCode).IsUnique();
            builder.Property(m => m.Title).IsRequired().HasMaxLength(200);
            builder.Property(m => m.RecordingMode).HasMaxLength(32);
            builder.Property(m => m.IsWaitingRoomEnabled).HasDefaultValue(false);
            builder.Property(m => m.IsWatermarkEnabled).HasDefaultValue(false);

            builder.OwnsOne(m => m.Policy, policyBuilder =>
            {
                policyBuilder.Property(p => p.AllowScreenShare).HasDefaultValue(true);
                policyBuilder.Property(p => p.AllowChat).HasDefaultValue(true);
                policyBuilder.Property(p => p.AllowUnmuteSelf).HasDefaultValue(true);
                policyBuilder.Property(p => p.MuteOnEntry).HasDefaultValue(false);
                policyBuilder.Property(p => p.AllowRename).HasDefaultValue(true);
            });

            builder.HasMany(m => m.Sessions)
                .WithOne(s => s.Meeting)
                .HasForeignKey(s => s.MeetingId)
                .OnDelete(DeleteBehavior.Cascade);

            builder.HasMany(m => m.Recordings)
                .WithOne()
                .HasForeignKey(r => r.MeetingId)
                .OnDelete(DeleteBehavior.Cascade);

            builder.HasMany(m => m.Polls)
                .WithOne()
                .HasForeignKey(p => p.MeetingId)
                .OnDelete(DeleteBehavior.Cascade);

            builder.HasMany(m => m.BreakoutRooms)
                .WithOne()
                .HasForeignKey(b => b.MeetingId)
                .OnDelete(DeleteBehavior.Cascade);
        });

        modelBuilder.Entity<MeetingRecording>(builder =>
        {
            builder.ToTable("meeting_recordings");
            builder.HasKey(r => r.Id);
            builder.Property(r => r.StoragePath).HasMaxLength(500);
            builder.Property(r => r.StorageProvider).HasMaxLength(50);
            builder.Property(r => r.Status).HasConversion<string>().HasMaxLength(32);
            builder.HasIndex(r => r.MeetingId);
        });

        modelBuilder.Entity<Session>(builder =>
        {
            builder.ToTable("sessions");
            builder.HasKey(s => s.Id);
            builder.Property(s => s.SfuNodeId).IsRequired().HasMaxLength(100);

            builder.HasMany(s => s.Participations)
                .WithOne()
                .HasForeignKey(p => p.SessionId)
                .OnDelete(DeleteBehavior.Cascade);

            builder.HasMany(s => s.ChatMessages)
                .WithOne()
                .HasForeignKey(c => c.SessionId)
                .OnDelete(DeleteBehavior.Cascade);
        });

        modelBuilder.Entity<Participation>(builder =>
        {
            builder.ToTable("participations");
            builder.HasKey(p => p.Id);
            builder.Property(p => p.DisplayName).IsRequired().HasMaxLength(150);
            builder.Property(p => p.Role).HasConversion<string>().HasMaxLength(32);
            builder.Property(p => p.Status).HasConversion<string>().HasMaxLength(32);
            builder.Property(p => p.LeaveReason).HasConversion<string>().HasMaxLength(32);
            builder.HasIndex(p => p.SessionId);
            builder.HasIndex(p => p.UserId);
        });

        modelBuilder.Entity<ChatMessage>(builder =>
        {
            builder.ToTable("chat_messages");
            builder.HasKey(c => c.Id);
            builder.Property(c => c.UserName).IsRequired().HasMaxLength(150);
            builder.Property(c => c.Body).IsRequired().HasMaxLength(2000);
            builder.HasIndex(c => new { c.SessionId, c.SentAt });
        });

        modelBuilder.Entity<Poll>(builder =>
        {
            builder.ToTable("polls");
            builder.HasKey(p => p.Id);
            builder.Property(p => p.Question).IsRequired().HasMaxLength(500);
            builder.HasIndex(p => p.MeetingId);

            builder.HasMany(p => p.Options)
                .WithOne()
                .HasForeignKey(o => o.PollId)
                .OnDelete(DeleteBehavior.Cascade);

            builder.HasMany(p => p.Votes)
                .WithOne()
                .HasForeignKey(v => v.PollId)
                .OnDelete(DeleteBehavior.Cascade);
        });

        modelBuilder.Entity<PollOption>(builder =>
        {
            builder.ToTable("poll_options");
            builder.HasKey(o => o.Id);
            builder.Property(o => o.Text).IsRequired().HasMaxLength(200);
            builder.HasIndex(o => o.PollId);
        });

        modelBuilder.Entity<PollVote>(builder =>
        {
            builder.ToTable("poll_votes");
            builder.HasKey(v => v.Id);
            builder.HasIndex(v => new { v.PollId, v.VoterId });
        });

        modelBuilder.Entity<BreakoutRoom>(builder =>
        {
            builder.ToTable("breakout_rooms");
            builder.HasKey(b => b.Id);
            builder.Property(b => b.Name).IsRequired().HasMaxLength(150);
            builder.HasIndex(b => b.MeetingId);
        });
    }
}
