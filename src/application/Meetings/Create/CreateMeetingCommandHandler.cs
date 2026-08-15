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

        var meetingResult = Meeting.Create(command.Title, command.OwnerId, command.MaxParticipants, command.CustomJoinCode);
        if (meetingResult.IsFailure)
            return Result.Failure<CreateMeetingResponse>(meetingResult.Error);

        var meeting = meetingResult.Value;
        dbContext.Meetings.Add(meeting);
        await dbContext.SaveChangesAsync(cancellationToken);

        return Result.Success(new CreateMeetingResponse(
            meeting.Id,
            meeting.JoinCode,
            meeting.Title,
            meeting.OwnerId,
            meeting.MaxParticipants,
            meeting.CreatedAt
        ));
    }
}
