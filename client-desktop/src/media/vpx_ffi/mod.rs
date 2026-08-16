//! Safe-ish Rust bindings for the minimal libvpx VP8 wrapper in `vpx_wrapper.c`.
//!
//! The wrapper only supports I420 input/output. Higher-level code is responsible
//! for RGB/YUV conversion (see `zenyuv` or the simple converters in `video.rs`).

use std::ffi::CStr;
use std::os::raw::{c_int, c_ulong, c_void};

#[derive(Debug, thiserror::Error)]
pub enum VpxError {
    #[error("failed to create VP8 encoder: {0}")]
    EncoderCreate(String),
    #[error("failed to encode frame: {0}")]
    Encode(String),
    #[error("failed to create VP8 decoder: {0}")]
    DecoderCreate(String),
    #[error("failed to decode frame: {0}")]
    Decode(String),
    #[error("invalid dimensions or null pointer")]
    InvalidInput,
}

pub type Result<T> = std::result::Result<T, VpxError>;

/// One VP8 encoded frame returned by the encoder.
#[derive(Debug, Clone)]
pub struct Vp8Packet {
    pub data: Vec<u8>,
    pub keyframe: bool,
}

/// A decoded I420 frame. Plane data is owned by Rust (copied out of libvpx).
#[derive(Debug, Clone)]
pub struct I420Frame {
    pub width: u32,
    pub height: u32,
    pub y: Vec<u8>,
    pub u: Vec<u8>,
    pub v: Vec<u8>,
    pub stride_y: u32,
    pub stride_u: u32,
    pub stride_v: u32,
}

/// VP8 realtime encoder.
pub struct Vp8Encoder {
    inner: *mut VpxEncT,
    pts: i64,
}

/// VP8 decoder.
pub struct Vp8Decoder {
    inner: *mut VpxDecT,
}

#[repr(C)]
struct VpxEncT(c_void);
#[repr(C)]
struct VpxDecT(c_void);

#[repr(C)]
struct VpxEncPacket {
    ok: c_int,
    data: *const u8,
    len: usize,
    keyframe: c_int,
    has_more: c_int,
}

#[repr(C)]
struct VpxDecFrame {
    ok: c_int,
    has_frame: c_int,
    width: u32,
    height: u32,
    stride_y: u32,
    stride_u: u32,
    stride_v: u32,
    plane_y: *const u8,
    plane_u: *const u8,
    plane_v: *const u8,
}

unsafe extern "C" {
    fn vpx_enc_create(
        width: u32,
        height: u32,
        bitrate_kbps: u32,
        fps_num: u32,
        fps_den: u32,
        deadline_us: u32,
    ) -> *mut VpxEncT;
    fn vpx_enc_destroy(enc: *mut VpxEncT);
    fn vpx_enc_error(enc: *mut VpxEncT) -> *const libc::c_char;
    fn vpx_enc_encode(
        enc: *mut VpxEncT,
        y: *const u8,
        stride_y: u32,
        u: *const u8,
        stride_u: u32,
        v: *const u8,
        stride_v: u32,
        pts: i64,
        duration: c_ulong,
        flags: i64,
    ) -> c_int;
    fn vpx_enc_get_packet(enc: *mut VpxEncT) -> VpxEncPacket;

    fn vpx_dec_create() -> *mut VpxDecT;
    fn vpx_dec_destroy(dec: *mut VpxDecT);
    fn vpx_dec_error(dec: *mut VpxDecT) -> *const libc::c_char;
    fn vpx_dec_decode(dec: *mut VpxDecT, data: *const u8, len: usize) -> c_int;
    fn vpx_dec_get_frame(dec: *mut VpxDecT) -> VpxDecFrame;
}

