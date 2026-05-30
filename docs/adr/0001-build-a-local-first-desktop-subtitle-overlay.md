# Build SubIs as a Local-First Desktop Subtitle Overlay

SubIs is a local-first desktop app that captures audio on the user's machine and renders subtitles in an always-on-top overlay, instead of joining meetings as a bot or integrating with specific meeting platform APIs. Local-first means the app, capture, and session control live on the desktop; it does not require offline-only AI inference, so cloud transcription or translation services remain compatible with the product direction. This keeps the MVP useful across meetings, live talks, calls, videos, and podcasts, at the cost of owning native audio capture and desktop window behavior directly.

This follows the product vision and MVP non-goals in [SubIs PRD](../prd.md).

**Considered Options**

- Meeting bot or platform API integration would provide structured meeting context, but would tie the product to specific platforms and meeting permissions.
- Browser extension delivery would fit some web meetings and video sites, but would not cover native calls or in-person microphone use as directly.
