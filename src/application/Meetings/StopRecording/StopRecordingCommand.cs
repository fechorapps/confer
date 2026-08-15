using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.StopRecording;

public record StopRecordingCommand(
    Guid MeetingId,
    Guid ActorId
) : ICommand<Result<StopRecordingResponse>>;
