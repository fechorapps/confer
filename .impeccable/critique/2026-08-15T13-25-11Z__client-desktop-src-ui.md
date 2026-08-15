---
target: desktop app
total_score: 28
max_score: 40
na_heuristics: 
p0_count: 1
p1_count: 1
timestamp: 2026-08-15T13-25-11Z
slug: client-desktop-src-ui
---
# Design Critique: Confer Native Desktop Client UI

**Target:** `client-desktop/src/ui/`
**Surface Mode:** Operate (Native GPU Immediate-Mode Video Collaboration Client)

## Design Health Score

| # | Heuristic | Score | Key Issue |
|---|-----------|:-----:|-----------|
| 1 | Visibility of System Status | 3.5/4 | Clear active-speaker emerald halo, live mic VU meter, and waiting room radar pulse. Outbound screen-share FPS/resolution metrics omitted. |
| 2 | Match Between System & Real World | 4.0/4 | Standard video conferencing terminology and clear tool metaphors throughout. |
| 3 | User Control & Freedom | 2.8/4 | Whiteboard supports full undo/clear and drawers dismiss easily, but "Leave Meeting" and host "Kick" trigger immediately without an undo buffer. |
| 4 | Consistency & Standards | 2.5/4 | **Split Navigation Inconsistency:** Drawers (Chat, Roster, Polls) are split between the top header and the bottom dock; Polls is duplicated in both places while Chat is absent from the bottom dock. |
| 5 | Error Prevention | 2.0/4 | **High Destructive Action Risk:** Accidental click on "✕ Leave" abruptly severs WebRTC connection with zero confirmation modal. Host "Clear Whiteboard" wipes canvas instantly. |
| 6 | Recognition Rather Than Recall | 3.5/4 | Prominently displayed 6-digit join code, active speaker names, and highlighted active whiteboard tools. |
| 7 | Flexibility & Efficiency of Use | 2.0/4 | **Zero Keyboard Shortcuts & PTT in Desktop UI:** Power users cannot use Spacebar Push-to-Talk or `Ctrl+D`/`Ctrl+E` hotkeys to toggle mic and camera. |
| 8 | Aesthetic & Minimalist Design | 3.0/4 | Obsidian palette is visually disciplined, calm, and uncluttered. Bottom floating dock packs 13 buttons into a single horizontal strip. |
| 9 | Error Recovery | 2.5/4 | Lobby displays clear error callouts; network disconnects in active rooms drop abruptly to lobby rather than showing an in-stage reconnecting spinner. |
| 10 | Help & Documentation | 2.2/4 | Input placeholders are descriptive; advanced features (DLP Watermark, AI Denoise) lack contextual tooltips or shortcuts help. |
| **Total** | | **28.0/40** | **Good (28–34)** |

---

## Design Specificity Verdict

- **LLM Assessment:** The desktop client presents a highly disciplined, cohesive **Obsidian & Deep Zinc** dark design language (`#0B0C0E` base, `#121417` cards, `#22262C` borders) tailored for pro-audio/engineering collaboration. The pre-join hardware cockpit (`lobby.rs`) integrating real-time VU monitoring and the <40MB RAM telemetry HUD give Confer strong product identity. Primary design gaps stem from dock clutter (13 items in a single horizontal strip) and split navigation between the top header and bottom dock.
- **Deterministic Scan:** Deterministic scanner verified zero AI-slop anti-patterns (no synthetic purple-on-dark gradients, no decorative nested cards, proper typographic scale ≥ 1.25x).
- **Visual Overlays:** Native immediate-mode GPU application (`egui` rendering via OpenGL/Vulkan directly to Wayland/X11); web DOM injection is not applicable.

---

## Overall Impression
Confer's desktop client feels exceptionally fast, responsive, and aesthetically pro-grade. It achieves a pro-software feel with an ultra-light memory footprint (<40MB RAM). However, ergonomic friction in dock navigation, the lack of keyboard shortcuts/PTT, and accidental disconnect hazards currently hold it back from being a 10/10 power-user tool.

