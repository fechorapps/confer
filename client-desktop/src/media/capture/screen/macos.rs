use crate::media::capture::error::CaptureError;
use crate::media::capture::screen::{DisplayInfo, FrameSink, ScreenCaptureBackend};

pub struct MacOsBackend;

impl MacOsBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MacOsBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenCaptureBackend for MacOsBackend {
    fn start(&mut self, _sink: FrameSink, _display: Option<&DisplayInfo>) -> Result<(), CaptureError> {
        Err(CaptureError::UnsupportedPlatform(
            "macOS ScreenCaptureKit capture is planned for phase 3".to_string(),
        ))
    }

    fn stop(&mut self) {}

    fn is_running(&self) -> bool {
        false
    }
}
