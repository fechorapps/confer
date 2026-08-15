package com.confer.mobile.features.meeting

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Lock
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.confer.mobile.core.theme.*
import com.confer.mobile.features.meeting.components.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MeetingScreen(
    viewModel: MeetingViewModel,
    onLeaveMeeting: () -> Unit
) {
    val state by viewModel.uiState.collectAsState()

    Scaffold(
        containerColor = BackgroundDark,
        topBar = {
            TopAppBar(
                title = {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text(
                            text = state.title,
                            color = Color.White,
                            fontSize = 16.sp,
                            fontWeight = FontWeight.Bold
                        )
                        if (state.isLocked) {
                            Spacer(modifier = Modifier.width(6.dp))
                            Icon(Icons.Default.Lock, contentDescription = "Locked", tint = WarningYellow, modifier = Modifier.size(14.dp))
                        }
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(containerColor = SurfaceDark)
            )
        }
    ) { padding ->
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
        ) {
            // Main Responsive Video Grid
            VideoGrid(
                myDisplayName = state.myDisplayName,
                myRole = state.myRole,
                isLocalMicMuted = state.isMicMuted,
                isLocalCameraOff = state.isCameraOff,
                isLocalScreenSharing = state.isScreenSharing,
                isLocalHandRaised = state.isHandRaised,
                roster = state.roster,
                activeSpeakerIds = state.activeSpeakerIds,
                modifier = Modifier
                    .fillMaxSize()
                    .padding(bottom = 80.dp)
            )

            // Floating Emoji Reaction Stream
            Box(
                modifier = Modifier
                    .fillMaxSize()
                    .padding(bottom = 100.dp),
                contentAlignment = Alignment.BottomCenter
            ) {
                state.activeReactions.takeLast(4).forEach { reaction ->
                    Text(
                        text = reaction.emoji,
                        fontSize = 32.sp,
                        modifier = Modifier.padding(bottom = 20.dp)
                    )
                }
            }

            // Bottom Floating Control Bar
            ControlBar(
                isMicMuted = state.isMicMuted,
                isCameraOff = state.isCameraOff,
                isScreenSharing = state.isScreenSharing,
                isHandRaised = state.isHandRaised,
                unreadChatCount = state.unreadChatCount,
                onToggleMic = { viewModel.toggleMic() },
                onToggleCamera = { viewModel.toggleCamera() },
                onFlipCamera = { viewModel.flipCamera() },
                onToggleScreenShare = { viewModel.toggleScreenShare() },
                onToggleHandRaise = { viewModel.toggleHandRaise() },
                onSendReaction = { viewModel.sendReaction(it) },
                onOpenChat = { viewModel.showChat(true) },
                onOpenRoster = { viewModel.showRoster(true) },
                onOpenDiagnostics = { viewModel.showDiagnostics(true) },
                onLeaveMeeting = {
                    viewModel.leaveMeeting()
                    onLeaveMeeting()
                },
                modifier = Modifier.align(Alignment.BottomCenter)
            )

            // Chat Bottom Sheet
            if (state.showChatSheet) {
                ChatBottomSheet(
                    messages = state.chatMessages,
                    myParticipantId = state.myParticipantId,
                    onSendMessage = { viewModel.sendChat(it) },
                    onDismiss = { viewModel.showChat(false) }
                )
            }

            // Participant Roster Bottom Sheet
            if (state.showRosterSheet) {
                RosterBottomSheet(
                    myDisplayName = state.myDisplayName,
                    myRole = state.myRole,
                    roster = state.roster,
                    onHostMute = { viewModel.hostMute(it) },
                    onHostKick = { viewModel.hostKick(it) },
                    onDismiss = { viewModel.showRoster(false) }
                )
            }

            // Diagnostics HUD Dialog
            if (state.showDiagnostics) {
                DiagnosticsDialog(
                    rttMs = state.rttMs,
                    packetLossPct = state.packetLossPct,
                    onDismiss = { viewModel.showDiagnostics(false) }
                )
            }
        }
    }
}
