# Make Click-Through Cover the Whole Subtitle Island

Status: accepted. Supersedes [ADR 0003](0003-separate-click-through-subtitles-from-session-controls.md).

Click-Through Mode lets pointer events pass through the whole Subtitle Island window, including its controls. The control path is a fixed global keyboard shortcut. The island shows that shortcut for as long as the mode is on. Tauri's `set_ignore_cursor_events` applies to the whole window, and a window that ignores the cursor receives no hover events, so ADR 0003's split between click-through subtitles and clickable controls cannot be built directly.

**Considered Options**

- Polling the cursor position from the backend and turning click-through off while the cursor is over the controls would keep the controls clickable. It adds a timer loop and edge cases around the window boundary for little user benefit.
- A separate always-clickable control window would satisfy ADR 0003 literally. It adds a second window, capability, and cross-window state sync.

**Consequences**

- If the global shortcut cannot be registered, the app must not let the user enter Click-Through Mode.
- Click-Through Mode is not persisted, so every app launch starts with a clickable island.
