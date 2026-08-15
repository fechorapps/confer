use egui::ColorImage;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use x11rb::connection::Connection;
use x11rb::protocol::shm::ConnectionExt as _;
use x11rb::protocol::xproto::ImageFormat;
use x11rb::rust_connection::RustConnection;

use crate::media::capture::convert;
use crate::media::capture::error::CaptureError;
use crate::media::capture::join_with_timeout;
use crate::media::capture::screen::{DisplayInfo, FrameSink, ScreenCaptureBackend};

pub struct XshmBackend {
    is_running: Arc<AtomicBool>,
    last_error: Arc<Mutex<Option<CaptureError>>>,
    worker_handle: Option<thread::JoinHandle<()>>,
}

impl XshmBackend {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            last_error: Arc::new(Mutex::new(None)),
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
    fn start(
        &mut self,
        sink: FrameSink,
        display: Option<&DisplayInfo>,
    ) -> Result<(), CaptureError> {
        self.stop();

        let (conn, screen_num) = RustConnection::connect(None)
            .map_err(|e| CaptureError::X11(format!("Failed to connect to X11 server: {e}")))?;

        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;

        let screen_w = screen.width_in_pixels as i32;
        let screen_h = screen.height_in_pixels as i32;

        let (x, y, width, height) = if let Some(d) = display {
            let clamped_x = d.x.clamp(0, (screen_w - 1).max(0));
            let clamped_y = d.y.clamp(0, (screen_h - 1).max(0));
            let max_w = (screen_w - clamped_x).max(1) as u32;
            let max_h = (screen_h - clamped_y).max(1) as u32;
            (
                clamped_x,
                clamped_y,
                d.width.min(max_w).max(1),
                d.height.min(max_h).max(1),
            )
        } else {
            (0, 0, screen_w.max(1) as u32, screen_h.max(1) as u32)
        };

        if width == 0 || height == 0 {
            return Err(CaptureError::X11("Invalid screen dimensions".to_string()));
        }

        let is_running = Arc::new(AtomicBool::new(true));
        self.is_running = is_running.clone();
        let last_error = Arc::new(Mutex::new(None));
        self.last_error = last_error.clone();

        let args = XshmArgs {
            conn,
            root,
            x,
            y,
            width,
            height,
        };
        let handle = thread::spawn(move || {
            capture_xshm_loop(args, sink, is_running, last_error);
        });

        self.worker_handle = Some(handle);
        Ok(())
    }

    fn stop(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.worker_handle.take() {
            join_with_timeout(handle, Duration::from_secs(2));
        }
    }

    fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }

    fn take_error(&mut self) -> Option<CaptureError> {
        self.last_error
            .lock()
            .ok()
            .and_then(|mut guard| guard.take())
    }
}

impl Drop for XshmBackend {
    fn drop(&mut self) {
        self.stop();
    }
}

struct XshmArgs {
    conn: RustConnection,
    root: u32,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
}

fn store_error(slot: &Arc<Mutex<Option<CaptureError>>>, error: CaptureError) {
    tracing::warn!("XShm capture failed: {error}");
    if let Ok(mut guard) = slot.lock() {
        *guard = Some(error);
    }
}

fn capture_xshm_loop(
    args: XshmArgs,
    sink: FrameSink,
    is_running: Arc<AtomicBool>,
    last_error: Arc<Mutex<Option<CaptureError>>>,
) {
    let XshmArgs {
        conn,
        root,
        x,
        y,
        width,
        height,
    } = args;

    // Whatever happens below, on exit the backend must report "not running" so
    // the UI never believes a dead capture thread is still sharing.
    let _running_guard = scopeguard(&is_running);

    let size = (width * height * 4) as usize;

    let shmid = unsafe { libc::shmget(libc::IPC_PRIVATE, size, libc::IPC_CREAT | 0o600) };
    if shmid < 0 {
        store_error(
            &last_error,
            CaptureError::X11("Failed to allocate shared memory segment (shmget)".to_string()),
        );
        return;
    }

    let shmaddr = unsafe { libc::shmat(shmid, std::ptr::null(), 0) };
    if shmaddr == libc::MAP_FAILED {
        store_error(
            &last_error,
            CaptureError::X11("Failed to attach shared memory segment (shmat)".to_string()),
        );
        unsafe { libc::shmctl(shmid, libc::IPC_RMID, std::ptr::null_mut()) };
        return;
    }

    let shmseg = match conn.generate_id() {
        Ok(id) => id,
        Err(e) => {
            store_error(
                &last_error,
                CaptureError::X11(format!("Failed to generate XID: {e}")),
            );
            cleanup_shm(shmid, shmaddr);
            return;
        }
    };

    if let Err(e) = conn.shm_attach(shmseg, shmid as u32, false) {
        store_error(
            &last_error,
            CaptureError::X11(format!("Failed to attach SHM to X server: {e}")),
        );
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
                    let mut frame_buf = vec![0u8; size];
                    // SAFETY: the X server finished writing the image before
                    // sending the shm_get_image reply (X11 requests on one
                    // connection are processed in order), so `shmaddr..shmaddr+size`
                    // holds a complete frame. `frame_buf` is a fresh, disjoint
                    // `Vec` of exactly `size` bytes, so the copy cannot overlap
                    // or overflow.
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            shmaddr as *const u8,
                            frame_buf.as_mut_ptr(),
                            size,
                        );
                    }
                    let w = width as usize;
                    let h = height as usize;

                    if let Some(pixels) = convert::bgrx_to_color32(&frame_buf, w, h) {
                        frame_seq += 1;
                        if let Ok(mut guard) = sink.lock() {
                            *guard = Some((
                                frame_seq,
                                Arc::new(ColorImage {
                                    size: [w, h],
                                    pixels,
                                }),
                            ));
                        }
                    }
                }
            }
            Err(e) => {
                store_error(
                    &last_error,
                    CaptureError::X11(format!("shm_get_image failed: {e}")),
                );
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

/// Resets the running flag when the capture loop exits for any reason
/// (error, `stop()`, or panic unwinding past the loop).
fn scopeguard(is_running: &Arc<AtomicBool>) -> impl Drop {
    struct FlagGuard(Arc<AtomicBool>);
    impl Drop for FlagGuard {
        fn drop(&mut self) {
            self.0.store(false, Ordering::Relaxed);
        }
    }
    FlagGuard(is_running.clone())
}

fn cleanup_shm(shmid: i32, shmaddr: *mut libc::c_void) {
    unsafe {
        libc::shmdt(shmaddr);
        libc::shmctl(shmid, libc::IPC_RMID, std::ptr::null_mut());
    }
}
