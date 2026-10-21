# Performance Baseline

This document records measured non-functional behavior for SubIs. Targets live in [prd.md](prd.md) (NFR-001 Latency, NFR-002 Resource Usage). Add a new row when a change may affect latency or resource usage, so results can be compared across phases.

## How to Measure

1. Run a release build: `bun run tauri dev --release`. Debug builds are unoptimized and overstate CPU usage.
2. Start a Subtitle Session with the Audio Source, Source Language, and Translate settings under test.
3. Read latency from the `subis::latency` lines in the terminal (stderr). They carry durations only, never transcript or Translation text:
   - `Final Transcript N ms after commit`: from the Realtime `input_audio_buffer.committed` event to the Final Transcript.
   - `Translation N ms`: one Translation request.
4. In another terminal, sample CPU and memory once per minute:

   ```sh
   while sleep 60; do date +%T; top -b -n1 | grep -Ei 'subis|webkit'; done
   ```

   Compute average CPU from the `TIME+` column (CPU time delta divided by wall time). A single `top` sample is noisy, and `ps` `%CPU` is a lifetime average.

End-to-end latency from the end of speech to a translated subtitle is estimated as:

```text
silence commit delay (350 ms) + Final Transcript after commit + Translation
```

Continuous speech without pauses is committed every 2.5 seconds (`OPENAI_MAX_COMMIT_DELAY`).

## Baseline

| Date | Commit | Platform | Setup | Final Transcript after commit | Translation | End-to-end (est.) | CPU (one core) | RSS | Session length |
|---|---|---|---|---|---|---|---|---|---|
| 2026-10-21 | `783231d` | Linux, release | System Audio, Translate on, English source | median ~500 ms (300-700 ms, rare 2-7 s) | median ~1.75 s (1.3-2.3 s after the first minute) | ~2.6 s | not re-measured | not re-measured | ~3 min |
| 2026-10-21 | `92b16d5` | Linux, release | System Audio, Translate on, English source, reasoning at model default | median ~500 ms | median ~2.6 s (1.6-5.3 s) | ~3.5 s | `subis` ~19%, WebKit ~21% | `subis` ~184 MB flat; WebKit 243 to 268 MB, growth slowing | ~8 min |

## Not Yet Measured

- A 30-60 minute session.
- Partial Transcript latency (NFR-001 target: under 1 second).
- Windows.
