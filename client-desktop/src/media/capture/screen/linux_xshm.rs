use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use egui::ColorImage;
use x11rb::connection::Connection;
use x11rb::protocol::shm::ConnectionExt as _;
use x11rb::protocol::xproto::ImageFormat;
use x11rb::rust_connection::RustConnection;

use crate::media::capture::convert;
use crate::media::capture::error::CaptureError;
use crate::media::capture::screen::{DisplayInfo, FrameSink, ScreenCaptureBackend};

pub struct XshmBackend {
    is_running: Arc<AtomicBool>,
    worker_handle: Option<thread::JoinHandle<()>>,
}

impl XshmBackend {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            worker_handle: None,
        }
    }
}

impl Default for XshmBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenCaptureBackend for XshmBackend {
    fn start(&mut self, sink: FrameSink, display: Option<&DisplayInfo>) -> Result<(), CaptureError> {
        self.stop();

        let (conn, screen_num) = RustConnection::connect(None)
            .map_err(|e| CaptureError::X11(format!("Failed to connect to X11 server: {e}")))?;

        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;

        let (x, y, width, height) = if let Some(d) = display {
            (d.x, d.y, d.width, d.height)
        } else {
            (0, 0, screen.width_in_pixels as u32, screen.height_in_pixels as u32)
        };

        if width == 0 || height == 0 {
            return Err(CaptureError::X11("Invalid screen dimensions".to_string()));
        }

        let is_running = Arc::new(AtomicBool::new(true));
        self.is_running = is_running.clone();

        let handle = thread::spawn(move || {
            capture_xshm_loop(conn, root, x, y, width, height, sink, is_running);
        });

        self.worker_handle = Some(handle);
        Ok(())
    }

    fn stop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }

    fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }
}

impl Drop for XshmBackend {
    fn drop(&mut self) {
        self.stop();
    }
}

fn capture_xshm_loop(
    conn: RustConnection,
    root: u32,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    sink: FrameSink,
    is_running: Arc<AtomicBool>,
) {
    let size = (width * height * 4) as usize;

    // Allocate shared memory segment, owner-only (0o600): this segment holds
    // live screen-share frame data and must not be readable/writable by other
    // local users or processes.
    let shmid = unsafe { libc::shmget(libc::IPC_PRIVATE, size, libc::IPC_CREAT | 0o600) };
    if shmid < 0 {
        tracing::error!("XShm: Failed to allocate shared memory segment (shmget failed)");
        return;
    }

    let shmaddr = unsafe { libc::shmat(shmid, std::ptr::null(), 0) };
    if shmaddr == libc::MAP_FAILED {
        tracing::error!("XShm: Failed to attach shared memory segment (shmat failed)");
        unsafe { libc::shmctl(shmid, libc::IPC_RMID, std::ptr::null_mut()) };
        return;
    }

    let shmseg = match conn.generate_id() {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("XShm: Failed to generate XID: {e}");
            cleanup_shm(shmid, shmaddr);
            return;
        }
    };

    if let Err(e) = conn.shm_attach(shmseg, shmid as u32, false) {
        tracing::error!("XShm: Failed to attach SHM to X server: {e}");
        cleanup_shm(shmid, shmaddr);
        return;
    }
    let _ = conn.flush();

    let mut frame_seq = 0u64;
    let target_frame_duration = Duration::from_millis(33); // ~30 FPS

    while is_running.load(Ordering::Relaxed) {
        let frame_start = Instant::now();

        let reply = conn.shm_get_image(
            root,
            x as i16,
            y as i16,
            width as u16,
            height as u16,
            !0,
            ImageFormat::Z_PIXMAP.into(),
            shmseg,
            0,
        );

        match reply {
            Ok(cookie) => {
                if cookie.reply().is_ok() {
                    // X11 ZPixmap is BGRx (B=0, G=1, R=2, x=3)
                    let raw_slice = unsafe { std::slice::from_raw_parts(shmaddr as *const u8, size) };
                    let w = width as usize;
                    let h = height as usize;

                    if let Some(pixels) = convert::bgrx_to_color32(raw_slice, w, h) {
                        frame_seq += 1;
                        if let Ok(mut guard) = sink.lock() {
                            *guard = Some((frame_seq, ColorImage { size: [w, h], pixels }));
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("XShm get_image error: {e}");
                break;
            }
        }

        let elapsed = frame_start.elapsed();
        if elapsed < target_frame_duration {
            thread::sleep(target_frame_duration - elapsed);
        }
    }

    let _ = conn.shm_detach(shmseg);
    let _ = conn.flush();
    cleanup_shm(shmid, shmaddr);
}

fn cleanup_shm(shmid: i32, shmaddr: *mut libc::c_void) {
    unsafe {
        libc::shmdt(shmaddr);
        libc::shmctl(shmid, libc::IPC_RMID, std::ptr::null_mut());
    }
}
