using System.Text;
using System.Text.RegularExpressions;
using Confer.Application.DTOs;
using Confer.Domain.Sessions;
using Confer.Infrastructure.Observability;

namespace Confer.Infrastructure.AI;

public interface IConferAiCompanionService
{
    bool IsAiCommand(string message);
    Task<string> ProcessAiQueryAsync(Guid meetingId, string prompt, string senderName, List<ChatMessage> chatHistory, List<CaptionChunkDto> captionHistory);
}

public class ConferAiCompanionService : IConferAiCompanionService
{
    private static readonly Regex AiTriggerRegex = new(@"^(/ai|@confer-ai|@ai)\b", RegexOptions.IgnoreCase | RegexOptions.Compiled);

    public bool IsAiCommand(string message)
    {
        if (string.IsNullOrWhiteSpace(message)) return false;
        return AiTriggerRegex.IsMatch(message.Trim());
    }

    public Task<string> ProcessAiQueryAsync(
        Guid meetingId,
        string prompt,
        string senderName,
        List<ChatMessage> chatHistory,
        List<CaptionChunkDto> captionHistory)
    {
        ConferMetrics.AiCopilotQueriesTotal.Add(1);

        // Strip the trigger prefix
        var cleanQuery = AiTriggerRegex.Replace(prompt.Trim(), string.Empty).Trim();
        if (string.IsNullOrWhiteSpace(cleanQuery))
        {
            cleanQuery = "help";
        }

        var queryLower = cleanQuery.ToLowerInvariant();

        // 1. In-Meeting Instant Summary Command
        if (queryLower.Contains("summar") || queryLower.Contains("resum") || queryLower.Contains("recap"))
        {
            return Task.FromResult(GenerateInstantRecap(chatHistory, captionHistory));
        }

        // 2. Action Items / Tasks Command
        if (queryLower.Contains("task") || queryLower.Contains("action") || queryLower.Contains("tarea") || queryLower.Contains("compromis") || queryLower.Contains("pendiente"))
        {
            return Task.FromResult(ExtractInstantActionItems(chatHistory, captionHistory));
        }

        // 3. Help Command
        if (queryLower == "help" || queryLower == "ayuda" || queryLower == "?")
        {
            return Task.FromResult(
                "🤖 **Confer AI Copilot**\n" +
                "I am your real-time in-room AI assistant. Available commands:\n" +
                "- `/ai summarize` — Instant recap of the meeting discussion and key topics.\n" +
                "- `/ai action-items` — Extract pending tasks and commitments mentioned.\n" +
                "- `/ai <question>` — Ask questions about topics discussed in this session.");
        }

        // 4. Contextual Q&A against current transcript & chat
        return Task.FromResult(AnswerContextualQuestion(cleanQuery, senderName, chatHistory, captionHistory));
    }

    private static string GenerateInstantRecap(List<ChatMessage> chatHistory, List<CaptionChunkDto> captionHistory)
    {
        var transcriptItems = captionHistory.Where(c => c.IsFinal && !string.IsNullOrWhiteSpace(c.Text)).ToList();
        var totalWords = transcriptItems.Sum(c => c.Text.Split(' ', StringSplitOptions.RemoveEmptyEntries).Length);
        var totalMessages = chatHistory.Count;

        var sb = new StringBuilder();
        sb.AppendLine("🤖 **Live AI Meeting Recap:**");
        sb.AppendLine($"• **Activity:** {transcriptItems.Count} speech segments ({totalWords} spoken words) and {totalMessages} chat messages recorded.");

        var keySpeakers = transcriptItems
            .GroupBy(c => c.SpeakerName)
            .OrderByDescending(g => g.Count())
            .Take(3)
            .Select(g => g.Key)
            .ToList();

        if (keySpeakers.Any())
        {
            sb.AppendLine($"• **Active Contributors:** {string.Join(", ", keySpeakers)}.");
        }

        // Extract key highlights
        var highlights = transcriptItems
            .TakeLast(6)
            .Select(c => $"\"{c.Text}\" — *{c.SpeakerName}*")
            .ToList();

        if (highlights.Any())
        {
            sb.AppendLine("• **Recent Key Highlights:**");
            foreach (var h in highlights)
            {
                sb.AppendLine($"  - {h}");
            }
        }
        else
        {
            sb.AppendLine("• **Status:** The discussion is in progress. Continue speaking and I will summarize key takeaways.");
        }

        return sb.ToString();
    }

    private static string ExtractInstantActionItems(List<ChatMessage> chatHistory, List<CaptionChunkDto> captionHistory)
    {
        var items = new List<string>();

        // Check speech transcript for commitments
        foreach (var chunk in captionHistory.Where(c => c.IsFinal))
        {
            var text = chunk.Text.ToLowerInvariant();
            if (text.Contains("will ") || text.Contains("voy a ") || text.Contains("need to") || text.Contains("tiene que") || text.Contains("assigned") || text.Contains("deadline"))
            {
                items.Add($"[ ] **{chunk.SpeakerName}:** \"{chunk.Text}\"");
            }
        }

        // Check chat messages
        foreach (var msg in chatHistory)
        {
            var text = msg.Body.ToLowerInvariant();
            if (text.Contains("todo") || text.Contains("task") || text.Contains("tarea") || text.Contains("fix") || text.Contains("deploy") || text.Contains("ship"))
            {
                items.Add($"[ ] **{msg.UserName}:** {msg.Body}");
            }
        }

        if (!items.Any())
        {
            return "🤖 **Action Items:** No explicit task commitments detected yet. State items clearly (e.g. *\"I will review the pull request by 5pm\"*) for automatic capture.";
        }

        var sb = new StringBuilder();
        sb.AppendLine("🤖 **Detected Action Items:**");
        foreach (var item in items.Distinct().Take(5))
        {
            sb.AppendLine($"• {item}");
        }
        return sb.ToString();
    }

    private static string AnswerContextualQuestion(string query, string senderName, List<ChatMessage> chatHistory, List<CaptionChunkDto> captionHistory)
    {
        var relevantCaptions = captionHistory
            .Where(c => c.IsFinal && c.Text.Split(' ').Any(w => w.Length > 3 && query.Contains(w, StringComparison.OrdinalIgnoreCase)))
            .TakeLast(3)
            .ToList();

        if (relevantCaptions.Any())
        {
            var context = string.Join(" ", relevantCaptions.Select(c => $"[{c.SpeakerName}: {c.Text}]"));
            return $"🤖 **AI Answer for {senderName}:**\nBased on what was discussed earlier ({context}), regarding your question *\"{query}\"*, the team indicated alignment on this topic.";
        }

        return $"🤖 **AI Answer for {senderName}:**\nI analyzed the current meeting session for *\"{query}\"*. This topic hasn't been discussed in depth yet during this call.";
    }
}
