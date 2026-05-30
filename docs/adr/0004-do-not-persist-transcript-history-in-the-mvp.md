# Do Not Persist Transcript History in the MVP

SubIs does not persist transcript history in the MVP; transcript text is treated as ephemeral session display data. This keeps the first version focused on live subtitles and avoids committing early to privacy, storage, search, export, or note-generation semantics before those workflows are designed.

**Considered Options**

- Persisting every transcript segment would make future notes and search easier, but would introduce sensitive data retention before the product has a clear user-facing storage model.
- Adding note generation now would expand the MVP beyond live subtitle and translation behavior.
