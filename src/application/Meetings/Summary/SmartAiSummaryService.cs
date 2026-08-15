using System.Text;
using System.Text.RegularExpressions;
using Confer.Domain.Meetings;
using Confer.Domain.Sessions;

namespace Confer.Application.Meetings.Summary;

public sealed partial class SmartAiSummaryService : IAiSummaryService
{
    private static readonly Regex ActionItemPrefixRegex = new(
        @"^(?:(?:TODO|Action Item|Action|Task|AI|Next Step|Follow[\s-]?up|\[[\s ]?\])\s*[:\-–]\s*)(.+)$",
        RegexOptions.IgnoreCase | RegexOptions.Compiled);

    private static readonly Regex AssignedToRegex = new(
        @"(?:assigned to|assignee\s*[:\-]?)\s*@?([A-Za-z0-9_\. -]+?)(?:\s*[:\-–]\s*|\s+to\s+|\s+for\s+)(.+)$",
        RegexOptions.IgnoreCase | RegexOptions.Compiled);

    private static readonly Regex MentionActionRegex = new(
        @"^@([A-Za-z0-9_\.-]+)\s+(?:please\s+|can you\s+|to\s+|will\s+)?(.+)$",
        RegexOptions.IgnoreCase | RegexOptions.Compiled);

    private static readonly Regex SelfCommitmentRegex = new(
        @"^(?:I will|I'll|we will|we'll)\s+(.+)$",
        RegexOptions.IgnoreCase | RegexOptions.Compiled);

    private static readonly Regex DecisionPrefixRegex = new(
        @"^(?:(?:Decision|Decided|Conclusion|Resolution|Agreed|Outcome)\s*[:\-–]\s*)(.+)$",
        RegexOptions.IgnoreCase | RegexOptions.Compiled);

    private static readonly Regex DecisionPhraseRegex = new(
        @"(?:(?:we decided to|we agreed to|agreed to|decided to|we have decided to|agreed that|concluded that|consensus is that|approved the)\s+)(.+)$",
        RegexOptions.IgnoreCase | RegexOptions.Compiled);

    public Task<AiSummaryResult> GenerateSummaryAsync(
        Meeting meeting,
        IReadOnlyList<ChatMessage> chatMessages,
        IReadOnlyList<Participation> participations,
        CancellationToken cancellationToken = default)
    {
        // 1. Calculate Meeting Duration
        int durationMinutes = CalculateDurationMinutes(meeting, participations);

        // 2. Calculate Unique Participants & Speaker Stats
        var participantList = participations ?? Array.Empty<Participation>();
        var uniqueUsers = participantList
            .GroupBy(p => p.UserId)
            .Select(g => g.First())
            .ToList();

        int participantCount = uniqueUsers.Count > 0 ? uniqueUsers.Count : 1;

        // 3. Extract Decisions and Action Items from Chat Transcripts
        var messages = chatMessages ?? Array.Empty<ChatMessage>();
        var keyDecisions = ExtractKeyDecisions(messages);
        var actionItems = ExtractActionItems(messages);

        // 4. Generate Comprehensive Overview
        var overview = GenerateOverview(meeting, durationMinutes, participantCount, uniqueUsers, messages, keyDecisions, actionItems);

        var result = new AiSummaryResult(
            Overview: overview,
            KeyDecisions: keyDecisions,
            ActionItems: actionItems,
            DurationMinutes: durationMinutes,
            ParticipantCount: participantCount
        );

        return Task.FromResult(result);
    }

    private static int CalculateDurationMinutes(Meeting meeting, IReadOnlyList<Participation> participations)
    {
        if (meeting.EndedAt.HasValue && meeting.CreatedAt != default)
        {
            var totalMins = (meeting.EndedAt.Value - meeting.CreatedAt).TotalMinutes;
            return Math.Max(1, (int)Math.Round(totalMins));
        }

        if (participations.Count > 0)
        {
            var earliestJoin = participations.Min(p => p.JoinedAt);
            var latestLeave = participations.Max(p => p.LeftAt ?? DateTime.UtcNow);
            var duration = (latestLeave - earliestJoin).TotalMinutes;
            return Math.Max(1, (int)Math.Round(duration));
        }

        return 1;
    }

    private static List<string> ExtractKeyDecisions(IReadOnlyList<ChatMessage> messages)
    {
        var decisions = new List<string>();
        var seen = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

        foreach (var msg in messages)
        {
            if (string.IsNullOrWhiteSpace(msg.Body)) continue;

            var lines = msg.Body.Split(new[] { '\r', '\n' }, StringSplitOptions.RemoveEmptyEntries);
            foreach (var rawLine in lines)
            {
                var line = rawLine.Trim();
                string? decisionText = null;

                var prefixMatch = DecisionPrefixRegex.Match(line);
                if (prefixMatch.Success)
                {
                    decisionText = prefixMatch.Groups[1].Value.Trim();
                }
                else
                {
                    var phraseMatch = DecisionPhraseRegex.Match(line);
                    if (phraseMatch.Success)
                    {
                        decisionText = phraseMatch.Groups[1].Value.Trim();
                    }
                }

                if (!string.IsNullOrWhiteSpace(decisionText))
                {
                    decisionText = CleanSentence(decisionText);
                    if (decisionText.Length >= 5 && seen.Add(decisionText))
                    {
                        decisions.Add(decisionText);
                    }
                }
            }
        }

        return decisions;
    }

