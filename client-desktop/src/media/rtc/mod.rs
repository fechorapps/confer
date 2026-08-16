//! WebRTC engine for the Confer desktop client (Step 2).
//!
//! This module owns the two peer connections required by the SFU:
//!   - **publish**: the client offers, the server answers with `publish_ok`.
//!   - **subscribe**: the server offers with `subscribe_offer`, the client answers.
//!
//! It does *not* implement real audio/video capture or encoding yet (Steps 3-4);
//! it only wires up the PeerConnection machinery, SDP negotiation and trickle ICE.

use std::fmt;
use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

use crate::sdk::client::IceServerConfig;
use crate::sdk::protocol::{TrackIntent, TrackMapping};

use rtc::peer_connection::configuration::media_engine::MediaEngine;
use rtc::rtp_transceiver::rtp_sender::{RtpCodecKind, RTCRtpEncodingParameters};
use webrtc::error::Error as WebRtcError;
use webrtc::media_stream::track_remote::TrackRemote;
use webrtc::peer_connection::{
    PeerConnection, PeerConnectionBuilder, PeerConnectionEventHandler, RTCConfigurationBuilder,
    RTCIceCandidateInit, RTCIceServer, RTCPeerConnectionIceEvent, RTCSessionDescription,
};
use webrtc::rtp_transceiver::{RTCRtpTransceiverDirection, RTCRtpTransceiverInit};

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
        track: Arc<dyn TrackRemote>,
    },
}

impl fmt::Debug for RtcEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RtcEvent::IceCandidate {
                target,
                candidate,
                sdp_mid,
                sdp_mline_index,
                username_fragment,
            } => f
                .debug_struct("IceCandidate")
                .field("target", target)
                .field("candidate", candidate)
                .field("sdp_mid", sdp_mid)
                .field("sdp_mline_index", sdp_mline_index)
                .field("username_fragment", username_fragment)
                .finish(),
            RtcEvent::RemoteTrack { publisher_id, kind, .. } => f
                .debug_struct("RemoteTrack")
                .field("publisher_id", publisher_id)
                .field("kind", kind)
                .field("track", &"<Arc<dyn TrackRemote>>")
                .finish(),
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
    #[error("WebRTC internal error: {0}")]
    WebRtc(#[from] WebRtcError),

    #[error("ICE server configuration missing URLs")]
    IceServerMissingUrls,

    #[error("Invalid candidate target: {0}")]
    InvalidCandidateTarget(String),

    #[error("Subscribe offer mapping does not match received remote tracks")]
    TrackMappingExhausted,
}

/// Internal handler that forwards WebRTC callbacks into the engine's event channel.
#[derive(Clone)]
struct RtcPeerHandler {
    target: CandidateTarget,
    event_tx: mpsc::Sender<RtcEvent>,
    subscribe_mapping: Arc<Mutex<Vec<TrackMapping>>>,
}

#[async_trait::async_trait]
impl PeerConnectionEventHandler for RtcPeerHandler {
    async fn on_ice_candidate(&self, event: RTCPeerConnectionIceEvent) {
        // The webrtc-rs 0.20 event only gives us the parsed candidate; convert it
        // back to the SDP attribute string.  sdp_mid / sdp_mline_index are not
        // filled in by this version of the crate, so we leave them as None and
        // rely on the server accepting a bare candidate string together with the
        // `target` field.
        let init = match event.candidate.to_json() {
            Ok(init) => init,
            Err(e) => {
                tracing::warn!("failed to serialize local ICE candidate: {e}");
                return;
            }
        };

        if init.candidate.is_empty() {
            // Ignore end-of-candidates markers for now.
            return;
        }

        let _ = self
            .event_tx
            .send(RtcEvent::IceCandidate {
                target: self.target,
                candidate: init.candidate,
                sdp_mid: None,
                sdp_mline_index: None,
                username_fragment: init.username_fragment,
            })
            .await;
    }

    async fn on_track(&self, track: Arc<dyn TrackRemote>) {
        let kind = match track.kind().await {
            RtpCodecKind::Audio => "audio",
            RtpCodecKind::Video => "video",
            _ => "unknown",
        }
        .to_string();

        let publisher_id = {
            let mut mapping = self.subscribe_mapping.lock().await;
            if mapping.is_empty() {
                tracing::warn!(
                    "remote track arrived but subscribe offer mapping is exhausted"
                );
                Uuid::nil()
            } else {
                mapping.remove(0).publisher_id
            }
        };

        let _ = self
            .event_tx
            .send(RtcEvent::RemoteTrack {
                publisher_id,
                kind,
                track,
            })
            .await;
    }
}

