use egui::ColorImage;
use std::process::Command;
use std::sync::{Arc, Mutex};

use crate::media::capture::error::CaptureError;

#[cfg(target_os = "linux")]
pub mod linux_portal;
#[cfg(target_os = "linux")]
pub mod linux_xshm;
#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "windows")]
pub mod windows;

/// Shared slot where capture backends publish their newest frame. The frame
/// sits behind an `Arc` so consumers only clone a refcount while holding the
/// lock; the expensive pixel copy happens lock-free on the consumer side.
pub type FrameSink = Arc<Mutex<Option<(u64, Arc<ColorImage>)>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerMode {
    Native,
    DisplayList,
}

pub trait ScreenCaptureBackend: Send {
    fn start(&mut self, sink: FrameSink, display: Option<&DisplayInfo>)
        -> Result<(), CaptureError>;
    fn stop(&mut self);
    fn is_running(&self) -> bool;

    /// Returns and clears any error that occurred asynchronously after `start()`
    /// already returned `Ok` — backends that defer negotiation to a worker
    /// thread (e.g. the portal backend, so an interactive picker dialog never
    /// blocks the caller) can only report failures this way. Callers should
    /// poll this periodically while capture is expected to be running.
    fn take_error(&mut self) -> Option<CaptureError> {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisplayInfo {
    pub id: usize,
    pub name: String,
    pub label: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

pub fn detect_displays() -> Vec<DisplayInfo> {
    let mut displays = Vec::new();

    // 1. Try xrandr --listmonitors (for X11 fallback display detection)
    if let Ok(output) = Command::new("xrandr").arg("--listmonitors").output() {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let name = parts.last().unwrap_or(&"Display").to_string();
                let is_primary = parts[1].contains('*');
                let geom_part = parts[2]; // e.g. 1920/340x1200/220+1920+1080

                if let Some((dims, offsets)) = geom_part.split_once('+') {
                    let (w_str, h_str) = dims.split_once('x').unwrap_or(("1920", "1080"));
                    let w: u32 = w_str
                        .split('/')
                        .next()
                        .unwrap_or("1920")
                        .parse()
                        .unwrap_or(1920);
                    let h: u32 = h_str
                        .split('/')
                        .next()
                        .unwrap_or("1080")
                        .parse()
                        .unwrap_or(1080);

                    let offset_parts: Vec<&str> = offsets.split('+').collect();
                    let x: i32 = offset_parts
                        .first()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    let y: i32 = offset_parts
                        .get(1)
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);

                    let id = displays.len();
                    let primary_tag = if is_primary { " ★ Primary" } else { "" };
                    let label = format!("🖥️ {} ({}x{}){}", name, w, h, primary_tag);

                    displays.push(DisplayInfo {
                        id,
                        name,
                        label,
                        x,
                        y,
                        width: w,
                        height: h,
                        is_primary,
                    });
                }
            }
        }
    }

    // Fallback default if none detected
    if displays.is_empty() {
        displays.push(DisplayInfo {
            id: 0,
            name: ":0.0".to_string(),
            label: "🖥️ Primary Display (1920x1080)".to_string(),
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            is_primary: true,
        });
    }

    // Add full multi-screen desktop option if multiple monitors exist
    if displays.len() > 1 {
        let max_w = displays
            .iter()
            .map(|d| d.x + d.width as i32)
            .max()
            .unwrap_or(1920) as u32;
        let max_h = displays
            .iter()
            .map(|d| d.y + d.height as i32)
            .max()
            .unwrap_or(1080) as u32;
        let id = displays.len();
        displays.push(DisplayInfo {
            id,
            name: "Combined".to_string(),
            label: format!("🔲 All Displays Combined ({}x{})", max_w, max_h),
            x: 0,
            y: 0,
            width: max_w,
            height: max_h,
            is_primary: false,
        });
    }

    displays
}

pub struct ScreenCapturer {
    latest_frame: FrameSink,
    backend: Box<dyn ScreenCaptureBackend>,
    picker_mode: PickerMode,
}

