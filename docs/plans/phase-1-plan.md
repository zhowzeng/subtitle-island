# Phase 1 Plan

## Goal

Build the smallest working version of live speech subtitles:

1. Capture microphone audio.
2. Transcribe speech in real time.
3. Show the result in a floating subtitle island.

Phase 1 is not expected to support translation, system audio capture, bilingual subtitles, transcript history, or full settings.

## Current Progress

Status: Milestones 1-5 are complete.

Completed implementation:

- Subtitle island UI replaced the starter SvelteKit page.
- Frontend supports Idle, Listening, and Error states.
- Start and Stop controls are wired.
- Browser development mode can show mocked transcript updates without Tauri.
- Tauri frontend calls `start_session` and `stop_session`.
- Rust backend owns session state and emits mocked `transcript-update` events.
- The main Tauri window is configured as a compact always-on-top transparent overlay.

Verified:

- `npx @sveltejs/mcp svelte-autofixer .\src\routes\+page.svelte --svelte-version 5`
- `bun run check`
- `bun run build`
- `cargo check` in `src-tauri`
- `agent-browser` against `http://127.0.0.1:5173`: Start enters Listening, subtitle text updates, Stop returns to Idle and updates stop.
- `bun run tauri dev`: desktop app opens as a small floating subtitle window; Windows probe showed the window around `736x269` and `TopMost=True`; visual screenshot confirmed no visible standard title bar.

## Assumptions

- Target platform for the first implementation is Windows.
- Audio input is microphone only.
- The first useful milestone should work without OpenAI by using mocked transcript events.
- OpenAI Realtime transcription integration should be added only after the app shell, UI, and event flow are working.
- The subtitle island can start as a single Tauri window configured as an overlay.

## Success Criteria

- The app can start and stop a subtitle session.
- The subtitle island displays changing transcript text.
- Microphone input can be captured and stopped cleanly.
- Realtime transcription appears with acceptable latency, targeting under 2 seconds.
- Errors are visible for missing microphone access, missing API key, or transcription connection failure.

## Milestone 1: Subtitle Island UI

Status: Complete.

Build the frontend surface first with mocked data.

### Scope

- Replace the starter SvelteKit page with the subtitle island interface.
- Add basic session states:
  - Idle
  - Listening
  - Error
- Render one or two subtitle lines.
- Use mocked transcript updates to prove layout behavior.
- Keep UI controls minimal:
  - Start
  - Stop

### Verification

- Run `bun run check`.
- Run the frontend and verify transcript text updates without layout jumps.

Result:

- `bun run check` passed with 0 errors and 0 warnings.
- `agent-browser` verified Start changes the UI to Listening and mocked subtitle text updates in the frontend.

## Milestone 2: Overlay Window

Status: Complete.

Configure the Tauri window to behave like a subtitle island.

### Scope

- Configure the main window as a compact overlay.
- Enable always-on-top behavior.
- Use a transparent or near-transparent background.
- Remove standard window decorations if practical.
- Keep the window resizable only if needed for debugging.

### Verification

- Run `bun run tauri dev`.
- Verify the app opens as a small floating subtitle window.
- Verify the UI remains usable and readable.

Result:

- `bun run tauri dev` launched the desktop app.
- The window opened as a compact floating subtitle island.
- Windows API probe showed `TopMost=True` and a window size around `736x269`.
- Visual screenshot confirmed the UI is readable and standard window chrome is not visibly present.

## Milestone 3: Tauri Session Plumbing

Status: Complete.

Connect Svelte to Rust with commands and events before adding real audio.

### Scope

- Add Tauri commands:
  - `start_session`
  - `stop_session`
- Add backend session state.
- Emit mocked transcript events from Rust to the frontend.
- Listen for transcript events in Svelte and update the subtitle island.

### Verification

- Press Start and verify transcript events appear in the UI.
- Press Stop and verify transcript updates stop.
- Run `bun run check`.
- Run Rust checks if commands or state are added.

Result:

