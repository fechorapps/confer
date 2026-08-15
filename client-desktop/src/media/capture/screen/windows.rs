use crate::media::capture::error::CaptureError;
use crate::media::capture::screen::{DisplayInfo, FrameSink, ScreenCaptureBackend};

pub struct WindowsBackend;

impl WindowsBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WindowsBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenCaptureBackend for WindowsBackend {
    fn start(
        &mut self,
        _sink: FrameSink,
        _display: Option<&DisplayInfo>,
    ) -> Result<(), CaptureError> {
        Err(CaptureError::UnsupportedPlatform(
            "Windows DXGI Desktop Duplication capture is planned for phase 2".to_string(),
        ))
    }

    fn stop(&mut self) {}

    fn is_running(&self) -> bool {
        false
    }
}
