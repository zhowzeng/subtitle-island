# Phase 3 Plan

Status: Planned as of 2026-10-21.

## Goal

Improve the Subtitle Island for daily use, building on the completed Phase 2 meeting translation:

1. Add Display Mode with Bilingual Mode.
2. Add a settings panel with persisted settings.
3. Let the user resize the island width and adjust font size and background opacity.
4. Add Click-Through Mode with global shortcuts.

## Current Starting Point

Phase 2 is complete. Its accepted gaps are handled as follows:

- Translated subtitle latency (about 2.6 s) and 30-60 minute sessions are out of Phase 3 scope.
- The user manually verified repeated Start/Stop across Microphone Capture and System Audio Capture after Phase 2 closed.

The app starts Phase 3 with:

- One undecorated, transparent, always-on-top `main` window (960x420, min 640x320). It cannot be resized on Linux because it has no decorations.
- All controls in the island status row: Audio Source, Source Language, `Translate`, Start, Stop, and Close.
- `start_session` takes `{ audioSource, sourceLanguage, translate }` from the frontend.
- With `translate` on, the island uses Chinese Mode and hides non-Chinese lines that are waiting for Translation. With it off, the island shows source text. This is Original Mode.
- The island shows the last 2 Transcript Segments, with a font size of `clamp(20px, 4.8vw, 30px)` and a background of `rgb(10 15 20 / 0.78)`.
- No settings persistence, shortcuts, or Click-Through Mode.

## Grilled Decisions

### Decision 1: Display Mode replaces the Translate toggle

The user picks a Display Mode: Original, Chinese, or Bilingual. Original Mode means no Translation; Chinese and Bilingual Mode set the backend `translate` flag. Display Mode is chosen before Start and cannot change during a Subtitle Session.

Why: the old toggle allowed combinations that mean nothing, such as Translate off with Bilingual Mode.

### Decision 2: Bilingual Mode layout

Each of the last 2 Transcript Segments shows source text on one line and Traditional Chinese on the next, so the island shows up to 4 lines. Both lines use the same font size. Source text appears before its Translation arrives. A segment whose Translation failed shows only source text. Chinese source text shows as one line.

### Decision 3: The settings panel lives inside the Subtitle Island

A settings button in the status row swaps the subtitle area for a settings panel. After this change, the status row holds only the status, Start/Stop, settings, Click-Through Mode, and Close.

The panel holds:

- Audio Source, Source Language, and Display Mode. These can only change while idle.
- Font size and background opacity. These can change during a session.

Why: a separate settings window would add a second window, capability, and cross-window state sync. A tray menu is unreliable on Linux and does not suit sliders.

### Decision 4: The backend owns settings persistence

`src-tauri/` reads and writes a small JSON settings file in the app config directory with `serde_json`, which is already a dependency. It stores Audio Source, Source Language, Display Mode, font size, and background opacity. If the file is missing or invalid, the app uses defaults.

The frontend loads settings through a `get_settings` command and saves each change through `save_settings`. `start_session` reads the stored settings instead of taking options, so the Start button and the Start/Stop shortcut use the same source.

Window width and position are not persisted in Phase 3.

Why: [tauri-architecture-boundary.md](../tauri-architecture-boundary.md) puts settings that persist across restarts in `src-tauri/`, and the global shortcut handler needs settings to start a session without a round trip through the frontend.

### Decision 5: Width resizes by dragging the island edges

Transparent handles on the island's left and right edges call `startResizeDragging` through a narrow command, matching the existing `begin_subtitle_island_drag`. The 920px island cap is removed. The minimum window width drops from 640px to about 400px, now that most controls are in the settings panel. Height stays fixed.

### Decision 6: Font size and opacity

- Font size uses a fixed px value instead of `vw`, so changing the width does not change the font size. Range: 14-32px in 2px steps. Default: 20px.
- Opacity applies only to the island background, not to the text. Range: 0-100%. Default: 78%. At 0%, the blur and border are removed too.

### Decision 7: Click-Through Mode covers the whole island

See [ADR 0005](../adr/0005-make-click-through-cover-the-whole-subtitle-island.md). The whole window ignores the cursor. The user turns the mode on with a status row button or the shortcut, and turns it off only with the shortcut. While the mode is on, the island keeps showing a hint such as `Click-through · Ctrl+Alt+C to exit`.

The button tooltip and the settings panel also list the shortcut. If the shortcut fails to register, the Click-Through Mode button is disabled. Click-Through Mode is not persisted.

### Decision 8: Shortcuts

