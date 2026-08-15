package com.confer.mobile.features.meeting.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material.icons.outlined.CheckCircle
import androidx.compose.material.icons.outlined.Poll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.confer.mobile.core.network.Poll
import com.confer.mobile.core.network.PollOption
import com.confer.mobile.core.theme.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PollsBottomSheet(
    polls: List<Poll>,
    myParticipantId: String,
    isHost: Boolean,
    onCreatePoll: (question: String, options: List<String>) -> Unit,
    onVote: (pollId: String, optionId: String) -> Unit,
    onEndPoll: (pollId: String) -> Unit,
    onDismiss: () -> Unit
) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    var selectedTab by remember { mutableIntStateOf(if (polls.isEmpty()) 1 else 0) }

    ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
        containerColor = SurfaceDark
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .fillMaxHeight(0.85f)
                .padding(horizontal = 16.dp, vertical = 8.dp)
        ) {
            // Header
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Icon(
                        Icons.Outlined.Poll,
                        contentDescription = null,
                        tint = PrimaryCyan,
                        modifier = Modifier.size(24.dp)
                    )
                    Spacer(modifier = Modifier.width(8.dp))
                    Text(
                        text = "Meeting Polls",
                        color = Color.White,
                        fontSize = 18.sp,
                        fontWeight = FontWeight.Bold
                    )
                }

                IconButton(onClick = onDismiss) {
                    Icon(Icons.Default.Close, contentDescription = "Close", tint = TextMuted)
                }
            }

            Spacer(modifier = Modifier.height(10.dp))

            // Navigation Tabs (Polls List vs Create Poll)
            TabRow(
                selectedTabIndex = selectedTab,
                containerColor = SurfaceVariantDark,
                contentColor = PrimaryCyan,
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(12.dp))
            ) {
                Tab(
                    selected = selectedTab == 0,
                    onClick = { selectedTab = 0 },
                    text = {
                        Text(
                            text = "Active & Results (${polls.count { it.isActive }})",
                            fontSize = 13.sp,
                            fontWeight = if (selectedTab == 0) FontWeight.Bold else FontWeight.Normal,
                            color = if (selectedTab == 0) PrimaryCyan else TextSecondary
                        )
                    }
                )
                Tab(
                    selected = selectedTab == 1,
                    onClick = { selectedTab = 1 },
                    text = {
                        Text(
                            text = "+ Create Poll",
                            fontSize = 13.sp,
                            fontWeight = if (selectedTab == 1) FontWeight.Bold else FontWeight.Normal,
                            color = if (selectedTab == 1) PrimaryCyan else TextSecondary
                        )
                    }
                )
            }

            Spacer(modifier = Modifier.height(14.dp))

            if (selectedTab == 0) {
                // Polls List View
                PollsListView(
                    polls = polls,
                    myParticipantId = myParticipantId,
                    isHost = isHost,
                    onVote = onVote,
                    onEndPoll = onEndPoll,
                    onSwitchToCreate = { selectedTab = 1 }
                )
            } else {
                // Create Poll View
                CreatePollView(
                    onCreatePoll = { question, options ->
                        onCreatePoll(question, options)
                        selectedTab = 0
                    }
                )
            }
        }
    }
}

@Composable
private fun PollsListView(
    polls: List<Poll>,
    myParticipantId: String,
    isHost: Boolean,
    onVote: (pollId: String, optionId: String) -> Unit,
    onEndPoll: (pollId: String) -> Unit,
    onSwitchToCreate: () -> Unit
) {
    if (polls.isEmpty()) {
        Box(
            modifier = Modifier
                .fillMaxSize()
                .padding(32.dp),
            contentAlignment = Alignment.Center
        ) {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                Icon(
                    Icons.Outlined.Poll,
                    contentDescription = null,
                    tint = TextMuted,
                    modifier = Modifier.size(56.dp)
                )
                Spacer(modifier = Modifier.height(12.dp))
                Text(
                    text = "No polls created yet",
                    color = Color.White,
                    fontSize = 16.sp,
                    fontWeight = FontWeight.SemiBold
                )
                Spacer(modifier = Modifier.height(6.dp))
                Text(
                    text = "Engage meeting participants by starting a live quick poll.",
                    color = TextMuted,
                    fontSize = 13.sp
                )
                Spacer(modifier = Modifier.height(16.dp))
                Button(
                    onClick = onSwitchToCreate,
                    colors = ButtonDefaults.buttonColors(containerColor = PrimaryCyan)
                ) {
                    Icon(Icons.Default.Add, contentDescription = null, tint = BackgroundDark)
                    Spacer(modifier = Modifier.width(6.dp))
                    Text("Create First Poll", color = BackgroundDark, fontWeight = FontWeight.Bold)
                }
            }
        }
    } else {
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            verticalArrangement = Arrangement.spacedBy(14.dp),
            contentPadding = PaddingValues(bottom = 24.dp)
        ) {
            items(polls, key = { it.id }) { poll ->
                PollCard(
                    poll = poll,
                    myParticipantId = myParticipantId,
                    isHost = isHost,
                    onVote = onVote,
                    onEndPoll = onEndPoll
                )
            }
        }
    }
}

