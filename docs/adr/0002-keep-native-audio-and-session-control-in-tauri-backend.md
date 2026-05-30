# Keep Native Audio and Session Control in the Tauri Backend

Native audio capture, subtitle session lifecycle, transcription connections, API key access, and trusted desktop capabilities belong in `src-tauri/`, while `src/` owns the subtitle island UI and display state. This preserves a narrow Tauri IPC boundary for OS permissions and secrets, while keeping Svelte focused on presentation and user interaction.

Operational rules for applying this boundary live in [Tauri Frontend and Backend Boundary](../tauri-architecture-boundary.md).

**Considered Options**

- Putting more orchestration in the WebView would make UI iteration easier, but would expose native capabilities and secrets through a less trusted surface.
- Splitting session ownership across frontend and backend would reduce backend state at first, but would make repeated Start/Stop behavior, cross-window consistency, and capture cleanup harder to reason about.
