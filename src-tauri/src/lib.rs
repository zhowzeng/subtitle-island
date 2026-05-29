use serde::Serialize;
use std::{
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
  },
  thread::{self, JoinHandle},
  time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{Emitter, Manager, State};

#[derive(Default)]
struct SessionState {
  worker: Option<SessionWorker>,
}

struct SessionWorker {
  running: Arc<AtomicBool>,
  handle: JoinHandle<()>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TranscriptSegment {
  id: String,
  source: &'static str,
  text: &'static str,
  language: &'static str,
  is_final: bool,
  timestamp: u128,
}

const MOCK_TRANSCRIPTS: [&str; 4] = [
  "Testing the subtitle island layout.",
  "Rust is emitting mocked transcript events.",
  "The latest subtitle lines remain visible.",
  "Stop should end transcript updates cleanly.",
];

fn now_millis() -> u128 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or_default()
    .as_millis()
}

#[tauri::command]
fn start_session(app: tauri::AppHandle, state: State<'_, Mutex<SessionState>>) -> Result<(), String> {
  let mut session = state
    .lock()
    .map_err(|_| "Session state is unavailable.".to_string())?;

  if session.worker.is_some() {
    return Ok(());
  }

  let running = Arc::new(AtomicBool::new(true));
  let worker_running = Arc::clone(&running);

  let handle = thread::spawn(move || {
    let mut index = 0usize;

    while worker_running.load(Ordering::SeqCst) {
      let timestamp = now_millis();
      let segment = TranscriptSegment {
        id: format!("mock-{timestamp}-{index}"),
        source: "mic",
        text: MOCK_TRANSCRIPTS[index % MOCK_TRANSCRIPTS.len()],
        language: "en",
        is_final: index % 2 == 1,
        timestamp,
      };

      if app.emit("transcript-update", segment).is_err() {
        break;
      }

      index += 1;
      thread::sleep(Duration::from_millis(900));
    }
  });

  session.worker = Some(SessionWorker { running, handle });
  Ok(())
}

#[tauri::command]
fn stop_session(state: State<'_, Mutex<SessionState>>) -> Result<(), String> {
  let worker = {
    let mut session = state
      .lock()
      .map_err(|_| "Session state is unavailable.".to_string())?;
    session.worker.take()
  };

  if let Some(worker) = worker {
    worker.running.store(false, Ordering::SeqCst);
    worker
      .handle
      .join()
      .map_err(|_| "Transcript worker stopped unexpectedly.".to_string())?;
  }

  Ok(())
}

#[tauri::command]
fn close_window(window: tauri::WebviewWindow, state: State<'_, Mutex<SessionState>>) -> Result<(), String> {
  stop_session(state)?;
  window.close().map_err(|error| error.to_string())
}

#[tauri::command]
fn start_window_drag(window: tauri::WebviewWindow) -> Result<(), String> {
  window.start_dragging().map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .manage(Mutex::new(SessionState::default()))
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
          .build(),
        )?;
      }
      if let Some(window) = app.get_webview_window("main") {
        let _ = window.set_always_on_top(true);
        let _ = window.set_decorations(false);
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      start_session,
      stop_session,
      close_window,
      start_window_drag
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
