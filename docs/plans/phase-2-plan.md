# Phase 2 Plan

## Goal

Build meeting translation on top of the completed Phase 1 subtitle session:

1. Capture system audio.
2. Transcribe that audio through the existing Realtime transcription path.
3. Translate non-Chinese transcript text into Traditional Chinese.
4. Show translated live subtitles in the Subtitle Island.

Phase 2 should keep the product local-first: SubIs captures audio on the user's desktop and sends audio/text only to the configured external API. It must not introduce a meeting bot, meeting platform integration, account system, transcript persistence, speaker diarization, or full settings surface.

## Current Starting Point

Phase 1 is complete and provides:

- A compact always-on-top Subtitle Island.
- Start and Stop session controls.
- Tauri backend session ownership.
- Microphone Capture through `cpal`.
- Audio normalization to 24 kHz mono PCM.
- OpenAI Realtime transcription with partial and final transcript events.
- Ephemeral frontend display of the latest subtitle lines.
- Error handling for missing microphone, missing API key, and Realtime failures.

Before Phase 2 implementation, the app was microphone-first:

- `start_session` has no options.
- `session.rs` directly starts microphone capture.
- Realtime transcription emits `audioSource: "microphone"`.
- Frontend types already allow `audioSource: "microphone" | "system"` and `translation?: string`.

## Implementation Progress

Milestones 1 and 2 are complete as of commit `c096d72 feat: add system audio capture source`. Linux System Audio Capture was added in `2b94a76 feat: add linux system audio capture`. Milestone 3 is complete; it needed no code changes beyond Milestone 2.

Implemented:

- `start_session` accepts typed session options with an explicit Audio Source.
- The frontend exposes a minimal Audio Source control for `Microphone` and `System Audio`.
- Browser development mode remains mocked and uses the selected Audio Source.
- `src-tauri/` starts either Microphone Capture or Windows System Audio Capture from the same session boundary.
- Windows System Audio Capture uses the default output device through CPAL/WASAPI loopback.
- Linux System Audio Capture records the default sink monitor (`@DEFAULT_MONITOR@`) through the PulseAudio simple API, which also works on PipeWire via pipewire-pulse.
- Capture sources return a shared `AudioCapture` handle; dropping it stops capture.
- System audio chunks reuse the existing `AudioChunk`, `AudioNormalizer`, and Realtime transcription path.
- Transcript events carry the selected `audioSource`.
- `audio-activity` is emitted for both microphone and system audio.
- Source-aware errors are surfaced for capture and Realtime failures.

Verified:

- `bun run check`
- From `src-tauri/`: `cargo fmt --check`
- From `src-tauri/`: `cargo check --locked`
- From `src-tauri/`: `cargo test --locked`
- Manual Tauri check: Microphone Capture and System Audio Capture both work.
- Manual Tauri check on Linux: System Audio Capture produces Live Subtitle text, partial text is replaced by final text, and Stop during playback does not hang.
- Manual Tauri check on Linux: with Translate on, the Subtitle Island shows Traditional Chinese translated lines in Chinese Mode.
- Manual Tauri check on Linux: Chinese source audio is not duplicated as a translation.
- Manual Tauri check on Linux: a Translation failure shows `Translation failed` while the Subtitle Session keeps running.

Milestone 4 is complete:

