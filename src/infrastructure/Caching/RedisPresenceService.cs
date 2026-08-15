using System.Collections.Concurrent;
using System.Text.Json;
using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Microsoft.Extensions.Logging;
using StackExchange.Redis;

namespace Confer.Infrastructure.Caching;

public sealed class RedisPresenceService : IPresenceService
{
    private const string RosterKeyPrefix = "confer:presence:";
    private const string AffinityKeyPrefix = "confer:affinity:";

    private readonly ILogger<RedisPresenceService> _logger;
    private readonly IConnectionMultiplexer? _redis;

    // Fallback used only when Redis isn't configured/reachable, so a single dev instance
    // (no Redis running) keeps working exactly as before. With Redis present, none of this
    // is touched, and roster/affinity state is shared across every instance in the pool.
    private readonly ConcurrentDictionary<Guid, ConcurrentDictionary<Guid, ParticipantStateDto>> _localRosters = new();
    private readonly ConcurrentDictionary<Guid, string> _localAffinities = new();

    public RedisPresenceService(IConnectionMultiplexer? redis, ILogger<RedisPresenceService> logger)
    {
        _redis = redis;
        _logger = logger;
    }

    public async Task SetParticipantOnlineAsync(Guid meetingId, ParticipantStateDto participant)
    {
        if (_redis is { } redis)
        {
            var db = redis.GetDatabase();
            await db.HashSetAsync(RosterKey(meetingId), participant.ParticipantId.ToString(), JsonSerializer.Serialize(participant));
            return;
        }

        var roster = _localRosters.GetOrAdd(meetingId, _ => new ConcurrentDictionary<Guid, ParticipantStateDto>());
        roster[participant.ParticipantId] = participant;
    }

    public async Task SetParticipantOfflineAsync(Guid meetingId, Guid participantId)
    {
        if (_redis is { } redis)
        {
            await redis.GetDatabase().HashDeleteAsync(RosterKey(meetingId), participantId.ToString());
            return;
        }

        if (_localRosters.TryGetValue(meetingId, out var roster))
        {
            roster.TryRemove(participantId, out _);
        }
    }

    public async Task<List<ParticipantStateDto>> GetRosterAsync(Guid meetingId)
    {
        if (_redis is { } redis)
        {
            var entries = await redis.GetDatabase().HashGetAllAsync(RosterKey(meetingId));
            var roster = new List<ParticipantStateDto>(entries.Length);
            foreach (var entry in entries)
            {
                try
                {
                    var participant = JsonSerializer.Deserialize<ParticipantStateDto>((string)entry.Value!);
                    if (participant is not null) roster.Add(participant);
                }
                catch (JsonException ex)
                {
                    _logger.LogWarning(ex, "Skipping malformed roster entry for meeting {MeetingId}", meetingId);
                }
            }
            return roster;
        }

        if (_localRosters.TryGetValue(meetingId, out var localRoster))
        {
            return localRoster.Values.ToList();
        }
        return new List<ParticipantStateDto>();
    }

    public async Task<int> GetActiveCountAsync(Guid meetingId)
    {
        if (_redis is { } redis)
        {
            return (int)await redis.GetDatabase().HashLengthAsync(RosterKey(meetingId));
        }

        if (_localRosters.TryGetValue(meetingId, out var roster))
        {
            return roster.Count;
        }
        return 0;
    }

    public async Task SetNodeAffinityAsync(Guid meetingId, string nodeId, TimeSpan ttl)
    {
        if (_redis is { } redis)
        {
            await redis.GetDatabase().StringSetAsync(AffinityKey(meetingId), nodeId, ttl);
            return;
        }

        _localAffinities[meetingId] = nodeId;
    }

    public async Task<string?> GetNodeAffinityAsync(Guid meetingId)
    {
        if (_redis is { } redis)
        {
            var value = await redis.GetDatabase().StringGetAsync(AffinityKey(meetingId));
            return value.IsNullOrEmpty ? null : value.ToString();
        }

        _localAffinities.TryGetValue(meetingId, out var nodeId);
        return nodeId;
    }

    private static string RosterKey(Guid meetingId) => $"{RosterKeyPrefix}{meetingId}";
    private static string AffinityKey(Guid meetingId) => $"{AffinityKeyPrefix}{meetingId}";
}