---

## What's Working
1. **Hardware Pre-Check Cockpit (`lobby.rs`)**: Direct live camera loopback with shader filters, virtual background previews, and a real-time RNNoise mic VU meter right on the pre-join canvas.
2. **Disciplined Obsidian Pro Aesthetic**: High-contrast Sky Blue, Emerald, and Amber accents against rich zinc surfaces render at a locked 60 FPS on GPU.
3. **Rich Collaboration Suite**: Native immediate-mode vector whiteboard, live multi-choice polling with animated tally bars, floating live captions, and DLP watermarking.

---

## Priority Issues

### [P0] Zero-Confirmation Destructive Actions (Accidental Disconnect & Host Kick)
- **Why it matters:** Accidentally clicking "✕ Leave" (immediately adjacent to Reactions) severs the live WebRTC room without confirmation. Host clicking "Kick" immediately evicts participants with no confirmation dialog.
- **Fix:** Add a non-blocking confirmation dialog or a hold-to-leave button for "Leave Meeting", and a standard modal for host Kick / Clear Whiteboard.
- **Suggested Command:** `/impeccable harden desktop-safety`

### [P1] Fragmented Navigation & Bottom Dock Clutter (13 Buttons)
- **Why it matters:** 13 buttons crammed in the bottom dock create visual fatigue and scanning friction. Splitting drawers (Chat in top header, Whiteboard in bottom dock, Polls in both) breaks user mental models.
- **Fix:** Consolidate panel toggles (Chat, Roster, Polls, Whiteboard) into the dock, and group secondary toggles (Filters, Backgrounds, AI Denoise, Security) into a unified "⚙ Settings / More" popup.
- **Suggested Command:** `/impeccable layout desktop-dock`

### [P2] Missing Desktop Keyboard Shortcuts & Push-to-Talk (PTT)
- **Why it matters:** Desktop power users and moderators expect Spacebar Push-to-Talk, `Ctrl+D` (mute), `Ctrl+E` (camera), `Ctrl+Shift+S` (screen share), and `Esc` (close panels).
- **Fix:** Bind `egui::Key` handlers in `app.rs` / `controls.rs` for global hotkeys and Spacebar PTT with shortcut hints in button tooltips.
- **Suggested Command:** `/impeccable enhance desktop-shortcuts`

### [P3] Small-Screen Viewport Clipping & Responsive Dock Collapse
- **Why it matters:** On smaller laptops or split-window tiling (width < 1000px), the 13-item pill dock and open side panels overflow the visible canvas.
- **Fix:** Implement responsive dock collapsing: automatically collapse secondary actions into an overflow popup (`"••• More"`) when `available_width < 960px`.
- **Suggested Command:** `/impeccable adapt desktop-responsive`

---

## Persona Red Flags

- **Alex (Power User / Host):** Cannot moderate rooms quickly due to lack of keyboard shortcuts, inability to search/filter the 20+ participant roster, and no batch-mute hotkey.
- **Jordan (First-Timer / Guest):** Looks for in-call "Chat" in the bottom dock where industry standards place it, getting confused because it is tucked in the top header.
- **Morgan (Accessibility / Low Vision):** Secondary slate text (`#64748B` on `#121417`) has a contrast ratio of ~3.3:1 (below WCAG AA 4.5:1). Captions font size (13.5px) is non-resizable.

---

## Minor Observations & Questions to Consider
1. **Watermark Drift:** Adding subtle animated drift to the DLP watermark prevents screen capture software from filtering it via static frame subtraction.
2. **Integrated Health HUD:** Docking latency/FPS metrics into the top header bar rather than a separate floating window would feel cleaner.
3. **Provocative Question:** *What if the Whiteboard could be toggled as an interactive collaborative overlay directly on top of shared screen content?*
