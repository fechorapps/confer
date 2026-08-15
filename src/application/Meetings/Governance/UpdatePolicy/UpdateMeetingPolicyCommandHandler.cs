using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using Microsoft.EntityFrameworkCore;

namespace Confer.Application.Meetings.Governance.UpdatePolicy;

public sealed class UpdateMeetingPolicyCommandHandler(
    IConferDbContext dbContext,
    ISignalingNotifier signalingNotifier)
    : ICommandHandler<UpdateMeetingPolicyCommand, Result<MeetingPolicyDto>>
{
    public async Task<Result<MeetingPolicyDto>> HandleAsync(
        UpdateMeetingPolicyCommand command,
        CancellationToken cancellationToken = default)
    {
        var meeting = await dbContext.Meetings
            .FirstOrDefaultAsync(m => m.Id == command.MeetingId, cancellationToken);

        if (meeting is null)
            return Result.Failure<MeetingPolicyDto>(MeetingErrors.NotFound);

        if (command.Policy is null)
            return Result.Failure<MeetingPolicyDto>(MeetingErrors.InvalidPolicy);

        var domainPolicy = new MeetingPolicy(
            command.Policy.AllowScreenShare,
            command.Policy.AllowChat,
            command.Policy.AllowUnmuteSelf,
            command.Policy.MuteOnEntry,
            command.Policy.AllowRename
        );

        var updateResult = meeting.UpdatePolicy(command.ActorId, domainPolicy);
        if (updateResult.IsFailure)
            return Result.Failure<MeetingPolicyDto>(updateResult.Error);

        if (command.IsWatermarkEnabled.HasValue)
        {
            var watermarkResult = meeting.ToggleWatermark(command.ActorId, command.IsWatermarkEnabled.Value);
            if (watermarkResult.IsFailure)
                return Result.Failure<MeetingPolicyDto>(watermarkResult.Error);
        }

        if (command.IsWaitingRoomEnabled.HasValue)
        {
            var waitingRoomResult = meeting.ToggleWaitingRoom(command.ActorId, command.IsWaitingRoomEnabled.Value);
            if (waitingRoomResult.IsFailure)
                return Result.Failure<MeetingPolicyDto>(waitingRoomResult.Error);
        }

        await dbContext.SaveChangesAsync(cancellationToken);

        var policyDto = new MeetingPolicyDto(
            meeting.Policy.AllowScreenShare,
            meeting.Policy.AllowChat,
            meeting.Policy.AllowUnmuteSelf,
            meeting.Policy.MuteOnEntry,
            meeting.Policy.AllowRename
        );

        await signalingNotifier.BroadcastPolicyChangedAsync(
            meeting.Id,
            policyDto,
            meeting.IsWatermarkEnabled,
            meeting.IsWaitingRoomEnabled
        );

        return Result.Success(policyDto);
    }
}