@Composable
private fun PollCard(
    poll: Poll,
    myParticipantId: String,
    isHost: Boolean,
    onVote: (pollId: String, optionId: String) -> Unit,
    onEndPoll: (pollId: String) -> Unit
) {
    var selectedOptionId by remember(poll.id, poll.votedOptionId) {
        mutableStateOf(poll.votedOptionId)
    }
    val hasVoted = poll.votedOptionId != null || !poll.isActive
    val totalVotes = if (poll.totalVotes > 0) poll.totalVotes else poll.options.sumOf { it.voteCount }
    val canEndPoll = (isHost || poll.createdBy == myParticipantId) && poll.isActive

    Card(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(16.dp))
            .border(1.dp, if (poll.isActive) PrimaryCyan.copy(alpha = 0.3f) else BorderDark, RoundedCornerShape(16.dp)),
        colors = CardDefaults.cardColors(containerColor = SurfaceVariantDark)
    ) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(16.dp)
        ) {
            // Poll Status Row
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Box(
                        modifier = Modifier
                            .clip(RoundedCornerShape(6.dp))
                            .background(if (poll.isActive) AccentGreen.copy(alpha = 0.2f) else TextMuted.copy(alpha = 0.2f))
                            .padding(horizontal = 8.dp, vertical = 3.dp)
                    ) {
                        Text(
                            text = if (poll.isActive) "● LIVE" else "ENDED",
                            color = if (poll.isActive) AccentGreen else TextMuted,
                            fontSize = 11.sp,
                            fontWeight = FontWeight.Bold
                        )
                    }

                    Spacer(modifier = Modifier.width(8.dp))

                    Text(
                        text = "$totalVotes ${if (totalVotes == 1) "vote" else "votes"}",
                        color = TextSecondary,
                        fontSize = 12.sp
                    )
                }

                if (canEndPoll) {
                    TextButton(
                        onClick = { onEndPoll(poll.id) },
                        colors = ButtonDefaults.textButtonColors(contentColor = AlertRed),
                        contentPadding = PaddingValues(horizontal = 8.dp, vertical = 2.dp)
                    ) {
                        Icon(Icons.Default.Stop, contentDescription = null, modifier = Modifier.size(16.dp))
                        Spacer(modifier = Modifier.width(4.dp))
                        Text("End Poll", fontSize = 12.sp, fontWeight = FontWeight.SemiBold)
                    }
                }
            }

            Spacer(modifier = Modifier.height(8.dp))

            // Question
            Text(
                text = poll.question,
                color = Color.White,
                fontSize = 16.sp,
                fontWeight = FontWeight.SemiBold
            )

            if (poll.createdByName.isNotBlank()) {
                Text(
                    text = "Asked by ${poll.createdByName}",
                    color = TextMuted,
                    fontSize = 11.sp,
                    modifier = Modifier.padding(top = 2.dp)
                )
            }

            Spacer(modifier = Modifier.height(14.dp))

            // Options List
            Column(
                modifier = Modifier.fillMaxWidth(),
                verticalArrangement = Arrangement.spacedBy(8.dp)
            ) {
                poll.options.forEach { option ->
                    val percentage = if (totalVotes > 0) (option.voteCount.toFloat() / totalVotes.toFloat()) else 0f
                    val percentageInt = (percentage * 100).toInt()
                    val isMyVote = poll.votedOptionId == option.id || selectedOptionId == option.id

                    if (hasVoted) {
                        // Results View with Animated Progress Bar
                        ResultOptionItem(
                            option = option,
                            percentage = percentage,
                            percentageInt = percentageInt,
                            isMyVote = poll.votedOptionId == option.id
                        )
                    } else {
                        // Interactive Voting Option
                        VotingOptionItem(
                            option = option,
                            isSelected = selectedOptionId == option.id,
                            onSelect = { selectedOptionId = option.id }
                        )
                    }
                }
            }

            // Submit Vote Button (only when voting is open and user hasn't voted yet)
            if (!hasVoted && poll.isActive) {
                Spacer(modifier = Modifier.height(14.dp))
                Button(
                    onClick = {
                        selectedOptionId?.let { optId ->
                            onVote(poll.id, optId)
                        }
                    },
                    enabled = selectedOptionId != null,
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(12.dp),
                    colors = ButtonDefaults.buttonColors(
                        containerColor = PrimaryCyan,
                        disabledContainerColor = SurfaceDark
                    )
                ) {
                    Text(
                        text = "Submit Vote",
                        color = if (selectedOptionId != null) BackgroundDark else TextMuted,
                        fontWeight = FontWeight.Bold,
                        fontSize = 14.sp
                    )
                }
            }
        }
    }
}

