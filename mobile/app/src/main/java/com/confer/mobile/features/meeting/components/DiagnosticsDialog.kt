package com.confer.mobile.features.meeting.components

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.confer.mobile.core.theme.*

@Composable
fun DiagnosticsDialog(
    rttMs: Long,
    packetLossPct: Float,
    onDismiss: () -> Unit
) {
    AlertDialog(
        onDismissRequest = onDismiss,
        containerColor = SurfaceDark,
        title = {
            Text("📊 Performance Diagnostics", color = Color.White, fontSize = 18.sp, fontWeight = FontWeight.Bold)
        },
        text = {
            Column(modifier = Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                    Text("RTT Latency:", color = TextSecondary, fontSize = 13.sp)
                    Text("${rttMs} ms", color = if (rttMs < 100) AccentGreen else WarningYellow, fontSize = 13.sp, fontWeight = FontWeight.Bold)
                }
                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                    Text("Packet Loss:", color = TextSecondary, fontSize = 13.sp)
                    Text("${packetLossPct}%", color = Color.White, fontSize = 13.sp)
                }
                HorizontalDivider(color = BorderDark, modifier = Modifier.padding(vertical = 4.dp))
                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                    Text("Simulcast Layer:", color = TextSecondary, fontSize = 13.sp)
                    Text("Layer F (720p)", color = PrimaryCyan, fontSize = 13.sp)
                }
                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                    Text("Audio Codec:", color = TextSecondary, fontSize = 13.sp)
                    Text("Opus 48kHz (FEC)", color = Color.White, fontSize = 13.sp)
                }
                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                    Text("ICE Protocol:", color = TextSecondary, fontSize = 13.sp)
                    Text("Direct UDP", color = AccentGreen, fontSize = 13.sp)
                }
            }
        },
        confirmButton = {
            TextButton(onClick = onDismiss) {
                Text("Close", color = PrimaryCyan)
            }
        },
        shape = RoundedCornerShape(16.dp)
    )
}
