package com.confer.mobile.features.lobby

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.confer.mobile.core.network.ApiClient
import com.confer.mobile.core.network.JoinMeetingResponse
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

data class LobbyUiState(
    val serverUrl: String = "http://10.0.2.2:5100",
    val email: String = "participant1@confer.local",
    val displayName: String = "Alice (Mobile)",
    val meetingTitleInput: String = "Mobile Sync",
    val joinCodeInput: String = "",
    val micLevel: Float = 0.4f,
    val isMicMuted: Boolean = false,
    val isCameraOff: Boolean = false,
    val isLoading: Boolean = false,
    val errorMessage: String? = null,
    val joinSuccess: JoinMeetingResponse? = null
)

class LobbyViewModel : ViewModel() {
    private val _uiState = MutableStateFlow(LobbyUiState())
    val uiState = _uiState.asStateFlow()

    fun updateServerUrl(url: String) = _uiState.update { it.copy(serverUrl = url) }
    fun updateEmail(email: String) = _uiState.update { it.copy(email = email) }
    fun updateDisplayName(name: String) = _uiState.update { it.copy(displayName = name) }
    fun updateMeetingTitle(title: String) = _uiState.update { it.copy(meetingTitleInput = title) }
    fun updateJoinCode(code: String) = _uiState.update { it.copy(joinCodeInput = code) }
    fun toggleMic() = _uiState.update { it.copy(isMicMuted = !it.isMicMuted) }
    fun toggleCamera() = _uiState.update { it.copy(isCameraOff = !it.isCameraOff) }

    fun selectPresetAccount(email: String, name: String) {
        _uiState.update { it.copy(email = email, displayName = name) }
    }

    fun createMeeting() {
        val state = _uiState.value
        _uiState.update { it.copy(isLoading = true, errorMessage = null) }

        viewModelScope.launch {
            val api = ApiClient(state.serverUrl)
            val loginRes = api.devLogin(state.email, state.displayName)
            loginRes.fold(
                onSuccess = { login ->
                    val createRes = api.createMeeting(state.meetingTitleInput, login.userId)
                    createRes.fold(
                        onSuccess = { created ->
                            val joinRes = api.joinMeeting(created.id, login.userId, state.displayName)
                            joinRes.fold(
                                onSuccess = { joinData ->
                                    _uiState.update { it.copy(isLoading = false, joinSuccess = joinData) }
                                },
                                onFailure = { e ->
                                    _uiState.update { it.copy(isLoading = false, errorMessage = "Join failed: ${e.message}") }
                                }
                            )
                        },
                        onFailure = { e ->
                            _uiState.update { it.copy(isLoading = false, errorMessage = "Create failed: ${e.message}") }
                        }
                    )
                },
                onFailure = { e ->
                    _uiState.update { it.copy(isLoading = false, errorMessage = "Login failed: ${e.message}") }
                }
            )
        }
    }

    fun joinByCode() {
        val state = _uiState.value
        if (state.joinCodeInput.isBlank()) {
            _uiState.update { it.copy(errorMessage = "Please enter a join code") }
            return
        }

        _uiState.update { it.copy(isLoading = true, errorMessage = null) }

        viewModelScope.launch {
            val api = ApiClient(state.serverUrl)
            val loginRes = api.devLogin(state.email, state.displayName)
            loginRes.fold(
                onSuccess = { login ->
                    val meetingInfoRes = api.getMeetingByCode(state.joinCodeInput.trim())
                    meetingInfoRes.fold(
                        onSuccess = { meetingInfo ->
                            val joinRes = api.joinMeeting(meetingInfo.id, login.userId, state.displayName)
                            joinRes.fold(
                                onSuccess = { joinData ->
                                    _uiState.update { it.copy(isLoading = false, joinSuccess = joinData) }
                                },
                                onFailure = { e ->
                                    _uiState.update { it.copy(isLoading = false, errorMessage = "Join failed: ${e.message}") }
                                }
                            )
                        },
                        onFailure = { e ->
                            _uiState.update { it.copy(isLoading = false, errorMessage = "Meeting not found: ${e.message}") }
                        }
                    )
                },
                onFailure = { e ->
                    _uiState.update { it.copy(isLoading = false, errorMessage = "Login failed: ${e.message}") }
                }
            )
        }
    }

    fun resetJoinSuccess() {
        _uiState.update { it.copy(joinSuccess = null) }
    }
}
