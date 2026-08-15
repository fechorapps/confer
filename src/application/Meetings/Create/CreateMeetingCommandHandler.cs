using Confer.Application.DTOs;
using Confer.Application.Interfaces;
using Confer.Domain.Meetings;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;
using FluentValidation;

namespace Confer.Application.Meetings.Create;

public sealed class CreateMeetingCommandHandler(
    IConferDbContext dbContext,
    IValidator<CreateMeetingCommand> validator)
    : ICommandHandler<CreateMeetingCommand, Result<CreateMeetingResponse>>
{
    public async Task<Result<CreateMeetingResponse>> HandleAsync(
        CreateMeetingCommand command,
        CancellationToken cancellationToken = default)
    {
        var validation = await validator.ValidateAsync(command, cancellationToken);
        if (!validation.IsValid)
        {
            var firstError = validation.Errors.First();
            return Result.Failure<CreateMeetingResponse>(Error.Validation(firstError.PropertyName, firstError.ErrorMessage));
        }

        MeetingPolicy? domainPolicy = command.Policy is not null
            ? new MeetingPolicy(
                command.Policy.AllowScreenShare,
                command.Policy.AllowChat,
                command.Policy.AllowUnmuteSelf,
                command.Policy.MuteOnEntry,
                command.Policy.AllowRename)
            : null;

        var meetingResult = Meeting.Create(
            command.Title,
            command.OwnerId,
            command.MaxParticipants,
            command.CustomJoinCode,
            command.IsWaitingRoomEnabled,
            command.IsWatermarkEnabled,
            domainPolicy);

        if (meetingResult.IsFailure)
            return Result.Failure<CreateMeetingResponse>(meetingResult.Error);

        var meeting = meetingResult.Value;
        dbContext.Meetings.Add(meeting);
        await dbContext.SaveChangesAsync(cancellationToken);

        var policyDto = new MeetingPolicyDto(
            meeting.Policy.AllowScreenShare,
            meeting.Policy.AllowChat,
            meeting.Policy.AllowUnmuteSelf,
            meeting.Policy.MuteOnEntry,
            meeting.Policy.AllowRename);

        return Result.Success(new CreateMeetingResponse(
            meeting.Id,
            meeting.JoinCode,
            meeting.Title,
            meeting.OwnerId,
            meeting.MaxParticipants,
            meeting.CreatedAt,
            meeting.IsWaitingRoomEnabled,
            meeting.IsWatermarkEnabled,
            policyDto
        ));
    }
}
