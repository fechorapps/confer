# Zero-Delay Hardware Capture Pipeline — Design

## Problem

`client-desktop/src/media/camera.rs` and `client-desktop/src/media/screen.rs`
both capture hardware video by spawning `ffmpeg` as a subprocess, piping MJPEG
bytes over its stdout, and manually scanning the byte stream for JPEG
SOI/EOI markers to extract frames. Despite in-code comments describing this
as "zero-latency"/"unbuffered," the approach carries real, avoidable
overhead:

- Subprocess spawn/kill churn, including a `pkill -9` hack in
  `camera.rs::capture_loop` to clear stale processes holding `/dev/video0`.
- A forced JPEG encode (by the driver or `ffmpeg`) followed by a JPEG decode
  (via the `image` crate) even when the source could deliver raw frames.
- A linear O(n) byte-scan for frame markers on every read chunk.
- No zero-copy path: every frame is copied from kernel buffer → pipe →
  userspace `Vec<u8>` → decoded `Vec<Color32>`.

This document specifies a replacement built on native, per-platform hardware
capture APIs, targeting minimal-copy delivery from driver to render surface.

## Goals

- Eliminate the `ffmpeg`/`v4l2-ctl`/`pkill`/`xrandr` subprocess dependency
  from the capture path.
- Use true zero-copy or minimal-copy capture primitives per platform
  (V4L2 mmap buffers, PipeWire DMA-BUF, DXGI Desktop Duplication GPU
  textures, ScreenCaptureKit IOSurfaces).
- Design the abstraction for three platforms (Linux, Windows, macOS); build
  and ship only the Linux backend in this phase. Windows/macOS are stubbed
  with documented future implementations.
- Preserve the existing consumer-facing API
  (`CameraCapturer`, `ScreenCapturer`, `get_latest_frame_if_newer`) so
  `ui/lobby.rs`, `ui/meeting_room.rs`, `app.rs` need no changes beyond the
  screen-share UI flow described below.

## Non-goals

- GPU-side YUV→RGB color conversion (fragment shader in the egui/wgpu paint
  callback). Flagged as a future optimization; this phase keeps the existing
  CPU `rayon`-parallel conversion.
- Windows and macOS implementations (design only, not built).

## Architecture

New module tree, replacing the current flat `media/camera.rs` +
`media/screen.rs`:

```
client-desktop/src/media/capture/
  camera.rs              // CameraCapturer (nokhwa-backed)
  screen/
    mod.rs                // ScreenCaptureBackend trait, ScreenCapturer dispatcher
    linux_portal.rs        // ashpd + pipewire-rs, DMA-BUF (primary Linux backend)
    linux_xshm.rs          // x11rb + XShm (fallback Linux backend)
    windows.rs              // stub, #[cfg(target_os = "windows")]
    macos.rs                // stub, #[cfg(target_os = "macos")]
```

`media/mod.rs` gains `pub mod capture;` in place of `pub mod camera;` /
`pub mod screen;` (the public types re-export from the new locations so
existing `use crate::media::{camera::CameraCapturer, screen::ScreenCapturer}`
call sites only need an import path update, not a behavior change).

### Screen capture backend trait

```rust
pub trait ScreenCaptureBackend: Send {
    /// Starts capture. On the portal backend this triggers the compositor's
    /// native source picker; the backend has no display pre-selection input.
    fn start(&mut self, sink: FrameSink) -> Result<(), CaptureError>;
    fn stop(&mut self);
}

type FrameSink = Arc<Mutex<Option<(u64, ColorImage)>>>;
```

`ScreenCapturer` picks a concrete backend at construction time:

1. On Linux, probe for a running `org.freedesktop.portal.Desktop` D-Bus
   service. If present, use `linux_portal::PortalBackend`. If absent, fall
   back to `linux_xshm::XshmBackend`.
2. On Windows/macOS (future phases), select the platform stub.

## Camera capture (`nokhwa`)

Replaces `capture_loop`'s ffmpeg spawn with the `nokhwa` crate
(cross-platform: V4L2 on Linux, Media Foundation on Windows, AVFoundation on
macOS), used only for its Linux backend in this phase.

- Request the device's native format first (YUYV/NV12) to avoid a forced
  JPEG round-trip; fall back to MJPEG (nokhwa's built-in decoder) only if
  the device offers nothing else.
- Use nokhwa's `open_stream(callback)` push model on a dedicated thread —
  this directly replaces the current hand-rolled `thread::spawn` +
  read-loop in `capture_loop`.
- The frame callback: decode to RGB (nokhwa `decode_image::<RgbFormat>()`
  or manual YUYV→RGB conversion), then apply the *existing* background
  effect (`background::BackgroundEffect::apply`) and video filter
  (`filters::VideoFilter::apply`) pipeline unchanged, then write into the
  same `Arc<Mutex<Option<(seq, ColorImage)>>>` sink consumed by
  `get_latest_frame_if_newer`.
- Device control tuning (sharpness, contrast, saturation, backlight
  compensation, white balance — currently set via a `v4l2-ctl` subprocess
  call) moves to nokhwa's `CameraControl` API. A control unsupported by the
  connected device logs a `tracing::warn!` and is skipped; it does not block
  capture startup.
- Retry behavior: wrap `Camera::new()` / `open_stream()` in a
  backoff-and-retry loop (e.g. 300ms, capped) to replace the current
  infinite retry-with-`pkill` loop. No subprocess killing is needed since
  nokhwa owns the device handle directly.

## Screen capture (Linux)

### Primary backend: PipeWire portal + DMA-BUF (`linux_portal.rs`)

- `ashpd::desktop::screencast::ScreenCastProxy` opens a session, calls
  `select_sources` (monitor type, no pre-supplied display — the portal's own
  dialog is the picker), then `start()`. This returns a PipeWire node ID.
- `pipewire-rs` connects to that node and negotiates a buffer format,
  preferring DMA-BUF (zero-copy GPU buffer import) and falling back to an
  `SPA` memory-mapped buffer type when DMA-BUF isn't negotiable (some
  compositor/GPU driver combinations).
