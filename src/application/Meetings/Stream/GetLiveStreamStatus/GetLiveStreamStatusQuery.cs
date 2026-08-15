using Confer.Application.DTOs;
using Confer.Shared.Application.Interfaces;
using Confer.Shared.Results;

namespace Confer.Application.Meetings.Stream.GetLiveStreamStatus;

public record GetLiveStreamStatusQuery(
    Guid MeetingId
) : IQuery<Result<LiveStreamResponse>>;