| Action | Keys | Scope |
|---|---|---|
| Toggle Click-Through Mode | `Ctrl+Alt+C` | global |
| Start or Stop the Subtitle Session | `Ctrl+Alt+S` | global |

Keys are fixed in Phase 3. The shortcuts are registered in `src-tauri/` through `tauri-plugin-global-shortcut`, installed according to [dependency-management.md](../dependency-management.md).

Why `Ctrl+Alt+`: a global shortcut takes those keys away from every other app, and few apps use `Ctrl+Alt+`. `Ctrl+Alt+T` is avoided because it opens a terminal on Ubuntu.

## Success Criteria

- The user can pick Original, Chinese, or Bilingual Mode before Start, and the island shows the matching text.
- Settings survive an app restart. A missing or invalid settings file falls back to defaults.
- The user can drag the island's left and right edges to change its width on Linux.
- Font size and background opacity change the island immediately, including during a session.
- `Ctrl+Alt+C` toggles Click-Through Mode. While it is on, clicks reach the window underneath, and the exit shortcut stays visible.
- `Ctrl+Alt+S` starts and stops a session with the stored settings, even when the island is click-through or not focused.
- Transcript and Translation text remain ephemeral and are not written to the settings file.

## Milestone 1: Settings Boundary and Display Mode

Scope:

- Add the backend settings file, `get_settings`, `save_settings`, and their capability permissions.
- Change `start_session` to read stored settings. Map Display Mode to the existing `translate` flag.
- Replace the `Translate` toggle with Display Mode. Keep Original and Chinese Mode behavior unchanged.
- Update the browser mock in `local-subtitle-session.ts`.

Verification:

- Rust unit tests for settings defaults, round trip, and invalid file fallback.
- Manual check: settings survive a restart, and Original and Chinese Mode behave as they did in Phase 2.

## Milestone 2: Settings Panel

Scope:

- Add the settings button and panel. Move Audio Source, Source Language, and Display Mode into the panel.
- Reduce the status row to the status, Start/Stop, settings, and Close.

Verification:

- Manual check: idle-only settings are disabled during a session, and the subtitle area returns when the panel closes.

## Milestone 3: Bilingual Mode

Scope:

- Render source text and Translation per segment, following Decision 2.

Verification:

- Frontend unit tests or mock checks for pending, translated, failed, and Chinese source segments.
- Manual check with System Audio Capture of English speech.

## Milestone 4: Width, Font Size, and Opacity

Scope:

- Add the edge resize handles and narrow resize command. Remove the 920px cap and lower the minimum width.
- Add font size and background opacity controls to the settings panel.

Verification:

- Manual check on Linux: width resizes from both edges, 4 Bilingual lines fit at 32px in the default height, and 0% opacity leaves readable text only.

## Milestone 5: Shortcuts and Click-Through Mode

Scope:

- Add `tauri-plugin-global-shortcut` and register `Ctrl+Alt+C` and `Ctrl+Alt+S` in `src-tauri/`.
- Toggle `set_ignore_cursor_events` in the backend and emit the Click-Through Mode state to the frontend.
- Add the Click-Through Mode button, the visible exit hint, and the disabled state when registration fails.

Verification:

- Manual check on Linux X11: clicks pass through to the app underneath, both shortcuts work while another app has focus, and a restart returns to a clickable island.

## Milestone 6: Phase 3 Hardening

Verification:

- `bun run check`
- From `src-tauri/`: `cargo fmt --check`
- From `src-tauri/`: `cargo check --locked`
- From `src-tauri/`: `cargo test --locked`
- From `src-tauri/`: `cargo clippy --all-targets --all-features --locked -- -D warnings`
- Manual check that the settings file contains no transcript or Translation text.

## Out of Scope

- Translation latency work, including Streaming Translation.
- 30-60 minute session measurement.
- Changing Display Mode during a session.
- Persisting window width, position, or Click-Through Mode.
- Height resizing.
- Custom shortcut keys.
- A separate settings window or tray menu.
- macOS and Wayland behavior for click-through and global shortcuts.

## ADR Assessment

- [ADR 0005](../adr/0005-make-click-through-cover-the-whole-subtitle-island.md) supersedes ADR 0003. Click-Through Mode covers the whole window, and the control path is a visible global shortcut.
- Backend-owned settings follow the existing Tauri boundary doc, so they need no new ADR.
- Settings persistence does not conflict with [ADR 0004](../adr/0004-do-not-persist-transcript-history-in-the-mvp.md), because no transcript or Translation text is stored.
