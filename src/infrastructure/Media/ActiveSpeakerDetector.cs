using System.Collections.Concurrent;
using Confer.Application.DTOs;

namespace Confer.Infrastructure.Media;

public sealed class ActiveSpeakerDetector
{
    private readonly ConcurrentDictionary<Guid, double> _smoothedEnergies = new();
    private readonly ConcurrentDictionary<Guid, DateTime> _lastActivity = new();

    private const double AttackAlpha = 0.6;   // Fast attack (200 ms)
    private const double ReleaseAlpha = 0.15; // Slow release (600 ms)
    private const double SpeechThresholdDbov = -50.0; // Above -50 dBov is considered speech

    public void ReportAudioLevel(Guid participantId, int levelDbov)
    {
        // levelDbov: 0 (loudest) to -127 (silence)
        var clampedLevel = Math.Clamp(levelDbov, -127, 0);
        _lastActivity[participantId] = DateTime.UtcNow;

        _smoothedEnergies.AddOrUpdate(
            participantId,
            clampedLevel,
            (_, currentSmoothed) =>
            {
                var alpha = clampedLevel > currentSmoothed ? AttackAlpha : ReleaseAlpha;
                return (alpha * clampedLevel) + ((1 - alpha) * currentSmoothed);
            });
    }

    public List<SpeakerInfoDto> GetTopSpeakers(int maxCount = 3)
    {
        var now = DateTime.UtcNow;

        // Expire inactive participants after 2 seconds of silence
        var activeEntries = _smoothedEnergies
            .Where(kvp => _lastActivity.TryGetValue(kvp.Key, out var lastTime) && now - lastTime < TimeSpan.FromSeconds(2.0))
            .Where(kvp => kvp.Value > SpeechThresholdDbov)
            .OrderByDescending(kvp => kvp.Value)
            .Take(maxCount)
            .Select(kvp => new SpeakerInfoDto(kvp.Key, (int)Math.Round(kvp.Value)))
            .ToList();

        return activeEntries;
    }

    public void RemoveParticipant(Guid participantId)
    {
        _smoothedEnergies.TryRemove(participantId, out _);
        _lastActivity.TryRemove(participantId, out _);
    }
}
