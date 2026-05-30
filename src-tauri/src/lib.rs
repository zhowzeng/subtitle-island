mod api_key;
mod app;
mod audio;
mod commands;
mod events;
mod microphone;
mod realtime_transcription;
mod session;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    app::run();
}
