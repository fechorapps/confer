# Catppuccin Mocha Palette Flip Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Retarget `client-desktop`'s hand-rolled `Theme` design tokens (`src/ui/theme.rs`) to the Catppuccin Mocha palette, and make that change visible across the whole app by first collapsing ~99 duplicated raw `Color32` literals (scattered across 8 other files) down to references to those same tokens.

**Architecture:** Two-step, behavior-preserving-then-visible rollout. Step A (Tasks 1-8, one per file): replace every `Color32::from_rgb(...)`/`from_rgba_*(...)` call whose value exactly matches an existing `Theme` constant with a reference to that constant — zero visual change, verified by exact-value matching. Step B (Task 9): retarget the 22 `Theme` constants themselves to Catppuccin Mocha values (sourced directly from the `catppuccin-egui` crate's `MOCHA` constant, not re-typed hex) and apply `catppuccin_egui::set_theme()` to native egui widget chrome inside the existing per-frame `update()` visuals block. Because Step A already routed everything through `Theme::*`, Step B's diff is the entire app's visible palette change in one commit.

**Tech Stack:** Rust, egui/eframe 0.29.1, `catppuccin-egui` 5.7.0 (feature `egui29`).

**Spec:** `docs/superpowers/specs/2026-08-15-catppuccin-ui-refresh-design.md` (§1 palette mapping, §2 two-step rollout — this plan implements those two sections only).

## Global Constraints

- `egui` is pinned at `0.29.1` in `client-desktop/Cargo.toml` — `catppuccin-egui` must be added with `default-features = false, features = ["egui29"]` to resolve a compatible version (verified: v5.7.0).
- Step A replacements are **exact-value matches only**. Do not "round" or approximate a literal to the nearest `Theme` constant unless it's called out explicitly in a task below — an approximate match would silently change the pre-Catppuccin baseline and make Step B's diff impossible to verify cleanly (per spec §2/Risks).
- The whiteboard pen-color palette (`src/ui/whiteboard.rs` lines 42-47: white/red/green/blue/yellow/orange) is user-facing drawing content, not UI chrome. **Never touch it in this plan.**
- Any literal that does not exactly match a `Theme` constant is out of scope for this plan (tracked as spec §2 Step C, a separate future plan) — leave it untouched.
- Every task's `cargo build` must be clean (no new errors or warnings) before its commit.

---

## Task 1: Step A dedupe — `src/app.rs`

**Files:**
- Modify: `client-desktop/src/app.rs:1-10` (imports), `client-desktop/src/app.rs:1331-1337` (`update()` visuals block)

**Interfaces:**
- Consumes: `Theme::CANVAS`, `Theme::BORDER_ACTIVE`, `Theme::SURFACE_1` (existing constants in `src/ui/theme.rs`, unchanged by this task)
- Produces: nothing new — internal literal→token substitution only

- [ ] **Step 1: Add the missing import**

`app.rs` has no `use crate::ui::theme::Theme;` today. Add it near the other `crate::ui::` import:

```rust
use crate::ui::whiteboard::{WhiteboardTool, WHITEBOARD_COLORS};
use crate::ui::{lobby, meeting_room, waiting_lobby};
use crate::ui::theme::Theme;
```

- [ ] **Step 2: Replace the three literals in `update()`**

Current (`app.rs:1333-1337`):

```rust
let mut visuals = Visuals::dark();
visuals.panel_fill = Color32::from_rgb(11, 12, 14);
visuals.window_fill = Color32::from_rgb(18, 20, 23);
visuals.selection.bg_fill = Color32::from_rgb(2, 132, 199);
ctx.set_visuals(visuals);
```

Replace with:

```rust
let mut visuals = Visuals::dark();
visuals.panel_fill = Theme::CANVAS;
visuals.window_fill = Theme::SURFACE_1;
visuals.selection.bg_fill = Theme::BORDER_ACTIVE;
ctx.set_visuals(visuals);
```

Note: `window_fill` was `Color32::from_rgb(18, 20, 23)` — one off from `Theme::SURFACE_1`'s `(18, 20, 24)` (drifted copy-paste, not an intentional distinct color). This step corrects that drift by routing it through the token, per Global Constraints' "called out explicitly" exception.

- [ ] **Step 3: Verify build**

Run: `cd client-desktop && cargo build 2>&1 | tail -30`
Expected: clean compile, no errors, no new warnings.

- [ ] **Step 4: Verify zero visual diff**

Run: `git diff client-desktop/src/app.rs`
Expected: only the import line and the three `Color32::from_rgb(...)` → `Theme::*` substitutions above. Confirm `Theme::CANVAS == (11,12,14)`, `Theme::SURFACE_1 == (18,20,24)`, `Theme::BORDER_ACTIVE == (2,132,199)` in the still-unmodified `src/ui/theme.rs` (they do, per current file).

- [ ] **Step 5: Commit**

```bash
git add client-desktop/src/app.rs
git commit -m "refactor(desktop): route app.rs visuals through Theme tokens"
```

---

## Task 2: Step A dedupe — `src/ui/captions.rs`

**Files:**
- Modify: `client-desktop/src/ui/captions.rs:1-3` (imports), and the 4 literal call sites below

**Interfaces:**
- Consumes: `Theme::PRIMARY_LIGHT`, `Theme::EMERALD_LIGHT`, `Theme::AMBER_LIGHT`, `Theme::TEXT_SECONDARY`
- Produces: nothing new

- [ ] **Step 1: Add the missing import**

Current top of file:

```rust
use crate::app::ConferApp;
use crate::sdk::protocol::CaptionChunkDto;
use egui::{Color32, Pos2, Rect, RichText, Stroke, Ui};
```

Add after the `ConferApp` import:

```rust
use crate::app::ConferApp;
use crate::ui::theme::Theme;
use crate::sdk::protocol::CaptionChunkDto;
use egui::{Color32, Pos2, Rect, RichText, Stroke, Ui};
```

- [ ] **Step 2: Replace all 4 matched literals**

Using find-and-replace (each pattern occurs once in this file — plain string replace, no regex needed):

| Find | Replace |
|---|---|
| `Color32::from_rgb(56, 189, 248)` | `Theme::PRIMARY_LIGHT` |
| `Color32::from_rgb(52, 211, 153)` | `Theme::EMERALD_LIGHT` |
| `Color32::from_rgb(251, 191, 36)` | `Theme::AMBER_LIGHT` |
| `Color32::from_rgb(148, 163, 184)` | `Theme::TEXT_SECONDARY` |

- [ ] **Step 3: Verify build**

Run: `cd client-desktop && cargo build 2>&1 | tail -30`
Expected: clean compile.

- [ ] **Step 4: Verify zero visual diff**

Run: `git diff client-desktop/src/ui/captions.rs` — confirm only the import + the 4 substitutions above changed, no other lines.

- [ ] **Step 5: Commit**

```bash
git add client-desktop/src/ui/captions.rs
git commit -m "refactor(desktop): route captions.rs colors through Theme tokens"
```

---

## Task 3: Step A dedupe — `src/ui/lobby.rs`

**Files:**
- Modify: `client-desktop/src/ui/lobby.rs` (import already present at line 5: `use crate::ui::theme::Theme;`)

**Interfaces:**
- Consumes: `Theme::BORDER_ACTIVE`, `Theme::EMERALD`, `Theme::TEXT_SECONDARY`, `Theme::AMBER`, `Theme::AMBER_LIGHT`
- Produces: nothing new

- [ ] **Step 1: Replace all matched literals**

| Find | Replace | Occurrences |
|---|---|---|
| `Color32::from_rgb(2, 132, 199)` | `Theme::BORDER_ACTIVE` | 1 |
| `Color32::from_rgb(16, 185, 129)` | `Theme::EMERALD` | 3 |
| `Color32::from_rgb(148, 163, 184)` | `Theme::TEXT_SECONDARY` | 1 |
| `Color32::from_rgb(245, 158, 11)` | `Theme::AMBER` | 1 |
| `Color32::from_rgb(251, 191, 36)` | `Theme::AMBER_LIGHT` | 1 |

(Import already present — no Step 1 for that here.)

- [ ] **Step 2: Verify build**

Run: `cd client-desktop && cargo build 2>&1 | tail -30`
Expected: clean compile.

- [ ] **Step 3: Verify zero visual diff**

Run: `git diff client-desktop/src/ui/lobby.rs` — confirm exactly 7 literal→token substitutions (no import line change), no other content touched.

- [ ] **Step 4: Commit**

```bash
git add client-desktop/src/ui/lobby.rs
git commit -m "refactor(desktop): route lobby.rs colors through Theme tokens"
```

---

## Task 4: Step A dedupe — `src/ui/meeting_room.rs`

**Files:**
- Modify: `client-desktop/src/ui/meeting_room.rs` (import already present at line 2)

**Interfaces:**
- Consumes: `Theme::CANVAS`, `Theme::SURFACE_1`, `Theme::BORDER_SUBTLE`, `Theme::BORDER_ACTIVE`, `Theme::TEXT_PRIMARY`, `Theme::TEXT_MUTED`, `Theme::AMBER`, `Theme::AMBER_LIGHT`, `Theme::EMERALD`, `Theme::CRIMSON`, `Theme::TEXT_SECONDARY`, `Theme::SURFACE_3`
- Produces: nothing new

- [ ] **Step 1: Replace all plain `from_rgb` matched literals**

| Find | Replace | Occurrences |
|---|---|---|
| `Color32::from_rgb(2, 132, 199)` | `Theme::BORDER_ACTIVE` | 1 |
| `Color32::from_rgb(11, 12, 14)` | `Theme::CANVAS` | 1 |
| `Color32::from_rgb(18, 20, 24)` | `Theme::SURFACE_1` | 3 |
| `Color32::from_rgb(34, 38, 44)` | `Theme::BORDER_SUBTLE` | 1 |
| `Color32::from_rgb(38, 42, 48)` | `Theme::SURFACE_3` | 2 |
| `Color32::from_rgb(100, 116, 139)` | `Theme::TEXT_MUTED` | 1 |
| `Color32::from_rgb(148, 163, 184)` | `Theme::TEXT_SECONDARY` | 2 |
| `Color32::from_rgb(225, 29, 72)` | `Theme::CRIMSON` | 4 |
| `Color32::from_rgb(245, 158, 11)` | `Theme::AMBER` | 1 |
| `Color32::from_rgb(248, 250, 252)` | `Theme::TEXT_PRIMARY` | 1 |
| `Color32::from_rgb(251, 191, 36)` | `Theme::AMBER_LIGHT` | 1 |

- [ ] **Step 2: Replace the 2 alpha-blended `from_rgba_premultiplied` sites**

These carry an alpha channel the plain `Theme` constant doesn't have — preserve the exact alpha value while sourcing the RGB from the token, so the value stays tied to `Theme::EMERALD` going forward:

| Find | Replace |
|---|---|
| `Color32::from_rgba_premultiplied(16, 185, 129, 70)` | `Color32::from_rgba_premultiplied(Theme::EMERALD.r(), Theme::EMERALD.g(), Theme::EMERALD.b(), 70)` |
| `Color32::from_rgba_premultiplied(16, 185, 129, 245)` | `Color32::from_rgba_premultiplied(Theme::EMERALD.r(), Theme::EMERALD.g(), Theme::EMERALD.b(), 245)` |

- [ ] **Step 3: Verify build**

Run: `cd client-desktop && cargo build 2>&1 | tail -30`
Expected: clean compile.

- [ ] **Step 4: Verify zero visual diff**

Run: `git diff client-desktop/src/ui/meeting_room.rs` — confirm exactly the 20 + 2 substitutions above, no other lines touched. For the two rgba sites, confirm the alpha (4th number) is unchanged from the original.

- [ ] **Step 5: Commit**

```bash
git add client-desktop/src/ui/meeting_room.rs
git commit -m "refactor(desktop): route meeting_room.rs colors through Theme tokens"
```

---

## Task 5: Step A dedupe — `src/ui/polls.rs`

**Files:**
- Modify: `client-desktop/src/ui/polls.rs:1-4` (imports), and the 30 matched literal call sites below

**Interfaces:**
- Consumes: `Theme::BORDER_ACTIVE`, `Theme::BORDER_SUBTLE`, `Theme::SURFACE_2`, `Theme::SURFACE_3`, `Theme::TEXT_PRIMARY`, `Theme::TEXT_SECONDARY`, `Theme::TEXT_MUTED`, `Theme::EMERALD_LIGHT`, `Theme::PRIMARY_LIGHT`, `Theme::CRIMSON_LIGHT`
- Produces: nothing new

- [ ] **Step 1: Add the missing import**

Current top of file:

```rust
use egui::{Color32, Rect, RichText, ScrollArea, Stroke, Ui, Vec2};
use uuid::Uuid;

use crate::app::ConferApp;
```

Add:

```rust
use egui::{Color32, Rect, RichText, ScrollArea, Stroke, Ui, Vec2};
use uuid::Uuid;

use crate::app::ConferApp;
use crate::ui::theme::Theme;
```

- [ ] **Step 2: Replace all 30 matched literals**

| Find | Replace | Occurrences |
|---|---|---|
| `Color32::from_rgb(2, 132, 199)` | `Theme::BORDER_ACTIVE` | 4 |
| `Color32::from_rgb(26, 29, 33)` | `Theme::SURFACE_2` | 3 |
| `Color32::from_rgb(34, 38, 44)` | `Theme::BORDER_SUBTLE` | 1 |
| `Color32::from_rgb(38, 42, 48)` | `Theme::SURFACE_3` | 5 |
| `Color32::from_rgb(52, 211, 153)` | `Theme::EMERALD_LIGHT` | 1 |
| `Color32::from_rgb(56, 189, 248)` | `Theme::PRIMARY_LIGHT` | 2 |
| `Color32::from_rgb(100, 116, 139)` | `Theme::TEXT_MUTED` | 2 |
| `Color32::from_rgb(148, 163, 184)` | `Theme::TEXT_SECONDARY` | 8 |
| `Color32::from_rgb(248, 250, 252)` | `Theme::TEXT_PRIMARY` | 3 |
| `Color32::from_rgb(254, 205, 211)` | `Theme::CRIMSON_LIGHT` | 1 |

- [ ] **Step 3: Verify build**

Run: `cd client-desktop && cargo build 2>&1 | tail -30`
Expected: clean compile.

- [ ] **Step 4: Verify zero visual diff**

Run: `git diff client-desktop/src/ui/polls.rs` — confirm the import + exactly the 30 substitutions above.

- [ ] **Step 5: Commit**

```bash
git add client-desktop/src/ui/polls.rs
git commit -m "refactor(desktop): route polls.rs colors through Theme tokens"
```

---

## Task 6: Step A dedupe — `src/ui/waiting_lobby.rs`

**Files:**
- Modify: `client-desktop/src/ui/waiting_lobby.rs` (import already present at line 3)

**Interfaces:**
- Consumes: `Theme::EMERALD`, `Theme::TEXT_SECONDARY`, `Theme::AMBER`, `Theme::PRIMARY_LIGHT`
- Produces: nothing new

- [ ] **Step 1: Replace all 3 plain `from_rgb` matched literals**

| Find | Replace | Occurrences |
|---|---|---|
| `Color32::from_rgb(16, 185, 129)` | `Theme::EMERALD` | 3 |
| `Color32::from_rgb(148, 163, 184)` | `Theme::TEXT_SECONDARY` | 1 |
| `Color32::from_rgb(245, 158, 11)` | `Theme::AMBER` | 1 |

- [ ] **Step 2: Replace the 1 alpha-blended site**

| Find | Replace |
|---|---|
| `Color32::from_rgba_premultiplied(56, 189, 248, 70)` | `Color32::from_rgba_premultiplied(Theme::PRIMARY_LIGHT.r(), Theme::PRIMARY_LIGHT.g(), Theme::PRIMARY_LIGHT.b(), 70)` |

- [ ] **Step 3: Verify build**

Run: `cd client-desktop && cargo build 2>&1 | tail -30`
Expected: clean compile.

- [ ] **Step 4: Verify zero visual diff**

Run: `git diff client-desktop/src/ui/waiting_lobby.rs` — confirm exactly the 5 + 1 substitutions above, alpha (70) unchanged on the rgba one.

- [ ] **Step 5: Commit**

```bash
git add client-desktop/src/ui/waiting_lobby.rs
git commit -m "refactor(desktop): route waiting_lobby.rs colors through Theme tokens"
```

---

## Task 7: Step A dedupe — `src/ui/watermark.rs`

**Files:**
- Modify: `client-desktop/src/ui/watermark.rs:1` (import), and the 1 matched literal site

**Interfaces:**
- Consumes: `Theme::TEXT_PRIMARY`
- Produces: nothing new

- [ ] **Step 1: Add the missing import**

Current top of file:

```rust
use egui::{Color32, FontId, Pos2, Rect, Ui};
```

Add:

```rust
use egui::{Color32, FontId, Pos2, Rect, Ui};

use crate::ui::theme::Theme;
```

- [ ] **Step 2: Replace the 1 alpha-blended site**

| Find | Replace |
|---|---|
| `Color32::from_rgba_premultiplied(248, 250, 252, 32)` | `Color32::from_rgba_premultiplied(Theme::TEXT_PRIMARY.r(), Theme::TEXT_PRIMARY.g(), Theme::TEXT_PRIMARY.b(), 32)` |

- [ ] **Step 3: Verify build**

Run: `cd client-desktop && cargo build 2>&1 | tail -30`
Expected: clean compile.

- [ ] **Step 4: Verify zero visual diff**

Run: `git diff client-desktop/src/ui/watermark.rs` — confirm import + the single substitution, alpha (32) unchanged.

- [ ] **Step 5: Commit**

```bash
git add client-desktop/src/ui/watermark.rs
git commit -m "refactor(desktop): route watermark.rs color through Theme tokens"
```

---

## Task 8: Step A dedupe — `src/ui/whiteboard.rs`

**Files:**
- Modify: `client-desktop/src/ui/whiteboard.rs:1-5` (imports), and the 19 matched literal call sites below

**Interfaces:**
- Consumes: `Theme::BORDER_SUBTLE`, `Theme::PRIMARY_LIGHT`, `Theme::BORDER_ACTIVE`, `Theme::SURFACE_2`, `Theme::TEXT_SECONDARY`, `Theme::SURFACE_3`, `Theme::CRIMSON`
- Produces: nothing new

**Constraint reminder:** the pen-color palette at lines 42-47 (`(255,255,255)` White, `(239,68,68)` Red, `(34,197,94)` Green, `(59,130,246)` Blue, `(234,179,8)` Yellow, `(249,115,22)` Orange) is content, not chrome — **do not touch these 6 lines.** None of them match a `Theme` constant anyway, so the table below never references them.

- [ ] **Step 1: Add the missing import**

Current top of file:

```rust
use egui::{Align2, Color32, FontId, Pos2, Rect, RichText, Stroke, Ui, Vec2};
use uuid::Uuid;

use crate::app::ConferApp;
use crate::sdk::protocol::{WhiteboardColorDto, WhiteboardShapeDto, WhiteboardStrokeDto};
```

Add:

```rust
use egui::{Align2, Color32, FontId, Pos2, Rect, RichText, Stroke, Ui, Vec2};
use uuid::Uuid;

use crate::app::ConferApp;
use crate::sdk::protocol::{WhiteboardColorDto, WhiteboardShapeDto, WhiteboardStrokeDto};
use crate::ui::theme::Theme;
```

- [ ] **Step 2: Replace all 19 matched literals (excluding the pen palette)**

| Find | Replace | Occurrences |
|---|---|---|
| `Color32::from_rgb(2, 132, 199)` | `Theme::BORDER_ACTIVE` | 2 |
| `Color32::from_rgb(26, 29, 33)` | `Theme::SURFACE_2` | 5 |
| `Color32::from_rgb(34, 38, 44)` | `Theme::BORDER_SUBTLE` | 1 |
| `Color32::from_rgb(38, 42, 48)` | `Theme::SURFACE_3` | 2 |
| `Color32::from_rgb(56, 189, 248)` | `Theme::PRIMARY_LIGHT` | 3 |
| `Color32::from_rgb(148, 163, 184)` | `Theme::TEXT_SECONDARY` | 5 |
| `Color32::from_rgb(225, 29, 72)` | `Theme::CRIMSON` | 1 |

- [ ] **Step 3: Verify build**

Run: `cd client-desktop && cargo build 2>&1 | tail -30`
Expected: clean compile.

- [ ] **Step 4: Verify zero visual diff and pen palette untouched**

Run: `git diff client-desktop/src/ui/whiteboard.rs` — confirm import + exactly the 19 substitutions above, and confirm lines 42-47 (the pen palette, now possibly shifted by the import insertion — check by content, not line number) show **no diff**.

- [ ] **Step 5: Commit**

```bash
git add client-desktop/src/ui/whiteboard.rs
git commit -m "refactor(desktop): route whiteboard.rs colors through Theme tokens"
```

---

## Task 9: Step B — retarget `Theme` to Catppuccin Mocha

**Files:**
- Modify: `client-desktop/Cargo.toml`
- Modify: `client-desktop/src/ui/theme.rs:10-48`
- Modify: `client-desktop/src/app.rs` (`update()`, now at the location left by Task 1)

**Interfaces:**
- Consumes: `catppuccin_egui::MOCHA` (crate constant, fields verified against `catppuccin-egui` v5.7.0 source: `crust, mantle, surface0, surface1, overlay0, overlay1, blue, sapphire, green, teal, red, flamingo, peach, yellow, text, subtext1, subtext0`), `catppuccin_egui::set_theme(ctx: &egui::Context, theme: catppuccin_egui::Theme)`
- Produces: `Theme::*` constants now hold Catppuccin Mocha values — every file touched in Tasks 1-8 (and any other consumer of `Theme::*`) picks this up automatically, no further changes needed there.

- [ ] **Step 1: Add the dependency**

In `client-desktop/Cargo.toml`, add to `[dependencies]` (after `zenyuv = "0.1"`):

```toml
catppuccin-egui = { version = "5.7.0", default-features = false, features = ["egui29"] }
```

Run: `cd client-desktop && cargo add --dry-run catppuccin-egui --no-default-features --features egui29`
Expected output includes `Adding catppuccin-egui v5.7.0` — confirms the resolved version before it's ever built. (This confirms what's already verified in the spec; re-run here as the final check before touching the lockfile.)

