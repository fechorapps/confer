<div align="center">

# 🌐 Confer
### Ultra-High-Performance, Zero-Delay Real-Time Video Collaboration Platform
*Enterprise-Grade Alternative to Zoom, Google Meet & Microsoft Teams*

[![CI Pipeline](https://github.com/fechorapps/confer/actions/workflows/ci.yml/badge.svg)](https://github.com/fechorapps/confer/actions)
[![Release](https://img.shields.io/github/v/release/fechorapps/confer?include_prereleases&color=blue)](https://github.com/fechorapps/confer/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-Desktop%20%3C40MB%20RAM-DEA584?logo=rust)](client-desktop/)
[![.NET 10](https://img.shields.io/badge/.NET-10.0%20Clean%20Architecture-512BD4?logo=dotnet)](src/)
[![Kotlin](https://img.shields.io/badge/Kotlin-Compose%20Android-7F52FF?logo=kotlin)](mobile/)

[✨ Features](#-key-features) •
[⚡ Benchmark](#-industry-benchmark) •
[🚀 Quick Start](#-quick-start) •
[🏛️ Architecture](#-system-architecture) •
[📦 Packaging & Releases](#-packaging--installers) •
[☸️ Kubernetes Deploy](#-kubernetes-deployment)

---

</div>

## 📖 Overview

**Confer** is a modern, privacy-first, zero-delay video conferencing ecosystem engineered for low latency, zero Electron/Chromium desktop overhead, and infinite cloud scalability.

Built across three native surfaces:
- **🖥️ Native Desktop Client (`Rust` + `egui` + `WebRTC` + `RNNoise`)**: Pure GPU immediate mode client running on OpenGL/Vulkan with **< 40 MB RAM** memory footprint (compared to Zoom's 350+ MB and Teams' 800+ MB). Publishes and decodes real Opus audio and VP8 video end to end; building it requires system libraries (`opus-devel`, `libvpx-devel`, `alsa-lib-devel` on Fedora).
- **☁️ Backend Server (`.NET 10` + Clean Architecture + CQRS + SIPSorcery SFU)**: High-throughput media routing, WebSocket signaling (`confer.v1`), PostgreSQL/SQLite persistence, and Redis presence.
- **📱 Mobile App (`Kotlin` + `Jetpack Compose` + `Ktor 3.x`)**: Responsive Android client with custom Confer dark design system.
- **🌐 Web Client (`HTML5` + `Vanilla ES6` + `WebRTC`)**: Zero-install browser interface with Web Speech STT and hardware acceleration.

---

## ✨ Key Features

### 1. 🎨 Real-Time Collaboration & Interactivity
- **Collaborative Whiteboard**: Immediate-mode GPU canvas in Rust (`egui::Painter`) and HTML5 Canvas with Pen, Line, Rectangle, Circle, Smart Eraser, Text, color picker, and Undo.
- **Live Polls**: Interactive voting with dynamic percentage progress bars, multi-choice support, anonymous voting, and host closing.
- **Breakout Rooms**: Instant sub-room creation, timed sessions, and automatic/manual participant distribution.

### 2. 🛡️ Enterprise Governance & Security
- **Zero-Knowledge E2EE (IETF SFrame RFC)**: Real-time media payload encryption (`AES-128-GCM` / `AES-256-GCM`) with HKDF key ratcheting and RFC 6479 replay protection.
- **Interactive Waiting Room**: Branded waiting lobby with audio/video device checks and host moderation ("Admit", "Admit All", "Reject").
- **Host Security Policies**: Lock meeting, restrict screen sharing, disable chat, prevent self-unmute, and mute-on-entry.
- **DLP Visual Watermarking**: Diagonally rendered viewer email/name watermark over video tiles and screen share to prevent leaks.
- **Enterprise SSO**: Single Sign-On with Google Workspace, Microsoft Entra ID (Azure AD), Okta, Keycloak, and SAML 2.0 with PKCE security.

### 3. 🧠 Smart Audio & Real-Time AI
- **RNNoise AI Noise Suppression**: 48 kHz recurrent neural network (RNN) denoiser processing audio frames in real time with < 1% CPU utilization.
- **Virtual Background & GPU Blur**: 2D separable Gaussian blur and alpha matte compositing.
- **Live Captions & Speech-to-Text (STT)**: Streaming real-time subtitles with speaker initials, interim typing indicators (`⚡`), and auto-fade.
- **In-Room AI Copilot (`@confer-ai` / `/ai`)**: In-meeting AI companion answering questions, generating instant discussion recaps (`/ai summarize`), and extracting action items (`/ai action-items`).
- **Post-Meeting AI Summaries**: Automated executive summaries, key decisions, and assigned action item generation.

### 4. 🪝 Integrations & Broadcasting
- **Developer Webhooks**: HMAC-SHA256 cryptographically signed event notifications (`X-Confer-Signature`).
- **iCalendar (.ics) Generator**: RFC 5545 calendar invitations with one-click direct join URLs.
- **RTMP Live Streaming**: Muxed broadcast output to YouTube Live, Twitch, or private RTMP servers.
- **SIP / PSTN Dial-In Telephony**: Inbound voice bridging (Twilio / SIP Trunking) allowing phone calls into meeting audio rooms.

---

## ⚡ Industry Benchmark

| Feature / Metric | **Confer** 🚀 | **Zoom** | **Google Meet** | **Microsoft Teams** | **Jitsi Meet** |
| :--- | :---: | :---: | :---: | :---: | :---: |
| **Desktop RAM Footprint** | **< 40 MB** (Rust GPU) | ~350 MB | Web Browser | ~800 MB (Electron) | Web Browser |
| **Cifrado E2EE** | **SFrame RFC (AEAD)** | Opcional | Opcional | Opcional | Experimental |
| **AI Audio Denoise** | **RNNoise 48kHz** | Propietario | En la nube | Propietario | WebRTC NS |
| **Pizarra en GPU** | **Nativa + Web** | Webview | Descontinuado | Whiteboard App | Excalidraw plugin |
| **Subtítulos en Vivo** | **Streaming + Diarización** | En la nube | En la nube | En la nube | VOSK plugin |
| **Copiloto IA en Sala** | **Integrado (`@confer-ai`)** | AI Companion | Gemini | Copilot | No |
| **Webhooks Firmados** | **HMAC-SHA256** | Sí | Google Cloud | Graph API | No |
| **Calendario .ics** | **RFC 5545 Nativo** | Sí | Google Cal | Outlook | No |
| **PSTN / SIP Dial-In** | **TwiML / SIP Bridge** | De pago | De pago | De pago | Jigasi |
| **Licencia & Código** | **100% Open Source** | Propietario | Propietario | Propietario | Apache 2.0 |

---

## 🚀 Quick Start

### Prerequisites
- [.NET 10 SDK](https://dotnet.microsoft.com/download/dotnet/10.0)
- [Rust & Cargo (stable)](https://rustup.rs/)
- `make` and `gcc`/`clang` (Linux)

### Developer Commands (`Makefile`)

```bash
# Clone the repository
git clone https://github.com/fechorapps/confer.git
cd confer

# 1. Run backend server (.NET 10) on http://localhost:5000
make run-backend

# 2. Launch the ultra-lightweight native Rust Desktop Client
make run-desktop

# 3. Run the complete automated test suite (174 .NET + 63 Rust tests)
make test
cargo test --manifest-path client-desktop/Cargo.toml
```

---

## 🏛️ System Architecture

```
Confer/
├── src/
│   ├── domain/               # DDD Aggregates, Value Objects, Domain Events & Errors
│   ├── application/          # CQRS Slices, FluentValidation, Dispatcher Interfaces
│   ├── infrastructure/       # SFU WebRTC Engine, SFrame, AI Copilot, Webhooks, Telephony
│   ├── api/                  # Minimal APIs, WebSocket /v1/signal, and Static Web Client
│   └── shared/               # Functional Result<T>, Error models, and CQRS interfaces
├── client-desktop/           # Native Rust Immediate Mode Desktop App (egui, WebRTC, Opus/VP8, RNNoise)
├── mobile/                   # Native Kotlin / Jetpack Compose Android Application
├── deploy/
│   ├── helm/confer/          # Production Kubernetes Helm Chart (API, HPA, Ingress, Coturn)
│   └── grafana/dashboards/   # Prometheus & Grafana WebRTC Observability Dashboard
└── packaging/                # Linux .deb, tarball, and self-contained .NET packaging scripts
```

---

## 📦 Packaging & Installers

Confer provides automated build scripts for producing self-contained distribution bundles:

```bash
# Build Debian package (.deb) for Linux desktop
make package-deb

# Build standalone portable Linux tarball (.tar.gz)
make package-tarball

# Build self-contained single-file servers for Linux and Windows
make package-backend

# Build all release packages
make package-all
```

---

## ☸️ Kubernetes Deployment

Deploy Confer in production using the official Helm chart:

```bash
# Install Confer on Kubernetes with Helm
helm install confer ./deploy/helm/confer \
  --set ingress.hosts[0].host="meet.yourcompany.com" \
  --set config.jwtSecret="your-production-jwt-secret-key-32-chars"
```

---

## 📄 License

Distributed under the **MIT License**. See `LICENSE` for more information.

Built with ❤️ by **[Fechor Apps](https://github.com/fechorapps)**.