- Translation is opt-in per Subtitle Session: `StartSessionOptions.translate` defaults to `false`, and a `Translate` toggle next to the Audio Source control sets it before Start. When it is off, the backend makes no Translation requests.
- `translation.rs` translates Final Transcript text into Traditional Chinese through the OpenAI Responses API with `gpt-6-luna`, reusing the session API key.
- A local heuristic skips Chinese text: Han characters count as one token each and runs of other letters count as one word, so code-switched Chinese is not translated. Text with kana or Hangul is always translated.
- The Final Transcript is emitted first without `translation`, then the same Transcript Segment is emitted again with `translation` when it arrives.
- Translation runs in a `JoinSet` on the session runtime, so Stop aborts in-flight requests.
- Translation failures are non-fatal: the backend emits `translation-error`, the line keeps its source text, and the status shows `Translation failed` until the next translated line.
- When Translate is on, the Subtitle Island uses Chinese Mode: it shows only lines with `translation` and Chinese source lines. The backend sets `sourceLanguage: "zh"` for Chinese text, including Partial Transcripts. Non-Chinese Partial Transcripts and Final Transcripts waiting for Translation are hidden, which trades 2-4 seconds of latency for a cleaner display. When Translate is off, the island shows source text.

Source Language follow-up (implemented after Milestone 4; manual Tauri check pending):

- `StartSessionOptions.sourceLanguage` accepts `auto`, `en`, or `zh` and defaults to `auto`. An `Auto | EN | ZH` toggle sets it before Start.
- `en` and `zh` send `language` in the Realtime transcription `session.update`, which should reduce misdetected languages. `auto` keeps auto-detect.
- The selected language becomes the segment `sourceLanguage`. `zh` segments are never translated. `auto` still uses the local Chinese heuristic.
- Simplified Chinese output from Chinese source audio is out of scope; the main use is English to Traditional Chinese.
- The active Audio Source and Source Language buttons stay highlighted while a session is running.

Not yet implemented:

- Chinese Mode display polish and mocked translated lines (Milestone 5).
- Long-session hardening and latency measurement.

## Assumptions

- Windows and Linux are the supported Phase 2 targets. Windows was first because the Phase 1 plan already used Windows as the initial platform and the PRD names WASAPI loopback for Windows system audio capture.
- macOS System Audio Capture remains a known risk and should not block the first Windows implementation slice.
- Phase 2 adds a minimal Audio Source choice, not a full settings panel. Full settings remain Phase 3 scope.
- Translation is derived from transcript text and remains ephemeral display data.
- The first translation implementation uses OpenAI only. A provider abstraction should wait until a second provider is real.
- Chinese Mode is enough for Phase 2: show Traditional Chinese when translation is available, and show source text while waiting.
- Bilingual Mode remains Phase 3 scope unless translation latency makes a source-plus-translation display necessary for usability.

## Grilled Decisions

### Decision 1: What does "meeting translation" mean?

Recommended answer: meeting translation means System Audio Capture plus Translation shown as Live Subtitle text. It does not mean joining a meeting, reading meeting metadata, identifying speakers, or saving a transcript.

Why this matters: it keeps Phase 2 aligned with Local-First, Subtitle Session, and the MVP non-goals.

### Decision 2: Should Phase 2 start with Windows-only system audio?

Recommended answer: yes. Implement Windows WASAPI loopback first, document macOS as a separate spike, and keep the capture boundary narrow enough to add macOS later.

Why this matters: macOS capture has product and packaging trade-offs around BlackHole versus ScreenCaptureKit. That decision is harder to reverse and should not be hidden inside the first Phase 2 implementation.

### Decision 3: Should Start accept an Audio Source?

Recommended answer: yes. Change `start_session` to accept an explicit Audio Source, defaulting to Microphone Capture in browser/mock development and allowing System Audio Capture in Tauri.

Why this matters: backend owns the session lifecycle and capture permissions. Passing a typed source keeps the Tauri command app-level while avoiding duplicate Start commands.

### Decision 4: Where should Translation run?

Recommended answer: run Translation in `src-tauri/` after transcript events are produced and before emitting display updates.

Why this matters: backend already owns API key access, long-running session work, and external API calls. Putting Translation in the frontend would expand the WebView trust surface and duplicate session state.

### Decision 5: Should partial transcripts be translated?

Recommended answer: start with final transcript translation first, then add partial translation only if observed latency or readability requires it.

