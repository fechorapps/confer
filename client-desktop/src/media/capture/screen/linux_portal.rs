use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use ashpd::desktop::{
    screencast::{CursorMode, Screencast, SelectSourcesOptions, SourceType},
    PersistMode,
};
use egui::ColorImage;
use pipewire::context::ContextRc;
use pipewire::main_loop::MainLoopRc;
use pipewire::properties::properties;
use pipewire::spa::pod::Pod;
use pipewire::spa::utils::Direction;
use pipewire::stream::{StreamFlags, StreamRc};
use tokio::runtime::Runtime;

use crate::media::capture::convert;
use crate::media::capture::error::CaptureError;
use crate::media::capture::join_with_timeout;
use crate::media::capture::screen::{DisplayInfo, FrameSink, ScreenCaptureBackend};

pub struct PortalBackend {
    is_running: Arc<AtomicBool>,
    last_error: Arc<Mutex<Option<CaptureError>>>,
    worker_handle: Option<thread::JoinHandle<()>>,
}

impl PortalBackend {
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(false)),
            last_error: Arc::new(Mutex::new(None)),
            worker_handle: None,
        }
    }
}

impl Default for PortalBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenCaptureBackend for PortalBackend {
    fn start(&mut self, sink: FrameSink, _display: Option<&DisplayInfo>) -> Result<(), CaptureError> {
        self.stop();

        let is_running = Arc::new(AtomicBool::new(false));
        self.is_running = is_running.clone();
        let last_error = Arc::new(Mutex::new(None));
        self.last_error = last_error.clone();

        let handle = thread::spawn(move || {
            let rt = match Runtime::new() {
                Ok(rt) => rt,
                Err(e) => {
                    store_error(&last_error, CaptureError::PortalFailed(e.to_string()));
                    return;
                }
            };

            let node_id = rt.block_on(async {
                let cast = Screencast::new()
                    .await
                    .map_err(|e| CaptureError::PortalFailed(format!("Portal service unavailable: {e}")))?;

                let session = cast
                    .create_session(Default::default())
                    .await
                    .map_err(|e| CaptureError::PortalFailed(format!("Failed to create portal session: {e}")))?;

                cast.select_sources(
                    &session,
                    SelectSourcesOptions::default()
                        .set_cursor_mode(CursorMode::Metadata)
                        .set_sources(SourceType::Monitor | SourceType::Window)
                        .set_multiple(false)
                        .set_persist_mode(PersistMode::DoNot),
                )
                .await
                .map_err(|e| CaptureError::PortalFailed(format!("Failed to select sources: {e}")))?;

                let start_response = cast
                    .start(&session, None, Default::default())
                    .await
                    .map_err(|e| match e {
                        ashpd::Error::Response(ashpd::desktop::ResponseError::Cancelled) => {
                            CaptureError::PortalCancelled
                        }
                        other => CaptureError::PortalFailed(format!("Portal start failed: {other}")),
                    })?;

                let response = start_response
                    .response()
                    .map_err(|e| match e {
                        ashpd::Error::Response(ashpd::desktop::ResponseError::Cancelled) => {
                            CaptureError::PortalCancelled
                        }
                        other => CaptureError::PortalFailed(format!(
                            "Failed to read portal start response: {other}"
                        )),
                    })?;

                let stream = response
                    .streams()
                    .first()
                    .ok_or_else(|| CaptureError::PortalFailed("No video stream returned from portal".to_string()))?;

                Ok::<u32, CaptureError>(stream.pipe_wire_node_id())
            });

            let node_id = match node_id {
                Ok(id) => id,
                Err(e) => {
                    store_error(&last_error, e);
                    return;
                }
            };

            is_running.store(true, Ordering::Relaxed);
            capture_pipewire_loop(node_id, sink, is_running);
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
        self.last_error.lock().ok().and_then(|mut guard| guard.take())
    }
}

fn store_error(slot: &Arc<Mutex<Option<CaptureError>>>, error: CaptureError) {
    tracing::warn!("Portal screencast negotiation failed: {error}");
    if let Ok(mut guard) = slot.lock() {
        *guard = Some(error);
    }
}

impl Drop for PortalBackend {
    fn drop(&mut self) {
        self.stop();
    }
}

struct StreamUserData {
    sink: FrameSink,
    is_running: Arc<AtomicBool>,
    width: usize,
    height: usize,
    frame_seq: u64,
}

fn capture_pipewire_loop(node_id: u32, sink: FrameSink, is_running: Arc<AtomicBool>) {
    pipewire::init();

    let mainloop = match MainLoopRc::new(None) {
        Ok(m) => m,
        Err(e) => {
            tracing::error!("PipeWire MainLoop creation failed: {e}");
            return;
        }
    };

    let context = match ContextRc::new(&mainloop, None) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("PipeWire Context creation failed: {e}");
            return;
        }
    };

