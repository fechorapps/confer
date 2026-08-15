package com.confer.mobile.features.meeting.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.HourglassEmpty
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.confer.mobile.core.theme.*

@Composable
fun WaitingLobbyView(
    title: String,
    myDisplayName: String,
    waitingRoomMessage: String? = null,
    isMicMuted: Boolean,
    isCameraOff: Boolean,
    onLeaveMeeting: () -> Unit,
    modifier: Modifier = Modifier
) {
    Box(
        modifier = modifier
            .fillMaxSize()
            .background(BackgroundDark)
            .padding(24.dp),
        contentAlignment = Alignment.Center
    ) {
        Card(
            modifier = Modifier.fillMaxWidth(),
            shape = RoundedCornerShape(20.dp),
            colors = CardDefaults.cardColors(containerColor = SurfaceDark),
            elevation = CardDefaults.cardElevation(defaultElevation = 8.dp)
        ) {
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(24.dp),
                horizontalAlignment = Alignment.CenterVertically
            ) {
                // Hourglass / Spinner Icon
                Box(
                    modifier = Modifier
                        .size(64.dp)
                        .clip(CircleShape)
                        .background(PrimaryCyan.copy(alpha = 0.15f)),
                    contentAlignment = Alignment.Center
                ) {
                    Icon(
                        Icons.Default.HourglassEmpty,
                        contentDescription = "Waiting",
                        tint = PrimaryCyan,
                        modifier = Modifier.size(32.dp)
                    )
                }

                Spacer(modifier = Modifier.height(16.dp))

                // Meeting Title
                Text(
                    text = title.ifBlank { "Meeting Room" },
                    color = Color.White,
                    fontSize = 20.sp,
                    fontWeight = FontWeight.Bold,
                    textAlign = TextAlign.Center
                )

                Spacer(modifier = Modifier.height(8.dp))

                // Waiting Status Message
                val statusText = waitingRoomMessage ?: "Please wait, the meeting host will let you in soon."
                Text(
                    text = statusText,
                    color = TextMuted,
                    fontSize = 13.sp,
                    textAlign = TextAlign.Center,
                    lineHeight = 18.sp
                )

                Spacer(modifier = Modifier.height(20.dp))
                HorizontalDivider(color = BorderDark)
                Spacer(modifier = Modifier.height(16.dp))

                // Participant Badge
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .background(SurfaceVariantDark, RoundedCornerShape(12.dp))
                        .padding(horizontal = 14.dp, vertical = 10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        val initial = myDisplayName.firstOrNull()?.uppercase() ?: "U"
                        Box(
                            modifier = Modifier
                                .size(32.dp)
                                .clip(CircleShape)
                                .background(PrimaryCyan),
                            contentAlignment = Alignment.Center
                        ) {
                            Text(initial, color = Color.White, fontSize = 14.sp, fontWeight = FontWeight.Bold)
                        }
                        Spacer(modifier = Modifier.width(10.dp))
                        Text(myDisplayName, color = Color.White, fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                    }

                    Surface(
                        shape = RoundedCornerShape(6.dp),
                        color = Color(0xFF451E0A)
                    ) {
                        Text(
                            text = "WAITING",
                            color = WarningYellow,
                            fontSize = 10.sp,
                            fontWeight = FontWeight.Bold,
                            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp)
                        )
                    }
                }

                Spacer(modifier = Modifier.height(14.dp))

                // Pre-join status checks
                Row(
                    horizontalArrangement = Arrangement.spacedBy(16.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        text = if (isMicMuted) "🔇 Mic Muted" else "🎙 Mic Ready",
                        color = if (isMicMuted) AlertRed else AccentGreen,
                        fontSize = 12.sp
                    )
                    Text("•", color = BorderDark)
                    Text(
                        text = if (isCameraOff) "📷 Camera Off" else "🎥 Camera Ready",
                        color = if (isCameraOff) AlertRed else AccentGreen,
                        fontSize = 12.sp
                    )
                }

                Spacer(modifier = Modifier.height(24.dp))

                // Leave Meeting Button
                Button(
                    onClick = onLeaveMeeting,
                    colors = ButtonDefaults.buttonColors(containerColor = AlertRed),
                    shape = RoundedCornerShape(12.dp),
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Text("✕ Leave Waiting Room", color = Color.White, fontWeight = FontWeight.Bold)
                }
            }
        }
    }
}