@Composable
private fun VotingOptionItem(
    option: PollOption,
    isSelected: Boolean,
    onSelect: () -> Unit
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(10.dp))
            .background(if (isSelected) PrimaryCyan.copy(alpha = 0.12f) else SurfaceDark)
            .border(
                width = if (isSelected) 1.5.dp else 1.dp,
                color = if (isSelected) PrimaryCyan else BorderDark,
                shape = RoundedCornerShape(10.dp)
            )
            .clickable { onSelect() }
            .padding(horizontal = 14.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        RadioButton(
            selected = isSelected,
            onClick = onSelect,
            colors = RadioButtonDefaults.colors(
                selectedColor = PrimaryCyan,
                unselectedColor = TextMuted
            ),
            modifier = Modifier.size(20.dp)
        )
        Spacer(modifier = Modifier.width(12.dp))
        Text(
            text = option.text,
            color = if (isSelected) Color.White else TextPrimary,
            fontSize = 14.sp,
            fontWeight = if (isSelected) FontWeight.SemiBold else FontWeight.Normal
        )
    }
}

@Composable
private fun ResultOptionItem(
    option: PollOption,
    percentage: Float,
    percentageInt: Int,
    isMyVote: Boolean
) {
    val animatedProgress by animateFloatAsState(
        targetValue = percentage,
        animationSpec = tween(durationMillis = 500),
        label = "pollProgress"
    )

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(10.dp))
            .background(SurfaceDark)
            .border(
                width = if (isMyVote) 1.5.dp else 1.dp,
                color = if (isMyVote) PrimaryCyan else BorderDark,
                shape = RoundedCornerShape(10.dp)
            )
    ) {
        // Progress Fill Bar
        Box(
            modifier = Modifier
                .matchParentSize()
                .fillMaxWidth(animatedProgress)
                .clip(RoundedCornerShape(10.dp))
                .background(
                    if (isMyVote) PrimaryCyan.copy(alpha = 0.25f) else SecondaryBlue.copy(alpha = 0.15f)
                )
        )

        // Text & Percentage row
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 14.dp, vertical = 12.dp),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Row(
                modifier = Modifier.weight(1f),
                verticalAlignment = Alignment.CenterVertically
            ) {
                if (isMyVote) {
                    Icon(
                        Icons.Outlined.CheckCircle,
                        contentDescription = "Your vote",
                        tint = PrimaryCyan,
                        modifier = Modifier.size(16.dp)
                    )
                    Spacer(modifier = Modifier.width(6.dp))
                }
                Text(
                    text = option.text,
                    color = Color.White,
                    fontSize = 14.sp,
                    fontWeight = if (isMyVote) FontWeight.SemiBold else FontWeight.Normal
                )
            }

            Spacer(modifier = Modifier.width(8.dp))

            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = "$percentageInt%",
                    color = if (isMyVote) PrimaryCyan else TextPrimary,
                    fontSize = 13.sp,
                    fontWeight = FontWeight.Bold
                )
                Spacer(modifier = Modifier.width(6.dp))
                Text(
                    text = "(${option.voteCount})",
                    color = TextMuted,
                    fontSize = 12.sp
                )
            }
        }
    }
}

