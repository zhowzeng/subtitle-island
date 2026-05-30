# SubIs

SubIs is a local-first desktop tool for live subtitles and translation. Its language centers on capturing audio from the user's machine, turning speech into readable subtitle text, and displaying that text as a low-interruption desktop overlay.

## Language

**Local-First**:
The app runs on the user's desktop, captures audio locally, and does not depend on meeting bots or meeting platform APIs. It may still use cloud transcription or translation services.
_Avoid_: offline-only, local model

**SubIs**:
The product and desktop app name. It includes the subtitle island surface and may later include settings, controls, and session note views.
_Avoid_: subis, Subtitle Island app

**Subtitle Island**:
A floating desktop subtitle surface inside SubIs that stays visible while the user works in another app. It is the product's primary viewing surface, not the whole app, a meeting participant, or a browser extension.
_Avoid_: caption widget, meeting bot, overlay app

**Subtitle Session**:
A user-started period during which audio is captured and converted into subtitles. A session begins with Start, ends with Stop, and is not defined by any single transcription connection.
_Avoid_: recording, meeting, job, connection

**Session Note**:
A future note artifact derived from a subtitle session. It is not limited to meetings and does not exist in the MVP.
_Avoid_: meeting note, transcript history, summary

**Audio Source**:
The single origin of captured audio for a subtitle session, either the microphone or system audio in the MVP.
_Avoid_: input, device, mixed source

**Source Language**:
The language detected or selected for speech recognition in a subtitle session.
_Avoid_: source, input language

**Microphone Capture**:
Audio capture from the user's microphone for in-person speech or local voice input.
_Avoid_: mic recording, voice recording

**System Audio Capture**:
Audio capture from the computer's playback output, such as online meetings, calls, videos, or podcasts.
_Avoid_: desktop audio, speaker recording, meeting audio

**Live Subtitle**:
Subtitle text shown while speech is still happening, optimized for low latency rather than transcript completeness.
_Avoid_: transcript, caption

**Subtitle Line**:
A display unit rendered inside the subtitle island. It may be derived from transcript text, translation text, or both, and does not have to match one transcript segment exactly.
_Avoid_: transcript segment, caption line

**Transcript**:
Text produced by speech recognition from captured audio. It is an input to subtitle display, not necessarily the exact text shown in the subtitle island.
_Avoid_: subtitle, caption

**Transcript Segment**:
A bounded piece of speech recognition text that may first appear as partial and later become final. The final form replaces the partial form for the same segment.
_Avoid_: subtitle line, message

**Partial Transcript**:
Non-final speech recognition text that may change as more audio arrives.
_Avoid_: draft subtitle, interim caption

**Final Transcript**:
Speech recognition text that the transcription service has marked as stable.
_Avoid_: completed subtitle, confirmed caption

**Translation**:
Text converted from the detected source language into Traditional Chinese when the source is not Chinese.
_Avoid_: localization, interpretation, Taiwan localization

**Partial Translation**:
Non-final translation text derived from partial transcript text. It may change when the underlying transcript segment changes.
_Avoid_: draft translation

**Final Translation**:
Translation text derived from final transcript text for the same transcript segment.
_Avoid_: confirmed translation

**Original Mode**:
A subtitle display mode that shows only source-language text, even when the source language is already Chinese.
_Avoid_: source mode, transcript mode

**Chinese Mode**:
A subtitle display mode that shows Chinese subtitle text. If the source language is already Chinese, it shows the source text without translation.
_Avoid_: translation mode, target mode

**Bilingual Mode**:
A subtitle display mode that shows both source-language text and Traditional Chinese text, with source text first and Chinese text second. Source text may appear before the Chinese text is available. If the source language is already Chinese, duplicate Chinese lines are collapsed into one display line.
_Avoid_: dual mode, mixed mode

**Click-Through Mode**:
A subtitle island mode that lets pointer interactions pass through the subtitle display area to the window underneath. It does not remove the need for a separate way to control or stop the subtitle session.
_Avoid_: transparent mode, passthrough, disabled controls