- [ ] **Step 2: Retarget the `Theme` constants**

Replace `client-desktop/src/ui/theme.rs` lines 10-48 (from `// --- Core Colors ---` through the `TEXT_DIM` line, i.e. everything between the struct's `impl` opening and the `// --- Corner Radii ---` section) with:

```rust
    // -------------------------------------------------------------------------
    // Core Colors — sourced from Catppuccin Mocha (catppuccin_egui::MOCHA)
    // -------------------------------------------------------------------------
    pub const CANVAS: Color32 = catppuccin_egui::MOCHA.crust; // #11111B
    pub const SURFACE_1: Color32 = catppuccin_egui::MOCHA.mantle; // #181825
    pub const SURFACE_2: Color32 = catppuccin_egui::MOCHA.surface0; // #313244
    pub const SURFACE_3: Color32 = catppuccin_egui::MOCHA.surface1; // #45475A
    pub const SURFACE_DOCK: Color32 = Color32::from_rgba_premultiplied(24, 24, 37, 250); // mantle @ 250 alpha

    // -------------------------------------------------------------------------
    // Borders & Hairlines
    // -------------------------------------------------------------------------
    pub const BORDER_SUBTLE: Color32 = catppuccin_egui::MOCHA.overlay0; // #6C7086
    pub const BORDER_ACTIVE: Color32 = catppuccin_egui::MOCHA.blue; // #89B4FA

    // -------------------------------------------------------------------------
    // Semantic Accents
    // -------------------------------------------------------------------------
    pub const PRIMARY: Color32 = catppuccin_egui::MOCHA.blue; // #89B4FA
    // Catppuccin's palette is flat (no hover variants). Hand-derived by
    // blending PRIMARY 15% toward MOCHA.crust — see spec §1.
    pub const PRIMARY_HOVER: Color32 = Color32::from_rgb(119, 156, 217);
    pub const PRIMARY_LIGHT: Color32 = catppuccin_egui::MOCHA.sapphire; // #74C7EC

    pub const EMERALD: Color32 = catppuccin_egui::MOCHA.green; // #A6E3A1
    pub const EMERALD_LIGHT: Color32 = catppuccin_egui::MOCHA.teal; // #94E2D5

    pub const CRIMSON: Color32 = catppuccin_egui::MOCHA.red; // #F38BA8
    // Derived the same way as PRIMARY_HOVER — see note above.
    pub const CRIMSON_HOVER: Color32 = Color32::from_rgb(209, 121, 147);
    pub const CRIMSON_LIGHT: Color32 = catppuccin_egui::MOCHA.flamingo; // #F2CDCD

    pub const AMBER: Color32 = catppuccin_egui::MOCHA.peach; // #FAB387
    pub const AMBER_LIGHT: Color32 = catppuccin_egui::MOCHA.yellow; // #F9E2AF

    // -------------------------------------------------------------------------
    // Typography Colors
    // -------------------------------------------------------------------------
    pub const TEXT_PRIMARY: Color32 = catppuccin_egui::MOCHA.text; // #CDD6F4
    pub const TEXT_SECONDARY: Color32 = catppuccin_egui::MOCHA.subtext1; // #BAC2DE
    pub const TEXT_MUTED: Color32 = catppuccin_egui::MOCHA.subtext0; // #A6ADC8
    pub const TEXT_DIM: Color32 = catppuccin_egui::MOCHA.overlay1; // #7F849C
```

- [ ] **Step 3: Apply the theme to native egui widget chrome**

In `client-desktop/src/app.rs`, inside `fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)`, the block Task 1 left looks like:

```rust
let mut visuals = Visuals::dark();
visuals.panel_fill = Theme::CANVAS;
visuals.window_fill = Theme::SURFACE_1;
visuals.selection.bg_fill = Theme::BORDER_ACTIVE;
ctx.set_visuals(visuals);
```

Replace it with (order matters — `set_theme` must run first so the subsequent overrides layer on top of it, not get discarded by a fresh `Visuals::dark()`):

```rust
catppuccin_egui::set_theme(ctx, catppuccin_egui::MOCHA);
let mut visuals = ctx.style().visuals.clone();
visuals.panel_fill = Theme::CANVAS;
visuals.window_fill = Theme::SURFACE_1;
visuals.selection.bg_fill = Theme::BORDER_ACTIVE;
ctx.set_visuals(visuals);
```

- [ ] **Step 4: Verify build**

Run: `cd client-desktop && cargo build 2>&1 | tail -40`
Expected: clean compile. If it fails on `catppuccin_egui::MOCHA.crust` field access inside a `const` context, confirm `Theme` (the crate's struct, not this app's) derives `Copy` and `MOCHA` is declared `pub const` (both true in v5.7.0 `themes.rs`) — field projection on a `const` struct value is valid Rust and should just work.

- [ ] **Step 5: Run the full test suite**

Run: `cd client-desktop && cargo test 2>&1 | tail -20`
Expected: all existing tests still pass (this task changes constant values and one widget-styling call, not logic — no test should reference specific RGB values).

- [ ] **Step 6: Run clippy**

Run: `cd client-desktop && cargo clippy --message-format=short 2>&1 | tail -40`
Expected: no new warnings.

- [ ] **Step 7: Manual visual check**

Run: `cd client-desktop && cargo run` — confirm the app launches into the Lobby view with the Catppuccin Mocha palette visible (near-black `crust` background `#11111B`, blue `#89B4FA` primary accents, pastel green/red/peach status colors) and no rendering panics. This is the one step in this plan where the diff is meant to be visible — screenshot or describe what changed if anything looks off before committing.

- [ ] **Step 8: Commit**

```bash
git add client-desktop/Cargo.toml client-desktop/Cargo.lock client-desktop/src/ui/theme.rs client-desktop/src/app.rs
git commit -m "feat(desktop): retarget Theme tokens to Catppuccin Mocha

Every screen already routes through Theme::* after the Task 1-8 dedupe,
so this single commit flips the whole app's palette. Native egui widget
chrome (sliders, checkboxes, text-edit cursor) now matches via
catppuccin_egui::set_theme(), layered under the app's existing
CANVAS/SURFACE_1/BORDER_ACTIVE overrides."
```

---

## Out of scope (tracked separately)

Per spec §2 Step C and §3-6: replacing the remaining ~57 one-off literals that don't match any `Theme` constant, adding `egui-phosphor` (icons), `egui-toast` (notifications), `egui_commonmark` (chat markdown), and `egui-file-dialog` (dependency only). These are independent, separately shippable slices — write as a follow-up plan once this one lands, so each plan produces working, testable software on its own (per writing-plans Scope Check).
