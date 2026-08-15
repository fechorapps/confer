package com.confer.mobile.features.meeting.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.MicOff
import androidx.compose.material.icons.filled.PersonRemove
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.confer.mobile.core.network.ParticipantState
import com.confer.mobile.core.network.WaitingParticipant
import com.confer.mobile.core.theme.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RosterBottomSheet(
    myDisplayName: String,
    myRole: String,
    roster: List<ParticipantState>,
    waitingParticipants: List<WaitingParticipant> = emptyList(),
    onHostMute: (String) -> Unit,
    onHostKick: (String) -> Unit,
    onHostAdmit: (String) -> Unit = {},
    onHostAdmitAll: () -> Unit = {},
    onHostReject: (String) -> Unit = {},
    onDismiss: () -> Unit
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val isHost = myRole == "host"
    var selectedTab by remember { mutableIntStateOf(0) } // 0: In Meeting, 1: Waiting Room

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = SurfaceDark
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .height(480.dp)
                .padding(16.dp)
        ) {
            Text(
                text = "Participants",
                color = Color.White,
                fontSize = 18.sp,
                fontWeight = FontWeight.Bold
            )
            Spacer(modifier = Modifier.height(8.dp))

            // Segmented Tabs for Hosts
            if (isHost) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(vertical = 4.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp)
                ) {
                    FilterChip(
                        selected = selectedTab == 0,
                        onClick = { selectedTab = 0 },
                        label = { Text("In Meeting (${roster.size + 1})", fontSize = 12.sp) },
                        colors = FilterChipDefaults.filterChipColors(
                            selectedContainerColor = PrimaryCyan,
                            selectedLabelColor = BackgroundDark,
                            containerColor = SurfaceVariantDark,
                            labelColor = Color.White
                        )
                    )

                    val waitingBadgeColor = if (waitingParticipants.isNotEmpty() && selectedTab != 1) WarningYellow else Color.White
                    FilterChip(
                        selected = selectedTab == 1,
                        onClick = { selectedTab = 1 },
                        label = {
                            Text(
                                text = if (waitingParticipants.isNotEmpty()) "⏳ Waiting (${waitingParticipants.size})" else "Waiting Room (0)",
                                fontSize = 12.sp,
                                color = if (selectedTab == 1) BackgroundDark else waitingBadgeColor
                            )
                        },
                        colors = FilterChipDefaults.filterChipColors(
                            selectedContainerColor = PrimaryCyan,
                            selectedLabelColor = BackgroundDark,
                            containerColor = if (waitingParticipants.isNotEmpty()) SurfaceVariantDark else SurfaceVariantDark,
                            labelColor = waitingBadgeColor
                        )
                    )
                }
                Spacer(modifier = Modifier.height(8.dp))
            }

            if (isHost && selectedTab == 1) {
                // --- WAITING ROOM TAB ---
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween
                ) {
                    Text(
                        text = "Waiting in Lobby (${waitingParticipants.size})",
                        color = WarningYellow,
                        fontSize = 14.sp,
                        fontWeight = FontWeight.SemiBold
                    )
                    if (waitingParticipants.isNotEmpty()) {
                        Button(
                            onClick = onHostAdmitAll,
                            colors = ButtonDefaults.buttonColors(containerColor = AccentGreen),
                            contentPadding = PaddingValues(horizontal = 12.dp, vertical = 4.dp),
                            shape = RoundedCornerShape(16.dp)
                        ) {
                            Text("✓ Admit All", color = Color.White, fontSize = 11.sp, fontWeight = FontWeight.Bold)
                        }
                    }
                }

                Spacer(modifier = Modifier.height(8.dp))

                if (waitingParticipants.isEmpty()) {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .weight(1f),
                        contentAlignment = Alignment.Center
                    ) {
                        Text(
                            text = "No participants currently waiting in the lobby.",
                            color = TextMuted,
                            fontSize = 13.sp
                        )
                    }
                } else {
                    LazyColumn(
                        modifier = Modifier.fillMaxWidth(),
                        verticalArrangement = Arrangement.spacedBy(8.dp)
                    ) {
                        items(waitingParticipants, key = { it.participantId }) { wp ->
                            Row(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .background(SurfaceVariantDark, RoundedCornerShape(8.dp))
                                    .padding(horizontal = 12.dp, vertical = 8.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.SpaceBetween
                            ) {
                                Column(modifier = Modifier.weight(1f)) {
                                    Text(wp.displayName, color = Color.White, fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                                    if (!wp.email.isNullOrBlank()) {
                                        Text(wp.email, color = TextMuted, fontSize = 11.sp)
                                    }
                                }

                                Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                                    IconButton(
                                        onClick = { onHostAdmit(wp.participantId) },
                                        colors = IconButtonDefaults.iconButtonColors(containerColor = AccentGreen.copy(alpha = 0.2f))
                                    ) {
                                        Icon(Icons.Default.Check, contentDescription = "Admit", tint = AccentGreen)
                                    }
                                    IconButton(
                                        onClick = { onHostReject(wp.participantId) },
                                        colors = IconButtonDefaults.iconButtonColors(containerColor = AlertRed.copy(alpha = 0.2f))
                                    ) {
                                        Icon(Icons.Default.Close, contentDescription = "Reject", tint = AlertRed)
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                // --- IN-MEETING PARTICIPANTS TAB ---
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
}
