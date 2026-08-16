//! Real-time video pipeline (Step 4/5).
//!
//! Captures camera / screen-share frames, converts them to I420, encodes VP8
//! through the libvpx wrapper and sends them on the WebRTC publish track.
//! Remote VP8 RTP packets received on the subscribe track are depacketized,
//! decoded and converted to RGBA `egui::ColorImage` for rendering.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use egui::ColorImage;
use tokio::sync::mpsc;
use tracing;
use uuid::Uuid;
use webrtc::media_stream::track_local::static_sample::TrackLocalStaticSample;
use webrtc::media_stream::track_remote::{TrackRemote, TrackRemoteEvent};

use rtc::media::Sample;

use crate::media::rtp::{RtpHeader, RtpPacket, Vp8FrameAssembler};
use crate::media::vpx_ffi::{I420Frame, Vp8Decoder, Vp8Encoder, Vp8Packet};

const VIDEO_SSRC: u32 = 2222222222;
// TODO: negotiate the VP8 payload type from the SDP answer instead of
// hard-coding the common dynamic value 96.
const VP8_PAYLOAD_TYPE: u8 = 96;

/// Errors that can occur while building or running the video pipeline.
#[derive(Debug, thiserror::Error)]
pub enum VideoError {
    #[error("vpx error: {0}")]
    Vpx(#[from] crate::media::vpx_ffi::VpxError),

    #[error("webrtc error: {0}")]
    WebRtc(#[from] webrtc::error::Error),

    #[error("rtp error: {0}")]
    Rtp(#[from] crate::media::rtp::RtpError),

    #[error("invalid frame dimensions")]
    InvalidDimensions,
}

/// A decoded remote video frame ready for rendering.
pub type RemoteFrame = (Uuid, ColorImage);

/// Local camera / screen-share encoder pipeline.
pub struct VideoEncoder {
    /// Tells the encoder task to stop.
    shutdown: Arc<AtomicBool>,
}

impl VideoEncoder {
    /// Starts an encoder task that consumes RGB frames from `frame_rx`, encodes
    /// them as VP8 and writes them to the supplied WebRTC video track.
    ///
    /// Frames must be provided as raw RGB bytes (`width × height × 3`).
    pub fn start(
        width: u32,
        height: u32,
        fps: u32,
        bitrate_kbps: u32,
        video_track: Arc<TrackLocalStaticSample>,
        mut frame_rx: mpsc::Receiver<Vec<u8>>,
    ) -> Result<Self, VideoError> {
        if width == 0 || height == 0 || fps == 0 {
            return Err(VideoError::InvalidDimensions);
        }

        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_for_task = Arc::clone(&shutdown);

        tokio::spawn(async move {
            let mut encoder = match Vp8Encoder::new(width, height, bitrate_kbps, fps, 1) {
                Ok(enc) => enc,
                Err(e) => {
                    tracing::warn!("failed to create VP8 encoder: {e}");
                    return;
                }
            };

            let mut yuv_ctx = zenyuv::YuvContext::new(zenyuv::Range::Full, zenyuv::Matrix::Bt601);

            while let Some(rgb) = frame_rx.recv().await {
                if shutdown_for_task.load(Ordering::Relaxed) {
                    break;
                }

                if rgb.len() < (width * height * 3) as usize {
                    tracing::debug!("dropping undersized video frame");
                    continue;
                }

                let mut i420 = I420Frame::new(width, height);
                yuv_ctx.encode_420_u8(
                    &rgb,
                    &mut i420.y,
                    &mut i420.u,
                    &mut i420.v,
                    width as usize,
                    height as usize,
                );

                match encoder.encode(&i420) {
                    Ok(packets) => {
                        for Vp8Packet { data, .. } in packets {
                            let sample = Sample {
                                data: data.into(),
                                duration: Duration::from_secs_f64(1.0 / f64::from(fps)),
                                ..Default::default()
                            };
                            if let Err(e) = video_track
                                .write_sample(VIDEO_SSRC, VP8_PAYLOAD_TYPE, &sample, &[])
                                .await
                            {
                                tracing::warn!("failed to write video sample: {e}");
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!("vp8 encode error: {e}");
                    }
                }
            }
        });

        Ok(Self { shutdown })
    }

    /// Requests a graceful shutdown of the encoder task.
    pub fn stop(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

impl Drop for VideoEncoder {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Remote video decoder pipeline.
pub struct VideoDecoder {
    /// Tells the decoder task to stop.
    shutdown: Arc<AtomicBool>,
}

impl VideoDecoder {
    /// Starts a decoder task that reads RTP packets from `track`, decodes the
    /// VP8 payload and sends RGBA `ColorImage`s on `frame_tx`.
    pub fn start(
        track: Arc<dyn TrackRemote>,
        frame_tx: mpsc::UnboundedSender<RemoteFrame>,
        publisher_id: Uuid,
    ) -> Result<Self, VideoError> {
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_for_task = Arc::clone(&shutdown);

        tokio::spawn(async move {
            let mut decoder = match Vp8Decoder::new() {
                Ok(dec) => dec,
                Err(e) => {
                    tracing::warn!("failed to create VP8 decoder: {e}");
                    return;
                }
            };
            let mut assembler = Vp8FrameAssembler::new();

            loop {
                if shutdown_for_task.load(Ordering::Relaxed) {
                    break;
                }

                match track.poll().await {
                    Some(TrackRemoteEvent::OnRtpPacket(rtp)) => {
                        let packet = RtpPacket {
                            header: RtpHeader {
                                version: rtp.header.version,
                                padding: rtp.header.padding,
                                extension: rtp.header.extension,
                                csrc_count: rtp.header.csrc.len() as u8,
                                marker: rtp.header.marker,
                                payload_type: rtp.header.payload_type,
                                sequence_number: rtp.header.sequence_number,
                                timestamp: rtp.header.timestamp,
                                ssrc: rtp.header.ssrc,
                                csrc_list: rtp.header.csrc.clone(),
                                extension_profile: None,
                                extension_data: None,
                            },
                            payload: rtp.payload.to_vec(),
                            padding_len: 0,
                        };
                        match assembler.push_packet(packet) {
                            Ok(Some(frame)) => {
                                if let Ok(Some(i420)) = decoder.decode(&frame.payload) {
                                    if let Some(rgba) = i420_to_rgba(&i420) {
                                        let image = ColorImage::from_rgba_unmultiplied(
                                            [i420.width as usize, i420.height as usize],
                                            &rgba,
                                        );
                                        if frame_tx.send((publisher_id, image)).is_err() {
                                            break;
                                        }
                                    }
                                }
                            }
                            Ok(None) => {}
                            Err(e) => {
                                tracing::debug!("vp8 assembler reset: {e}");
                                assembler.reset();
                            }
                        }
                    }
                    Some(_) => {}
                    None => break,
                }
            }
        });

        Ok(Self { shutdown })
    }

    /// Requests a graceful shutdown of the decoder task.
    pub fn stop(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

impl Drop for VideoDecoder {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Convert an I420 frame to premultiplied RGBA bytes suitable for `ColorImage`.
fn i420_to_rgba(frame: &I420Frame) -> Option<Vec<u8>> {
    let w = frame.width as usize;
    let h = frame.height as usize;
    let mut rgba = vec![0u8; w * h * 4];

    for y in 0..h {
        for x in 0..w {
            let cy = frame.y[y * frame.stride_y as usize + x] as f32;
            let cu = frame.u[(y / 2) * frame.stride_u as usize + (x / 2)] as f32 - 128.0;
            let cv = frame.v[(y / 2) * frame.stride_v as usize + (x / 2)] as f32 - 128.0;

            let r = (cy + 1.402 * cv).clamp(0.0, 255.0) as u8;
            let g = (cy - 0.344136 * cu - 0.714136 * cv).clamp(0.0, 255.0) as u8;
            let b = (cy + 1.772 * cu).clamp(0.0, 255.0) as u8;

            let idx = (y * w + x) * 4;
            rgba[idx] = r;
            rgba[idx + 1] = g;
            rgba[idx + 2] = b;
            rgba[idx + 3] = 255;
        }
    }

    Some(rgba)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn i420_to_rgba_roundtrip_colors() {
        let mut frame = I420Frame::new(4, 4);
        frame.y.fill(128);
        frame.u.fill(128);
        frame.v.fill(128);

        let rgba = i420_to_rgba(&frame).unwrap();
        // Neutral gray: R == G == B.
        assert_eq!(rgba[0], rgba[1]);
        assert_eq!(rgba[1], rgba[2]);
        assert_eq!(rgba[3], 255);
    }
}
