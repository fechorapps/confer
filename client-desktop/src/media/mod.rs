pub mod background;
pub mod capture;
pub mod crypto;
pub mod filters;
pub mod noise_suppression;
pub mod rtp;
pub mod virtual_background;

pub use capture::camera::CameraCapturer;
pub use capture::error::CaptureError;
pub use capture::screen::{detect_displays, DisplayInfo, PickerMode, ScreenCapturer};
pub use capture::{camera, screen};
pub use crypto::{
    decrypt_frame, encrypt_frame, ratchet_key, rotate_epoch_key, CipherSuite, ReplayFilter,
    SFrameEngine, SFrameError, SFrameHeader, SFrameKey,
};
pub use noise_suppression::AiNoiseSuppressor;
pub use rtp::{
    audio_samples_to_encrypted_opus_rtp_packet, audio_samples_to_opus_rtp_packet,
    capture_frame_to_encrypted_vp8_rtp_packets, capture_frame_to_vp8_rtp_packets,
    compute_pcm_audio_level_dbov, decrypt_opus_frame, decrypt_vp8_frame,
    AudioLevelExtension, OneByteHeaderExtension, OpusAudioFrame, OpusDepacketizer,
    OpusPacketizer, RtpError, RtpHeader, RtpPacket, RtpStreamStats, Vp8Frame,
    Vp8FrameAssembler, Vp8FrameHeader, Vp8Packetizer, Vp8PayloadDescriptor,
};
pub use virtual_background::{VirtualBackgroundMode, VirtualBackgroundProcessor};
