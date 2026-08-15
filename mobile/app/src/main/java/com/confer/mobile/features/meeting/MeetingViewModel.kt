package com.confer.mobile.features.meeting

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.confer.mobile.core.network.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import java.text.SimpleDateFormat
import java.util.*

data class ActiveReactionItem(val emoji: String, val id: String = UUID.randomUUID().toString())

data class MeetingUiState(
    val meetingId: String = "",
    val title: String = "Meeting",
    val myParticipantId: String = "",
    val myDisplayName: String = "",
    val myRole: String = "participant",
    val isLocked: Boolean = false,
    val isMicMuted: Boolean = false,
    val isCameraOff: Boolean = false,
    val isScreenSharing: Boolean = false,
    val isHandRaised: Boolean = false,
    val isFrontCamera: Boolean = true,
    val roster: List<ParticipantState> = emptyList(),
    val activeSpeakerIds: Set<String> = emptySet(),
    val chatMessages: List<ChatMessage> = emptyList(),
    val unreadChatCount: Int = 0,
    val activeReactions: List<ActiveReactionItem> = emptyList(),
    val showChatSheet: Boolean = false,
    val showRosterSheet: Boolean = false,
    val showDiagnostics: Boolean = false,
    val rttMs: Long = 32,
    val packetLossPct: Float = 0.0f
)

class MeetingViewModel : ViewModel() {
    private val _uiState = MutableStateFlow(MeetingUiState())
    val uiState = _uiState.asStateFlow()

    private var signalingClient: SignalingClient? = null

    fun initialize(joinData: JoinMeetingResponse, serverUrl: String, displayName: String) {
        _uiState.update {
            it.copy(
                meetingId = joinData.meetingId,
                title = joinData.title,
                myParticipantId = joinData.participantId,
                myDisplayName = displayName,
                myRole = joinData.role,
                isLocked = joinData.isLocked
            )
        }

        signalingClient = SignalingClient(serverUrl)
        viewModelScope.launch {
            signalingClient?.connect(joinData.roomToken)
            listenIncomingMessages()
        }
    }

    private fun listenIncomingMessages() {
        viewModelScope.launch {
            signalingClient?.incomingMessages?.collect { msg ->
                when (msg) {
                    is ServerMessage.Joined -> {
                        _uiState.update { state ->
                            state.copy(
                                roster = msg.roster.filter { it.participantId != state.myParticipantId }
                            )
                        }
                    }
                    is ServerMessage.ParticipantJoined -> {
                        _uiState.update { state ->
                            if (msg.participant.participantId != state.myParticipantId) {
                                val updated = state.roster.filter { it.participantId != msg.participant.participantId } + msg.participant
                                state.copy(roster = updated)
                            } else state
                        }
                    }
                    is ServerMessage.ParticipantLeft -> {
                        _uiState.update { state ->
                            state.copy(
                                roster = state.roster.filter { it.participantId != msg.participantId },
                                activeSpeakerIds = state.activeSpeakerIds - msg.participantId
                            )
                        }
                    }
                    is ServerMessage.ParticipantMuteChanged -> {
                        _uiState.update { state ->
                            val updated = state.roster.map {
                                if (it.participantId == msg.participantId) {
                                    when (msg.kind) {
                                        "audio" -> it.copy(isAudioMuted = msg.muted)
                                        "video" -> it.copy(isVideoMuted = msg.muted)
                                        "screen_share" -> it.copy(isScreenSharing = !msg.muted)
                                        else -> it
                                    }
                                } else it
                            }
                            state.copy(roster = updated)
                        }
                    }
                    is ServerMessage.ActiveSpeakers -> {
                        val ids = msg.ranked.map { it.participantId }.toSet()
                        _uiState.update { it.copy(activeSpeakerIds = ids) }
                    }
                    is ServerMessage.Chat -> {
                        _uiState.update { state ->
                            val newUnread = if (state.showChatSheet) 0 else state.unreadChatCount + 1
                            state.copy(
                                chatMessages = state.chatMessages + ChatMessage(
                                    id = msg.id,
                                    fromId = msg.fromId,
                                    fromName = msg.fromName,
                                    body = msg.body,
                                    sentAt = msg.sentAt.takeLast(8).take(5) // HH:mm
                                ),
                                unreadChatCount = newUnread
                            )
                        }
                    }
                    is ServerMessage.Reaction -> {
                        val reactionItem = ActiveReactionItem(msg.emoji)
                        _uiState.update { it.copy(activeReactions = it.activeReactions + reactionItem) }
                    }
                    is ServerMessage.MeetingLocked -> {
                        _uiState.update { it.copy(isLocked = msg.isLocked) }
                    }
                    else -> {}
                }
            }
        }
    }

    fun toggleMic() {
        val newMuted = !_uiState.value.isMicMuted
        _uiState.update { it.copy(isMicMuted = newMuted) }
        viewModelScope.launch {
            signalingClient?.sendMessage(ClientMessage.SetMute("audio", newMuted))
        }
    }

    fun toggleCamera() {
        val newOff = !_uiState.value.isCameraOff
        _uiState.update { it.copy(isCameraOff = newOff) }
        viewModelScope.launch {
            signalingClient?.sendMessage(ClientMessage.SetMute("video", newOff))
        }
    }

    fun flipCamera() {
        _uiState.update { it.copy(isFrontCamera = !it.isFrontCamera) }
    }

    fun toggleScreenShare() {
        val newShare = !_uiState.value.isScreenSharing
        _uiState.update { it.copy(isScreenSharing = newShare) }
        viewModelScope.launch {
            signalingClient?.sendMessage(ClientMessage.SetMute("screen_share", !newShare))
        }
    }

    fun toggleHandRaise() {
        val newHand = !_uiState.value.isHandRaised
        _uiState.update { it.copy(isHandRaised = newHand) }
        sendReaction(if (newHand) "✋" else "👋")
    }

    fun sendReaction(emoji: String) {
        val reactionItem = ActiveReactionItem(emoji)
        _uiState.update { it.copy(activeReactions = it.activeReactions + reactionItem) }
        viewModelScope.launch {
            signalingClient?.sendMessage(ClientMessage.Reaction(emoji))
        }
    }

    fun sendChat(body: String) {
        if (body.isBlank()) return
        viewModelScope.launch {
            signalingClient?.sendMessage(ClientMessage.Chat(body, UUID.randomUUID().toString()))
        }
    }

    fun hostMute(participantId: String) {
        viewModelScope.launch {
            signalingClient?.sendMessage(ClientMessage.HostAction("mute", participantId))
        }
    }

    fun hostKick(participantId: String) {
        viewModelScope.launch {
            signalingClient?.sendMessage(ClientMessage.HostAction("kick", participantId))
        }
    }

    fun showChat(show: Boolean) {
        _uiState.update { it.copy(showChatSheet = show, unreadChatCount = if (show) 0 else it.unreadChatCount) }
    }

    fun showRoster(show: Boolean) {
        _uiState.update { it.copy(showRosterSheet = show) }
    }

    fun showDiagnostics(show: Boolean) {
        _uiState.update { it.copy(showDiagnostics = show) }
    }

    fun leaveMeeting() {
        signalingClient?.disconnect()
    }
}
