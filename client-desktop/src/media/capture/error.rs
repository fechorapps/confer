use thiserror::Error;

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("Camera device error: {0}")]
    CameraDevice(String),
    #[error("Camera stream error: {0}")]
    CameraStream(String),
    #[error("Portal negotiation error: {0}")]
    PortalFailed(String),
    #[error("Screen capture was cancelled by user")]
    PortalCancelled,
    #[error("PipeWire error: {0}")]
    PipeWire(String),
    #[error("X11 error: {0}")]
    X11(String),
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
