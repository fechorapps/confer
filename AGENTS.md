# AGENTS.md — Confer Developer & Architecture Guide

Clean Architecture + DDD + CQRS with Minimal APIs on .NET 10, Native Rust Desktop Client & Kotlin Mobile App

## Architecture Overview

Confer is structured across three primary surfaces:

### 1. .NET Core Backend (`Confer.slnx`)
- **Domain Layer (`src/domain/`)**: Pure entities & aggregates (`Meeting`, `Session`, `Participation`, `ChatMessage`, `User`), Value Objects, Domain Events, Domain Errors (`MeetingErrors`, `SessionErrors`, `AuthErrors`).
- **Application Layer (`src/application/`)**: CQRS slices (`Meetings/Create/`, `Meetings/Join/`, `Meetings/GetByCode/`, `Meetings/Lock/`), `ICqrsDispatcher`, FluentValidation, and abstraction interfaces (`ISfuRoomManager`, `ISignalingNotifier`, `ITokenProvider`, `IPresenceService`).
- **Infrastructure Layer (`src/infrastructure/`)**:
  - `Media/`: SIPSorcery SFU WebRTC room engine (`RoomMediaSession`), VP8 simulcast layer selector with 2s hysteresis, and RFC 6464 active speaker detector.
  - `Signal/`: WebSocket signaling handler implementing `confer.v1` wire protocol.
  - `Persistence/`: EF Core `ConferDbContext` (PostgreSQL / SQLite).
  - `Caching/`: Redis presence and pub/sub.
  - `Security/`: JWT token provider (120s room tokens).
- **API Layer (`src/api/`)**: Minimal API modules (`MeetingsEndpoint`, `AuthEndpoint`) inheriting `BaseModule, IEndpoint` + WebSocket `/v1/signal` endpoint.
- **Shared Layer (`src/shared/`)**: Functional `Result`, `Result<T>`, `Error`, and `BaseModule`.

### 2. Native Rust Desktop Client (`client-desktop/`)
- **GPU Immediate Mode GUI**: Built with **`egui` / `eframe`** (< 40 MB RAM footprint on Linux, zero Electron/Chromium overhead).
- **Real-time Protocol**: Async WebSocket client communicating via `confer.v1` protocol.
- **WebRTC Engine**: `webrtc-rs` / `rtc` peer-connection engine for the SFU (publish/subscribe + trickle ICE).
- **Audio Pipeline**: `cpal` mic capture → Opus encode → publish track; remote Opus tracks decoded and mixed to playback.
- **Video Pipeline**: `nokhwa` camera capture + `ashpd`/`pipewire` screen share → I420 → VP8 encode (libvpx via a minimal C wrapper, `media/vpx_ffi/`) → publish track. Remote VP8 tracks are depacketized (RFC 7741), decoded, and rendered as `egui` textures in the meeting grid.
- **UI Views**:
  - **Lobby View**: Server selector, user accounts, audio input meter, meeting creator / join code.
  - **Meeting Room View**: Auto-fitting responsive video grid, active speaker highlighting, screen share view.
  - **Bottom Dock**: Mic, camera, screen share, hand raise, emoji reactions, chat toggle, roster toggle, diagnostics HUD toggle, leave call.
  - **Side Panels**: Live in-call chat and participant roster with host moderation (Mute, Kick, Lock).
  - **Diagnostics HUD**: Latency RTT (real, from ping/pong round-trip), packet loss %, FPS, memory footprint in MB.

> **Build note (current state):** the desktop client compiles, passes `cargo test`/`cargo clippy`, and publishes/decodes real audio (Opus) and video (VP8) end to end. Building it requires the system development libraries: on Fedora `sudo dnf install -y opus-devel libvpx-devel alsa-lib-devel` (or `libopus-dev libvpx-dev libasound2-dev` on Debian/Ubuntu), plus `cmake` and a C compiler for the libvpx wrapper. Known gaps: the VP8 RTP payload type is hard-coded to the common default (96) instead of negotiated from the SDP answer, and there's no PLI-triggered keyframe request, so a participant joining mid-call waits up to ~2s for the next automatic keyframe.

### 3. Native Kotlin Mobile App (`mobile/`)
- **UI Toolkit**: **Jetpack Compose + Material 3** with custom Confer dark design system.
- **Networking**: **Ktor 3.x** for REST API calls (`/api/meetings`, `/api/auth`) and WebSockets (`/v1/signal`).
- **Architecture**: MVI/MVVM with StateFlow (`LobbyViewModel`, `MeetingViewModel`).
- **Features**:
  - Pre-join device check and live mic meter.
  - Responsive mobile video grid (1x1, 2x1, 2x2, Stage mode).
  - Floating bottom dock with mic, camera, flip camera, screen share, reactions, chat, and roster.
  - Modal bottom sheets for live chat and participant roster with host actions.
  - Diagnostics dialog (RTT, packet loss, active layer).

---

## Developer Command Center (`Makefile`)

You can control all aspects of the project via `make`:

```bash
make help              # Show all available commands
make run-backend       # (Alias: make backend) Start .NET backend on http://localhost:5000
make watch-backend     # Start .NET backend with hot reload
make test              # Run all 9 backend automated unit & integration tests
make run-desktop       # (Alias: make desktop) Launch native Rust desktop app (<40MB RAM)
make check-desktop     # Typecheck the Rust desktop client
make build-mobile      # (Alias: make mobile) Build the Kotlin/Compose Android debug APK
make docker-up         # Start PostgreSQL, Redis, Coturn and API in Docker
make docker-down       # Stop all Docker services
make clean             # Clean all temporary build artifacts across .NET, Rust, and Gradle
```
