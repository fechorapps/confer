package com.confer.mobile.core.network

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
data class TrackIntent(
    @SerialName("track_id") val trackId: String,
    val kind: String,
    val simulcast: Boolean = false
)

@Serializable
data class TileSpec(
    @SerialName("participant_id") val participantId: String,
    val width: Int,
    val height: Int,
    val visible: Boolean = true
)

@Serializable
data class ParticipantState(
    @SerialName("participant_id") val participantId: String,
    @SerialName("user_id") val userId: String,
    @SerialName("display_name") val displayName: String,
    val role: String,
    @SerialName("is_audio_muted") val isAudioMuted: Boolean = false,
    @SerialName("is_video_muted") val isVideoMuted: Boolean = false,
    @SerialName("is_screen_sharing") val isScreenSharing: Boolean = false,
    @SerialName("is_hand_raised") val isHandRaised: Boolean = false
)

@Serializable
data class SpeakerInfo(
    @SerialName("participant_id") val participantId: String,
    @SerialName("level_dbov") val levelDbov: Int
)

@Serializable
data class ChatMessage(
    val id: String,
    @SerialName("from_id") val fromId: String,
    @SerialName("from_name") val fromName: String,
    val body: String,
    @SerialName("sent_at") val sentAt: String
)

@Serializable
sealed class ClientMessage {
    @Serializable
    @SerialName("publish")
    data class Publish(
        val sdp: String,
        val tracks: List<TrackIntent>
    ) : ClientMessage()

    @Serializable
    @SerialName("subscribe_answer")
    data class SubscribeAnswer(val sdp: String) : ClientMessage()

    @Serializable
    @SerialName("update_viewport")
    data class UpdateViewport(val tiles: List<TileSpec>) : ClientMessage()

    @Serializable
    @SerialName("set_mute")
    data class SetMute(val kind: String, val muted: Boolean) : ClientMessage()

    @Serializable
    @SerialName("chat")
    data class Chat(
        val body: String,
        @SerialName("client_msg_id") val clientMsgId: String
    ) : ClientMessage()

    @Serializable
    @SerialName("reaction")
    data class Reaction(val emoji: String) : ClientMessage()

    @Serializable
    @SerialName("host_action")
    data class HostAction(
        val action: String,
        @SerialName("target_participant_id") val targetParticipantId: String
    ) : ClientMessage()

    @Serializable
    @SerialName("ping")
    data class Ping(val seq: Long) : ClientMessage()
}

@Serializable
sealed class ServerMessage {
    @Serializable
    @SerialName("joined")
    data class Joined(
        @SerialName("participant_id") val participantId: String,
        @SerialName("meeting_id") val meetingId: String,
        @SerialName("room_title") val roomTitle: String,
        val role: String,
        val roster: List<ParticipantState> = emptyList()
    ) : ServerMessage()

    @Serializable
    @SerialName("publish_ok")
    data class PublishOk(val sdp: String) : ServerMessage()

    @Serializable
    @SerialName("participant_joined")
    data class ParticipantJoined(val participant: ParticipantState) : ServerMessage()

    @Serializable
    @SerialName("participant_left")
    data class ParticipantLeft(
        @SerialName("participant_id") val participantId: String,
        val reason: String = "left"
    ) : ServerMessage()

    @Serializable
    @SerialName("participant_mute_changed")
    data class ParticipantMuteChanged(
        @SerialName("participant_id") val participantId: String,
        val kind: String,
        val muted: Boolean
    ) : ServerMessage()

    @Serializable
    @SerialName("active_speakers")
    data class ActiveSpeakers(val ranked: List<SpeakerInfo>) : ServerMessage()

    @Serializable
    @SerialName("chat")
    data class Chat(
        val id: String,
        @SerialName("from_id") val fromId: String,
        @SerialName("from_name") val fromName: String,
        val body: String,
        @SerialName("sent_at") val sentAt: String
    ) : ServerMessage()

    @Serializable
    @SerialName("reaction")
    data class Reaction(
        @SerialName("from_id") val fromId: String,
        @SerialName("from_name") val fromName: String,
        val emoji: String
    ) : ServerMessage()

    @Serializable
    @SerialName("meeting_locked")
    data class MeetingLocked(@SerialName("is_locked") val isLocked: Boolean) : ServerMessage()

    @Serializable
    @SerialName("meeting_ended")
    data class MeetingEnded(val reason: String) : ServerMessage()

    @Serializable
    @SerialName("pong")
    data class Pong(val seq: Long) : ServerMessage()
}
