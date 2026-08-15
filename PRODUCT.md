# Product

<!-- impeccable:product-schema 1 -->

## Platform

adaptive

## Stack

- **Backend**: .NET 10 Minimal APIs, SIPSorcery SFU WebRTC Engine (VP8 simulcast with hysteresis, RFC 6464 active speaker detection), WebSocket signaling (`confer.v1` protocol), EF Core (PostgreSQL / SQLite), Redis presence & pub/sub, HMAC-SHA256 webhooks, RFC 5545 iCalendar.
- **Desktop Client**: Native Rust (`egui`/`eframe` GPU immediate mode, `tokio`, `cpal` audio, `v4l2`/`pipewire`/`ashpd` screen and webcam capture, `nnnoiseless` RNNoise 48kHz denoiser, `aes-gcm` RFC SFrame E2EE cipher) with < 40 MB RAM footprint.
- **Mobile Client**: Native Android Kotlin (`Jetpack Compose`, `Ktor 3.x`, `Material 3`, WebSockets).
- **Web Client**: Ultra-lightweight Vanilla JS/CSS client with WebRTC media streaming and Web Speech API subtitles.

## Users

- **Meeting Host / Organizer**: Creates meetings, shares PIN codes, manages waiting rooms, sets room security policies (lock meeting, mute on entry, disable screen share, toggle watermarks), moderates participants (mute, kick), starts recordings, broadcasts to RTMP, and generates AI meeting summaries.
- **Team Participants / Collaborators**: Joins meetings instantly via 6-character codes, shares video/audio with neural noise suppression, shares screen with zero delay, draws on collaborative whiteboard, votes in live polls, raises hand, sends emoji reactions, in-call chat, and views real-time captions.
- **Developers / Integrators**: Subscribes to signed HMAC-SHA256 event webhooks (`meeting.started`, `participant.joined`, `poll.created`, `recording.completed`), generates `.ics` calendar files, and integrates with the REST / WebSocket API.

## Product Purpose

Confer is an ultra-lightweight, high-performance video conferencing and collaboration platform designed for engineering teams and privacy-conscious organizations. It eliminates the heavy memory bloat and sluggishness of Electron/Chromium-based tools by providing native, sub-40MB RAM immediate-mode clients coupled with a robust, Clean Architecture .NET WebRTC SFU media backend.

## Positioning

Unlike mass-market conferencing apps that consume 500MB–1.5GB of RAM and enforce proprietary cloud lock-in, Confer delivers:
1. **Sub-40MB RAM Native Footprint**: Instant startup and near-zero CPU idle overhead on Linux, Windows, and macOS.
2. **Zero-Knowledge E2EE Media Security**: End-to-end frame-level encryption (RFC SFrame AES-128/256-GCM) where media payloads are unreadable even by the SFU routing server.
3. **All-in-One Engineering Collaboration**: Native GPU whiteboard, live polls, AI meeting summaries with action items, and live streaming built right into the core protocol.

## Operating Context

- **Use Cases**: Daily standups, architecture deep-dives, sprint retrospectives, live design reviews, pair programming, and private enterprise meetings.
- **Environments**: Linux (Wayland & X11 with PipeWire/PulseAudio/ALSA), Android devices, and modern web browsers.

## Capabilities and Constraints

### Media & Transports
- WebRTC SFU with VP8 simulcast layer selection (High, Medium, Low) with 2-second switching hysteresis.
- RFC 6464 client-to-mixer audio levels and server-side active speaker detection.
- Sub-30ms hardware video capture with dynamic color tone filters and Gaussian blur virtual backgrounds.
- Opus audio streaming with RNNoise 48kHz neural noise suppression.
- Screen sharing via PipeWire portal (Wayland) and XSHM (X11) with aspect-ratio preserving display.

### Collaboration & Governance
- **Collaborative Whiteboard**: Multi-tool GPU immediate-mode drawing canvas (Pen, Line, Rect, Circle, Text, Eraser) synced in real-time.
- **Live Polls**: Single and multi-choice polls with anonymous voting option and live percentage bars.
- **AI Meeting Summaries**: Automated extraction of meeting overview, key decisions, and action items with assignees from chat transcripts and session data.
- **Host Security Controls**: Waiting room lobby queue with admit/reject, meeting lock, visual DLP watermarking, and participant moderation.
- **Captions & Live Subtitles**: Real-time streaming speech-to-text chunk delivery.
- **Recordings & RTMP Streaming**: Archival to local storage and live broadcast to YouTube/Twitch/custom RTMP endpoints.

## Brand Commitments

- **Visual World**: Obsidian & Deep Zinc (`#0B0C0E` base, `#121418` cards, `#0284C7` electric blue accents, `#10B981` emerald active speaker halos).
- **Tone**: Focused, minimalist, precise, engineering-first, distraction-free.

## Product Principles

1. **Native Performance First**: Zero Electron overhead; the client must remain lightweight (<40MB RAM) and responsive at 60 FPS.
2. **Privacy & Security by Default**: Room tokens expire in 120s, media supports zero-knowledge SFrame E2EE encryption, and host security policies are strictly enforced.
3. **Frictionless Collaboration**: 1-click room entry with 6-character codes, instant Push-to-Talk spacebar hotkey, and synchronized whiteboard/polls.
4. **Resilient & Observable**: Real-time diagnostics HUD showing sub-second RTT latency, packet loss %, FPS, and memory footprint.
