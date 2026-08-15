package com.confer.mobile.features.meeting.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Send
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.confer.mobile.core.network.ChatMessage
import com.confer.mobile.core.theme.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ChatBottomSheet(
    messages: List<ChatMessage>,
    myParticipantId: String,
    onSendMessage: (String) -> Unit,
    onDismiss: () -> Unit
) {
    var textInput by remember { mutableStateOf("") }
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)

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
                text = "In-Meeting Chat",
                color = Color.White,
                fontSize = 18.sp,
                fontWeight = FontWeight.Bold
            )
            Spacer(modifier = Modifier.height(12.dp))

            // Messages list
            LazyColumn(
                modifier = Modifier
                    .weight(1f)
                    .fillMaxWidth(),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                if (messages.isEmpty()) {
                    item {
                        Box(modifier = Modifier.fillMaxWidth().padding(top = 32.dp), contentAlignment = Alignment.Center) {
                            Text("No messages yet. Start the conversation! 👋", color = TextMuted, fontSize = 13.sp)
                        }
                    }
                }
                items(messages, key = { it.id }) { msg ->
                    val isMe = msg.fromId == myParticipantId
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(8.dp))
                            .background(if (isMe) SurfaceVariantDark else SurfaceVariantDark.copy(alpha = 0.5f))
                            .padding(8.dp)
                    ) {
                        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                            Text(
                                text = if (isMe) "${msg.fromName} (You)" else msg.fromName,
                                color = if (isMe) PrimaryCyan else AccentGreen,
                                fontSize = 12.sp,
                                fontWeight = FontWeight.SemiBold
                            )
                            Text(text = msg.sentAt, color = TextMuted, fontSize = 10.sp)
                        }
                        Spacer(modifier = Modifier.height(2.dp))
                        Text(text = msg.body, color = Color.White, fontSize = 13.sp)
                    }
                }
            }

            Spacer(modifier = Modifier.height(12.dp))

            // Input Bar
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                OutlinedTextField(
                    value = textInput,
                    onValueChange = { textInput = it },
                    placeholder = { Text("Send a message...") },
                    modifier = Modifier.weight(1f),
                    shape = RoundedCornerShape(24.dp),
                    colors = OutlinedTextFieldDefaults.colors(
                        focusedBorderColor = PrimaryCyan,
                        unfocusedBorderColor = BorderDark,
                        focusedContainerColor = SurfaceVariantDark,
                        unfocusedContainerColor = SurfaceVariantDark
                    )
                )
                Spacer(modifier = Modifier.width(8.dp))
                IconButton(
                    onClick = {
                        if (textInput.isNotBlank()) {
                            onSendMessage(textInput.trim())
                            textInput = ""
                        }
                    },
                    colors = IconButtonDefaults.iconButtonColors(containerColor = PrimaryCyan)
                ) {
                    Icon(Icons.Default.Send, contentDescription = "Send", tint = BackgroundDark)
                }
            }
        }
    }
}
