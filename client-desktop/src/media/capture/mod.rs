pub mod camera;
pub(crate) mod convert;
pub mod error;
pub mod screen;

use std::thread;
use std::time::Duration;

pub use camera::CameraCapturer;
pub use error::CaptureError;
pub use screen::{detect_displays, DisplayInfo, FrameSink, PickerMode, ScreenCaptureBackend, ScreenCapturer};

/// Joins a worker thread but gives up after `timeout` instead of blocking the
/// caller (often the UI thread) indefinitely if the thread is stuck inside a
/// blocking hardware/IPC call. The thread is abandoned (not killed) on timeout;
/// it may still finish and release its resources later.
pub(crate) fn join_with_timeout(handle: thread::JoinHandle<()>, timeout: Duration) {
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        if handle.join().is_err() {
            tracing::error!("Capture worker thread panicked while shutting down");
        }
        let _ = done_tx.send(());
    });
    if done_rx.recv_timeout(timeout).is_err() {
        tracing::warn!(
            "Capture worker thread did not shut down within {:?}; abandoning join",
            timeout
        );
    }
}
