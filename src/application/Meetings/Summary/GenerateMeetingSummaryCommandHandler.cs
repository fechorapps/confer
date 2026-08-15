using Confer.Application.Interfaces;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.Summary;

public sealed class GenerateMeetingSummaryCommandHandler(
    IConferDbContext dbContext,
    IAiSummaryService aiSummaryService)
    : ICommandHandler<GenerateMeetingSummaryCommand, Result<MeetingSummaryDto>>
{
    public async Task<Result<MeetingSummaryDto>> HandleAsync(
        GenerateMeetingSummaryCommand command,
        CancellationToken cancellationToken = default)
    {
        var meeting = await dbContext.Meetings
            .Include(m => m.Summary)
            .Include(m => m.Sessions)
                .ThenInclude(s => s.ChatMessages)
            .Include(m => m.Sessions)
                .ThenInclude(s => s.Participations)
            .FirstOrDefaultAsync(m => m.Id == command.MeetingId, cancellationToken);

        if (meeting is null)
            return Result.Failure<MeetingSummaryDto>(MeetingErrors.NotFound);

        // Check actor authorization: meeting owner or admitted participant
        var isOwner = meeting.OwnerId == command.ActorId;
        var isParticipant = meeting.Sessions.Any(s => s.Participations.Any(p => p.UserId == command.ActorId));
        if (!isOwner && !isParticipant)
            return Result.Failure<MeetingSummaryDto>(MeetingErrors.Unauthorized);

        // Meeting must have ended before generating a summary
        if (!meeting.EndedAt.HasValue)
            return Result.Failure<MeetingSummaryDto>(MeetingErrors.MeetingNotEnded);

        // Return existing summary if already generated and ForceRegenerate is false
        if (meeting.Summary is not null && !command.ForceRegenerate)
        {
            return Result.Success(MapToDto(meeting.Summary));
        }

        // Gather all session chats and participations
        var chatMessages = meeting.Sessions
            .SelectMany(s => s.ChatMessages)
            .OrderBy(c => c.SentAt)
            .ToList();

        var participations = meeting.Sessions
            .SelectMany(s => s.Participations)
            .ToList();

        // Run AI Summary generation engine
        var aiResult = await aiSummaryService.GenerateSummaryAsync(
            meeting,
            chatMessages,
            participations,
            cancellationToken);

        var domainActionItems = aiResult.ActionItems
            .Select(a => new ActionItem(a.Title, a.Assignee, a.Status))
            .ToList();

        if (meeting.Summary is not null)
        {
            dbContext.MeetingSummaries.Remove(meeting.Summary);
        }

        var summaryResult = meeting.AddSummary(
            aiResult.Overview,
            aiResult.KeyDecisions,
            domainActionItems,
            aiResult.DurationMinutes,
            aiResult.ParticipantCount);

        if (summaryResult.IsFailure)
            return Result.Failure<MeetingSummaryDto>(summaryResult.Error);

        var summary = summaryResult.Value;
        dbContext.MeetingSummaries.Add(summary);
        await dbContext.SaveChangesAsync(cancellationToken);

        return Result.Success(MapToDto(summary));
    }

    private static MeetingSummaryDto MapToDto(MeetingSummary summary)
    {
        return new MeetingSummaryDto(
            summary.Id,
            summary.MeetingId,
            summary.Overview,
            summary.KeyDecisions,
            summary.ActionItems.Select(a => new ActionItemDto(a.Title, a.Assignee, a.Status)).ToList(),
            summary.GeneratedAt,
            summary.DurationMinutes,
            summary.ParticipantCount
        );
    }
}
