package com.confer.mobile.features.meeting.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.confer.mobile.core.network.ParticipantState
import com.confer.mobile.core.theme.*

@Composable
fun VideoGrid(
    myDisplayName: String,
    myRole: String,
    isLocalMicMuted: Boolean,
    isLocalCameraOff: Boolean,
    isLocalScreenSharing: Boolean,
    isLocalHandRaised: Boolean,
    roster: List<ParticipantState>,
    activeSpeakerIds: Set<String>,
    modifier: Modifier = Modifier
) {
    val totalCount = roster.size + 1
    val columns = when (totalCount) {
        1 -> 1
        2 -> 1
        3, 4 -> 2
        else -> 2
    }

    LazyVerticalGrid(
        columns = GridCells.Fixed(columns),
        modifier = modifier.padding(8.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp)
    ) {
        // Local participant tile
        item {
            VideoTile(
                displayName = myDisplayName,
                role = myRole,
                isAudioMuted = isLocalMicMuted,
                isVideoMuted = isLocalCameraOff,
                isScreenSharing = isLocalScreenSharing,
                isHandRaised = isLocalHandRaised,
                isLocal = true,
                isActiveSpeaker = false
            )
        }

        // Remote participant tiles
        items(roster, key = { it.participantId }) { p ->
            val isActiveSpeaker = activeSpeakerIds.contains(p.participantId)
            VideoTile(
                displayName = p.displayName,
                role = p.role,
                isAudioMuted = p.isAudioMuted,
                isVideoMuted = p.isVideoMuted,
                isScreenSharing = p.isScreenSharing,
                isHandRaised = p.isHandRaised,
                isLocal = false,
                isActiveSpeaker = isActiveSpeaker
            )
        }
    }
}

@Composable
fun VideoTile(
    displayName: String,
    role: String,
    isAudioMuted: Boolean,
    isVideoMuted: Boolean,
    isScreenSharing: Boolean,
    isHandRaised: Boolean,
    isLocal: Boolean,
    isActiveSpeaker: Boolean
) {
    val borderColor = if (isActiveSpeaker) AccentGreen else SurfaceVariantDark
    val borderWidth = if (isActiveSpeaker) 2.dp else 1.dp

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(200.dp)
            .clip(RoundedCornerShape(12.dp))
            .background(SurfaceDark)
            .border(borderWidth, borderColor, RoundedCornerShape(12.dp)),
        contentAlignment = Alignment.Center
    ) {
        if (isScreenSharing) {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                Icon(Icons.Default.ScreenShare, contentDescription = null, tint = PrimaryCyan, modifier = Modifier.size(36.dp))
                Spacer(modifier = Modifier.height(6.dp))
                Text("Sharing Screen", color = PrimaryCyan, fontSize = 13.sp)
            }
        } else if (isVideoMuted) {
            // Avatar Circle
            val initial = displayName.firstOrNull()?.uppercase() ?: "?"
            Box(
                modifier = Modifier
                    .size(56.dp)
                    .clip(CircleShape)
                    .background(SurfaceVariantDark),
                contentAlignment = Alignment.Center
            ) {
                Text(initial, color = Color.White, fontSize = 22.sp, fontWeight = FontWeight.Bold)
            }
        } else {
            // Active Camera Video Simulator
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                Icon(Icons.Default.Videocam, contentDescription = null, tint = TextMuted, modifier = Modifier.size(36.dp))
                Spacer(modifier = Modifier.height(4.dp))
                Text(if (isLocal) "You" else displayName, color = TextSecondary, fontSize = 12.sp)
            }
        }

        // Bottom Info Pill
        Row(
            modifier = Modifier
                .align(Alignment.BottomStart)
                .padding(8.dp)
                .clip(RoundedCornerShape(6.dp))
                .background(Color.Black.copy(alpha = 0.6f))
                .padding(horizontal = 8.dp, vertical = 4.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = if (isLocal) "$displayName (You)" else displayName,
                color = Color.White,
                fontSize = 11.sp,
                fontWeight = FontWeight.Medium
            )

            if (role == "host") {
                Spacer(modifier = Modifier.width(4.dp))
                Text("★", color = WarningYellow, fontSize = 10.sp)
            }

            Spacer(modifier = Modifier.width(6.dp))
            if (isAudioMuted) {
                Icon(Icons.Default.MicOff, contentDescription = "Muted", tint = AlertRed, modifier = Modifier.size(12.dp))
            } else if (isActiveSpeaker) {
                Icon(Icons.Default.VolumeUp, contentDescription = "Speaking", tint = AccentGreen, modifier = Modifier.size(12.dp))
            }

            if (isHandRaised) {
                Spacer(modifier = Modifier.width(4.dp))
                Text("✋", fontSize = 12.sp)
            }
        }
    }
}
