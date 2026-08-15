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
- **Native Audio**: Powered by **`cpal`** (PulseAudio/ALSA/PipeWire) with Opus audio streaming.
- **Real-time Protocol**: Async WebSocket client communicating via `confer.v1` protocol.
- **UI Views**:
  - **Lobby View**: Server selector, user accounts, audio input meter, meeting creator / join code.
  - **Meeting Room View**: Auto-fitting responsive video grid, active speaker highlighting, screen share view.
  - **Bottom Dock**: Mic, camera, screen share, hand raise, emoji reactions, chat toggle, roster toggle, diagnostics HUD toggle, leave call.
  - **Side Panels**: Live in-call chat and participant roster with host moderation (Mute, Kick, Lock).
  - **Diagnostics HUD**: Latency RTT, packet loss %, FPS, memory footprint in MB.

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
