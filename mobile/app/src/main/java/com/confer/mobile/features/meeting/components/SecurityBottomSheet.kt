package com.confer.mobile.features.meeting.components

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.confer.mobile.core.network.MeetingPolicy
import com.confer.mobile.core.theme.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SecurityBottomSheet(
    policy: MeetingPolicy,
    onToggleLock: (Boolean) -> Unit,
    onToggleWaitingRoom: (Boolean) -> Unit,
    onToggleScreenShare: (Boolean) -> Unit,
    onToggleChat: (Boolean) -> Unit,
    onToggleUnmute: (Boolean) -> Unit,
    onToggleWatermark: (Boolean) -> Unit,
    onDismiss: () -> Unit
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = SurfaceDark
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 20.dp, vertical = 12.dp)
                .verticalScroll(rememberScrollState())
        ) {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier = Modifier.fillMaxWidth()
            ) {
                Icon(Icons.Default.Security, contentDescription = "Security", tint = PrimaryCyan, modifier = Modifier.size(24.dp))
                Spacer(modifier = Modifier.width(10.dp))
                Text(
                    text = "Meeting Security & Policies",
                    color = Color.White,
                    fontSize = 18.sp,
                    fontWeight = FontWeight.Bold
                )
            }

            Spacer(modifier = Modifier.height(16.dp))

            Text(
                text = "ROOM ACCESS",
                color = PrimaryCyan,
                fontSize = 11.sp,
                fontWeight = FontWeight.Bold,
                letterSpacing = 1.sp
            )
            Spacer(modifier = Modifier.height(8.dp))

            SecurityToggleItem(
                icon = Icons.Default.Lock,
                title = "Lock Meeting",
                description = "Prevent any new participants from joining",
                checked = policy.isLocked,
                onCheckedChange = onToggleLock
            )

            SecurityToggleItem(
                icon = Icons.Default.HourglassEmpty,
                title = "Waiting Room",
                description = "Hold new joiners in lobby until host admits them",
                checked = policy.waitingRoomEnabled,
                onCheckedChange = onToggleWaitingRoom
            )

            HorizontalDivider(modifier = Modifier.padding(vertical = 12.dp), color = BorderDark)

            Text(
                text = "PARTICIPANT PERMISSIONS",
                color = PrimaryCyan,
                fontSize = 11.sp,
                fontWeight = FontWeight.Bold,
                letterSpacing = 1.sp
            )
            Spacer(modifier = Modifier.height(8.dp))

            SecurityToggleItem(
                icon = Icons.Default.ScreenShare,
                title = "Share Screen",
                description = "Allow participants to share their screen",
                checked = policy.allowScreenShare,
                onCheckedChange = onToggleScreenShare
            )

            SecurityToggleItem(
                icon = Icons.Default.Chat,
                title = "Send Chat Messages",
                description = "Allow participants to send in-call chat",
                checked = policy.allowChat,
                onCheckedChange = onToggleChat
            )

            SecurityToggleItem(
                icon = Icons.Default.Mic,
                title = "Unmute Themselves",
                description = "Allow participants to unmute their mic",
                checked = policy.allowUnmute,
                onCheckedChange = onToggleUnmute
            )

            HorizontalDivider(modifier = Modifier.padding(vertical = 12.dp), color = BorderDark)

            Text(
                text = "DATA LOSS PREVENTION (DLP)",
                color = PrimaryCyan,
                fontSize = 11.sp,
                fontWeight = FontWeight.Bold,
                letterSpacing = 1.sp
            )
            Spacer(modifier = Modifier.height(8.dp))

            SecurityToggleItem(
                icon = Icons.Default.Badge,
                title = "Visual Watermark",
                description = "Tiled semi-transparent participant identity on video",
                checked = policy.watermarkEnabled,
                onCheckedChange = onToggleWatermark
            )

            Spacer(modifier = Modifier.height(24.dp))
        }
    }
}

@Composable
private fun SecurityToggleItem(
    icon: ImageVector,
    title: String,
    description: String,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Row(
            modifier = Modifier.weight(1f),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(icon, contentDescription = title, tint = TextMuted, modifier = Modifier.size(20.dp))
            Spacer(modifier = Modifier.width(12.dp))
            Column {
                Text(text = title, color = Color.White, fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                Text(text = description, color = TextMuted, fontSize = 11.sp)
            }
        }
        Switch(
            checked = checked,
            onCheckedChange = onCheckedChange,
            colors = SwitchDefaults.colors(
                checkedThumbColor = Color.White,
                checkedTrackColor = PrimaryCyan,
                uncheckedThumbColor = TextMuted,
                uncheckedTrackColor = SurfaceVariantDark
            )
        )
    }
}
