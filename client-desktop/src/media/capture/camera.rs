use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use egui::ColorImage;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{
    CameraFormat, CameraIndex, FrameFormat,
    RequestedFormat, RequestedFormatType, Resolution,
};
use nokhwa::Camera;

use crate::media::background::BackgroundEffect;
use crate::media::capture::convert;
use crate::media::filters::VideoFilter;

pub struct CameraCapturer {
    /// Requested capture resolution. The camera negotiates the closest format
    /// it actually supports, so a decoded frame's real size (see the
    /// `ColorImage` returned by `get_latest_frame_if_newer`) may differ
    /// slightly from this request.
    pub width: usize,
    pub height: usize,
    latest_frame: Arc<Mutex<Option<(u64, ColorImage)>>>,
    active_filter_atomic: Arc<AtomicU8>,
    active_bg_mutex: Arc<Mutex<BackgroundEffect>>,
    is_running: Arc<AtomicBool>,
    worker_handle: Option<thread::JoinHandle<()>>,
}

impl CameraCapturer {
    pub fn new(width: usize, height: usize) -> Self {
        let latest_frame = Arc::new(Mutex::new(None));
        let active_filter_atomic = Arc::new(AtomicU8::new(0));
        let active_bg_mutex = Arc::new(Mutex::new(BackgroundEffect::None));
        let is_running = Arc::new(AtomicBool::new(true));

        let frame_sink = latest_frame.clone();
        let filter_ref = active_filter_atomic.clone();
        let bg_ref = active_bg_mutex.clone();
        let running_flag = is_running.clone();

        let worker_handle = thread::spawn(move || {
            capture_loop(width, height, frame_sink, filter_ref, bg_ref, running_flag);
        });

        Self {
            width,
            height,
            latest_frame,
            active_filter_atomic,
            active_bg_mutex,
            is_running,
            worker_handle: Some(worker_handle),
        }
    }

    pub fn set_filter(&self, filter: VideoFilter) {
        let val = match filter {
            VideoFilter::None => 0,
            VideoFilter::StudioGlow => 1,
            VideoFilter::WarmSunset => 2,
            VideoFilter::CoolNordic => 3,
            VideoFilter::NoirBw => 4,
            VideoFilter::VibrantPop => 5,
            VideoFilter::VignetteFocus => 6,
            VideoFilter::VintageFilm => 7,
        };
        self.active_filter_atomic.store(val, Ordering::Relaxed);
    }

    pub fn set_background(&self, bg: BackgroundEffect) {
        if let Ok(mut guard) = self.active_bg_mutex.lock() {
            *guard = bg;
        }
    }

    /// Fetches the newest frame only if a newer frame has been decoded since `last_seen_id`.
    /// This prevents redundant GPU texture uploads on 60 FPS UI repaints.
    pub fn get_latest_frame_if_newer(&self, last_seen_id: u64) -> Option<(u64, ColorImage)> {
        if let Ok(guard) = self.latest_frame.lock() {
            if let Some((frame_id, img)) = &*guard {
                if *frame_id > last_seen_id {
                    return Some((*frame_id, img.clone()));
                }
            }
        }
        None
    }
}

impl Drop for CameraCapturer {
    fn drop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.worker_handle.take() {
            crate::media::capture::join_with_timeout(handle, Duration::from_secs(2));
        }
    }
}

