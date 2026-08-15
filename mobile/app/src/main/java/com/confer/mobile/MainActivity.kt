package com.confer.mobile

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.*
import androidx.lifecycle.viewmodel.compose.viewModel
import com.confer.mobile.core.network.JoinMeetingResponse
import com.confer.mobile.core.theme.ConferTheme
import com.confer.mobile.features.lobby.LobbyScreen
import com.confer.mobile.features.lobby.LobbyViewModel
import com.confer.mobile.features.meeting.MeetingScreen
import com.confer.mobile.features.meeting.MeetingViewModel

enum class MobileScreen {
    Lobby,
    Meeting
}

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()

        setContent {
            ConferTheme {
                var currentScreen by remember { mutableStateOf(MobileScreen.Lobby) }
                val lobbyViewModel: LobbyViewModel = viewModel()
                val meetingViewModel: MeetingViewModel = viewModel()

                when (currentScreen) {
                    MobileScreen.Lobby -> {
                        LobbyScreen(
                            viewModel = lobbyViewModel,
                            onJoinSuccess = { joinData, serverUrl ->
                                meetingViewModel.initialize(
                                    joinData = joinData,
                                    serverUrl = serverUrl,
                                    displayName = lobbyViewModel.uiState.value.displayName
                                )
                                currentScreen = MobileScreen.Meeting
                            }
                        )
                    }
                    MobileScreen.Meeting -> {
                        MeetingScreen(
                            viewModel = meetingViewModel,
                            onLeaveMeeting = {
                                currentScreen = MobileScreen.Lobby
                            }
                        )
                    }
                }
            }
        }
    }
}
