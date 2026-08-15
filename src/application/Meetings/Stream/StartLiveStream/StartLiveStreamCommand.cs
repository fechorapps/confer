using Confer.Application.DTOs;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Stream.StartLiveStream;

public record StartLiveStreamCommand(
    Guid MeetingId,
    Guid ActorId,
    string RtmpUrl,
    string StreamKey
) : ICommand<Result<LiveStreamResponse>>;