    private static List<ActionItemDto> ExtractActionItems(IReadOnlyList<ChatMessage> messages)
    {
        var actionItems = new List<ActionItemDto>();
        var seen = new HashSet<string>(StringComparer.OrdinalIgnoreCase);

        foreach (var msg in messages)
        {
            if (string.IsNullOrWhiteSpace(msg.Body)) continue;

            var lines = msg.Body.Split(new[] { '\r', '\n' }, StringSplitOptions.RemoveEmptyEntries);
            foreach (var rawLine in lines)
            {
                var line = rawLine.Trim();
                string? title = null;
                string assignee = !string.IsNullOrWhiteSpace(msg.UserName) ? msg.UserName : "Unassigned";
                string status = "pending";

                // Check prefix
                var prefixMatch = ActionItemPrefixRegex.Match(line);
                if (prefixMatch.Success)
                {
                    var content = prefixMatch.Groups[1].Value.Trim();

                    // Check if content has assigned to or mention
                    var assignedMatch = AssignedToRegex.Match(content);
                    if (assignedMatch.Success)
                    {
                        assignee = assignedMatch.Groups[1].Value.Trim();
                        title = assignedMatch.Groups[2].Value.Trim();
                    }
                    else
                    {
                        var mentionMatch = MentionActionRegex.Match(content);
                        if (mentionMatch.Success)
                        {
                            assignee = mentionMatch.Groups[1].Value.Trim();
                            title = mentionMatch.Groups[2].Value.Trim();
                        }
                        else
                        {
                            title = content;
                        }
                    }
                }
                else
                {
                    var assignedMatch = AssignedToRegex.Match(line);
                    if (assignedMatch.Success)
                    {
                        assignee = assignedMatch.Groups[1].Value.Trim();
                        title = assignedMatch.Groups[2].Value.Trim();
                    }
                    else
                    {
                        var mentionMatch = MentionActionRegex.Match(line);
                        if (mentionMatch.Success)
                        {
                            assignee = mentionMatch.Groups[1].Value.Trim();
                            title = mentionMatch.Groups[2].Value.Trim();
                        }
                        else
                        {
                            var selfMatch = SelfCommitmentRegex.Match(line);
                            if (selfMatch.Success)
                            {
                                assignee = !string.IsNullOrWhiteSpace(msg.UserName) ? msg.UserName : "Self";
                                title = selfMatch.Groups[1].Value.Trim();
                            }
                        }
                    }
                }

                if (!string.IsNullOrWhiteSpace(title))
                {
                    title = CleanSentence(title);

                    // Check for completion status tags
                    if (line.Contains("[x]", StringComparison.OrdinalIgnoreCase) ||
                        line.Contains("(done)", StringComparison.OrdinalIgnoreCase) ||
                        line.Contains("completed", StringComparison.OrdinalIgnoreCase))
                    {
                        status = "completed";
                    }
                    else if (line.Contains("in progress", StringComparison.OrdinalIgnoreCase) ||
                             line.Contains("wip", StringComparison.OrdinalIgnoreCase))
                    {
                        status = "in_progress";
                    }

                    var key = $"{assignee}:{title}";
                    if (title.Length >= 4 && seen.Add(key))
                    {
                        actionItems.Add(new ActionItemDto(title, assignee, status));
                    }
                }
            }
        }

        return actionItems;
    }

    private static string GenerateOverview(
        Meeting meeting,
        int durationMinutes,
        int participantCount,
        IReadOnlyList<Participation> participants,
        IReadOnlyList<ChatMessage> chatMessages,
        IReadOnlyList<string> keyDecisions,
        IReadOnlyList<ActionItemDto> actionItems)
    {
        var sb = new StringBuilder();

        sb.Append($"Meeting '{meeting.Title}' was held with {participantCount} participant(s) ");
        sb.Append($"for a duration of approximately {durationMinutes} minute(s). ");

        if (participants.Count > 0)
        {
            var names = participants
                .Select(p => p.DisplayName)
                .Where(n => !string.IsNullOrWhiteSpace(n))
                .Distinct()
                .Take(5)
                .ToList();

            if (names.Count > 0)
            {
                sb.Append($"Attendees included: {string.Join(", ", names)}");
                if (participants.Count > 5)
                {
                    sb.Append($" and {participants.Count - 5} other(s)");
                }
                sb.Append(". ");
            }
        }

        if (chatMessages.Count > 0)
        {
            sb.Append($"During the session, {chatMessages.Count} chat message(s) were exchanged covering discussion topics and collaborative feedback. ");
        }
        else
        {
            sb.Append("No in-meeting chat messages were logged. ");
        }

        if (keyDecisions.Count > 0)
        {
            sb.Append($"The group established {keyDecisions.Count} key decision(s). ");
        }

        if (actionItems.Count > 0)
        {
            sb.Append($"{actionItems.Count} action item(s) were identified and assigned for follow-up.");
        }
        else
        {
            sb.Append("All agenda topics were reviewed and recorded.");
        }

        return sb.ToString();
    }

    private static string CleanSentence(string input)
    {
        var cleaned = input.Trim().TrimEnd('.', ';', ',');
        if (cleaned.Length == 0) return cleaned;
        return char.ToUpperInvariant(cleaned[0]) + cleaned[1..];
    }
}
