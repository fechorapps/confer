using System.Reflection;
using Confer.Application;
using Confer.Domain.Identity;
using Confer.Infrastructure;
using Confer.Infrastructure.Persistence;
using Confer.Shared.Api;
using Microsoft.EntityFrameworkCore;

var builder = WebApplication.CreateBuilder(args);

// Add Layers
builder.Services.AddApplication();
builder.Services.AddInfrastructure(builder.Configuration);

// Add Services
builder.Services.AddEndpointsApiExplorer();
builder.Services.AddSwaggerGen();
builder.Services.AddCors(options =>
{
    options.AddDefaultPolicy(policy =>
    {
        policy.AllowAnyOrigin()
              .AllowAnyHeader()
              .AllowAnyMethod();
    });
});

var app = builder.Build();

// Auto-migrate & seed test users
using (var scope = app.Services.CreateScope())
{
    var db = scope.ServiceProvider.GetRequiredService<ConferDbContext>();
    await db.Database.EnsureCreatedAsync();

    if (db.Database.IsSqlite())
    {
        try { await db.Database.ExecuteSqlRawAsync(@"ALTER TABLE ""meetings"" ADD COLUMN ""IsWaitingRoomEnabled"" INTEGER NOT NULL DEFAULT 0;"); } catch { }
        try { await db.Database.ExecuteSqlRawAsync(@"ALTER TABLE ""meetings"" ADD COLUMN ""IsWatermarkEnabled"" INTEGER NOT NULL DEFAULT 0;"); } catch { }
        try { await db.Database.ExecuteSqlRawAsync(@"ALTER TABLE ""meetings"" ADD COLUMN ""Policy_AllowScreenShare"" INTEGER NOT NULL DEFAULT 1;"); } catch { }
        try { await db.Database.ExecuteSqlRawAsync(@"ALTER TABLE ""meetings"" ADD COLUMN ""Policy_AllowChat"" INTEGER NOT NULL DEFAULT 1;"); } catch { }
        try { await db.Database.ExecuteSqlRawAsync(@"ALTER TABLE ""meetings"" ADD COLUMN ""Policy_AllowUnmuteSelf"" INTEGER NOT NULL DEFAULT 1;"); } catch { }
        try { await db.Database.ExecuteSqlRawAsync(@"ALTER TABLE ""meetings"" ADD COLUMN ""Policy_MuteOnEntry"" INTEGER NOT NULL DEFAULT 0;"); } catch { }
        try { await db.Database.ExecuteSqlRawAsync(@"ALTER TABLE ""meetings"" ADD COLUMN ""Policy_AllowRename"" INTEGER NOT NULL DEFAULT 1;"); } catch { }
        try { await db.Database.ExecuteSqlRawAsync(@"ALTER TABLE ""participations"" ADD COLUMN ""Status"" TEXT NOT NULL DEFAULT 'Admitted';"); } catch { }
        try { await db.Database.ExecuteSqlRawAsync(@"ALTER TABLE ""participations"" ADD COLUMN ""AdmittedAt"" TEXT NULL;"); } catch { }
        try { await db.Database.ExecuteSqlRawAsync(@"ALTER TABLE ""meetings"" ADD COLUMN ""LiveStreamConfig_IsStreamingEnabled"" INTEGER NOT NULL DEFAULT 0;"); } catch { }
        try { await db.Database.ExecuteSqlRawAsync(@"ALTER TABLE ""meetings"" ADD COLUMN ""LiveStreamConfig_RtmpUrl"" TEXT NOT NULL DEFAULT '';"); } catch { }
        try { await db.Database.ExecuteSqlRawAsync(@"ALTER TABLE ""meetings"" ADD COLUMN ""LiveStreamConfig_StreamKey"" TEXT NOT NULL DEFAULT '';"); } catch { }
        try { await db.Database.ExecuteSqlRawAsync(@"ALTER TABLE ""meetings"" ADD COLUMN ""LiveStreamConfig_Status"" TEXT NOT NULL DEFAULT 'Idle';"); } catch { }
        try { await db.Database.ExecuteSqlRawAsync(@"ALTER TABLE ""meetings"" ADD COLUMN ""LiveStreamConfig_StartedAt"" TEXT NULL;"); } catch { }

        await db.Database.ExecuteSqlRawAsync(@"
            CREATE TABLE IF NOT EXISTS ""polls"" (
                ""Id"" TEXT NOT NULL PRIMARY KEY,
                ""MeetingId"" TEXT NOT NULL,
                ""CreatorId"" TEXT NOT NULL,
                ""Question"" TEXT NOT NULL,
                ""IsAnonymous"" INTEGER NOT NULL,
                ""IsMultiChoice"" INTEGER NOT NULL,
                ""IsActive"" INTEGER NOT NULL,
                ""CreatedAt"" TEXT NOT NULL,
                ""ClosedAt"" TEXT NULL,
                CONSTRAINT ""FK_polls_meetings_MeetingId"" FOREIGN KEY (""MeetingId"") REFERENCES ""meetings"" (""Id"") ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS ""IX_polls_MeetingId"" ON ""polls"" (""MeetingId"");

            CREATE TABLE IF NOT EXISTS ""poll_options"" (
                ""Id"" TEXT NOT NULL PRIMARY KEY,
                ""PollId"" TEXT NOT NULL,
                ""Text"" TEXT NOT NULL,
                ""Index"" INTEGER NOT NULL,
                ""VoteCount"" INTEGER NOT NULL,
                CONSTRAINT ""FK_poll_options_polls_PollId"" FOREIGN KEY (""PollId"") REFERENCES ""polls"" (""Id"") ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS ""IX_poll_options_PollId"" ON ""poll_options"" (""PollId"");

            CREATE TABLE IF NOT EXISTS ""poll_votes"" (
                ""Id"" TEXT NOT NULL PRIMARY KEY,
                ""PollId"" TEXT NOT NULL,
                ""OptionId"" TEXT NOT NULL,
                ""VoterId"" TEXT NOT NULL,
                ""VotedAt"" TEXT NOT NULL,
                CONSTRAINT ""FK_poll_votes_polls_PollId"" FOREIGN KEY (""PollId"") REFERENCES ""polls"" (""Id"") ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS ""IX_poll_votes_PollId_VoterId"" ON ""poll_votes"" (""PollId"", ""VoterId"");

            CREATE TABLE IF NOT EXISTS ""breakout_rooms"" (
                ""Id"" TEXT NOT NULL PRIMARY KEY,
                ""MeetingId"" TEXT NOT NULL,
                ""SessionId"" TEXT NOT NULL,
                ""Name"" TEXT NOT NULL,
                ""Index"" INTEGER NOT NULL,
                ""CreatedAt"" TEXT NOT NULL,
                ""EndsAt"" TEXT NULL,
                ""MaxDurationMinutes"" INTEGER NOT NULL,
                CONSTRAINT ""FK_breakout_rooms_meetings_MeetingId"" FOREIGN KEY (""MeetingId"") REFERENCES ""meetings"" (""Id"") ON DELETE CASCADE
            );
            CREATE INDEX IF NOT EXISTS ""IX_breakout_rooms_MeetingId"" ON ""breakout_rooms"" (""MeetingId"");

            CREATE TABLE IF NOT EXISTS ""meeting_summaries"" (
                ""Id"" TEXT NOT NULL PRIMARY KEY,
                ""MeetingId"" TEXT NOT NULL,
                ""Overview"" TEXT NOT NULL,
                ""KeyDecisions"" TEXT NOT NULL,
                ""ActionItems"" TEXT NOT NULL,
                ""GeneratedAt"" TEXT NOT NULL,
                ""DurationMinutes"" INTEGER NOT NULL,
                ""ParticipantCount"" INTEGER NOT NULL,
                CONSTRAINT ""FK_meeting_summaries_meetings_MeetingId"" FOREIGN KEY (""MeetingId"") REFERENCES ""meetings"" (""Id"") ON DELETE CASCADE
            );
            CREATE UNIQUE INDEX IF NOT EXISTS ""IX_meeting_summaries_MeetingId"" ON ""meeting_summaries"" (""MeetingId"");

            CREATE TABLE IF NOT EXISTS ""webhook_subscriptions"" (
                ""Id"" TEXT NOT NULL PRIMARY KEY,
                ""UserId"" TEXT NOT NULL,
                ""TargetUrl"" TEXT NOT NULL,
                ""Secret"" TEXT NOT NULL,
                ""SubscribedEvents"" TEXT NOT NULL,
                ""IsActive"" INTEGER NOT NULL DEFAULT 1,
                ""CreatedAt"" TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS ""IX_webhook_subscriptions_UserId"" ON ""webhook_subscriptions"" (""UserId"");
        ");
    }

    if (!await db.Users.AnyAsync())
    {
        db.Users.AddRange(
            User.CreateWithId(Guid.Parse("11111111-1111-1111-1111-111111111111"), "host@confer.local", "Host User (Dev)"),
            User.CreateWithId(Guid.Parse("22222222-2222-2222-2222-222222222222"), "participant1@confer.local", "Alice (Dev)"),
            User.CreateWithId(Guid.Parse("33333333-3333-3333-3333-333333333333"), "participant2@confer.local", "Bob (Dev)")
        );
        await db.SaveChangesAsync();
    }
}

if (app.Environment.IsDevelopment())
{
    app.UseSwagger();
    app.UseSwaggerUI();
}

app.UseDefaultFiles();
app.UseStaticFiles();

app.UseWebSockets(new WebSocketOptions
{
    KeepAliveInterval = TimeSpan.FromSeconds(15)
});

// Health probes
app.MapGet("/v1/health", () => TypedResults.Ok(new { status = "healthy", timestamp = DateTime.UtcNow }));

// Auto-map all IEndpoint implementations
var endpointTypes = Assembly.GetExecutingAssembly().GetTypes()
    .Where(t => typeof(IEndpoint).IsAssignableFrom(t) && !t.IsInterface && !t.IsAbstract);

foreach (var type in endpointTypes)
{
    if (Activator.CreateInstance(type) is IEndpoint endpoint)
    {
        endpoint.MapEndpoint(app);
    }
}

app.Run();

// Required for WebApplicationFactory in integration tests
public partial class Program { }