/// The Confer WebRTC engine.
pub struct RtcEngine {
    publish_pc: Arc<dyn PeerConnection>,
    subscribe_pc: Arc<dyn PeerConnection>,
    event_tx: mpsc::Sender<RtcEvent>,
    /// FIFO of mappings from the last `subscribe_offer`, in m-line order.
    subscribe_mapping: Arc<Mutex<Vec<TrackMapping>>>,
    /// Serializes `createAnswer` calls on the subscribe peer connection.
    subscribe_answer_lock: Arc<Mutex<()>>,
    /// Candidates received before the remote description was set.
    pending_publish_candidates: Arc<Mutex<Vec<RTCIceCandidateInit>>>,
    pending_subscribe_candidates: Arc<Mutex<Vec<RTCIceCandidateInit>>>,
}

impl RtcEngine {
    /// Creates the publish and subscribe peer connections.
    pub async fn new(
        ice_servers: Vec<IceServerConfig>,
        event_tx: mpsc::Sender<RtcEvent>,
    ) -> Result<Self, RtcError> {
        let rtc_ice_servers: Vec<RTCIceServer> = ice_servers
            .into_iter()
            .map(|cfg| RTCIceServer {
                urls: cfg.urls,
                username: cfg.username.unwrap_or_default(),
                credential: cfg.credential.unwrap_or_default(),
            })
            .collect();

        let configuration = RTCConfigurationBuilder::default()
            .with_ice_servers(rtc_ice_servers)
            .build();

        let mut media_engine = MediaEngine::default();
        media_engine.register_default_codecs()?;

        // TODO: Register the ssrc-audio-level RTP header extension once the
        // public webrtc-rs API exposes `RTCRtpHeaderExtensionCapability` without
        // reaching into the internal `rtc` crate.  Until then the SFU will not
        // include this client in active-speaker calculations.
        //
        // media_engine.register_header_extension(
        //     RTCRtpHeaderExtensionCapability {
        //         uri: "urn:ietf:params:rtp-hdrext:ssrc-audio-level".to_string(),
        //     },
        //     RtpCodecKind::Audio,
        //     Some(RTCRtpTransceiverDirection::Sendrecv),
        // )?;

        let subscribe_mapping = Arc::new(Mutex::new(Vec::new()));
        let pending_publish_candidates: Arc<Mutex<Vec<RTCIceCandidateInit>>> =
            Arc::new(Mutex::new(Vec::new()));
        let pending_subscribe_candidates: Arc<Mutex<Vec<RTCIceCandidateInit>>> =
            Arc::new(Mutex::new(Vec::new()));

        let publish_handler = RtcPeerHandler {
            target: CandidateTarget::Publish,
            event_tx: event_tx.clone(),
            subscribe_mapping: Arc::clone(&subscribe_mapping),
        };

        let subscribe_handler = RtcPeerHandler {
            target: CandidateTarget::Subscribe,
            event_tx: event_tx.clone(),
            subscribe_mapping: Arc::clone(&subscribe_mapping),
        };

        let publish_pc: Arc<dyn PeerConnection> = Arc::new(
            PeerConnectionBuilder::new()
                .with_configuration(configuration.clone())
                .with_media_engine(media_engine.clone())
                .with_handler(Arc::new(publish_handler))
                .with_udp_addrs(vec!["0.0.0.0:0"])
                .build()
                .await?,
        );

        let subscribe_pc: Arc<dyn PeerConnection> = Arc::new(
            PeerConnectionBuilder::new()
                .with_configuration(configuration)
                .with_media_engine(media_engine)
                .with_handler(Arc::new(subscribe_handler))
                .with_udp_addrs(vec!["0.0.0.0:0"])
                .build()
                .await?,
        );

        Ok(Self {
            publish_pc,
            subscribe_pc,
            event_tx,
            subscribe_mapping,
            subscribe_answer_lock: Arc::new(Mutex::new(())),
            pending_publish_candidates,
            pending_subscribe_candidates,
        })
    }

    /// Returns a clone of the event sender (useful for tests).
    pub fn event_tx(&self) -> mpsc::Sender<RtcEvent> {
        self.event_tx.clone()
    }

