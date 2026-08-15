using Confer.Application.DTOs;

namespace Confer.Application.Interfaces;

public interface ISignalingNotifier
{
    Task BroadcastJoinedAsync(Guid meetingId, ParticipantStateDto participant);
    Task BroadcastLeftAsync(Guid meetingId, Guid participantId, string reason);
    Task BroadcastMuteChangedAsync(Guid meetingId, Guid participantId, string kind, bool isMuted);
    Task BroadcastActiveSpeakersAsync(Guid meetingId, List<SpeakerInfoDto> speakers);
    Task BroadcastChatAsync(Guid meetingId, Guid messageId, Guid fromId, string fromName, string body, DateTime sentAt);
    Task BroadcastReactionAsync(Guid meetingId, Guid fromId, string fromName, string emoji);
    Task BroadcastMeetingLockedAsync(Guid meetingId, bool isLocked);
    Task BroadcastMeetingEndedAsync(Guid meetingId, string reason);
    Task BroadcastRecordingStateAsync(Guid meetingId, bool isRecording, Guid? recordingId = null);
    Task SendSubscribeOfferAsync(Guid meetingId, Guid participantId, string sdpOffer, List<TrackMappingDto> mappings);
    Task BroadcastPollCreatedAsync(Guid meetingId, PollDto poll);
    Task BroadcastPollUpdatedAsync(Guid meetingId, PollDto poll);
    Task BroadcastPollClosedAsync(Guid meetingId, Guid pollId);
    Task BroadcastBreakoutInviteAsync(Guid meetingId, Guid participantId, BreakoutInviteDto invite);
    Task BroadcastWaitingRoomUpdateAsync(Guid meetingId, bool isWaitingRoomEnabled, int waitingCount);
    Task BroadcastPolicyChangedAsync(Guid meetingId, MeetingPolicyDto policy, bool isWatermarkEnabled = false, bool isWaitingRoomEnabled = false);
    Task BroadcastParticipantAdmittedAsync(Guid meetingId, Guid participantId);
    Task BroadcastStreamStatusAsync(Guid meetingId, bool isStreaming, string rtmpUrl, string status, DateTime? startedAt = null);
    Task BroadcastCaptionAsync(Guid meetingId, CaptionChunkDto caption);
}

