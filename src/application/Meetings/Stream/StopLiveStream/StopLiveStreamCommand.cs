using Confer.Application.DTOs;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Stream.StopLiveStream;

public record StopLiveStreamCommand(
    Guid MeetingId,
    Guid ActorId
) : ICommand<Result<LiveStreamResponse>>;
