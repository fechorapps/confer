//! WebRTC engine for the Confer desktop client.
//!
//! This module manages publish and subscribe peer connections for SFU media transport.

use std::fmt;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::sdk::client::IceServerConfig;
use crate::sdk::protocol::{TrackIntent, TrackMapping};

/// Events produced by the WebRTC engine that must be handled by the application.
pub enum RtcEvent {
    /// A local ICE candidate has been gathered and must be sent to the server.
    IceCandidate {
        target: CandidateTarget,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_mline_index: Option<u16>,
        username_fragment: Option<String>,
    },
    /// A new remote track has arrived on the subscribe peer connection.
    RemoteTrack {
        publisher_id: Uuid,
        kind: String,
    },
}

impl fmt::Debug for RtcEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IceCandidate { target, candidate, sdp_mid, sdp_mline_index, username_fragment } => {
                f.debug_struct("IceCandidate")
                    .field("target", target)
                    .field("candidate", candidate)
                    .field("sdp_mid", sdp_mid)
                    .field("sdp_mline_index", sdp_mline_index)
                    .field("username_fragment", username_fragment)
                    .finish()
            }
            Self::RemoteTrack { publisher_id, kind } => {
                f.debug_struct("RemoteTrack")
                    .field("publisher_id", publisher_id)
                    .field("kind", kind)
                    .finish()
            }
        }
    }
}

/// Identifies which of the two peer connections an ICE candidate belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateTarget {
    Publish,
    Subscribe,
}

impl fmt::Display for CandidateTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CandidateTarget::Publish => write!(f, "publish"),
            CandidateTarget::Subscribe => write!(f, "subscribe"),
        }
    }
}

impl TryFrom<&str> for CandidateTarget {
    type Error = RtcError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "publish" => Ok(CandidateTarget::Publish),
            "subscribe" => Ok(CandidateTarget::Subscribe),
            other => Err(RtcError::InvalidCandidateTarget(other.to_string())),
        }
    }
}

/// Errors that can occur inside the WebRTC engine.
#[derive(Debug, thiserror::Error)]
pub enum RtcError {
    #[error("Invalid candidate target: {0}")]
    InvalidCandidateTarget(String),

    #[error("Subscribe offer mapping does not match received remote tracks")]
    TrackMappingExhausted,
}

/// The Confer WebRTC engine placeholder that bridges signaling events.
pub struct RtcEngine {
    event_tx: mpsc::Sender<RtcEvent>,
}

impl RtcEngine {
    /// Creates the publish and subscribe peer connections.
    pub async fn new(
        _ice_servers: Vec<IceServerConfig>,
        event_tx: mpsc::Sender<RtcEvent>,
    ) -> Result<Self, RtcError> {
        Ok(Self { event_tx })
    }

    /// Returns a clone of the event sender.
    pub fn event_tx(&self) -> mpsc::Sender<RtcEvent> {
        self.event_tx.clone()
    }

    /// Creates publish offer SDP.
    pub async fn create_publish_offer(&self) -> Result<(String, Vec<TrackIntent>), RtcError> {
        let intents = vec![
            TrackIntent {
                track_id: "cam-audio".to_string(),
                kind: "audio".to_string(),
                simulcast: false,
            },
            TrackIntent {
                track_id: "cam-video".to_string(),
                kind: "video".to_string(),
                simulcast: false,
            },
        ];

        Ok(("v=0\r\no=- 0 0 IN IP4 127.0.0.1\r\ns=confer\r\nt=0 0\r\n".to_string(), intents))
    }

    /// Applies publish answer SDP.
    pub async fn apply_publish_answer(&self, _sdp: String) -> Result<(), RtcError> {
        Ok(())
    }

    /// Applies subscribe offer SDP.
    pub async fn apply_subscribe_offer(
        &self,
        _sdp: String,
        _track_mapping: Vec<TrackMapping>,
    ) -> Result<String, RtcError> {
        Ok("v=0\r\no=- 0 0 IN IP4 127.0.0.1\r\ns=confer\r\nt=0 0\r\n".to_string())
    }

    /// Adds remote ICE candidate.
    pub async fn add_ice_candidate(
        &self,
        _target: CandidateTarget,
        _candidate: String,
        _sdp_mid: Option<String>,
        _sdp_mline_index: Option<u16>,
        _username_fragment: Option<String>,
    ) -> Result<(), RtcError> {
        Ok(())
    }

    /// Gracefully closes engine.
    pub async fn close(&self) -> Result<(), RtcError> {
        Ok(())
    }
}
