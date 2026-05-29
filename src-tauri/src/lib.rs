use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use serde::Serialize;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
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
struct AudioActivity {
    level: f32,
    peak: f32,
    timestamp: u128,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionError {
    message: String,
}

fn now_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn activity_from_f32(samples: &[f32]) -> AudioActivity {
    if samples.is_empty() {
        return AudioActivity {
            level: 0.0,
            peak: 0.0,
            timestamp: now_millis(),
        };
    }

    let mut sum = 0.0f32;
    let mut peak = 0.0f32;

    for sample in samples {
        let value = sample.clamp(-1.0, 1.0).abs();
        sum += value * value;
        peak = peak.max(value);
    }

    AudioActivity {
        level: (sum / samples.len() as f32).sqrt().clamp(0.0, 1.0),
        peak,
        timestamp: now_millis(),
    }
}

fn activity_from_i16(samples: &[i16]) -> AudioActivity {
    let converted: Vec<f32> = samples
        .iter()
        .map(|sample| *sample as f32 / i16::MAX as f32)
        .collect();
    activity_from_f32(&converted)
}

fn activity_from_u16(samples: &[u16]) -> AudioActivity {
    let converted: Vec<f32> = samples
        .iter()
        .map(|sample| (*sample as f32 - 32768.0) / 32768.0)
        .collect();
    activity_from_f32(&converted)
}

fn emit_activity(app: &tauri::AppHandle, last_emit: &mut Instant, activity: AudioActivity) {
    if last_emit.elapsed() < Duration::from_millis(100) {
        return;
    }

    *last_emit = Instant::now();
    let _ = app.emit("audio-activity", activity);
}

fn emit_session_error(app: &tauri::AppHandle, error: cpal::StreamError) {
    let _ = app.emit(
        "session-error",
        SessionError {
            message: format!("Microphone capture failed: {error}"),
        },
    );
}

fn start_microphone_capture(app: tauri::AppHandle) -> Result<cpal::Stream, String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| "No default microphone input device was found.".to_string())?;
    let supported_config = device
        .default_input_config()
        .map_err(|error| format!("Could not read default microphone config: {error}"))?;
    let config = supported_config.config();

    let stream = match supported_config.sample_format() {
        cpal::SampleFormat::F32 => {
            let app_for_audio = app.clone();
            let app_for_error = app.clone();
            let mut last_emit = Instant::now() - Duration::from_millis(100);
            device.build_input_stream(
                &config,
                move |data: &[f32], _| {
                    emit_activity(&app_for_audio, &mut last_emit, activity_from_f32(data))
                },
                move |error| emit_session_error(&app_for_error, error),
                None,
            )
        }
        cpal::SampleFormat::I16 => {
            let app_for_audio = app.clone();
            let app_for_error = app.clone();
            let mut last_emit = Instant::now() - Duration::from_millis(100);
            device.build_input_stream(
                &config,
                move |data: &[i16], _| {
                    emit_activity(&app_for_audio, &mut last_emit, activity_from_i16(data))
                },
                move |error| emit_session_error(&app_for_error, error),
                None,
            )
        }
        cpal::SampleFormat::U16 => {
            let app_for_audio = app.clone();
            let app_for_error = app.clone();
            let mut last_emit = Instant::now() - Duration::from_millis(100);
            device.build_input_stream(
                &config,
                move |data: &[u16], _| {
                    emit_activity(&app_for_audio, &mut last_emit, activity_from_u16(data))
                },
                move |error| emit_session_error(&app_for_error, error),
                None,
            )
        }
        sample_format => {
            return Err(format!(
                "Unsupported microphone sample format: {sample_format:?}"
            ));
        }
    }
    .map_err(|error| format!("Could not start microphone stream: {error}"))?;

    stream
        .play()
        .map_err(|error| format!("Could not play microphone stream: {error}"))?;

    Ok(stream)
}

#[tauri::command]
fn start_session(
    app: tauri::AppHandle,
    state: State<'_, Mutex<SessionState>>,
) -> Result<(), String> {
    let mut session = state
        .lock()
        .map_err(|_| "Session state is unavailable.".to_string())?;

    if session.worker.is_some() {
        return Ok(());
    }

    let running = Arc::new(AtomicBool::new(true));
    let worker_running = Arc::clone(&running);
    let (ready_sender, ready_receiver) = mpsc::channel();

    let handle = thread::spawn(move || {
        let stream = match start_microphone_capture(app) {
            Ok(stream) => stream,
            Err(error) => {
                let _ = ready_sender.send(Err(error));
                return;
            }
        };

        let _ = ready_sender.send(Ok(()));

        while worker_running.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_millis(50));
        }

        drop(stream);
    });

    match ready_receiver.recv() {
        Ok(Ok(())) => {
            session.worker = Some(SessionWorker { running, handle });
        }
        Ok(Err(error)) => {
            let _ = handle.join();
            return Err(error);
        }
        Err(_) => {
            let _ = handle.join();
            return Err("Microphone capture worker stopped unexpectedly.".to_string());
        }
    }

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
            .map_err(|_| "Microphone capture worker stopped unexpectedly.".to_string())?;
    }

    Ok(())
}

#[tauri::command]
fn close_window(
    window: tauri::WebviewWindow,
    state: State<'_, Mutex<SessionState>>,
) -> Result<(), String> {
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
