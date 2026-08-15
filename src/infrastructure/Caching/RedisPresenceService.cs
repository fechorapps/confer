using System.Collections.Concurrent;
using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.Logging;
using StackExchange.Redis;

namespace Confer.Infrastructure.Caching;

public sealed class RedisPresenceService : IPresenceService
{
    private readonly ILogger<RedisPresenceService> _logger;
    private readonly IConnectionMultiplexer? _redis;
    private readonly ConcurrentDictionary<Guid, ConcurrentDictionary<Guid, ParticipantStateDto>> _localRosters = new();
    private readonly ConcurrentDictionary<Guid, string> _localAffinities = new();

    public RedisPresenceService(IConfiguration configuration, ILogger<RedisPresenceService> logger)
    {
        _logger = logger;
        var redisConn = configuration.GetConnectionString("Redis");
        if (!string.IsNullOrWhiteSpace(redisConn))
        {
            try
            {
                _redis = ConnectionMultiplexer.Connect(redisConn);
                _logger.LogInformation("Connected to Redis at {RedisConnection}", redisConn);
            }
            catch (Exception ex)
            {
                _logger.LogWarning(ex, "Failed to connect to Redis. Falling back to in-memory presence tracking.");
            }
        }
    }

    public Task SetParticipantOnlineAsync(Guid meetingId, ParticipantStateDto participant)
    {
        var roster = _localRosters.GetOrAdd(meetingId, _ => new ConcurrentDictionary<Guid, ParticipantStateDto>());
        roster[participant.ParticipantId] = participant;
        return Task.CompletedTask;
    }

    public Task SetParticipantOfflineAsync(Guid meetingId, Guid participantId)
    {
        if (_localRosters.TryGetValue(meetingId, out var roster))
        {
            roster.TryRemove(participantId, out _);
        }
        return Task.CompletedTask;
    }

    public Task<List<ParticipantStateDto>> GetRosterAsync(Guid meetingId)
    {
        if (_localRosters.TryGetValue(meetingId, out var roster))
        {
            return Task.FromResult(roster.Values.ToList());
        }
        return Task.FromResult(new List<ParticipantStateDto>());
    }

    public Task<int> GetActiveCountAsync(Guid meetingId)
    {
        if (_localRosters.TryGetValue(meetingId, out var roster))
        {
            return Task.FromResult(roster.Count);
        }
        return Task.FromResult(0);
    }

    public Task SetNodeAffinityAsync(Guid meetingId, string nodeId, TimeSpan ttl)
    {
        _localAffinities[meetingId] = nodeId;
        return Task.CompletedTask;
    }

    public Task<string?> GetNodeAffinityAsync(Guid meetingId)
    {
        _localAffinities.TryGetValue(meetingId, out var nodeId);
        return Task.FromResult(nodeId);
    }
}