    let core = match context.connect_rc(None) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("PipeWire Core connect failed: {e}");
            return;
        }
    };

    let props = properties! {
        *pipewire::keys::MEDIA_TYPE => "Video",
        *pipewire::keys::MEDIA_CATEGORY => "Capture",
        *pipewire::keys::MEDIA_ROLE => "Screen",
    };

    let stream = match StreamRc::new(core, "confer-screencast", props) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("PipeWire Stream create failed: {e}");
            return;
        }
    };

    let user_data = StreamUserData {
        sink,
        is_running: is_running.clone(),
        width: 1920,
        height: 1080,
        frame_seq: 0,
    };

    let mainloop_weak = mainloop.downgrade();
    let state_mainloop_weak = mainloop.downgrade();

    let _listener = stream
        .add_local_listener_with_user_data(user_data)
        .state_changed(move |_stream, _user_data, _old, new| {
            tracing::debug!("PipeWire screencast stream state changed: {:?}", new);
            match new {
                pipewire::stream::StreamState::Error(ref msg) => {
                    tracing::warn!("PipeWire screencast stream error: {msg}");
                    if let Some(ml) = state_mainloop_weak.upgrade() {
                        ml.quit();
                    }
                }
                pipewire::stream::StreamState::Unconnected => {
                    if let Some(ml) = state_mainloop_weak.upgrade() {
                        ml.quit();
                    }
                }
                _ => {}
            }
        })
        .param_changed(|_stream, user_data, id, param| {
            if id == pipewire::spa::param::ParamType::Format.as_raw() {
                if let Some(param) = param {
                    if let Some((w, h)) = parse_video_dimensions(param) {
                        user_data.width = w;
                        user_data.height = h;
                    }
                }
            }
        })
        .process(move |stream, user_data| {
            if !user_data.is_running.load(Ordering::Relaxed) {
                if let Some(ml) = mainloop_weak.upgrade() {
                    ml.quit();
                }
                return;
            }

            if let Some(mut buffer) = stream.dequeue_buffer() {
                let datas = buffer.datas_mut();
                if let Some(data) = datas.first_mut() {
                    let chunk_size = data.chunk().size() as usize;
                    if let Some(slice) = data.data() {
                        let w = user_data.width.max(1);
                        let h = user_data.height.max(1);
                        let expected_size = w * h * 4;

                        let (actual_w, actual_h) = if chunk_size >= expected_size {
                            (w, h)
                        } else if chunk_size >= 4 {
                            let total_px = chunk_size / 4;
                            if total_px >= w {
                                (w, total_px / w)
                            } else {
                                (w, h)
                            }
                        } else {
                            (w, h)
                        };

                        if let Some(pixels) = convert::bgrx_to_color32(slice, actual_w, actual_h) {
                            user_data.frame_seq += 1;
                            if let Ok(mut guard) = user_data.sink.lock() {
                                *guard = Some((user_data.frame_seq, Arc::new(ColorImage { size: [actual_w, actual_h], pixels })));
                            }
                        }
                    }
                }
            }
        })
        .register();

    let flags = StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS;
    if let Err(e) = stream.connect(Direction::Input, Some(node_id), flags, &mut []) {
        tracing::error!("Failed to connect PipeWire stream to node {node_id}: {e}");
        return;
    }

    mainloop.run();
}

fn parse_video_dimensions(param: &Pod) -> Option<(usize, usize)> {
    use pipewire::spa::param::format::FormatProperties;
    use pipewire::spa::utils::Id;

    if let Ok(object) = param.as_object() {
        if let Some(prop) = object.find_prop(Id(FormatProperties::VideoSize.0)) {
            if let Ok(rect) = prop.value().get_rectangle() {
                if rect.width > 0 && rect.height > 0 {
                    return Some((rect.width as usize, rect.height as usize));
                }
            }
        }
    }
    None
}