unsafe fn cstr_err(ptr: *const libc::c_char) -> String {
    if ptr.is_null() {
        "unknown error".to_string()
    } else {
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}

impl Vp8Encoder {
    /// Create a realtime VP8 encoder.
    ///
    /// `fps_num`/`fps_den` is the frame rate fraction (e.g. 30/1).
    /// `deadline_us` is passed straight to libvpx (e.g. `VPX_DL_REALTIME`).
    pub fn new(
        width: u32,
        height: u32,
        bitrate_kbps: u32,
        fps_num: u32,
        fps_den: u32,
    ) -> Result<Self> {
        if width == 0 || height == 0 || fps_num == 0 || fps_den == 0 {
            return Err(VpxError::InvalidInput);
        }
        // SAFETY: inputs are validated; the wrapper returns NULL on failure.
        let inner = unsafe {
            vpx_enc_create(
                width,
                height,
                bitrate_kbps,
                fps_num,
                fps_den,
                1, // VPX_DL_REALTIME == 1
            )
        };
        if inner.is_null() {
            return Err(VpxError::EncoderCreate(
                "libvpx encoder initialization failed".into(),
            ));
        }
        Ok(Self { inner, pts: 0 })
    }

    /// Encode one I420 frame. Returns all produced VP8 packets.
    pub fn encode(&mut self, frame: &I420Frame) -> Result<Vec<Vp8Packet>> {
        if frame.width == 0 || frame.height == 0 {
            return Err(VpxError::InvalidInput);
        }
        // SAFETY: `inner` is valid and the wrapper copies the plane data.
        let rc = unsafe {
            vpx_enc_encode(
                self.inner,
                frame.y.as_ptr(),
                frame.stride_y,
                frame.u.as_ptr(),
                frame.stride_u,
                frame.v.as_ptr(),
                frame.stride_v,
                self.pts,
                1,
                0,
            )
        };
        if rc != 0 {
            let msg = unsafe { cstr_err(vpx_enc_error(self.inner)) };
            return Err(VpxError::Encode(msg));
        }
        self.pts += 1;

        let mut packets = Vec::new();
        loop {
            // SAFETY: encoder is valid; returned pointers are only valid until
            // the next call into the encoder, so we copy the data immediately.
            let pkt = unsafe { vpx_enc_get_packet(self.inner) };
            if pkt.ok == 0 {
                break;
            }
            if !pkt.data.is_null() && pkt.len > 0 {
                let data = unsafe { std::slice::from_raw_parts(pkt.data, pkt.len) }.to_vec();
                packets.push(Vp8Packet {
                    data,
                    keyframe: pkt.keyframe != 0,
                });
            }
            if pkt.has_more == 0 {
                break;
            }
        }
        Ok(packets)
    }
}

impl Drop for Vp8Encoder {
    fn drop(&mut self) {
        // SAFETY: `inner` was created by `vpx_enc_create` and is non-null.
        unsafe { vpx_enc_destroy(self.inner) };
    }
}

unsafe impl Send for Vp8Encoder {}
unsafe impl Sync for Vp8Encoder {}

impl Vp8Decoder {
    /// Create a VP8 decoder.
    pub fn new() -> Result<Self> {
        // SAFETY: wrapper returns NULL on failure.
        let inner = unsafe { vpx_dec_create() };
        if inner.is_null() {
            return Err(VpxError::DecoderCreate(
                "libvpx decoder initialization failed".into(),
            ));
        }
        Ok(Self { inner })
    }

    /// Decode one VP8 frame. Returns the decoded I420 frame if available.
    pub fn decode(&mut self, data: &[u8]) -> Result<Option<I420Frame>> {
        // SAFETY: `inner` is valid; `data`/`len` describe a valid slice.
        let rc = unsafe { vpx_dec_decode(self.inner, data.as_ptr(), data.len()) };
        if rc != 0 {
            let msg = unsafe { cstr_err(vpx_dec_error(self.inner)) };
            return Err(VpxError::Decode(msg));
        }

        // SAFETY: decoder is valid; returned plane pointers are only valid
        // until the next decoder call, so we copy immediately.
        let frame = unsafe { vpx_dec_get_frame(self.inner) };
        if frame.ok == 0 || frame.has_frame == 0 {
            return Ok(None);
        }

        let y_len = (frame.height * frame.stride_y) as usize;
        let uv_len = (frame.height / 2 * frame.stride_u) as usize;
        let v_len = (frame.height / 2 * frame.stride_v) as usize;

        if frame.plane_y.is_null() || frame.plane_u.is_null() || frame.plane_v.is_null() {
            return Err(VpxError::Decode("decoder returned null planes".into()));
        }

        let y = unsafe { std::slice::from_raw_parts(frame.plane_y, y_len) }.to_vec();
        let u = unsafe { std::slice::from_raw_parts(frame.plane_u, uv_len) }.to_vec();
        let v = unsafe { std::slice::from_raw_parts(frame.plane_v, v_len) }.to_vec();

        Ok(Some(I420Frame {
            width: frame.width,
            height: frame.height,
            y,
            u,
            v,
            stride_y: frame.stride_y,
            stride_u: frame.stride_u,
            stride_v: frame.stride_v,
        }))
    }
}

impl Default for Vp8Decoder {
    fn default() -> Self {
        Self::new().expect("VP8 decoder creation failed")
    }
}

impl Drop for Vp8Decoder {
    fn drop(&mut self) {
        // SAFETY: `inner` was created by `vpx_dec_create` and is non-null.
        unsafe { vpx_dec_destroy(self.inner) };
    }
}

unsafe impl Send for Vp8Decoder {}
unsafe impl Sync for Vp8Decoder {}

impl I420Frame {
    /// Allocate an empty I420 frame with the given dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        let y_size = width * height;
        let uv_size = (width / 2) * (height / 2);
        Self {
            width,
            height,
            y: vec![0; y_size as usize],
            u: vec![0; uv_size as usize],
            v: vec![0; uv_size as usize],
            stride_y: width,
            stride_u: width / 2,
            stride_v: width / 2,
        }
    }

    /// Fill from tightly-packed I420 data (e.g. from `zenyuv`).
    pub fn from_packed_i420(data: &[u8], width: u32, height: u32) -> Option<Self> {
        let y_size = (width * height) as usize;
        let uv_size = ((width / 2) * (height / 2)) as usize;
        if data.len() < y_size + 2 * uv_size {
            return None;
        }
        Some(Self {
            width,
            height,
            y: data[0..y_size].to_vec(),
            u: data[y_size..y_size + uv_size].to_vec(),
            v: data[y_size + uv_size..y_size + 2 * uv_size].to_vec(),
            stride_y: width,
            stride_u: width / 2,
            stride_v: width / 2,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let width = 320;
        let height = 240;
        let mut encoder = Vp8Encoder::new(width, height, 500, 30, 1).unwrap();
        let mut decoder = Vp8Decoder::new().unwrap();

        // Build a simple colored gradient frame.
        let mut frame = I420Frame::new(width, height);
        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                frame.y[idx] = ((x + y) % 256) as u8;
            }
        }
        frame.u.fill(128);
        frame.v.fill(128);

        let packets = encoder.encode(&frame).unwrap();
        assert!(!packets.is_empty(), "encoder produced no packets");
        assert!(packets[0].keyframe, "first packet should be a keyframe");

        let mut decoded: Option<I420Frame> = None;
        for pkt in packets {
            decoded = decoder.decode(&pkt.data).unwrap();
        }
        let decoded = decoded.expect("decoder should output a frame");
        assert_eq!(decoded.width, width);
        assert_eq!(decoded.height, height);
    }
}
