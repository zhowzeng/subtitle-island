use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Emitter;
use tracing::error;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AudioActivity {
    pub(crate) level: f32,
    pub(crate) peak: f32,
    pub(crate) timestamp: u128,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionError {
    message: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TranscriptSegment {
    pub(crate) id: String,
    pub(crate) audio_source: &'static str,
    pub(crate) text: String,
    pub(crate) source_language: String,
    pub(crate) is_final: bool,
    pub(crate) timestamp: u128,
}

pub(crate) fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

pub(crate) fn log_session_error(message: &str) {
    error!(target: "subis::session", "{message}");
}

pub(crate) fn emit_session_error_message(app: &tauri::AppHandle, message: String) {
    log_session_error(&message);
    let _ = app.emit("session-error", SessionError { message });
}

pub(crate) fn emit_audio_activity(app: &tauri::AppHandle, activity: AudioActivity) {
    let _ = app.emit("audio-activity", activity);
}

pub(crate) fn emit_transcript_segment(app: &tauri::AppHandle, segment: TranscriptSegment) {
    let _ = app.emit("transcript-update", segment);
}
