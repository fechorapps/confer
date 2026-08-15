package com.confer.mobile.features.lobby

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.confer.mobile.core.network.JoinMeetingResponse
import com.confer.mobile.core.theme.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun LobbyScreen(
    viewModel: LobbyViewModel,
    onJoinSuccess: (JoinMeetingResponse, String) -> Unit
) {
    val state by viewModel.uiState.collectAsState()

    LaunchedEffect(state.joinSuccess) {
        state.joinSuccess?.let { joinData ->
            onJoinSuccess(joinData, state.serverUrl)
            viewModel.resetJoinSuccess()
        }
    }

    Scaffold(
        containerColor = BackgroundDark
    ) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .verticalScroll(rememberScrollState())
                .padding(20.dp),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            Spacer(modifier = Modifier.height(10.dp))

            // Logo & Branding
            Text(
                text = "CONFER",
                color = PrimaryCyan,
                fontSize = 28.sp,
                fontWeight = FontWeight.Bold,
                letterSpacing = 2.sp
            )
            Text(
                text = "High-Performance Mobile Conferencing",
                color = TextSecondary,
                fontSize = 12.sp
            )

            Spacer(modifier = Modifier.height(20.dp))

            // Camera / Audio Preview Box
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .height(180.dp)
                    .clip(RoundedCornerShape(16.dp))
                    .background(SurfaceDark),
                contentAlignment = Alignment.Center
            ) {
                if (state.isCameraOff) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Icon(
                            Icons.Default.VideocamOff,
                            contentDescription = null,
                            tint = TextMuted,
                            modifier = Modifier.size(48.dp)
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        Text("Camera is off", color = TextMuted, fontSize = 13.sp)
                    }
                } else {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Icon(
                            Icons.Default.Videocam,
                            contentDescription = null,
                            tint = PrimaryCyan,
                            modifier = Modifier.size(48.dp)
                        )
                        Spacer(modifier = Modifier.height(8.dp))
                        Text("Camera Preview", color = TextSecondary, fontSize = 13.sp)
                    }
                }

                // Floating quick toggles
                Row(
                    modifier = Modifier
                        .align(Alignment.BottomCenter)
                        .padding(bottom = 12.dp)
                ) {
                    IconButton(
                        onClick = { viewModel.toggleMic() },
                        colors = IconButtonDefaults.iconButtonColors(
                            containerColor = if (state.isMicMuted) AlertRed else SurfaceVariantDark
                        )
                    ) {
                        Icon(
                            if (state.isMicMuted) Icons.Default.MicOff else Icons.Default.Mic,
                            contentDescription = "Toggle Mic",
                            tint = Color.White
                        )
                    }
                    Spacer(modifier = Modifier.width(12.dp))
                    IconButton(
                        onClick = { viewModel.toggleCamera() },
                        colors = IconButtonDefaults.iconButtonColors(
                            containerColor = if (state.isCameraOff) AlertRed else SurfaceVariantDark
                        )
                    ) {
                        Icon(
                            if (state.isCameraOff) Icons.Default.VideocamOff else Icons.Default.Videocam,
                            contentDescription = "Toggle Camera",
                            tint = Color.White
                        )
                    }
                }
            }

            Spacer(modifier = Modifier.height(16.dp))

            // Microphone Level Indicator
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Text("Mic Level:", color = TextSecondary, fontSize = 12.sp)
                Spacer(modifier = Modifier.width(8.dp))
                LinearProgressIndicator(
                    progress = { if (state.isMicMuted) 0f else state.micLevel },
                    modifier = Modifier
                        .weight(1f)
                        .height(6.dp)
                        .clip(RoundedCornerShape(3.dp)),
                    color = AccentGreen,
                    trackColor = SurfaceVariantDark
                )
            }

            Spacer(modifier = Modifier.height(20.dp))

            // User Profile Presets
            Text("Select Account", color = TextPrimary, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, modifier = Modifier.align(Alignment.Start))
            Spacer(modifier = Modifier.height(8.dp))
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                FilterChip(
                    selected = state.email == "host@confer.local",
                    onClick = { viewModel.selectPresetAccount("host@confer.local", "Host User (Mobile)") },
                    label = { Text("Host", fontSize = 12.sp) }
                )
                FilterChip(
                    selected = state.email == "participant1@confer.local",
                    onClick = { viewModel.selectPresetAccount("participant1@confer.local", "Alice (Mobile)") },
                    label = { Text("Alice", fontSize = 12.sp) }
                )
                FilterChip(
                    selected = state.email == "participant2@confer.local",
                    onClick = { viewModel.selectPresetAccount("participant2@confer.local", "Bob (Mobile)") },
                    label = { Text("Bob", fontSize = 12.sp) }
                )
            }

            Spacer(modifier = Modifier.height(12.dp))

            OutlinedTextField(
                value = state.displayName,
                onValueChange = { viewModel.updateDisplayName(it) },
                label = { Text("Display Name") },
                modifier = Modifier.fillMaxWidth(),
                colors = OutlinedTextFieldDefaults.colors(
                    focusedBorderColor = PrimaryCyan,
                    unfocusedBorderColor = BorderDark
                )
            )

            Spacer(modifier = Modifier.height(12.dp))

            OutlinedTextField(
                value = state.serverUrl,
                onValueChange = { viewModel.updateServerUrl(it) },
                label = { Text("Server URL") },
                modifier = Modifier.fillMaxWidth(),
                colors = OutlinedTextFieldDefaults.colors(
                    focusedBorderColor = PrimaryCyan,
                    unfocusedBorderColor = BorderDark
                )
            )

            Spacer(modifier = Modifier.height(24.dp))

            // Actions: New Meeting & Join by Code
            Button(
                onClick = { viewModel.createMeeting() },
                modifier = Modifier
                    .fillMaxWidth()
                    .height(48.dp),
                colors = ButtonDefaults.buttonColors(containerColor = PrimaryCyan),
                shape = RoundedCornerShape(12.dp),
                enabled = !state.isLoading
            ) {
                Icon(Icons.Default.Add, contentDescription = null, tint = BackgroundDark)
                Spacer(modifier = Modifier.width(8.dp))
                Text("Start New Meeting", color = BackgroundDark, fontWeight = FontWeight.Bold)
            }

            Spacer(modifier = Modifier.height(16.dp))

            Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                HorizontalDivider(modifier = Modifier.weight(1f), color = BorderDark)
                Text(" OR ", color = TextMuted, fontSize = 12.sp, modifier = Modifier.padding(horizontal = 8.dp))
                HorizontalDivider(modifier = Modifier.weight(1f), color = BorderDark)
            }

            Spacer(modifier = Modifier.height(16.dp))

            Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                OutlinedTextField(
                    value = state.joinCodeInput,
                    onValueChange = { viewModel.updateJoinCode(it) },
                    placeholder = { Text("Enter code (e.g. abc-defg-hij)") },
                    modifier = Modifier.weight(1f),
                    colors = OutlinedTextFieldDefaults.colors(
                        focusedBorderColor = PrimaryCyan,
                        unfocusedBorderColor = BorderDark
                    )
                )
                Spacer(modifier = Modifier.width(8.dp))
                Button(
                    onClick = { viewModel.joinByCode() },
                    modifier = Modifier.height(56.dp),
                    colors = ButtonDefaults.buttonColors(containerColor = SurfaceVariantDark),
                    shape = RoundedCornerShape(12.dp),
                    enabled = !state.isLoading
                ) {
                    Text("Join", color = PrimaryCyan, fontWeight = FontWeight.Bold)
                }
            }

            if (state.errorMessage != null) {
                Spacer(modifier = Modifier.height(12.dp))
                Text(
                    text = state.errorMessage ?: "",
                    color = AlertRed,
                    fontSize = 12.sp
                )
            }

            if (state.isLoading) {
                Spacer(modifier = Modifier.height(16.dp))
                CircularProgressIndicator(color = PrimaryCyan)
            }
        }
    }
}
