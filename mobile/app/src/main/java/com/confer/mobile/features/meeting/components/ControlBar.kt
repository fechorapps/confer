package com.confer.mobile.features.meeting.components

import androidx.compose.foundation.background
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material.icons.outlined.Poll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.confer.mobile.core.theme.*

@Composable
fun ControlBar(
    isMicMuted: Boolean,
    isCameraOff: Boolean,
    isScreenSharing: Boolean,
    isHandRaised: Boolean,
    isWhiteboardActive: Boolean = false,
    isHost: Boolean = false,
    unreadChatCount: Int = 0,
    unreadPollCount: Int = 0,
    onToggleMic: () -> Unit,
    onToggleCamera: () -> Unit,
    onFlipCamera: () -> Unit,
    onToggleScreenShare: () -> Unit,
    onToggleWhiteboard: () -> Unit,
    onToggleHandRaise: () -> Unit,
    onSendReaction: (String) -> Unit,
    onOpenChat: () -> Unit,
    onOpenPolls: () -> Unit,
    onOpenRoster: () -> Unit,
    onOpenSecurity: () -> Unit = {},
    onOpenDiagnostics: () -> Unit,
    onLeaveMeeting: () -> Unit,
    modifier: Modifier = Modifier
) {
    var showReactionMenu by remember { mutableStateOf(false) }

    Surface(
        modifier = modifier
            .fillMaxWidth()
            .padding(12.dp)
            .clip(RoundedCornerShape(28.dp)),
        color = SurfaceDark.copy(alpha = 0.95f),
        tonalElevation = 8.dp
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 8.dp, vertical = 8.dp)
                .horizontalScroll(rememberScrollState()),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            // Mic Toggle
            IconButton(
                onClick = onToggleMic,
                colors = IconButtonDefaults.iconButtonColors(
                    containerColor = if (isMicMuted) AlertRed else SurfaceVariantDark
                )
            ) {
                Icon(if (isMicMuted) Icons.Default.MicOff else Icons.Default.Mic, contentDescription = "Mic", tint = Color.White)
            }

            // Camera Toggle
            IconButton(
                onClick = onToggleCamera,
                colors = IconButtonDefaults.iconButtonColors(
                    containerColor = if (isCameraOff) AlertRed else SurfaceVariantDark
                )
            ) {
                Icon(if (isCameraOff) Icons.Default.VideocamOff else Icons.Default.Videocam, contentDescription = "Camera", tint = Color.White)
            }

            // Flip Camera
            IconButton(
                onClick = onFlipCamera,
                colors = IconButtonDefaults.iconButtonColors(containerColor = SurfaceVariantDark)
            ) {
                Icon(Icons.Default.FlipCameraAndroid, contentDescription = "Flip Camera", tint = Color.White)
            }

            // Screen Share
            IconButton(
                onClick = onToggleScreenShare,
                colors = IconButtonDefaults.iconButtonColors(
                    containerColor = if (isScreenSharing) PrimaryCyan else SurfaceVariantDark
                )
            ) {
                Icon(Icons.Default.ScreenShare, contentDescription = "Screen Share", tint = if (isScreenSharing) BackgroundDark else Color.White)
            }

            // Whiteboard Toggle
            IconButton(
                onClick = onToggleWhiteboard,
                colors = IconButtonDefaults.iconButtonColors(
                    containerColor = if (isWhiteboardActive) PrimaryCyan else SurfaceVariantDark
                )
            ) {
                Icon(
                    Icons.Default.Draw,
                    contentDescription = "Whiteboard",
                    tint = if (isWhiteboardActive) BackgroundDark else Color.White
                )
            }

            // Hand Raise
            IconButton(
                onClick = onToggleHandRaise,
                colors = IconButtonDefaults.iconButtonColors(
                    containerColor = if (isHandRaised) WarningYellow else SurfaceVariantDark
                )
            ) {
                Text(if (isHandRaised) "✋" else "👋", fontSize = 18.sp)
            }

            // Reactions Trigger
            Box {
                IconButton(
                    onClick = { showReactionMenu = !showReactionMenu },
                    colors = IconButtonDefaults.iconButtonColors(containerColor = SurfaceVariantDark)
                ) {
                    Text("😀", fontSize = 18.sp)
                }

                DropdownMenu(
                    expanded = showReactionMenu,
                    onDismissRequest = { showReactionMenu = false }
                ) {
                    Row(modifier = Modifier.padding(horizontal = 8.dp)) {
                        listOf("👍", "❤️", "👏", "🎉", "🚀", "💡").forEach { emoji ->
                            TextButton(onClick = {
                                onSendReaction(emoji)
                                showReactionMenu = false
                            }) {
                                Text(emoji, fontSize = 20.sp)
                            }
                        }
                    }
                }
            }

            // Chat Drawer Toggle (with Badge)
            BadgedBox(
                badge = {
                    if (unreadChatCount > 0) {
                        Badge(containerColor = PrimaryCyan) {
                            Text(unreadChatCount.toString(), color = BackgroundDark)
                        }
                    }
                }
            ) {
                IconButton(
                    onClick = onOpenChat,
                    colors = IconButtonDefaults.iconButtonColors(containerColor = SurfaceVariantDark)
                ) {
                    Icon(Icons.Default.Chat, contentDescription = "Chat", tint = Color.White)
                }
            }

            // Polls Drawer Toggle (with Badge)
            BadgedBox(
                badge = {
                    if (unreadPollCount > 0) {
                        Badge(containerColor = AccentGreen) {
                            Text(unreadPollCount.toString(), color = BackgroundDark)
                        }
                    }
                }
            ) {
                IconButton(
                    onClick = onOpenPolls,
                    colors = IconButtonDefaults.iconButtonColors(containerColor = SurfaceVariantDark)
                ) {
                    Icon(Icons.Outlined.Poll, contentDescription = "Polls", tint = Color.White)
                }
            }

            // Participants Roster
            IconButton(
                onClick = onOpenRoster,
                colors = IconButtonDefaults.iconButtonColors(containerColor = SurfaceVariantDark)
            ) {
                Icon(Icons.Default.People, contentDescription = "Participants", tint = Color.White)
            }

            // Host Security Policies
            if (isHost) {
                IconButton(
                    onClick = onOpenSecurity,
                    colors = IconButtonDefaults.iconButtonColors(containerColor = SurfaceVariantDark)
                ) {
                    Icon(Icons.Default.Security, contentDescription = "Security", tint = WarningYellow)
                }
            }

            // Diagnostics HUD
            IconButton(
                onClick = onOpenDiagnostics,
                colors = IconButtonDefaults.iconButtonColors(containerColor = SurfaceVariantDark)
            ) {
                Icon(Icons.Default.Speed, contentDescription = "Diagnostics", tint = Color.White)
            }

            Spacer(modifier = Modifier.width(4.dp))

            // Leave Meeting Button
            IconButton(
                onClick = onLeaveMeeting,
                colors = IconButtonDefaults.iconButtonColors(containerColor = AlertRed)
            ) {
                Icon(Icons.Default.CallEnd, contentDescription = "Leave Call", tint = Color.White)
            }
        }
    }
}