@Composable
private fun CreatePollView(
    onCreatePoll: (question: String, options: List<String>) -> Unit
) {
    var questionText by remember { mutableStateOf("") }
    var options by remember { mutableStateOf(listOf("", "")) }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(bottom = 16.dp),
        verticalArrangement = Arrangement.SpaceBetween
    ) {
        LazyColumn(
            modifier = Modifier
                .weight(1f)
                .fillMaxWidth(),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            item {
                Text(
                    text = "Poll Question",
                    color = TextPrimary,
                    fontSize = 13.sp,
                    fontWeight = FontWeight.SemiBold
                )
                Spacer(modifier = Modifier.height(6.dp))
                OutlinedTextField(
                    value = questionText,
                    onValueChange = { questionText = it },
                    placeholder = { Text("What would you like to ask the room?", color = TextMuted, fontSize = 13.sp) },
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(12.dp),
                    colors = OutlinedTextFieldDefaults.colors(
                        focusedBorderColor = PrimaryCyan,
                        unfocusedBorderColor = BorderDark,
                        focusedContainerColor = SurfaceVariantDark,
                        unfocusedContainerColor = SurfaceVariantDark
                    ),
                    singleLine = false,
                    maxLines = 3
                )
            }

            item {
                Text(
                    text = "Options",
                    color = TextPrimary,
                    fontSize = 13.sp,
                    fontWeight = FontWeight.SemiBold
                )
            }

            items(options.size) { index ->
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    OutlinedTextField(
                        value = options[index],
                        onValueChange = { newText ->
                            options = options.toMutableList().also { it[index] = newText }
                        },
                        placeholder = { Text("Option ${index + 1}", color = TextMuted, fontSize = 13.sp) },
                        modifier = Modifier.weight(1f),
                        shape = RoundedCornerShape(10.dp),
                        colors = OutlinedTextFieldDefaults.colors(
                            focusedBorderColor = PrimaryCyan,
                            unfocusedBorderColor = BorderDark,
                            focusedContainerColor = SurfaceVariantDark,
                            unfocusedContainerColor = SurfaceVariantDark
                        ),
                        singleLine = true
                    )

                    if (options.size > 2) {
                        Spacer(modifier = Modifier.width(6.dp))
                        IconButton(
                            onClick = {
                                options = options.filterIndexed { i, _ -> i != index }
                            }
                        ) {
                            Icon(Icons.Default.DeleteOutline, contentDescription = "Remove option", tint = AlertRed)
                        }
                    }
                }
            }

            if (options.size < 6) {
                item {
                    TextButton(
                        onClick = { options = options + "" },
                        colors = ButtonDefaults.textButtonColors(contentColor = PrimaryCyan)
                    ) {
                        Icon(Icons.Default.AddCircleOutline, contentDescription = null, modifier = Modifier.size(18.dp))
                        Spacer(modifier = Modifier.width(6.dp))
                        Text("+ Add Another Option", fontSize = 13.sp, fontWeight = FontWeight.SemiBold)
                    }
                }
            }
        }

        val isValid = questionText.isNotBlank() && options.count { it.isNotBlank() } >= 2

        Button(
            onClick = {
                val cleanOptions = options.map { it.trim() }.filter { it.isNotBlank() }
                if (questionText.isNotBlank() && cleanOptions.size >= 2) {
                    onCreatePoll(questionText.trim(), cleanOptions)
                }
            },
            enabled = isValid,
            modifier = Modifier
                .fillMaxWidth()
                .height(48.dp),
            shape = RoundedCornerShape(12.dp),
            colors = ButtonDefaults.buttonColors(
                containerColor = PrimaryCyan,
                disabledContainerColor = SurfaceVariantDark
            )
        ) {
            Icon(
                Icons.Default.Send,
                contentDescription = null,
                tint = if (isValid) BackgroundDark else TextMuted,
                modifier = Modifier.size(18.dp)
            )
            Spacer(modifier = Modifier.width(8.dp))
            Text(
                text = "Launch Poll to Room",
                color = if (isValid) BackgroundDark else TextMuted,
                fontWeight = FontWeight.Bold,
                fontSize = 14.sp
            )
        }
    }
}
