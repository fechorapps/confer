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
    val isWaitingInLobby: Boolean = false,
    val waitingRoomMessage: String? = null,
    val waitingParticipants: List<WaitingParticipant> = emptyList(),
    val meetingPolicy: MeetingPolicy = MeetingPolicy(),
    val isWatermarkEnabled: Boolean = false,
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
    val polls: List<Poll> = emptyList(),
    val unreadPollCount: Int = 0,
    val whiteboardStrokes: List<WhiteboardStroke> = emptyList(),
    val isWhiteboardActive: Boolean = false,
    val showChatSheet: Boolean = false,
    val showRosterSheet: Boolean = false,
    val showPollsSheet: Boolean = false,
    val showSecuritySheet: Boolean = false,
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
                        _uiState.update { it.copy(isLocked = msg.isLocked, meetingPolicy = it.meetingPolicy.copy(isLocked = msg.isLocked)) }
                    }
                    is ServerMessage.WaitingRoomStatus -> {
                        _uiState.update { it.copy(isWaitingInLobby = msg.isWaiting, waitingRoomMessage = msg.message) }
                    }
                    is ServerMessage.ParticipantWaiting -> {
                        _uiState.update { state ->
                            if (state.waitingParticipants.none { it.participantId == msg.participant.participantId }) {
                                state.copy(waitingParticipants = state.waitingParticipants + msg.participant)
                            } else state
                        }
                    }
                    is ServerMessage.ParticipantAdmitted -> {
                        _uiState.update { state ->
                            val stillWaiting = if (msg.participantId == state.myParticipantId) false else state.isWaitingInLobby
                            state.copy(
                                isWaitingInLobby = stillWaiting,
                                waitingParticipants = state.waitingParticipants.filter { it.participantId != msg.participantId }
                            )
                        }
                    }
                    is ServerMessage.ParticipantRejected -> {
                        _uiState.update { state ->
                            state.copy(
                                waitingParticipants = state.waitingParticipants.filter { it.participantId != msg.participantId }
                            )
                        }
                    }
                    is ServerMessage.MeetingPolicyChanged -> {
                        _uiState.update {
                            it.copy(
                                isLocked = msg.policy.isLocked,
                                isWatermarkEnabled = msg.policy.watermarkEnabled,
                                meetingPolicy = msg.policy
                            )
                        }
                    }
                    is ServerMessage.PollCreated -> {
                        _uiState.update { state ->
                            val exists = state.polls.any { it.id == msg.poll.id }
                            val updatedPolls = if (exists) {
                                state.polls.map { if (it.id == msg.poll.id) msg.poll else it }
                            } else {
                                state.polls + msg.poll
                            }
                            val newUnread = if (state.showPollsSheet) 0 else state.unreadPollCount + 1
                            state.copy(polls = updatedPolls, unreadPollCount = newUnread)
                        }
                    }
                    is ServerMessage.PollUpdated -> {
                        _uiState.update { state ->
                            val updatedPolls = state.polls.map { current ->
                                if (current.id == msg.poll.id) {
                                    msg.poll.copy(votedOptionId = current.votedOptionId ?: msg.poll.votedOptionId)
                                } else current
                            }
                            state.copy(polls = updatedPolls)
                        }
                    }
                    is ServerMessage.PollEnded -> {
                        _uiState.update { state ->
                            val updatedPolls = state.polls.map { current ->
                                if (current.id == msg.pollId) current.copy(isActive = false) else current
                            }
                            state.copy(polls = updatedPolls)
                        }
                    }
                    is ServerMessage.WhiteboardDraw -> {
                        _uiState.update { state ->
                            if (state.whiteboardStrokes.none { it.id == msg.stroke.id }) {
                                state.copy(whiteboardStrokes = state.whiteboardStrokes + msg.stroke)
                            } else state
                        }
                    }
                    is ServerMessage.WhiteboardClear -> {
                        _uiState.update { it.copy(whiteboardStrokes = emptyList()) }
                    }
                    is ServerMessage.WhiteboardUndo -> {
                        _uiState.update { state ->
                            val updatedStrokes = if (msg.strokeId != null) {
                                state.whiteboardStrokes.filter { it.id != msg.strokeId }
                            } else {
                                state.whiteboardStrokes.dropLast(1)
                            }
                            state.copy(whiteboardStrokes = updatedStrokes)
                        }
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

    fun createPoll(question: String, options: List<String>) {
        if (question.isBlank() || options.size < 2) return
        val pollId = UUID.randomUUID().toString()
        val pollOptions = options.mapIndexed { idx, text ->
            PollOption(id = "opt_${pollId}_$idx", text = text, voteCount = 0)
        }
        val newPoll = Poll(
            id = pollId,
            question = question,
            options = pollOptions,
            createdBy = _uiState.value.myParticipantId,
            createdByName = _uiState.value.myDisplayName,
            isActive = true,
            votedOptionId = null,
            totalVotes = 0
        )
        _uiState.update { it.copy(polls = it.polls + newPoll) }
        viewModelScope.launch {
            signalingClient?.sendMessage(ClientMessage.CreatePoll(question, options, pollId = pollId))
        }
    }

    fun votePoll(pollId: String, optionId: String) {
        _uiState.update { state ->
            val updated = state.polls.map { poll ->
                if (poll.id == pollId) {
                    val updatedOptions = poll.options.map { opt ->
                        if (opt.id == optionId) opt.copy(voteCount = opt.voteCount + 1) else opt
                    }
                    poll.copy(
                        options = updatedOptions,
                        votedOptionId = optionId,
                        totalVotes = poll.totalVotes + 1
                    )
                } else poll
            }
            state.copy(polls = updated)
        }
        viewModelScope.launch {
            signalingClient?.sendMessage(ClientMessage.VotePoll(pollId, optionId))
        }
    }

    fun endPoll(pollId: String) {
        _uiState.update { state ->
            val updated = state.polls.map { poll ->
                if (poll.id == pollId) poll.copy(isActive = false) else poll
            }
            state.copy(polls = updated)
        }
        viewModelScope.launch {
            signalingClient?.sendMessage(ClientMessage.EndPoll(pollId))
        }
    }

    fun toggleWhiteboard() {
        _uiState.update { it.copy(isWhiteboardActive = !it.isWhiteboardActive) }
    }

    fun showWhiteboard(show: Boolean) {
        _uiState.update { it.copy(isWhiteboardActive = show) }
    }

    fun sendWhiteboardStroke(stroke: WhiteboardStroke) {
        _uiState.update { it.copy(whiteboardStrokes = it.whiteboardStrokes + stroke) }
        viewModelScope.launch {
            signalingClient?.sendMessage(ClientMessage.WhiteboardDraw(stroke))
        }
    }

    fun clearWhiteboard() {
        _uiState.update { it.copy(whiteboardStrokes = emptyList()) }
        viewModelScope.launch {
            signalingClient?.sendMessage(ClientMessage.WhiteboardClear())
        }
    }

    fun undoWhiteboard() {
        val lastStroke = _uiState.value.whiteboardStrokes.lastOrNull()
        _uiState.update { it.copy(whiteboardStrokes = it.whiteboardStrokes.dropLast(1)) }
        viewModelScope.launch {
            signalingClient?.sendMessage(ClientMessage.WhiteboardUndo(lastStroke?.id))
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

    fun showPolls(show: Boolean) {
        _uiState.update { it.copy(showPollsSheet = show, unreadPollCount = if (show) 0 else it.unreadPollCount) }
    }

    fun showRoster(show: Boolean) {
        _uiState.update { it.copy(showRosterSheet = show) }
    }

    fun showDiagnostics(show: Boolean) {
        _uiState.update { it.copy(showDiagnostics = show) }
    }

    fun showSecurity(show: Boolean) {
        _uiState.update { it.copy(showSecuritySheet = show) }
    }

    fun admitParticipant(participantId: String) {
        _uiState.update { state ->
            state.copy(waitingParticipants = state.waitingParticipants.filter { it.participantId != participantId })
        }
        viewModelScope.launch {
            signalingClient?.sendMessage(ClientMessage.AdmitParticipant(participantId))
        }
    }

    fun admitAllWaiting() {
        _uiState.update { it.copy(waitingParticipants = emptyList()) }
        viewModelScope.launch {
            signalingClient?.sendMessage(ClientMessage.AdmitAll())
        }
    }

    fun rejectParticipant(participantId: String) {
        _uiState.update { state ->
            state.copy(waitingParticipants = state.waitingParticipants.filter { it.participantId != participantId })
        }
        viewModelScope.launch {
            signalingClient?.sendMessage(ClientMessage.RejectParticipant(participantId))
        }
    }

    fun updateMeetingPolicy(policy: MeetingPolicy) {
        _uiState.update {
            it.copy(
                isLocked = policy.isLocked,
                isWatermarkEnabled = policy.watermarkEnabled,
                meetingPolicy = policy
            )
        }
        viewModelScope.launch {
            signalingClient?.sendMessage(ClientMessage.UpdatePolicy(policy))
        }
    }

    fun toggleWaitingRoom(enabled: Boolean) {
        updateMeetingPolicy(_uiState.value.meetingPolicy.copy(waitingRoomEnabled = enabled))
    }

    fun toggleLockMeeting(locked: Boolean) {
        updateMeetingPolicy(_uiState.value.meetingPolicy.copy(isLocked = locked))
    }

    fun toggleAllowScreenShare(allow: Boolean) {
        updateMeetingPolicy(_uiState.value.meetingPolicy.copy(allowScreenShare = allow))
    }

    fun toggleAllowChat(allow: Boolean) {
        updateMeetingPolicy(_uiState.value.meetingPolicy.copy(allowChat = allow))
    }

    fun toggleAllowUnmute(allow: Boolean) {
        updateMeetingPolicy(_uiState.value.meetingPolicy.copy(allowUnmute = allow))
    }

    fun toggleWatermark(enabled: Boolean) {
        updateMeetingPolicy(_uiState.value.meetingPolicy.copy(watermarkEnabled = enabled))
    }

    fun leaveMeeting() {
        signalingClient?.disconnect()
    }
}