Why this matters: translating every partial update can increase cost, flicker, and latency pressure. Final-first translation gives a stable baseline and keeps Phase 2 smaller.

### Decision 6: Should translated subtitles be emitted as separate events?

Recommended answer: keep a single `transcript-update` payload with optional `translation` for Phase 2.

Why this matters: the existing frontend type already models this, and one event preserves segment replacement semantics for partial/final updates. A separate event can be introduced later only if translation becomes asynchronous enough to need independent lifecycle state.

## Success Criteria

- The user can choose System Audio Capture and start a Subtitle Session.
- System audio from a meeting, video, or call produces Live Subtitle text.
- Non-Chinese final transcript text is translated into Traditional Chinese and shown in the Subtitle Island.
- Chinese source speech is not duplicated as a translation.
- Missing system audio device, missing API key, capture failure, transcription failure, and translation failure produce concise visible errors.
- Stop ends capture, transcription, and translation work cleanly.
- Start/Stop can be repeated without leaving stale workers.
- Observed translated subtitle latency is close to the PRD target of under 2 seconds for normal speech.
- Transcript and Translation text remain ephemeral and are not persisted.

## Milestone 1: Audio Source Boundary

Status: Complete.

Refactor the session boundary before adding new capture code.

### Scope

- Add an `AudioSource` type in Rust with `microphone` and `system`.
- Change `start_session` to accept typed session options, including Audio Source.
- Preserve existing microphone behavior as the default path.
- Update frontend Tauri wrapper types.
- Add a minimal Audio Source control in the Subtitle Island or route state.
- Keep browser development mode mocked and deterministic.

### Verification

- Run `bun run check`.
- From `src-tauri/`, run `cargo check --locked`.
- Start a microphone subtitle session and verify existing Phase 1 behavior still works.
- Confirm old repeated Start/Stop behavior still works.

## Milestone 2: Windows System Audio Capture

Status: Complete.

Linux follow-up: complete. Linux uses the PulseAudio default sink monitor instead of CPAL, because CPAL's Linux host is ALSA-only and has no loopback capture. Builds link `libpulse.so.0` directly when the PulseAudio dev package is missing, so only the runtime `libpulse0` library is required. On PipeWire the monitor follows sink mute, so a muted output device produces silent captured audio.

Add the first real System Audio Capture path.

### Scope

- Implement Windows WASAPI loopback capture in `src-tauri/`.
- Reuse the existing `AudioChunk` and `AudioNormalizer` path.
- Emit `audio-activity` for system audio.
- Pass the selected Audio Source through to emitted transcript segments.
- Keep microphone capture behavior unchanged.
- Surface concise errors when loopback capture is unavailable.

### Verification

- Play audio from a meeting, video, or browser tab.
- Start a session with System Audio Capture and verify the audio meter responds.
- Verify transcript events emit `audioSource: "system"`.
- Stop the session and verify capture stops cleanly.
- From `src-tauri/`, run `cargo check --locked` and `cargo test --locked`.

## Milestone 3: Transcription Pipeline Reuse

Status: Complete.

Prove that System Audio Capture can use the existing Realtime transcription path without forking transcription behavior.

### Scope

- Feed system audio chunks into `run_realtime_transcription`.
- Keep 24 kHz mono PCM normalization centralized.
- Preserve partial and final transcript replacement semantics.
- Make transcription error messages source-aware enough to be useful without exposing implementation details.

### Verification

- Speak or play English audio through system output and verify Live Subtitle text appears.
- Confirm partial text is replaced by final text for the same Transcript Segment.
- Confirm stopping during active playback does not hang.
- From `src-tauri/`, run `cargo clippy --all-targets --all-features --locked -- -D warnings`.

## Milestone 4: Final Transcript Translation

Status: Complete.

Add Translation for final transcript text.

### Scope

