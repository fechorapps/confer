package com.confer.mobile.core.network

import io.ktor.client.*
import io.ktor.client.call.*
import io.ktor.client.engine.okhttp.*
import io.ktor.client.plugins.contentnegotiation.*
import io.ktor.client.request.*
import io.ktor.http.*
import io.ktor.serialization.kotlinx.json.*
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

@Serializable
data class DevLoginRequest(val email: String, val displayName: String)

@Serializable
data class DevLoginResponse(val userId: String, val email: String, val displayName: String, val token: String)

@Serializable
data class CreateMeetingRequest(val title: String, val ownerId: String, val maxParticipants: Int = 50)

@Serializable
data class CreateMeetingResponse(val id: String, val joinCode: String, val title: String, val ownerId: String)

@Serializable
data class MeetingInfoResponse(val id: String, val joinCode: String, val title: String, val ownerId: String, val isLocked: Boolean, val hasEnded: Boolean)

@Serializable
data class JoinMeetingRequest(val userId: String, val displayName: String, val clientInfo: String? = null)

@Serializable
data class JoinMeetingResponse(
    val meetingId: String,
    val title: String,
    val participantId: String,
    val role: String,
    val isLocked: Boolean,
    val wsUrl: String,
    val roomToken: String
)

class ApiClient(private val baseUrl: String) {
    private val httpClient = HttpClient(OkHttp) {
        install(ContentNegotiation) {
            json(Json {
                ignoreUnknownKeys = true
                isLenient = true
            })
        }
    }

    suspend fun devLogin(email: String, displayName: String): Result<DevLoginResponse> = runCatching {
        httpClient.post("$baseUrl/api/auth/dev-login") {
            contentType(ContentType.Application.Json)
            setBody(DevLoginRequest(email, displayName))
        }.body()
    }

    suspend fun createMeeting(title: String, ownerId: String): Result<CreateMeetingResponse> = runCatching {
        httpClient.post("$baseUrl/api/meetings") {
            contentType(ContentType.Application.Json)
            setBody(CreateMeetingRequest(title, ownerId))
        }.body()
    }

    suspend fun getMeetingByCode(code: String): Result<MeetingInfoResponse> = runCatching {
        httpClient.get("$baseUrl/api/meetings/$code").body()
    }

    suspend fun joinMeeting(meetingId: String, userId: String, displayName: String): Result<JoinMeetingResponse> = runCatching {
        httpClient.post("$baseUrl/api/meetings/$meetingId/join") {
            contentType(ContentType.Application.Json)
            setBody(JoinMeetingRequest(userId, displayName, clientInfo = "Confer Android (Kotlin)"))
        }.body()
    }
}
