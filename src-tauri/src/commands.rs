use crate::{
    session::{start_subtitle_session, stop_subtitle_session, SessionState},
    settings::{self, Settings},
};
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
pub(crate) fn start_session(
    app: tauri::AppHandle,
    state: State<'_, Mutex<SessionState>>,
) -> Result<(), String> {
    let options = settings::load_settings(&app).session_options();
    start_subtitle_session(app, state, options)
}

#[tauri::command]
pub(crate) fn stop_session(state: State<'_, Mutex<SessionState>>) -> Result<(), String> {
    stop_subtitle_session(state)
}

#[tauri::command]
pub(crate) fn get_settings(app: tauri::AppHandle) -> Settings {
    settings::load_settings(&app)
}

#[tauri::command]
pub(crate) fn save_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    settings::save_settings(&app, settings)
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