- Add a backend Translation module that calls OpenAI with the existing API key.
- Translate final non-Chinese transcript text into Traditional Chinese used in Taiwan.
- Attach `translation` to the same Transcript Segment payload.
- Skip Translation when the source is Chinese or when the transcript text is empty.
- Hide non-Chinese source text while Translation is pending. Mixed source and translated lines were too noisy in manual testing.
- Keep Translation failures visible but scoped to the session, without persisting text.

### Verification

- Use English system audio and verify Traditional Chinese translation appears.
- Use Chinese source audio and verify the UI does not duplicate Chinese lines.
- Simulate missing API key and translation API failure, then verify concise errors.
- Run `bun run check`.
- From `src-tauri/`, run `cargo check --locked` and `cargo test --locked`.

## Milestone 5: Subtitle Island Display for Phase 2

Make the Subtitle Island display translated meeting subtitles clearly without implementing full Phase 3 display modes.

### Scope

- In Chinese Mode, render `translation` when present.
- Revisit a dimmed source-text placeholder only if Chinese Mode latency feels too slow.
- Preserve only the latest visible subtitle lines.
- Label the active Audio Source concisely.
- Avoid full settings, shortcuts, click-through changes, or Bilingual Mode controls.

### Verification

- In browser development mode, verify mocked system-audio translated lines fit the existing island.
- In Tauri, verify System Audio Capture with translation remains readable over another desktop window.
- Run `bun run check`.

## Milestone 6: Phase 2 Hardening

Make meeting translation repeatable enough to use in real sessions.

### Scope

- Test repeated Start/Stop across microphone and system audio.
- Ensure worker cleanup covers capture, Realtime transcription, and Translation.
- Tune error messages for system audio and Translation.
- Measure rough latency from audible speech to translated subtitle.
- Document macOS System Audio Capture status if it remains unimplemented.

### Verification

- Run a 30-60 minute system audio subtitle session.
- Verify CPU/memory behavior is acceptable for a desktop overlay.
- Verify no transcript or Translation history is written to disk.
- Run:
  - `bun run check`
  - From `src-tauri/`: `cargo fmt --check`
  - From `src-tauri/`: `cargo check --locked`
  - From `src-tauri/`: `cargo test --locked`
  - From `src-tauri/`: `cargo clippy --all-targets --all-features --locked -- -D warnings`

## Out of Scope

- Bilingual Mode controls.
- Font size, opacity, width, and position settings.
- Shortcut keys.
- Click-Through Mode changes.
- Transcript persistence.
- Session Note generation.
- Speaker diarization.
- Meeting platform APIs.
- Provider abstraction.
- Automatic reconnect.
- macOS System Audio Capture implementation, unless Windows-first scope changes.

## Recommended First Implementation Slice

Start with Milestones 1 and 2 only:

1. Add typed Audio Source selection while preserving Microphone Capture.
2. Implement Windows System Audio Capture and prove audio activity works.

This creates the smallest useful Phase 2 base:

```text
Start with system audio
  -> Rust session starts selected capture source
  -> audio chunks flow through existing normalizer
  -> Realtime transcription emits system transcript segments
  -> Svelte renders live subtitle text
Stop
  -> capture and transcription stop
```

After that path is stable, add final transcript Translation and then tune the Subtitle Island display.

## ADR Assessment

No new ADR is required for the initial Phase 2 plan.

- Keeping capture, transcription, Translation, and session ownership in `src-tauri/` follows the existing Tauri boundary ADR.
- Not persisting transcript or Translation text follows the MVP data retention ADR.
- Windows-first WASAPI loopback is an implementation slice, not yet a hard-to-reverse product decision.
- Linux PulseAudio monitor capture is an implementation slice behind the same `start_system_audio_capture` boundary; switching to native PipeWire later only touches `system_audio.rs`.

Create a new ADR only if the project commits to a macOS System Audio Capture strategy with meaningful trade-offs, such as BlackHole versus ScreenCaptureKit, or if Translation provider strategy expands beyond OpenAI.
