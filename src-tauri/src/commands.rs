use crate::session::{start_subtitle_session, stop_subtitle_session, SessionState};
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub(crate) fn start_session(
    app: tauri::AppHandle,
    state: State<'_, Mutex<SessionState>>,
) -> Result<(), String> {
    start_subtitle_session(app, state)
}

#[tauri::command]
pub(crate) fn stop_session(state: State<'_, Mutex<SessionState>>) -> Result<(), String> {
    stop_subtitle_session(state)
}

#[tauri::command]
pub(crate) fn close_app(
    window: tauri::WebviewWindow,
    state: State<'_, Mutex<SessionState>>,
) -> Result<(), String> {
    stop_subtitle_session(state)?;
    window.close().map_err(|error| error.to_string())
}

#[tauri::command]
pub(crate) fn begin_subtitle_island_drag(window: tauri::WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(|error| error.to_string())
}
