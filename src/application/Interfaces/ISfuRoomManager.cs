using Confer.Application.DTOs;
using Confer.Domain.Enums;
using Confer.Shared.Results;

namespace Confer.Application.Interfaces;

public interface ISfuRoomManager
{
    Task<Result> CreateOrGetRoomAsync(Guid meetingId);
    Task<Result<string>> HandlePublishOfferAsync(Guid meetingId, Guid participantId, string sdpOffer, List<TrackIntent> tracks);
    Task<Result> HandleSubscribeAnswerAsync(Guid meetingId, Guid participantId, string sdpAnswer);
    Task<Result> UpdateViewportAsync(Guid meetingId, Guid participantId, List<TileSpec> tiles);
    Task<Result> SetMuteAsync(Guid meetingId, Guid participantId, MediaKind kind, bool isMuted);
    Task<Result> RemoveParticipantAsync(Guid meetingId, Guid participantId);
    Task<Result> CloseRoomAsync(Guid meetingId);
}
