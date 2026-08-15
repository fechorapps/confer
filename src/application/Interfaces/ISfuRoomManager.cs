using Confer.Application.DTOs;
using Confer.Domain.Enums;
using Confer.Shared.Results;

namespace Confer.Application.Interfaces;

public interface ISfuRoomManager
{
    Task<Result> CreateOrGetRoomAsync(Guid meetingId);

    /// <summary>
    /// Registers a participant with the room's media session as soon as they connect (before
    /// they necessarily publish anything), and offers them any tracks already published by
    /// other participants so they don't have to wait for a re-publish to see/hear the room.
    /// </summary>
    Task<Result> JoinRoomAsync(Guid meetingId, Guid participantId);

    Task<Result<string>> HandlePublishOfferAsync(Guid meetingId, Guid participantId, string sdpOffer, List<TrackIntent> tracks);
    Task<Result> HandleSubscribeAnswerAsync(Guid meetingId, Guid participantId, string sdpAnswer);
    Task<Result> HandleIceCandidateAsync(Guid meetingId, Guid participantId, string target, IceCandidateDto candidate);
    Task<Result> UpdateViewportAsync(Guid meetingId, Guid participantId, List<TileSpec> tiles);
    Task<Result> SetMuteAsync(Guid meetingId, Guid participantId, MediaKind kind, bool isMuted);
    Task<Result> RemoveParticipantAsync(Guid meetingId, Guid participantId);
    Task<Result> CloseRoomAsync(Guid meetingId);
}
