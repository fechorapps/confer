using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.StartRecording;

public record StartRecordingCommand(
    Guid MeetingId,
    Guid ActorId
) : ICommand<Result<StartRecordingResponse>>;