- A `process` callback fires per frame: for the DMA-BUF case, the buffer is
  handed off for GPU-side use (future optimization; this phase maps it to
  CPU-visible memory when a direct texture upload path isn't wired yet); for
  the mmap fallback, existing rayon-parallel RGB conversion applies. Result
  is written into the same `Arc<Mutex<Option<(seq, ColorImage)>>>` sink used
  by `screen.rs` today.
- **UX change**: `start()` invokes the compositor's native screen/window
  picker dialog. This replaces both custom dropdowns:
  - `ui/controls.rs:32-49` ("🖥 Share" `egui::menu_button` listing
    `app.available_displays`) becomes a single "Share Screen" button that
    calls the portal backend's `start()` directly — no pre-built display
    list needed for this path.
  - `ui/meeting_room.rs:138-154` ("🔄 Switch Display" dropdown) becomes a
    button that stops the current portal session and starts a new one,
    re-invoking the native picker.
  - `app.rs`'s `available_displays: Vec<DisplayInfo>` /
    `detect_displays()` remain in place but are only populated/used when the
    XSHM fallback backend is active (see below); the portal path does not
    consume them.

### Fallback backend: XShm (`linux_xshm.rs`)

- Used automatically (no user-visible toggle) when no
  `org.freedesktop.portal.Desktop` D-Bus service is detected at
  `ScreenCapturer` construction.
- Implements the same `ScreenCaptureBackend` trait using `x11rb` + the X
  SHM extension: attach a shared-memory segment, request frames into it
  directly (no subprocess, no JPEG), convert via the existing
  rayon-parallel path.
- In this fallback case only, the existing `detect_displays()` /
  `DisplayInfo` dropdown UI in `controls.rs`/`meeting_room.rs` continues to
  be shown, since XShm capture still needs an app-selected display/region.

### UI dispatch

`app.rs` / `ui/controls.rs` / `ui/meeting_room.rs` branch on which backend
`ScreenCapturer` is using (exposed via a small
`ScreenCapturer::picker_mode() -> PickerMode { Native, DisplayList }`
accessor) to decide whether to render the single "Share Screen" button or
the existing dropdown.

## Buffer / threading model (unchanged)

The existing pattern in both `camera.rs` and `screen.rs` —
`Arc<Mutex<Option<(seq, ColorImage)>>>` written by the capture thread,
read via `get_latest_frame_if_newer(last_seen_id)` from the UI thread — is
kept as-is. It is the correct pattern for this use case: always overwrite
with the newest frame (no queue, no backlog), and decouple capture-thread
delivery cadence from the UI's 60fps repaint pull. Native backends (nokhwa
callback, PipeWire `process` callback) become new producers into the same
sink structure; no consumer-side changes are needed.

## Dependencies

Add to `client-desktop/Cargo.toml`:

- `nokhwa` (camera, cross-platform; Linux backend used this phase)
- `ashpd` (portal session negotiation)
- `pipewire` (`pipewire-rs`; PipeWire stream connect/negotiate/callback)
- `x11rb` (XShm fallback backend only)

Remove at runtime (no longer shelled out to on the Linux path): `ffmpeg`,
`v4l2-ctl`, `pkill`, `xrandr` (the latter remains a soft dependency only for
the XShm fallback's `detect_displays()`).

## Error handling

- Camera: device-open failures retry with capped backoff; unsupported
  controls warn-and-skip (see above).
- Screen, portal path: if `ashpd` session negotiation fails (portal absent,
  user cancels the picker, D-Bus error), `ScreenCapturer::start()` returns
  `Err(CaptureError)`; the UI surfaces this as a toast/log rather than
  silently retrying (matches user cancel = intentional).
- Screen, backend selection: portal-probe failure at construction time is
  the sole trigger for falling back to XShm — this happens once, at
  startup, not per-capture-attempt.

## Testing

- Manual: verify camera capture on a V4L2 UVC webcam with both a
  YUYV-native and MJPEG-only device if available, confirm filter/background
  pipeline still applies correctly, confirm control tuning warnings appear
  for unsupported controls without blocking startup.
- Manual: verify screen share on a Wayland compositor with a running
  `xdg-desktop-portal` (portal path, native picker, DMA-BUF or mmap
  fallback negotiated), and on an X11 session/WM without a portal running
  (XShm fallback path, existing dropdown UI).
- Manual: verify "Switch Display" now correctly stops the current portal
  session and re-invokes the picker without leaking the previous PipeWire
  stream connection.
- No new automated test suite is proposed for the native capture paths
  themselves (they require real hardware/compositor state); the existing
  `background`/`filters` unit-testable logic is unaffected and keeps its
  current test coverage (if any).

## Rollout phases

- **Phase 1 (this plan)**: Linux only — `nokhwa` camera backend,
  `linux_portal` + `linux_xshm` screen backends, UI dispatch on picker mode.
- **Phase 2 (future, not built)**: Windows — `nokhwa` camera backend
  (Media Foundation), `win_desktop_duplication` (DXGI Desktop Duplication)
  screen backend behind the same `ScreenCaptureBackend` trait.
- **Phase 3 (future, not built)**: macOS — `nokhwa` camera backend
  (AVFoundation), `screencapturekit-rs` screen backend behind the same
  trait.
