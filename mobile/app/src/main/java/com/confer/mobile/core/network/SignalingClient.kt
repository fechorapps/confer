package com.confer.mobile.core.network

import io.ktor.client.*
import io.ktor.client.engine.okhttp.*
import io.ktor.client.plugins.websocket.*
import io.ktor.websocket.*
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.serialization.json.Json

class SignalingClient(private val serverUrl: String) {
    private val json = Json {
        ignoreUnknownKeys = true
        isLenient = true
        encodeDefaults = true
    }

    private val httpClient = HttpClient(OkHttp) {
        install(WebSockets) {
            pingInterval = 10.seconds
        }
    }

    private val _incomingMessages = MutableSharedFlow<ServerMessage>()
    val incomingMessages = _incomingMessages.asSharedFlow()

    private var session: DefaultClientWebSocketSession? = null
    private var connectionScope: CoroutineScope? = null

    suspend fun connect(roomToken: String) = withContext(Dispatchers.IO) {
        val wsBase = serverUrl
            .replace("http://", "ws://")
            .replace("https://", "wss://")
            .trimEnd('/')

        val wsUrl = "$wsBase/v1/signal?token=$roomToken"

        connectionScope = CoroutineScope(Dispatchers.IO + SupervisorJob())
        connectionScope?.launch {
            try {
                httpClient.webSocket(urlString = wsUrl) {
                    session = this

                    for (frame in incoming) {
                        if (frame is Frame.Text) {
                            val text = frame.readText()
                            try {
                                val msg = json.decodeFromString<ServerMessage>(text)
                                _incomingMessages.emit(msg)
                            } catch (e: Exception) {
                                // Log / ignore malformed message
                            }
                        }
                    }
                }
            } catch (e: Exception) {
                // Handle disconnection
            }
        }
    }

    suspend fun sendMessage(message: ClientMessage) = withContext(Dispatchers.IO) {
        val text = json.encodeToString(ClientMessage.serializer(), message)
        session?.send(Frame.Text(text))
    }

    fun disconnect() {
        connectionScope?.cancel()
        session = null
    }
}