fn capture_loop(
    width: usize,
    height: usize,
    frame_sink: Arc<Mutex<Option<(u64, ColorImage)>>>,
    filter_ref: Arc<AtomicU8>,
    bg_ref: Arc<Mutex<BackgroundEffect>>,
    is_running: Arc<AtomicBool>,
) {
    let mut backoff = Duration::from_millis(100);
    let max_backoff = Duration::from_millis(1000);
    let mut frame_seq = 0u64;

    while is_running.load(Ordering::Relaxed) {
        let index = CameraIndex::Index(0);

        // 1. Prioritize uncompressed native format (YUYV / NV12) at the
        // requested resolution to avoid unnecessary JPEG encode/decode roundtrips
        let requested_format = RequestedFormat::new::<RgbFormat>(
            RequestedFormatType::Closest(CameraFormat::new(
                Resolution::new(width as u32, height as u32),
                FrameFormat::YUYV,
                30,
            )),
        );

        let mut camera = match Camera::new(index.clone(), requested_format) {
            Ok(cam) => cam,
            Err(_) => {
                // Fallback to MJPEG if uncompressed format fails
                let mjpeg_format = RequestedFormat::new::<RgbFormat>(
                    RequestedFormatType::Closest(CameraFormat::new(
                        Resolution::new(width as u32, height as u32),
                        FrameFormat::MJPEG,
                        30,
                    )),
                );
                match Camera::new(index, mjpeg_format) {
                    Ok(cam) => cam,
                    Err(e) => {
                        tracing::warn!("Native camera init retry after error: {e}");
                        thread::sleep(backoff);
                        backoff = (backoff * 2).min(max_backoff);
                        continue;
                    }
                }
            }
        };

        // Tune hardware camera controls (sharpness, contrast, saturation, backlight, white balance)
        #[cfg(target_os = "linux")]
        tune_v4l2_hardware_controls("/dev/video0");

        if let Err(e) = camera.open_stream() {
            tracing::warn!("Failed to open native camera stream: {e}");
            thread::sleep(backoff);
            backoff = (backoff * 2).min(max_backoff);
            continue;
        }

        // Reset backoff on successful stream start
        backoff = Duration::from_millis(100);

        while is_running.load(Ordering::Relaxed) {
            match camera.frame() {
                Ok(buffer) => {
                    let decoded = match buffer.decode_image::<RgbFormat>() {
                        Ok(img) => img,
                        Err(e) => {
                            tracing::warn!("Frame decode error: {e}");
                            continue;
                        }
                    };

                    let (w, h) = (decoded.width() as usize, decoded.height() as usize);
                    let mut pixels = match convert::rgb_to_color32(decoded.as_raw(), w, h) {
                        Some(p) => p,
                        None => {
                            tracing::warn!("Camera frame buffer smaller than expected {w}x{h}; skipping frame");
                            continue;
                        }
                    };

                    // 1. Apply background blur / virtual background replacement
                    if let Ok(bg_guard) = bg_ref.lock() {
                        bg_guard.apply(&mut pixels, w, h);
                    }

                    // 2. Apply visual tone filter
                    let filter_val = filter_ref.load(Ordering::Relaxed);
                    let filter = match filter_val {
                        1 => VideoFilter::StudioGlow,
                        2 => VideoFilter::WarmSunset,
                        3 => VideoFilter::CoolNordic,
                        4 => VideoFilter::NoirBw,
                        5 => VideoFilter::VibrantPop,
                        6 => VideoFilter::VignetteFocus,
                        7 => VideoFilter::VintageFilm,
                        _ => VideoFilter::None,
                    };
                    filter.apply(&mut pixels, w, h);

                    let color_image = ColorImage {
                        size: [w, h],
                        pixels,
                    };

                    frame_seq += 1;
                    if let Ok(mut guard) = frame_sink.lock() {
                        *guard = Some((frame_seq, color_image));
                    }
                }
                Err(e) => {
                    tracing::warn!("Camera frame capture error: {e}");
                    break;
                }
            }
        }

        let _ = camera.stop_stream();
        thread::sleep(Duration::from_millis(200));
    }
}

#[cfg(target_os = "linux")]
fn tune_v4l2_hardware_controls(device_path: &str) {
    use std::os::unix::io::AsRawFd;
    if let Ok(file) = std::fs::OpenOptions::new().read(true).write(true).open(device_path) {
        let fd = file.as_raw_fd();

        #[repr(C)]
        struct V4l2Control {
            id: u32,
            value: i32,
        }

        // _IOWR('V', 28, struct v4l2_control)
        const VIDIOC_S_CTRL: libc::c_ulong = 0xc008561c;

        const V4L2_CID_BASE: u32 = 0x00980900;
        const V4L2_CID_CONTRAST: u32 = V4L2_CID_BASE + 1;
        const V4L2_CID_SATURATION: u32 = V4L2_CID_BASE + 2;
        const V4L2_CID_AUTO_WHITE_BALANCE: u32 = V4L2_CID_BASE + 12;
        const V4L2_CID_SHARPNESS: u32 = V4L2_CID_BASE + 27;
        const V4L2_CID_BACKLIGHT_COMPENSATION: u32 = V4L2_CID_BASE + 28;

        let controls = [
            (V4L2_CID_SHARPNESS, 60),
            (V4L2_CID_CONTRAST, 54),
            (V4L2_CID_SATURATION, 54),
            (V4L2_CID_BACKLIGHT_COMPENSATION, 1),
            (V4L2_CID_AUTO_WHITE_BALANCE, 1),
        ];

        for (id, val) in controls {
            let mut ctrl = V4l2Control {
                id,
                value: val,
            };
            unsafe {
                let _ = libc::ioctl(fd, VIDIOC_S_CTRL, &mut ctrl);
            }
        }
    }
}
