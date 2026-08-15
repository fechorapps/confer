using Confer.Application.Interfaces;
using Confer.Infrastructure.Caching;
using Confer.Infrastructure.Media;
using Confer.Infrastructure.Persistence;
using Confer.Infrastructure.Security;
using Confer.Infrastructure.Signal;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;

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
            }
            else
            {
                options.UseSqlite(sqliteConn);
            }
        });

        services.AddScoped<IConferDbContext>(sp => sp.GetRequiredService<ConferDbContext>());

        // Security & Tokens
        services.AddSingleton<ITokenProvider, JwtTokenProvider>();

        // Presence & Caching
        services.AddSingleton<IPresenceService, RedisPresenceService>();

        // Signaling Handler (Singleton so WebSocket sessions are retained)
        services.AddSingleton<WebSocketSignalingHandler>();
        services.AddSingleton<ISignalingNotifier>(sp => sp.GetRequiredService<WebSocketSignalingHandler>());

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

        return services;
    }
}
