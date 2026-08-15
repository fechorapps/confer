use thiserror::Error;

#[derive(Debug, Error)]
pub enum SdkError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Protocol JSON serialization/deserialization failed: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Server returned status {status}: {message}")]
    Server {
        status: reqwest::StatusCode,
        message: String,
    },

    #[error("Invalid request configuration: {0}")]
    InvalidRequest(String),

    #[allow(dead_code)]
    #[error("Signaling connection closed")]
    ConnectionClosed,
}
