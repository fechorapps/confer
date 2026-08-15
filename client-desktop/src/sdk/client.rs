use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use futures_util::{SinkExt, StreamExt};
use reqwest::Client as HttpClient;
use tokio::sync::mpsc;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::Message;
use uuid::Uuid;

use crate::sdk::error::SdkError;
use crate::sdk::protocol::{ClientMessage, ServerMessage};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DevLoginResponse {
    #[serde(rename = "userId")]
    pub user_id: Uuid,
    pub email: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub token: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreateMeetingResponse {
    pub id: Uuid,
    #[serde(rename = "joinCode")]
    pub join_code: String,
    pub title: String,
    #[serde(rename = "ownerId")]
    pub owner_id: Uuid,
    #[serde(rename = "maxParticipants")]
    pub max_participants: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MeetingInfoResponse {
    pub id: Uuid,
    #[serde(rename = "joinCode")]
    pub join_code: String,
    pub title: String,
    #[serde(rename = "ownerId")]
    pub owner_id: Uuid,
    #[serde(rename = "maxParticipants")]
    pub max_participants: i32,
    #[serde(rename = "isLocked")]
    pub is_locked: bool,
    #[serde(rename = "hasEnded")]
    pub has_ended: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JoinMeetingResponse {
    #[serde(rename = "meetingId")]
    pub meeting_id: Uuid,
    pub title: String,
    #[serde(rename = "participantId")]
    pub participant_id: Uuid,
    pub role: String,
    #[serde(rename = "isLocked")]
    pub is_locked: bool,
    #[serde(rename = "wsUrl")]
    pub ws_url: String,
    #[serde(rename = "roomToken")]
    pub room_token: String,
}

pub struct ConferClient {
    pub server_url: String,
    pub http_client: HttpClient,
    pub outgoing_tx: Option<mpsc::UnboundedSender<ClientMessage>>,
    pub incoming_rx: Option<mpsc::UnboundedReceiver<ServerMessage>>,
    pub is_connected: Arc<AtomicBool>,
}

impl ConferClient {
    pub fn new(server_url: &str) -> Self {
        Self {
            server_url: server_url.trim_end_matches('/').to_string(),
            http_client: HttpClient::new(),
            outgoing_tx: None,
            incoming_rx: None,
            is_connected: Arc::new(AtomicBool::new(false)),
        }
    }

    pub async fn dev_login(&self, email: &str, display_name: &str) -> Result<DevLoginResponse, SdkError> {
        let url = format!("{}/api/auth/dev-login", self.server_url);
        let resp = self.http_client
            .post(&url)
            .json(&serde_json::json!({
                "email": email,
                "displayName": display_name
            }))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(SdkError::Server {
                status: resp.status(),
                message: "Dev login failed".to_string(),
            });
        }

        let body = resp.json::<DevLoginResponse>().await?;
        Ok(body)
    }

    pub async fn create_meeting(&self, title: &str, owner_id: Uuid, max_participants: i32) -> Result<CreateMeetingResponse, SdkError> {
        let url = format!("{}/api/meetings", self.server_url);
        let resp = self.http_client
            .post(&url)
            .json(&serde_json::json!({
                "title": title,
                "ownerId": owner_id,
                "maxParticipants": max_participants
            }))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(SdkError::Server {
                status: resp.status(),
                message: "Create meeting failed".to_string(),
            });
        }

        let body = resp.json::<CreateMeetingResponse>().await?;
        Ok(body)
    }

    pub async fn get_meeting_by_code(&self, code: &str) -> Result<MeetingInfoResponse, SdkError> {
        let url = format!("{}/api/meetings/{code}", self.server_url);
        let resp = self.http_client
            .get(&url)
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(SdkError::Server {
                status: resp.status(),
                message: "Meeting not found".to_string(),
            });
        }

        let body = resp.json::<MeetingInfoResponse>().await?;
        Ok(body)
    }

    pub async fn join_meeting(&self, meeting_id: Uuid, user_id: Uuid, display_name: &str) -> Result<JoinMeetingResponse, SdkError> {
        let url = format!("{}/api/meetings/{meeting_id}/join", self.server_url);
        let resp = self.http_client
            .post(&url)
            .json(&serde_json::json!({
                "userId": user_id,
                "displayName": display_name,
                "clientInfo": "Confer Desktop Linux (Rust)"
            }))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(SdkError::Server {
                status: resp.status(),
                message: "Join meeting failed".to_string(),
            });
        }

        let body = resp.json::<JoinMeetingResponse>().await?;
        Ok(body)
    }

    pub async fn connect_signaling(&mut self, room_token: &str) -> Result<(), SdkError> {
        let ws_base = self.server_url.replace("http://", "ws://").replace("https://", "wss://");
        let ws_url = format!("{ws_base}/v1/signal?token={room_token}");

        let mut req = ws_url.into_client_request()
            .map_err(|e| SdkError::InvalidRequest(e.to_string()))?;
        req.headers_mut().insert(
            "Sec-WebSocket-Protocol",
            "confer.v1".parse().map_err(|e| SdkError::InvalidRequest(format!("{e}")))?,
        );

        let (ws_stream, _) = connect_async(req).await?;

        let (mut write, mut read) = ws_stream.split();
        let (out_tx, mut out_rx) = mpsc::unbounded_channel::<ClientMessage>();
        let (in_tx, in_rx) = mpsc::unbounded_channel::<ServerMessage>();

        let is_connected = self.is_connected.clone();
        is_connected.store(true, Ordering::SeqCst);

        // Spawn writer task
        let is_connected_writer = is_connected.clone();
        tokio::spawn(async move {
            while let Some(msg) = out_rx.recv().await {
                if let Ok(json_text) = serde_json::to_string(&msg) {
                    if write.send(Message::Text(json_text.into())).await.is_err() {
                        break;
                    }
                }
            }
            is_connected_writer.store(false, Ordering::SeqCst);
        });

        // Spawn reader task
        let is_connected_reader = is_connected.clone();
        let out_tx_ping = out_tx.clone();
        tokio::spawn(async move {
            let mut ping_interval = tokio::time::interval(Duration::from_secs(5));
            let mut seq = 0u64;

            loop {
                tokio::select! {
                    _ = ping_interval.tick() => {
                        seq += 1;
                        let _ = out_tx_ping.send(ClientMessage::Ping { seq });
                    }
                    msg = read.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                if let Ok(server_msg) = serde_json::from_str::<ServerMessage>(&text) {
                                    let _ = in_tx.send(server_msg);
                                }
                            }
                            Some(Ok(Message::Close(_))) | None | Some(Err(_)) => {
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }
            is_connected_reader.store(false, Ordering::SeqCst);
        });

        self.outgoing_tx = Some(out_tx);
        self.incoming_rx = Some(in_rx);

        Ok(())
    }

    pub fn send_message(&self, message: ClientMessage) {
        if let Some(tx) = &self.outgoing_tx {
            let _ = tx.send(message);
        }
    }
}
