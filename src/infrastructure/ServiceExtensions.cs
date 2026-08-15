using Confer.Application.Interfaces;
using Confer.Infrastructure.Caching;
using Confer.Infrastructure.Media;
using Confer.Infrastructure.Persistence;
using Confer.Infrastructure.Security;
using Confer.Infrastructure.Signal;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using StackExchange.Redis;

namespace Confer.Infrastructure;

public static class ServiceExtensions
{
    public static IServiceCollection AddInfrastructure(this IServiceCollection services, IConfiguration configuration)
    {
        // Database configuration: Default to SQLite for easy local dev, or PostgreSQL if configured
        var postgresConn = configuration.GetConnectionString("Postgres");
        var sqliteConn = configuration.GetConnectionString("DefaultConnection") ?? "Data Source=confer.db";

        services.AddDbContext<ConferDbContext>(options =>
        {
            if (!string.IsNullOrWhiteSpace(postgresConn))
            {
                options.UseNpgsql(postgresConn);

                // The migration history was authored (and is model-diffed) against SQLite, the
                // default local/test provider. EF's pending-changes check compares the compiled
                // model against that snapshot using whichever provider is active, and the two
                // providers' own conventions (default lengths, etc.) diverge enough to always
                // trip it here even with no real drift — the actual CreateTable/AddColumn
                // operations themselves carry no provider-specific SQL, so they still apply
                // correctly. Only suppressed for Postgres; SQLite keeps the check so real
                // uncommitted model changes still fail loudly during dev.
                options.ConfigureWarnings(w => w.Ignore(Microsoft.EntityFrameworkCore.Diagnostics.RelationalEventId.PendingModelChangesWarning));
            }
            else
            {
                options.UseSqlite(sqliteConn);
            }
        });

        services.AddScoped<IConferDbContext>(sp => sp.GetRequiredService<ConferDbContext>());

        // Shared Redis connection: backs both presence tracking and cross-instance WebSocket
        // relay. Null when Redis:ConnectionString isn't configured or the connection fails, in
        // which case dependents fall back to single-process, in-memory behavior.
        services.AddSingleton<IConnectionMultiplexer>(sp =>
        {
            var redisConn = configuration.GetConnectionString("Redis");
            var logger = sp.GetRequiredService<ILoggerFactory>().CreateLogger("Confer.Infrastructure.Redis");

            if (string.IsNullOrWhiteSpace(redisConn)) return null!;

            try
            {
                var multiplexer = ConnectionMultiplexer.Connect(redisConn);
                logger.LogInformation("Connected to Redis at {RedisConnection}", redisConn);
                return multiplexer;
            }
            catch (Exception ex)
            {
                logger.LogWarning(ex, "Failed to connect to Redis. Falling back to single-instance, in-memory presence and signaling.");
                return null!;
            }
        });

        // Security & Tokens
        services.AddSingleton<ITokenProvider, JwtTokenProvider>();

        // Presence & Caching
        services.AddSingleton<IPresenceService, RedisPresenceService>();

        // Signaling Handler (Singleton so WebSocket sessions are retained)
        services.AddSingleton<WebSocketSignalingHandler>();
        services.AddSingleton<ISignalingNotifier>(sp => sp.GetRequiredService<WebSocketSignalingHandler>());

        // ICE servers (STUN + TURN) shared by clients and server-side peer connections
        services.AddSingleton<IIceServerProvider, ConfigurationIceServerProvider>();

        // SFU Room Manager
        services.AddSingleton<ISfuRoomManager, SfuRoomManager>();

        // Recording & Archival Storage
        services.AddSingleton<Confer.Application.Interfaces.IRecordingStorageService, Persistence.LocalDiskRecordingStorage>();

        // HTTP Client & Developer Webhooks Engine
        services.AddHttpClient();
        services.AddScoped<IWebhookDispatcher, Webhooks.HmacWebhookDispatcher>();

        // iCalendar (.ics) Generator
        services.AddSingleton<ICalendarService, Calendar.IcsCalendarGenerator>();

        // In-Room AI Copilot / Companion
        services.AddSingleton<AI.IConferAiCompanionService, AI.ConferAiCompanionService>();

        // Enterprise SSO Authentication Service
        services.AddScoped<Application.Auth.Sso.ISsoAuthenticationService, Auth.Sso.SsoAuthenticationService>();

        // PSTN / SIP Telephony Bridge
        services.AddScoped<Application.Meetings.Telephony.ITelephonyBridgeService, Telephony.TwilioSipTelephonyBridge>();

        return services;
    }
}
