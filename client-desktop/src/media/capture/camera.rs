use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use egui::{Color32, ColorImage};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{
    CameraFormat, CameraIndex, FrameFormat,
    RequestedFormat, RequestedFormatType, Resolution,
};
use nokhwa::Camera;

use crate::media::capture::convert;
use crate::media::filters::VideoFilter;
use crate::media::virtual_background::{VirtualBackgroundMode, VirtualBackgroundProcessor};

pub struct CameraCapturer {
    pub width: usize,
    pub height: usize,
    latest_frame: Arc<Mutex<Option<(u64, ColorImage)>>>,
    active_filter_atomic: Arc<AtomicU8>,
    active_bg_mutex: Arc<Mutex<VirtualBackgroundMode>>,
    is_running: Arc<AtomicBool>,
    worker_handle: Option<thread::JoinHandle<()>>,
}

impl CameraCapturer {
    pub fn new(width: usize, height: usize) -> Self {
        let latest_frame = Arc::new(Mutex::new(None));
        let active_filter_atomic = Arc::new(AtomicU8::new(0));
        let active_bg_mutex = Arc::new(Mutex::new(VirtualBackgroundMode::None));
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

    pub fn set_background(&self, bg: VirtualBackgroundMode) {
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

fn try_init_hardware_camera(width: usize, height: usize) -> Option<Camera> {
    let index = CameraIndex::Index(0);

    // 1. Try uncompressed YUYV format
    let yuyv_fmt = RequestedFormat::new::<RgbFormat>(
        RequestedFormatType::Closest(CameraFormat::new(
            Resolution::new(width as u32, height as u32),
            FrameFormat::YUYV,
            30,
        )),
    );
    if let Ok(mut cam) = Camera::new(index.clone(), yuyv_fmt) {
        if cam.open_stream().is_ok() {
            return Some(cam);
        }
    }

    // 2. Try MJPEG format
    let mjpeg_fmt = RequestedFormat::new::<RgbFormat>(
        RequestedFormatType::Closest(CameraFormat::new(
            Resolution::new(width as u32, height as u32),
            FrameFormat::MJPEG,
            30,
        )),
    );
    if let Ok(mut cam) = Camera::new(index.clone(), mjpeg_fmt) {
        if cam.open_stream().is_ok() {
            return Some(cam);
        }
    }

    // 3. Try device default format (None)
    let default_fmt = RequestedFormat::new::<RgbFormat>(RequestedFormatType::None);
    if let Ok(mut cam) = Camera::new(index, default_fmt) {
        if cam.open_stream().is_ok() {
            return Some(cam);
        }
    }

    None
}

fn capture_loop(
    width: usize,
    height: usize,
    frame_sink: Arc<Mutex<Option<(u64, ColorImage)>>>,
    filter_ref: Arc<AtomicU8>,
    bg_ref: Arc<Mutex<VirtualBackgroundMode>>,
    is_running: Arc<AtomicBool>,
) {
    let mut frame_seq = 0u64;
    let start_time = Instant::now();
    let mut last_hw_probe = Instant::now() - Duration::from_secs(10);
    let mut hw_camera: Option<Camera> = None;
    let bg_processor = VirtualBackgroundProcessor::new();

    while is_running.load(Ordering::Relaxed) {
        // Probe for hardware camera periodically if not connected
        if hw_camera.is_none() && last_hw_probe.elapsed() > Duration::from_secs(4) {
            last_hw_probe = Instant::now();
            hw_camera = try_init_hardware_camera(width, height);
            if hw_camera.is_some() {
                tracing::info!("Hardware camera initialized and streaming successfully.");
                #[cfg(target_os = "linux")]
                tune_v4l2_hardware_controls("/dev/video0");
            }
        }

        if let Some(cam) = &mut hw_camera {
            match cam.frame() {
                Ok(buffer) => {
                    if let Ok(decoded) = buffer.decode_image::<RgbFormat>() {
                        let (w, h) = (decoded.width() as usize, decoded.height() as usize);
                        if let Some(mut pixels) = convert::rgb_to_color32(decoded.as_raw(), w, h) {
                            // Virtual background & filter
                            if let Ok(bg_guard) = bg_ref.lock() {
                                bg_processor.process_frame(&bg_guard, &mut pixels, w, h);
                            }
                            apply_active_filter(&filter_ref, &mut pixels, w, h);

                            frame_seq += 1;
                            if let Ok(mut guard) = frame_sink.lock() {
                                *guard = Some((frame_seq, ColorImage { size: [w, h], pixels }));
                            }
                            continue;
                        }
                    }
                }
                Err(_) => {
                    // Camera stream failed/disconnected, drop and fall back to synthetic
                    let _ = cam.stop_stream();
                    hw_camera = None;
                }
            }
        }

        // --- Graceful Synthetic Frame Generator (when no webcam is present) ---
        let w = width.max(320);
        let h = height.max(180);
        let elapsed = start_time.elapsed().as_secs_f32();
        let mut pixels = generate_synthetic_avatar_frame(w, h, elapsed);

        // Apply background / filter to synthetic frame
        if let Ok(bg_guard) = bg_ref.lock() {
            bg_processor.process_frame(&bg_guard, &mut pixels, w, h);
        }
        apply_active_filter(&filter_ref, &mut pixels, w, h);

        frame_seq += 1;
        if let Ok(mut guard) = frame_sink.lock() {
            *guard = Some((frame_seq, ColorImage { size: [w, h], pixels }));
        }

        // 30 FPS rate limiter for synthetic generator
        thread::sleep(Duration::from_millis(33));
    }

    if let Some(mut cam) = hw_camera {
        let _ = cam.stop_stream();
    }
}

fn apply_active_filter(filter_ref: &Arc<AtomicU8>, pixels: &mut [Color32], w: usize, h: usize) {
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
    filter.apply(pixels, w, h);
}

fn generate_synthetic_avatar_frame(w: usize, h: usize, elapsed: f32) -> Vec<Color32> {
    let mut pixels = Vec::with_capacity(w * h);
    let cx = (w as f32) / 2.0;
    let cy = (h as f32) / 2.0;
    let radius = (h as f32 * 0.28).min(cx * 0.5);
    let pulse = 1.0 + 0.04 * (elapsed * 3.0).sin();
    let effective_radius_sq = (radius * pulse).powi(2);

    for y in 0..h {
        let dy = y as f32 - cy;
        let y_ratio = y as f32 / h as f32;
        for x in 0..w {
            let dx = x as f32 - cx;
            let dist_sq = dx * dx + dy * dy;

            if dist_sq <= effective_radius_sq {
                // Avatar Circle (Confer Blue gradient)
                let factor = (1.0 - (dist_sq / effective_radius_sq).sqrt()).clamp(0.0, 1.0);
                let r = (2.0 + factor * 50.0) as u8;
                let g = (132.0 + factor * 56.0) as u8;
                let b = (199.0 + factor * 56.0) as u8;
                pixels.push(Color32::from_rgb(r, g, b));
            } else {
                // Luxury Dark Background with subtle ambient gradient
                let bg_val = (16.0 + y_ratio * 12.0 + (elapsed * 0.5).sin() * 4.0).clamp(12.0, 32.0) as u8;
                pixels.push(Color32::from_rgb(bg_val, bg_val + 2, bg_val + 6));
            }
        }
    }
    pixels
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
            let mut ctrl = V4l2Control { id, value: val };
            unsafe {
                let _ = libc::ioctl(fd, VIDIOC_S_CTRL, &mut ctrl);
            }
        }
    }
}
