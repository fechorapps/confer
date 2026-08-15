# Product

<!-- impeccable:product-schema 1 -->

## Platform

adaptive

## Stack

- **Backend**: .NET 10 Minimal APIs, SIPSorcery SFU WebRTC Engine, WebSocket signaling (`confer.v1`), EF Core, Redis.
- **Desktop Client**: Native Rust (`egui`/`eframe`, `tokio`, `v4l2`, `image`) with zero Electron/Chromium overhead (< 40 MB RAM).
- **Mobile Client**: Native Android Kotlin (`Jetpack Compose`, `Ktor 3.x`, `Material 3`).

## Purpose & Positioning

Confer is an ultra-lightweight, high-performance video conferencing platform designed for engineering teams and internal deployments. It combines the low resource footprint of native clients (< 40 MB RAM on desktop) with enterprise-grade WebRTC SFU media routing, sub-30ms hardware camera capture, and sleek glassmorphic UI craft.

## Users & Jobs

- **Host User**: Creates secure meetings, shares join codes, moderates calls (mute participants, lock room), monitors call health diagnostics.
- **Participants**: Join by 6-character room codes, share live video & audio, raise hand, send instant reactions and in-call chat.

## Key Surfaces & Workflows

1. **Lobby & Pre-call Cockpit**: Live webcam feed preview, microphone audio level meter, user identity selection, server connectivity, meeting creation & join by code.
2. **Meeting Stage & Grid**: Auto-fitting responsive video grid with active speaker green glow halos, screen share spotlight, participant status chips.
3. **Floating Control Dock**: Mute/unmute mic, toggle video, screen share, hand raise, floating emoji reaction animations, panel toggles, leave meeting.
4. **Side Panels & Drawers**: Real-time message stream with sender badges, participant roster with host moderation controls.
5. **Diagnostics HUD**: Sub-second latency RTT, packet loss %, framerate (FPS), and process memory footprint.
