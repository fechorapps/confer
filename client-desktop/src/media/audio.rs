//! Real-time audio pipeline (Step 3).
//!
//! Captures microphone input through cpal, encodes it with Opus and sends it
//! through the WebRTC publish track.  Remote Opus RTP packets received on the
//! subscribe track are decoded and played back through the default output
//! device.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    BufferSize, Device, SampleFormat, SampleRate, Stream, StreamConfig, SupportedStreamConfigRange,
};
use opus::{Application, Channels, Decoder as OpusDecoder, Encoder as OpusEncoder};
use tokio::sync::mpsc;
use tracing;
use webrtc::media_stream::track_local::static_sample::TrackLocalStaticSample;
use webrtc::media_stream::track_remote::{TrackRemote, TrackRemoteEvent};

use rtc::media::Sample;

const SAMPLE_RATE: u32 = 48000;
const FRAME_SIZE: usize = 960; // 20 ms @ 48 kHz
const STEREO_CHANNELS: usize = 2;
const MONO_CHANNELS: usize = 1;
const AUDIO_SSRC: u32 = 1111111111;
// TODO: negotiate the Opus payload type from the SDP answer instead of
// hard-coding the common dynamic value 111.
const OPUS_PAYLOAD_TYPE: u8 = 111;

/// Errors that can occur while building or running the audio pipeline.
#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("cpal build stream error: {0}")]
    CpalBuildStream(#[from] cpal::BuildStreamError),

    #[error("cpal play stream error: {0}")]
    CpalPlayStream(#[from] cpal::PlayStreamError),

    #[error("cpal supported configs error: {0}")]
    CpalSupportedConfigs(#[from] cpal::SupportedStreamConfigsError),

    #[error("cpal device not available")]
    CpalDeviceUnavailable,

    #[error("opus error: {0}")]
    Opus(#[from] opus::Error),

    #[error("webrtc error: {0}")]
    WebRtc(#[from] webrtc::error::Error),

    #[error("unsupported sample format: {0:?}")]
    UnsupportedSampleFormat(SampleFormat),

    #[error("no supported 48 kHz audio configuration")]
    UnsupportedConfig,
}

/// Shared ring buffer used by the output stream callback.
type SampleRing = Arc<Mutex<VecDeque<f32>>>;

/// Wrapper that makes a cpal `Stream` `Send` so the pipeline can cross the
/// async connect task → UI thread boundary.
///
/// Safety: the streams are only played once on creation and are afterwards kept
/// alive exclusively to prevent their audio callbacks from stopping.  Their
/// methods are never invoked concurrently from multiple threads.
#[allow(dead_code)]
struct SendStream(Stream);
unsafe impl Send for SendStream {}

/// Local microphone capture + remote audio playback pipeline.
pub struct AudioPipeline {
    /// Channel used by remote decode tasks to deliver decoded stereo f32 samples.
    remote_sample_tx: mpsc::Sender<Vec<f32>>,
    /// Local microphone mute flag.
    muted: Arc<AtomicBool>,
    /// Keeps the cpal capture stream alive.
    _capture_stream: SendStream,
    /// Keeps the cpal output stream alive.
    _output_stream: SendStream,
}

impl AudioPipeline {
    /// Builds the capture and playback streams and starts the encoder task.
    pub async fn new(audio_track: Arc<TrackLocalStaticSample>) -> Result<Self, AudioError> {
        let host = cpal::default_host();

        let input_device = host
            .default_input_device()
            .ok_or(AudioError::CpalDeviceUnavailable)?;
        let output_device = host
            .default_output_device()
            .ok_or(AudioError::CpalDeviceUnavailable)?;

        // Try stereo first; fall back to mono and upmix in the callback.
        let (input_config, input_channels) =
            pick_input_config(&input_device)?.ok_or(AudioError::UnsupportedConfig)?;

        let output_config =
            pick_output_config(&output_device)?.ok_or(AudioError::UnsupportedConfig)?;

        let muted = Arc::new(AtomicBool::new(false));
        let ring_buffer: SampleRing = Arc::new(Mutex::new(VecDeque::new()));

        // Bridge task: decoded remote samples arrive here and are mixed into the
        // shared playback ring buffer.
        let (remote_sample_tx, mut remote_sample_rx) = mpsc::channel::<Vec<f32>>(64);
        let ring_for_bridge = Arc::clone(&ring_buffer);
        tokio::spawn(async move {
            while let Some(samples) = remote_sample_rx.recv().await {
                mix_into_ring(&ring_for_bridge, &samples);
            }
        });

        // Capture -> Opus encoder -> WebRTC track.
        let (capture_tx, mut capture_rx) = mpsc::channel::<Vec<f32>>(8);
        let muted_for_encoder = Arc::clone(&muted);
        tokio::spawn(async move {
            let mut encoder =
                match OpusEncoder::new(SAMPLE_RATE, Channels::Stereo, Application::Audio) {
                    Ok(enc) => enc,
                    Err(e) => {
                        tracing::warn!("failed to create opus encoder: {e}");
                        return;
                    }
                };

            while let Some(frame) = capture_rx.recv().await {
                if muted_for_encoder.load(Ordering::Relaxed) {
                    continue;
                }

                let mut encoded = vec![0u8; 1275]; // max single Opus frame
                match encoder.encode_float(&frame, &mut encoded) {
                    Ok(len) => {
                        encoded.truncate(len);
                        let sample = Sample {
                            data: encoded.into(),
                            duration: Duration::from_millis(20),
                            ..Default::default()
                        };
                        if let Err(e) = audio_track
                            .write_sample(AUDIO_SSRC, OPUS_PAYLOAD_TYPE, &sample, &[])
                            .await
                        {
                            tracing::warn!("failed to write audio sample: {e}");
                        }
                    }
                    Err(e) => {
                        tracing::warn!("opus encode error: {e}");
                    }
                }
            }
        });

        let capture_stream =
            build_capture_stream(&input_device, &input_config, input_channels, capture_tx)?;

        let output_stream = build_output_stream(&output_device, &output_config, ring_buffer)?;

        capture_stream.play()?;
        output_stream.play()?;

        Ok(Self {
            remote_sample_tx,
            muted,
            _capture_stream: SendStream(capture_stream),
            _output_stream: SendStream(output_stream),
        })
    }

    /// Starts decoding a remote audio track and feeding its samples into this
    /// pipeline's playback path.
    pub fn start_remote_decode(track: Arc<dyn TrackRemote>, output_tx: mpsc::Sender<Vec<f32>>) {
        tokio::spawn(async move {
            let mut decoder = match OpusDecoder::new(SAMPLE_RATE, Channels::Stereo) {
                Ok(dec) => dec,
                Err(e) => {
                    tracing::warn!("failed to create opus decoder: {e}");
                    return;
                }
            };

            loop {
                match track.poll().await {
                    Some(TrackRemoteEvent::OnRtpPacket(rtp)) => {
                        let payload = rtp.payload;
                        if payload.is_empty() {
                            continue;
                        }

                        let mut decoded = vec![0.0_f32; FRAME_SIZE * STEREO_CHANNELS];
                        match decoder.decode_float(&payload, &mut decoded, false) {
                            Ok(samples_per_channel) => {
                                decoded.truncate(samples_per_channel * STEREO_CHANNELS);
                                if output_tx.send(decoded).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                tracing::warn!("opus decode error: {e}");
                            }
                        }
                    }
                    Some(TrackRemoteEvent::OnEnded) | None => break,
                    Some(_) => continue,
                }
            }
        });
    }

    /// Enables or disables local microphone capture.
    pub fn set_muted(&self, muted: bool) {
        self.muted.store(muted, Ordering::Relaxed);
    }

    /// Returns a clone of the channel used to feed decoded remote samples into
    /// the pipeline.
    pub fn remote_sample_tx(&self) -> mpsc::Sender<Vec<f32>> {
        self.remote_sample_tx.clone()
    }
}

/// Picks a 48 kHz input configuration.  Stereo is preferred; mono is returned
/// if the device does not support stereo and will be upmixed in the callback.
fn pick_input_config(device: &Device) -> Result<Option<(StreamConfig, usize)>, AudioError> {
    let supported = device.supported_input_configs()?;
    let mut mono: Option<SupportedStreamConfigRange> = None;

    for range in supported {
        if range.sample_format() != SampleFormat::F32 {
            continue;
        }
        if range.min_sample_rate().0 > SAMPLE_RATE || range.max_sample_rate().0 < SAMPLE_RATE {
            continue;
        }

        if range.channels() as usize >= STEREO_CHANNELS {
            return Ok(Some((
                StreamConfig {
                    channels: STEREO_CHANNELS as u16,
                    sample_rate: SampleRate(SAMPLE_RATE),
                    buffer_size: BufferSize::Default,
                },
                STEREO_CHANNELS,
            )));
        }
        if range.channels() as usize >= MONO_CHANNELS && mono.is_none() {
            mono = Some(range);
        }
    }

    if mono.is_some() {
        return Ok(Some((
            StreamConfig {
                channels: MONO_CHANNELS as u16,
                sample_rate: SampleRate(SAMPLE_RATE),
                buffer_size: BufferSize::Default,
            },
            MONO_CHANNELS,
        )));
    }

    Ok(None)
}

/// Picks a 48 kHz stereo output configuration.
fn pick_output_config(device: &Device) -> Result<Option<StreamConfig>, AudioError> {
    let supported = device.supported_output_configs()?;

    for range in supported {
        if range.sample_format() != SampleFormat::F32 {
            continue;
        }
        if range.min_sample_rate().0 > SAMPLE_RATE || range.max_sample_rate().0 < SAMPLE_RATE {
            continue;
        }
        if range.channels() as usize >= STEREO_CHANNELS {
            return Ok(Some(StreamConfig {
                channels: STEREO_CHANNELS as u16,
                sample_rate: SampleRate(SAMPLE_RATE),
                buffer_size: BufferSize::Default,
            }));
        }
    }

    Ok(None)
}

/// Builds the microphone capture stream.  Samples are accumulated into 20 ms
/// stereo frames and forwarded to the encoder task.
fn build_capture_stream(
    device: &Device,
    config: &StreamConfig,
    input_channels: usize,
    frame_tx: mpsc::Sender<Vec<f32>>,
) -> Result<Stream, AudioError> {
    let mut accum: Vec<f32> = Vec::with_capacity(FRAME_SIZE * STEREO_CHANNELS);

    let stream = device.build_input_stream(
        config,
        move |data: &[f32], _info| {
            for chunk in data.chunks(input_channels) {
                let sample = if input_channels == MONO_CHANNELS || chunk.len() == 1 {
                    // Upmix mono to stereo by duplicating the single channel.
                    [chunk[0], chunk[0]]
                } else {
                    [chunk[0], chunk[1]]
                };
                accum.extend_from_slice(&sample);

                if accum.len() >= FRAME_SIZE * STEREO_CHANNELS {
                    let frame = accum.split_off(0);
                    if frame_tx.try_send(frame).is_err() {
                        tracing::warn!("audio encoder queue full; dropping capture frame");
                    }
                }
            }
        },
        |err| {
            tracing::warn!("capture stream error: {err}");
        },
        None,
    )?;

    Ok(stream)
}

/// Builds the speaker output stream.  It consumes samples from the shared ring
/// buffer, filling with silence when data is not yet available.
fn build_output_stream(
    device: &Device,
    config: &StreamConfig,
    ring_buffer: SampleRing,
) -> Result<Stream, AudioError> {
    let stream = device.build_output_stream(
        config,
        move |data: &mut [f32], _info| {
            let mut ring = ring_buffer.lock().expect("audio ring buffer poisoned");
            for sample in data.iter_mut() {
                *sample = ring.pop_front().unwrap_or(0.0);
            }
        },
        |err| {
            tracing::warn!("output stream error: {err}");
        },
        None,
    )?;

    Ok(stream)
}

/// Mixes a freshly decoded remote frame into the existing playback ring buffer.
/// Samples are summed and clamped to [-1.0, 1.0].
fn mix_into_ring(ring: &SampleRing, samples: &[f32]) {
    let mut ring = ring.lock().expect("audio ring buffer poisoned");
    for (i, &sample) in samples.iter().enumerate() {
        if i < ring.len() {
            let mixed = (ring[i] + sample).clamp(-1.0, 1.0);
            ring[i] = mixed;
        } else {
            ring.push_back(sample);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opus_roundtrip_silence() {
        let mut encoder =
            OpusEncoder::new(SAMPLE_RATE, Channels::Stereo, Application::Audio).unwrap();
        let mut decoder = OpusDecoder::new(SAMPLE_RATE, Channels::Stereo).unwrap();

        let silence = vec![0.0_f32; FRAME_SIZE * STEREO_CHANNELS];
        let mut encoded = vec![0u8; 1275];
        let encoded_len = encoder.encode_float(&silence, &mut encoded).unwrap();
        assert!(
            encoded_len > 0,
            "encoded silence frame should produce non-empty Opus packet"
        );
        encoded.truncate(encoded_len);

        let mut decoded = vec![0.0_f32; FRAME_SIZE * STEREO_CHANNELS];
        let samples_per_channel = decoder.decode_float(&encoded, &mut decoded, false).unwrap();

        assert_eq!(
            samples_per_channel, FRAME_SIZE,
            "decoder should return one 20 ms frame per channel"
        );
        assert_eq!(
            decoded.len(),
            FRAME_SIZE * STEREO_CHANNELS,
            "decoded buffer should contain interleaved stereo samples"
        );
    }
}