- Rust commands `start_session` and `stop_session` are registered in Tauri.
- Rust session state starts and stops a mocked transcript worker.
- Svelte listens for `transcript-update` events and keeps the latest two subtitle lines visible.
- `agent-browser` verified Start/Stop behavior in the running frontend.
- `bun run check` passed.
- `cargo check` passed.

## Milestone 4: Microphone Capture

Status: Complete.

Replace mocked backend activity with real microphone capture.

### Scope

- Add microphone capture in Rust, likely with `cpal`.
- Use the default input device first.
- Capture PCM audio chunks.
- Track a simple audio activity signal for debugging.
- Stop capture cleanly when the session stops.

### Verification

- Start a session and speak into the microphone.
- Verify audio activity changes when speaking.
- Stop the session and verify capture stops without hanging.

Result:

- Rust starts a `cpal` stream from the default input device when the session starts.
- The backend emits `audio-activity` events with a simple RMS level and peak signal.
- Svelte shows the current microphone activity as a compact meter.
- Stop drops the capture stream through the session worker and returns the UI to Idle.
- Browser development mode keeps mocked transcript lines and mocked meter activity for local UI checks.

## Milestone 5: Realtime Transcription

Status: Complete.

Connect microphone chunks to OpenAI Realtime transcription.

### Scope

- Read the OpenAI API key from `OPENAI_API_KEY`.
- Support a root `.env` file as a development override for `OPENAI_API_KEY`.
- Open an OpenAI Realtime transcription websocket session with `gpt-realtime-whisper`.
- Use manual audio commits based on local speech and silence timing.
- Normalize microphone audio to 24 kHz mono PCM before streaming.
- Stream microphone PCM chunks to the Realtime session.
- Receive input audio transcription delta and completed events.
- Emit partial and final transcript updates to the frontend, replacing partial text with final text for the same transcript segment.
- Show a UI-only starting state while the backend opens the transcription session.

### Verification

- Speak into the microphone and verify subtitles appear.
- Confirm partial text updates before final text when available.
- Confirm missing API key and connection errors are shown in the UI.
- Check observed latency against the Phase 1 target.

### Decisions

- Do not add a provider abstraction yet. Milestone 5 supports OpenAI Realtime only; future Google support can introduce an abstraction once there is a real second provider.
- Do not add automatic reconnect yet. Connection and API errors stop the session and surface a session error so the user can restart.
- Do not persist transcript history. Transcript text remains ephemeral session display data.
- Do not add a new ADR for audio normalization or Realtime wiring because these choices follow the existing backend boundary ADR and are reversible implementation details.

Result:

- Rust loads `OPENAI_API_KEY`, with root `.env` overriding the process environment when present.
- The backend opens an OpenAI Realtime transcription websocket session, streams microphone audio as 24 kHz mono PCM, and emits input audio transcription delta/completed events as transcript updates.
- Svelte shows a starting state while the backend opens the Realtime session and replaces partial transcript text with final text for the same transcript segment.
- Automated checks pass.
- Manual Realtime transcription verification passed with a valid OpenAI API key and microphone input: subtitles appeared, partial transcript text reached a final state, missing API key and connection errors were visible, and observed latency was acceptable for the Phase 1 target.

## Milestone 6: Phase 1 Hardening

Only add polish needed to make the first version usable.

### Scope

- Show clear recording/API status.
- Show concise error messages.
- Keep only the most recent subtitle lines visible.
- Avoid persistent transcript storage.
- Avoid device selection unless default microphone handling is not enough.

### Verification

- Run the app for a longer manual test session.
- Verify Start/Stop can be repeated.
- Verify the UI remains readable while other desktop windows are active.

## Recommended First Implementation Slice

Start with Milestones 1 to 3 only:

1. Build the subtitle island UI.
2. Configure the Tauri window as an overlay.
3. Add Start/Stop commands and mocked transcript events from Rust.

This creates a working end-to-end skeleton:

```text
Start
  -> Rust session starts
  -> transcript events emit
  -> Svelte receives events
  -> subtitle island updates
Stop
  -> transcript events stop
```

After this skeleton is stable, add microphone capture, then OpenAI Realtime transcription.
