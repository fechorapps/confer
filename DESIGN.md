---
name: Confer
description: Ultra-lightweight native video collaboration system with Obsidian precision aesthetics
colors:
  primary: "#0284c7"
  primary-hover: "#0369a1"
  primary-light: "#38bdf8"
  canvas: "#0b0c0e"
  surface-1: "#121418"
  surface-2: "#1a1d21"
  surface-3: "#262a30"
  border-subtle: "#22262c"
  border-active: "#0284c7"
  speaker-active: "#10b981"
  destructive: "#e11d48"
  destructive-hover: "#be123c"
  warning: "#f59e0b"
  text-primary: "#f8fafc"
  text-secondary: "#94a3b8"
  text-muted: "#64748b"
typography:
  display:
    fontFamily: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
    fontSize: "20px"
    fontWeight: 700
    lineHeight: 1.2
    letterSpacing: "-0.02em"
  heading:
    fontFamily: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
    fontSize: "15px"
    fontWeight: 600
    lineHeight: 1.3
    letterSpacing: "-0.01em"
  body:
    fontFamily: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: "normal"
  caption:
    fontFamily: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif"
    fontSize: "11px"
    fontWeight: 500
    lineHeight: 1.3
    letterSpacing: "0.01em"
  mono:
    fontFamily: "'JetBrains Mono', ui-monospace, monospace"
    fontSize: "13.5px"
    fontWeight: 500
    lineHeight: 1.2
    letterSpacing: "0.05em"
rounded:
  sm: "6px"
  md: "8px"
  lg: "12px"
  xl: "16px"
  pill: "9999px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "12px"
  lg: "16px"
  xl: "20px"
  2xl: "24px"
  3xl: "36px"
components:
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.md}"
    padding: "8px 16px"
  button-primary-hover:
    backgroundColor: "{colors.primary-hover}"
  button-destructive:
    backgroundColor: "{colors.destructive}"
    textColor: "{colors.text-primary}"
    rounded: "{rounded.md}"
    padding: "8px 16px"
  button-secondary:
    backgroundColor: "{colors.surface-2}"
    textColor: "{colors.text-secondary}"
    rounded: "{rounded.md}"
    padding: "8px 14px"
  card-surface:
    backgroundColor: "{colors.surface-1}"
    rounded: "{rounded.xl}"
    padding: "20px 20px"
  dock-capsule:
    backgroundColor: "{colors.surface-1}"
    rounded: "{rounded.pill}"
    padding: "8px 16px"
---

# Design System — Confer

<!-- impeccable:design-schema 1 -->

## Overview

Confer's visual language is engineered for high-focus, distraction-free video communication. It rejects heavy glassmorphic ornament and decorative clutter in favor of an **Obsidian Precision & Deep Zinc** dark aesthetic tailored for native GPU immediate-mode rendering (`egui`/Rust) and fluid responsive cross-platform clients.

Brand identity is expressed through meticulous micro-interactions, crisp typographic hierarchy, high-contrast status feedback, and zero latency visual responsiveness.

---

## Colors

The palette is rooted in true black and deep obsidian neutrals, punctuated with functional status accents:

- **Canvas Base (`#0B0C0E`)**: The foundational deep obsidian backdrop that ensures maximum contrast with video textures.
- **Surface Elevation 1 (`#121418`)**: Primary card, panel, and floating dock container background.
- **Surface Elevation 2 (`#1A1D21`)**: Secondary controls, unselected pill badges, and input containers.
- **Surface Elevation 3 (`#262A30`)**: Hover states, interactive highlights, and active segment backgrounds.
- **Borders & Dividers**:
  - `Border Subtle`: `#22262C` (1.0px hairline separation between surfaces).
  - `Border Active / Focus`: `#0284C7` (Electric Blue outline on active card/input).
- **Functional Semantics**:
  - `Active Speaker Halo`: `#10B981` (Emerald Pulse 2.5px ring).
  - `Destructive / Disconnect`: `#E11D48` (Crimson Rose for leaving/kicking).
  - `Warning / Moderation`: `#F59E0B` (Amber Flame for room lock & hand raises).
  - `Primary Accent`: `#0284C7` (Confer Electric Blue for primary actions).

---

## Typography

Typographic hierarchy prioritizes rapid scannability, legibility across varying DPI displays, and distinct character recognition:

- **Display (`20px`, Bold, `-0.02em` tracking)**: Top application title and logomark (`CONFER PRO STUDIO`).
- **Heading (`15px` - `16px`, Semi-Bold, `-0.01em` tracking)**: Card headers, modal titles, and section dividers.
- **Body (`12.5px` - `13.5px`, Regular/Medium)**: Participant names, input text, button labels, and chat messages.
- **Caption / Meta (`10.5px` - `11.5px`, Medium, `+0.01em` tracking)**: Status indicators, stream metadata (`● 720p HD 30 FPS`), timestamps, and hotkey hints.
- **Monospace (`13.5px` - `14px`, Medium, `+0.05em` tracking)**: 6-character room codes (`[CODE: ABC123]`) and diagnostic metrics (`12ms RTT`).

---

## Layout

- **Responsive Multi-Resolution Layout**:
  - **Lobby Cockpit**: Centered max-width container (`1220px`) with 54%/46% split for Viewfinder and Action Cards on desktop, auto-stacking into a single vertical column on viewports `< 940px`.
  - **Meeting Grid**: Auto-fitting responsive video grid computing optimal column/row matrix (1x1, 2x1, 2x2, 3x2, 3x3) with 12px gutters, preserving 16:9 / 4:3 camera aspect ratios without letterbox distortion.
  - **Side Drawers**: Fixed 320px–340px right panel for Chat, Roster, and Polls that smoothly yields video grid space.
  - **Floating Dock**: Centered capsule dock hovering 14px above the viewport bottom with `padding: 8px 16px`.

---

## Elevation & Depth

Confer employs tonal surface elevation rather than artificial fuzzy drop shadows to preserve GPU rendering performance:

1. **Level 0 (Canvas)**: Obsidian `#0B0C0E`.
2. **Level 1 (Cards & Side Panels)**: Deep Zinc `#121418` with `#22262C` 1px border.
3. **Level 2 (Pills & Control Buttons)**: Slate `#1A1D21` with 6px–8px radius.
4. **Level 3 (Popovers & Modals)**: Frosted elevated Zinc `#181A20` with 1.5px crimson/blue focus stroke and 180-alpha backdrop scrim.

---

## Shapes

- **Micro Radii (`6px` - `8px`)**: Action buttons, text edit fields, status chips, and code copy pills.
- **Medium Radii (`12px` - `16px`)**: Video tile frames, presentation cards, and safety confirmation modals.
- **Pill Radii (`9999px` / `24px`)**: Floating bottom control dock, user identity avatars, and toggle pill filters.

---

## Components

### 1. Studio Viewfinder Card
- 16:9 live webcam surface with rounded GPU texture mask.
- Real-time 20-segment LED audio VU meter transitioning from Emerald to Amber to Red.
- Horizontal scrollable rails for Cinematic Color Tones and Virtual Background Modes.

### 2. Video Grid Tile
- Auto-sized tile container with 14px rounding.
- Active speaker 2.5px Emerald halo border.
- Bottom-left frosted identity pill showing name, `(You)` tag, `HOST` badge, and reactive mic icon.
- Top-right amber hand-raise badge.

### 3. Floating Control Dock
- Frosted glass capsule floating at the viewport bottom.
- Logical clustering: Media toggles (Mic, Cam, Share) → Tools (Whiteboard, Polls, CC, Hand, React) → Panels (Chat, People, Settings) → Destructive Leave.

### 4. Safety Modals
- High-contrast centered dialogs with dark backdrop scrim, keyboard accelerators (`Esc` to cancel, `Enter` to confirm), and distinct crimson warning borders.

---

## Do's and Don'ts

### Do:
- **Do** maintain a strict `< 40 MB` memory footprint by using native GPU primitives and avoiding web view wrappers.
- **Do** provide immediate visual feedback for all hotkeys (e.g. glowing PTT banner during spacebar hold).
- **Do** clamp all layout coordinates and preserve aspect ratios for screen share and video feeds.
- **Do** use monospaced fonts for room codes, latency numbers, and token identifiers.

### Don't:
- **Don't** add decorative glowing background circles, grid patterns, or neon gradients that distract from video content.
- **Don't** use compound emojis with variation selectors that generate missing glyph tofu boxes on Linux.
- **Don't** allow horizontal overflow or clipped buttons inside cards.
- **Don't** allow destructive actions (leave meeting, kick user) without explicit confirmation guards.
