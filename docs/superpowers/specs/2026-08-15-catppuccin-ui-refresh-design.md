# Confer Desktop — Catppuccin Mocha Reskin + UI Library Additions

Status: approved by user, ready for implementation planning.

## Overview

`client-desktop/` renders almost entirely with hand-drawn `egui::Frame`s and
literal `Color32` values — it does not lean on egui's stock widget styling.
This spec covers:

1. Re-skinning the existing design-token system (`src/ui/theme.rs`) to the
   Catppuccin Mocha palette, and propagating that change through every
   screen that currently duplicates token values as raw literals instead of
   referencing `Theme::*`.
2. Adding four third-party egui crates to fill gaps the survey turned up:
   `egui-phosphor` (icons), `egui-toast` (notifications), `egui_commonmark`
   (chat markdown), `egui-file-dialog` (added per explicit user request,
   not wired to a flow yet — see §6).

Crates surveyed and explicitly **excluded**: `egui_dock`/`egui_tiles`
(docking — no use case in a video-call UI), `egui_table`/`egui-data-table`
(spreadsheet-style tables — no use case), `egui_plot` (charts — no current
need), `egui_taffy`/`egui_flex` (CSS-style layout engines — the app's
layout is already hand-built and working), `egui-notify`/`egui_file`
(redundant with the chosen `egui-toast`/`rfd` respectively).

## Goals

- Ship a real, verifiable visual change (Catppuccin Mocha) across the whole
  app, not just the handful of screens that consume `Theme::*` directly.
- Fix the pre-existing duplication where ~160 call sites hand-copy `Theme`
  values as raw `Color32::from_rgb(...)` literals, so future palette or
  token changes propagate from one place.
- Replace remaining emoji glyphs with a proper icon font.
- Add a toast queue for transient feedback that doesn't exist today.
- Render chat messages as markdown instead of plain text.

## Non-goals

- Recoloring content/data colors that aren't part of the UI chrome (the
  whiteboard pen color palette in `whiteboard.rs` lines 42-47: white, red,
  green, blue, yellow, orange). These are user-selectable drawing colors,
  not theme colors, and stay as-is.
- Wiring `egui-file-dialog` into an actual UI flow. `rfd::FileDialog`
  already covers the one existing file-picker use case (`app.rs:324`,
  virtual background image) with the native OS dialog, which is better UX
  on desktop. `egui-file-dialog` is added as a dependency with a thin,
  unused-for-now wrapper so it's ready when a real embedded-picker use case
  (e.g. chat file sharing) exists. Do not remove or replace the `rfd` call.
- Restyling egui's stock widget visuals beyond the one `set_theme()` call
  described in §1 (no custom widget skinning work).

## Design

### 1. Palette mapping (Theme → Catppuccin Mocha)

Verified against `catppuccin-egui` v5.7.0 (`themes.rs`), feature `egui29`,
compatible with the pinned `egui = "0.29.1"`.

| `Theme` constant | Catppuccin Mocha field | Value |
|---|---|---|
| `CANVAS` | `crust` | `#11111B` (17,17,27) |
| `SURFACE_1` | `mantle` | `#181825` (24,24,37) |
| `SURFACE_2` | `surface0` | `#313244` (49,50,68) |
| `SURFACE_3` | `surface1` | `#45475A` (69,71,90) |
| `SURFACE_DOCK` | `mantle` at existing alpha (250) | — |
| `BORDER_SUBTLE` | `overlay0` | `#6C7086` (108,112,134) |
| `BORDER_ACTIVE` | `blue` | `#89B4FA` (137,180,250) |
| `PRIMARY` | `blue` | `#89B4FA` |
| `PRIMARY_HOVER` | **derived**: `blue` blended 15% toward `crust` | computed |
| `PRIMARY_LIGHT` | `sapphire` | `#74C7EC` (116,199,236) |
| `EMERALD` | `green` | `#A6E3A1` (166,227,161) |
| `EMERALD_LIGHT` | `teal` | `#94E2D5` (148,226,213) |
| `CRIMSON` | `red` | `#F38BA8` (243,139,168) |
| `CRIMSON_HOVER` | **derived**: `red` blended 15% toward `crust` | computed |
| `CRIMSON_LIGHT` | `flamingo` | `#F2CDCD` (242,205,205) |
| `AMBER` | `peach` | `#FAB387` (250,179,135) |
| `AMBER_LIGHT` | `yellow` | `#F9E2AF` (249,226,175) |
| `TEXT_PRIMARY` | `text` | `#CDD6F4` (205,214,244) |
| `TEXT_SECONDARY` | `subtext1` | `#BAC2DE` (186,194,222) |
| `TEXT_MUTED` | `subtext0` | `#A6ADC8` (166,173,200) |
| `TEXT_DIM` | `overlay1` | `#7F849C` (127,132,156) |

`PRIMARY_HOVER`/`CRIMSON_HOVER` have no direct Catppuccin token (the
palette is flat, no built-in hover variants) — derive them with a small
channel-wise `lerp(color, Theme::CANVAS_TARGET, 0.15)` helper added to
`theme.rs`, rather than picking an unrelated named color that would drift
the hue.

Also call `catppuccin_egui::set_theme(ctx, catppuccin_egui::MOCHA)` once
during `eframe::App` setup so native egui widget chrome (sliders,
checkboxes, text-edit cursor, scrollbars) matches the custom frames.

### 2. Two-step propagation (not a single 180-site edit)