    /// Ensures audio and video transceivers exist on the publish peer connection,
    /// creates an offer, sets it as local description and returns it.
    pub async fn create_publish_offer(&self) -> Result<(String, Vec<TrackIntent>), RtcError> {
        let audio_init = RTCRtpTransceiverInit {
            direction: RTCRtpTransceiverDirection::Sendrecv,
            send_encodings: vec![RTCRtpEncodingParameters::default()],
            ..Default::default()
        };
        self.publish_pc
            .add_transceiver_from_kind(RtpCodecKind::Audio, Some(audio_init))
            .await?;

        let video_init = RTCRtpTransceiverInit {
            direction: RTCRtpTransceiverDirection::Sendrecv,
            send_encodings: vec![RTCRtpEncodingParameters::default()],
            ..Default::default()
        };
        self.publish_pc
            .add_transceiver_from_kind(RtpCodecKind::Video, Some(video_init))
            .await?;

        let offer = self.publish_pc.create_offer(None).await?;
        self.publish_pc.set_local_description(offer.clone()).await?;

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

        Ok((offer.sdp, intents))
    }

    /// Applies the server's answer to the publish peer connection.
    pub async fn apply_publish_answer(&self, sdp: String) -> Result<(), RtcError> {
        let answer = RTCSessionDescription::answer(sdp)?;
        self.publish_pc.set_remote_description(answer).await?;
        Self::drain_pending_candidates(&self.publish_pc, &self.pending_publish_candidates).await?;
        Ok(())
    }

    /// Applies a subscribe offer, creates an answer under a serialization lock and
    /// returns the SDP answer to be sent back to the server.
    pub async fn apply_subscribe_offer(
        &self,
        sdp: String,
        mapping: Vec<TrackMapping>,
    ) -> Result<String, RtcError> {
        {
            let mut current_mapping = self.subscribe_mapping.lock().await;
            *current_mapping = mapping;
        }

        let _guard = self.subscribe_answer_lock.lock().await;

        let offer = RTCSessionDescription::offer(sdp)?;
        self.subscribe_pc.set_remote_description(offer).await?;

        let answer = self.subscribe_pc.create_answer(None).await?;
        self.subscribe_pc.set_local_description(answer.clone()).await?;

        Self::drain_pending_candidates(&self.subscribe_pc, &self.pending_subscribe_candidates)
            .await?;

        Ok(answer.sdp)
    }

    /// Adds a remote ICE candidate to the appropriate peer connection.
    /// If the remote description has not been set yet, the candidate is queued
    /// and applied once the description arrives.
    pub async fn add_ice_candidate(
        &self,
        target: CandidateTarget,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_mline_index: Option<u16>,
        username_fragment: Option<String>,
    ) -> Result<(), RtcError> {
        let init = RTCIceCandidateInit {
            candidate,
            sdp_mid,
            sdp_mline_index,
            username_fragment,
            url: None,
        };

        match target {
            CandidateTarget::Publish => {
                if self.publish_pc.current_remote_description().await.is_some() {
                    self.publish_pc.add_ice_candidate(init).await?;
                } else {
                    self.pending_publish_candidates.lock().await.push(init);
                }
            }
            CandidateTarget::Subscribe => {
                if self.subscribe_pc.current_remote_description().await.is_some() {
                    self.subscribe_pc.add_ice_candidate(init).await?;
                } else {
                    self.pending_subscribe_candidates.lock().await.push(init);
                }
            }
        }

        Ok(())
    }

    /// Closes both peer connections.
    pub async fn close(&self) -> Result<(), RtcError> {
        self.publish_pc.close().await?;
        self.subscribe_pc.close().await?;
        Ok(())
    }

    async fn drain_pending_candidates(
        pc: &Arc<dyn PeerConnection>,
        pending: &Arc<Mutex<Vec<RTCIceCandidateInit>>>,
    ) -> Result<(), RtcError> {
        let candidates = {
            let mut guard = pending.lock().await;
            std::mem::take(&mut *guard)
        };
        for candidate in candidates {
            pc.add_ice_candidate(candidate).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create_publish_offer_includes_vp8_and_opus() {
        let (tx, _rx) = mpsc::channel(16);
        let engine = RtcEngine::new(Vec::new(), tx).await.expect("engine creation");

        let (sdp, intents) = engine
            .create_publish_offer()
            .await
            .expect("create_publish_offer");

        assert!(
            sdp.contains("VP8"),
            "publish offer should advertise VP8 video codec"
        );
        assert!(
            sdp.to_ascii_lowercase().contains("opus"),
            "publish offer should advertise opus audio codec"
        );

        assert_eq!(intents.len(), 2);
        assert_eq!(intents[0].kind, "audio");
        assert_eq!(intents[1].kind, "video");

        // Cleanly close the engine so the test does not leak driver tasks.
        engine.close().await.expect("close engine");
    }
}
