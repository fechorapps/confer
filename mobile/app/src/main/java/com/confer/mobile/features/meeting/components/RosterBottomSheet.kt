package com.confer.mobile.features.meeting.components

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.MicOff
import androidx.compose.material.icons.filled.PersonRemove
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.confer.mobile.core.network.ParticipantState
import com.confer.mobile.core.theme.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RosterBottomSheet(
    myDisplayName: String,
    myRole: String,
    roster: List<ParticipantState>,
    onHostMute: (String) -> Unit,
    onHostKick: (String) -> Unit,
    onDismiss: () -> Unit
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val isHost = myRole == "host"

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = SurfaceDark
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .height(450.dp)
                .padding(16.dp)
        ) {
            Text(
                text = "Participants (${roster.size + 1})",
                color = Color.White,
                fontSize = 18.sp,
                fontWeight = FontWeight.Bold
            )
            Spacer(modifier = Modifier.height(12.dp))

            LazyColumn(
                modifier = Modifier.fillMaxWidth(),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                // Local participant
                item {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Text(myDisplayName, color = Color.White, fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                        Text(" (You)", color = TextMuted, fontSize = 12.sp)
                        if (myRole == "host") {
                            Spacer(modifier = Modifier.width(4.dp))
                            Text("★ Host", color = WarningYellow, fontSize = 11.sp)
                        }
                    }
                    HorizontalDivider(modifier = Modifier.padding(vertical = 8.dp), color = BorderDark)
                }

                // Remote participants
                items(roster, key = { it.participantId }) { p ->
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.SpaceBetween
                    ) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Text(p.displayName, color = Color.White, fontSize = 14.sp)
                            if (p.role == "host") {
                                Spacer(modifier = Modifier.width(4.dp))
                                Text("★ Host", color = WarningYellow, fontSize = 11.sp)
                            }
                            if (p.isAudioMuted) {
                                Spacer(modifier = Modifier.width(6.dp))
                                Text("🔇", fontSize = 12.sp)
                            }
                            if (p.isHandRaised) {
                                Spacer(modifier = Modifier.width(6.dp))
                                Text("✋", fontSize = 12.sp)
                            }
                        }

                        if (isHost) {
                            Row {
                                IconButton(onClick = { onHostMute(p.participantId) }) {
                                    Icon(Icons.Default.MicOff, contentDescription = "Mute", tint = TextMuted)
                                }
                                IconButton(onClick = { onHostKick(p.participantId) }) {
                                    Icon(Icons.Default.PersonRemove, contentDescription = "Kick", tint = AlertRed)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
