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
use pipewire::spa::param::video::VideoFormat;
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
            capture_pipewire_loop(node_id, sink, is_running, &last_error);
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
    format: VideoFormat,
    frame_seq: u64,
}

fn capture_pipewire_loop(
    node_id: u32,
    sink: FrameSink,
    is_running: Arc<AtomicBool>,
    last_error: &Arc<Mutex<Option<CaptureError>>>,
) {
    pipewire::init();

    let mainloop = match MainLoopRc::new(None) {
        Ok(m) => m,
        Err(e) => {
            store_error(last_error, CaptureError::PortalFailed(format!("PipeWire MainLoop failed: {e}")));
            return;
        }
    };

    let context = match ContextRc::new(&mainloop, None) {
        Ok(c) => c,
        Err(e) => {
            store_error(last_error, CaptureError::PortalFailed(format!("PipeWire Context failed: {e}")));
            return;
        }
    };

    let core = match context.connect_rc(None) {
        Ok(c) => c,
        Err(e) => {
            store_error(last_error, CaptureError::PortalFailed(format!("PipeWire Core connect failed: {e}")));
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
            store_error(last_error, CaptureError::PortalFailed(format!("PipeWire Stream create failed: {e}")));
            return;
        }
    };

    let user_data = StreamUserData {
        sink,
        is_running: is_running.clone(),
        width: 1920,
        height: 1080,
        format: VideoFormat::BGRx,
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
            let Some(param) = param else { return; };
            if id != pipewire::spa::param::ParamType::Format.as_raw() {
                return;
            }

            let mut info = pipewire::spa::param::video::VideoInfoRaw::default();
            if info.parse(param).is_ok() {
                let size = info.size();
                if size.width > 0 && size.height > 0 {
                    user_data.width = size.width as usize;
                    user_data.height = size.height as usize;
                    user_data.format = info.format();
                    tracing::info!(
                        "Negotiated PipeWire screen share format: {:?} {}x{}",
                        info.format(),
                        size.width,
                        size.height
                    );
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

                        let pixels = match user_data.format {
                            VideoFormat::RGBA | VideoFormat::RGBx => {
                                convert::rgba_to_color32(slice, actual_w, actual_h)
                            }
                            VideoFormat::RGB => {
                                convert::rgb_to_color32(slice, actual_w, actual_h)
                            }
                            _ => {
                                convert::bgrx_to_color32(slice, actual_w, actual_h)
                                    .or_else(|| convert::rgba_to_color32(slice, actual_w, actual_h))
                            }
                        };

                        if let Some(pixels) = pixels {
                            user_data.frame_seq += 1;
                            if let Ok(mut guard) = user_data.sink.lock() {
                                *guard = Some((
                                    user_data.frame_seq,
                                    Arc::new(ColorImage {
                                        size: [actual_w, actual_h],
                                        pixels,
                                    }),
                                ));
                            }
                        }
                    }
                }
            }
        })
        .register();

    let obj = pipewire::spa::pod::object!(
        pipewire::spa::utils::SpaTypes::ObjectParamFormat,
        pipewire::spa::param::ParamType::EnumFormat,
        pipewire::spa::pod::property!(
            pipewire::spa::param::format::FormatProperties::MediaType,
            Id,
            pipewire::spa::param::format::MediaType::Video
        ),
        pipewire::spa::pod::property!(
            pipewire::spa::param::format::FormatProperties::MediaSubtype,
            Id,
            pipewire::spa::param::format::MediaSubtype::Raw
        ),
        pipewire::spa::pod::property!(
            pipewire::spa::param::format::FormatProperties::VideoFormat,
            Choice,
            Enum,
            Id,
            pipewire::spa::param::video::VideoFormat::BGRx,
            pipewire::spa::param::video::VideoFormat::BGRx,
            pipewire::spa::param::video::VideoFormat::BGRA,
            pipewire::spa::param::video::VideoFormat::RGBA,
            pipewire::spa::param::video::VideoFormat::RGBx,
            pipewire::spa::param::video::VideoFormat::RGB,
        ),
        pipewire::spa::pod::property!(
            pipewire::spa::param::format::FormatProperties::VideoSize,
            Choice,
            Range,
            Rectangle,
            pipewire::spa::utils::Rectangle {
                width: 1920,
                height: 1080
            },
            pipewire::spa::utils::Rectangle {
                width: 1,
                height: 1
            },
            pipewire::spa::utils::Rectangle {
                width: 4096,
                height: 4096
            }
        ),
        pipewire::spa::pod::property!(
            pipewire::spa::param::format::FormatProperties::VideoFramerate,
            Choice,
            Range,
            Fraction,
            pipewire::spa::utils::Fraction { num: 30, denom: 1 },
            pipewire::spa::utils::Fraction { num: 1, denom: 1 },
            pipewire::spa::utils::Fraction { num: 60, denom: 1 }
        ),
    );

    let values: Vec<u8> = match pipewire::spa::pod::serialize::PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &pipewire::spa::pod::Value::Object(obj),
    ) {
        Ok((cursor, _)) => cursor.into_inner(),
        Err(e) => {
            store_error(last_error, CaptureError::PortalFailed(format!("Failed to serialize SPA format pod: {e}")));
            return;
        }
    };

    let mut params = [match Pod::from_bytes(&values) {
        Some(p) => p,
        None => {
            store_error(last_error, CaptureError::PortalFailed("Failed to parse Pod from serialized format bytes".to_string()));
            return;
        }
    }];

    let flags = StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS;
    if let Err(e) = stream.connect(Direction::Input, Some(node_id), flags, &mut params) {
        store_error(last_error, CaptureError::PortalFailed(format!("Failed to connect PipeWire stream to node {node_id}: {e}")));
        return;
    }

    mainloop.run();
}