Grepped counts of raw `Color32::from_rgb`/`from_rgba_*` outside
`theme.rs`: `polls.rs` 42, `whiteboard.rs` 29, `lobby.rs` 23,
`meeting_room.rs` 26, `waiting_lobby.rs` 13, `captions.rs` 13, `roster.rs`
4, `chat.rs` 3, `app.rs` 3, `components.rs` 1, `watermark.rs` 1.
Cross-referencing values against `theme.rs`'s existing 20 documented RGB
triples shows most of these are literal duplicates of `Theme::*` values
(e.g. `(18,20,23)` == `SURFACE_1`, `(2,132,199)` == `PRIMARY`,
`(148,163,184)` == `TEXT_SECONDARY`).

- **Step A — dedupe (behavior-preserving):** in each of the 11 files, find
  every `Color32::from_rgb(r,g,b)` whose triple exactly matches a current
  `Theme` constant and replace it with `Theme::CONSTANT`. No visual change;
  verify by confirming the replaced constant's current value equals the
  literal it replaced. Anything that doesn't match a `Theme` constant
  exactly is left as a literal for step C to classify.
- **Step B — retarget `Theme`:** apply the §1 mapping to the 22 constants
  in `theme.rs` (plus `set_theme()` call at startup). This is the one
  commit where the app's look actually changes, and it changes everywhere
  because of step A.
- **Step C — classify leftover literals:** for any literal that survived
  step A unmatched, decide per-site whether it's UI chrome (add a new
  semantic `Theme` constant if none fits, or map to the nearest step-A
  constant) or content color (leave alone — e.g. the whiteboard pen
  palette, and the two accent colors already blended per-pixel in
  `virtual_background.rs`/`filters.rs` if any turn up, which are pixel
  processing, not UI).

### 3. Icons — `egui-phosphor` v0.13.0

`cargo add --dry-run` confirms `regular` feature set, compatible with
`egui 0.29.1`. Integration:
- Call `egui_phosphor::add_to_fonts()` (or equivalent font-registration
  helper) once during `eframe::App` font setup.
- Replace emoji occurrences found in `controls.rs` (12+), `lobby.rs` (4),
  `waiting_lobby.rs` (3), `roster.rs` (2), `whiteboard.rs` (2), `polls.rs`
  (2), `meeting_room.rs` (2) with the matching `egui_phosphor::regular::*`
  constant, keeping the same `format!("{icon} label")` string-interpolation
  pattern already in use — no widget restructuring needed.

### 4. Toasts — `egui-toast` v0.22.0

New, additive subsystem — does not replace `error_message`. Add a
`Toasts` instance to `ConferApp`, call its `.show(ctx)` once per frame in
`update()`, and add a small `push_toast(kind, message)` helper. First
consumers (all currently silent/missing feedback):
- "Join code copied" (lobby)
- Participant joined/left
- Screen share started/stopped/failed (currently only sets
  `error_message` on failure — keep that for the blocking case, but the
  success/stop paths get a toast instead of nothing)
- Host action taken on you (muted, kicked warning, waiting-room admit)

### 5. Markdown — `egui_commonmark` v0.25.0

Swap `ui.label(text)` for message bodies in `chat.rs` for a
`CommonMarkViewer` render. Scoped to `chat.rs` only.

### 6. File dialog — `egui-file-dialog` v0.15.0

Added as a dependency per explicit user decision, with a thin wrapper
module (e.g. `src/ui/file_picker.rs`) exposing a reusable
`egui_file_dialog::FileDialog` instance, but **not** wired into any
existing screen — `rfd::FileDialog` stays as the virtual-background
picker. This crate has no consumer until a future embedded-picker use
case (e.g. chat file sharing) exists.

## Files touched (expected)

- `client-desktop/Cargo.toml` — 5 new dependencies.
- `client-desktop/src/ui/theme.rs` — retargeted constants, hover-blend
  helper, `set_theme()` call site reference.
- `client-desktop/src/app.rs` — font/theme setup at startup, `Toasts`
  field + per-frame `.show()`, toast calls at the feedback points listed
  in §4.
- `client-desktop/src/ui/{controls,lobby,waiting_lobby,roster,whiteboard,
  polls,meeting_room,captions,chat,components,watermark}.rs` — step A
  dedupe + step C classification; `controls.rs`/`lobby.rs`/etc. also get
  icon-constant swaps per §3.
- `client-desktop/src/ui/chat.rs` — commonmark rendering.
- `client-desktop/src/ui/file_picker.rs` — new, thin wrapper (§6).

## Rollout / testing order

1. Step A (dedupe) alone, per file, `cargo build` + visual sanity check —
   zero diff expected. Cheap to verify, de-risks step B.
2. Step B (retarget `Theme` + `set_theme()`) as one commit — this is the
   single point where the whole app visibly changes.
3. Step C (classify leftover literals) per file.
4. Icons, toasts, markdown, file-dialog dependency as independent
   commits/slices — each buildable and reviewable on its own, no
   ordering dependency between them or on steps A-C.

## Risks

- `catppuccin-egui`, `egui-phosphor`, `egui-toast`, `egui_commonmark`,
  `egui-file-dialog` are third-party crates version-pinned to specific
  egui minor releases; each `cargo add` must be dry-run first (as done in
  this spec) to confirm the resolved version still matches `egui 0.29.1`
  before landing.
- Step A's literal→constant matching must be exact-value, not
  visually-close — an approximate match would silently change the
  pre-Catppuccin baseline and make step B's diff impossible to verify
  cleanly.
- `egui-file-dialog` will sit unused in the dependency graph — flagged
  explicitly above so it isn't mistaken for dead code and removed later
  without checking this spec first.
