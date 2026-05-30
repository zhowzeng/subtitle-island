# Separate Click-Through Subtitles from Session Controls

Click-through behavior applies to the subtitle display area, not to the product's ability to control an active subtitle session. The app must preserve a non-click-through control path, such as a temporary control state, keyboard shortcut, tray/menu action, or separate control surface, so users can stop or adjust a session even when the subtitle island is letting pointer events pass to the app underneath.

**Considered Options**

- Making the entire overlay click-through would be visually clean, but could leave users without an obvious way to stop the session.
- Keeping all controls permanently non-click-through would be simpler, but would make the overlay more intrusive during meetings, talks, or video playback.