impl ScreenCapturer {
    pub fn new() -> Self {
        let latest_frame = Arc::new(Mutex::new(None));

        #[cfg(target_os = "linux")]
        let (picker_mode, backend): (PickerMode, Box<dyn ScreenCaptureBackend>) = {
            if probe_portal_available() {
                tracing::info!(
                    "ScreenCapturer: Using Linux Wayland/Portal backend (Native Picker)"
                );
                (
                    PickerMode::Native,
                    Box::new(linux_portal::PortalBackend::new()),
                )
            } else {
                tracing::info!(
                    "ScreenCapturer: Using Linux X11 MIT-SHM fallback backend (DisplayList)"
                );
                (
                    PickerMode::DisplayList,
                    Box::new(linux_xshm::XshmBackend::new()),
                )
            }
        };

        #[cfg(target_os = "windows")]
        let (picker_mode, backend): (PickerMode, Box<dyn ScreenCaptureBackend>) = {
            (
                PickerMode::DisplayList,
                Box::new(windows::WindowsBackend::new()),
            )
        };

        #[cfg(target_os = "macos")]
        let (picker_mode, backend): (PickerMode, Box<dyn ScreenCaptureBackend>) =
            { (PickerMode::Native, Box::new(macos::MacOsBackend::new())) };

        #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
        let (picker_mode, backend): (PickerMode, Box<dyn ScreenCaptureBackend>) =
            { (PickerMode::DisplayList, Box::new(NullBackend::new())) };

        Self {
            latest_frame,
            backend,
            picker_mode,
        }
    }

    pub fn picker_mode(&self) -> PickerMode {
        self.picker_mode
    }

    pub fn start_native(&mut self) -> Result<(), CaptureError> {
        self.backend.start(self.latest_frame.clone(), None)
    }

    pub fn start_display(&mut self, display: &DisplayInfo) -> Result<(), CaptureError> {
        self.backend.start(self.latest_frame.clone(), Some(display))
    }

    /// Returns and clears any error that occurred asynchronously after the
    /// last `start_native`/`start_display` call already returned `Ok`.
    pub fn take_error(&mut self) -> Option<CaptureError> {
        self.backend.take_error()
    }

    pub fn stop_capture(&mut self) {
        self.backend.stop();
        if let Ok(mut guard) = self.latest_frame.lock() {
            *guard = None;
        }
    }

    pub fn is_running(&self) -> bool {
        self.backend.is_running()
    }

    pub fn get_latest_frame_if_newer(&self, last_seen_id: u64) -> Option<(u64, ColorImage)> {
        // Under the lock only the sequence id is checked and the `Arc` is
        // cloned (a refcount bump); the expensive pixel-buffer clone happens
        // after the lock is released so the capture worker is never blocked
        // by a UI-side copy.
        let latest = self.latest_frame.lock().ok().and_then(|guard| {
            guard.as_ref().and_then(|(frame_id, img)| {
                (*frame_id > last_seen_id).then(|| (*frame_id, img.clone()))
            })
        })?;
        Some((latest.0, (*latest.1).clone()))
    }
}

impl Default for ScreenCapturer {
    fn default() -> Self {
        Self::new()
    }
}

/// Fallback backend for platforms without a native screen capture
/// implementation: every `start` fails cleanly with
/// [`CaptureError::UnsupportedPlatform`] instead of referencing a
/// platform-gated backend that does not compile there.
#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
pub struct NullBackend;

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
impl NullBackend {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
impl ScreenCaptureBackend for NullBackend {
    fn start(
        &mut self,
        _sink: FrameSink,
        _display: Option<&DisplayInfo>,
    ) -> Result<(), CaptureError> {
        Err(CaptureError::UnsupportedPlatform(format!(
            "Screen capture is not supported on {}",
            std::env::consts::OS
        )))
    }

    fn stop(&mut self) {}

    fn is_running(&self) -> bool {
        false
    }
}

#[cfg(target_os = "linux")]
fn probe_portal_available() -> bool {
    // 1. Check D-Bus for org.freedesktop.portal.Desktop
    if let Ok(conn) = zbus::blocking::Connection::session() {
        let reply = conn.call_method(
            Some("org.freedesktop.DBus"),
            "/org/freedesktop/DBus",
            Some("org.freedesktop.DBus"),
            "NameHasOwner",
            &("org.freedesktop.portal.Desktop",),
        );
        if let Ok(msg) = reply {
            if let Ok(has_owner) = msg.body().deserialize::<bool>() {
                if has_owner {
                    return true;
                }
            }
        }
    }

    // 2. Check if running under Wayland
    std::env::var("WAYLAND_DISPLAY").is_ok()
}
