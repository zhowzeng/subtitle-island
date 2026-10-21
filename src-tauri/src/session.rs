use crate::{
    api_key::load_openai_api_key,
    audio::AudioChunk,
    events::{emit_session_error_message, log_session_error},
    microphone::start_microphone_capture,
    realtime_transcription::run_realtime_transcription,
    system_audio::start_system_audio_capture,
};
use serde::Deserialize;
use std::{
    any::Any,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};
use tauri::State;

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum AudioSource {
    #[default]
    Microphone,
    System,
}

impl AudioSource {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            AudioSource::Microphone => "microphone",
            AudioSource::System => "system",
        }
    }

    fn capture_name(self) -> &'static str {
        match self {
            AudioSource::Microphone => "Microphone Capture",
            AudioSource::System => "System Audio Capture",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) enum SourceLanguage {
    #[default]
    Auto,
    En,
    Zh,
}

impl SourceLanguage {
    /// ISO-639-1 code sent to transcription, or `None` for auto-detect.
    pub(crate) fn code(self) -> Option<&'static str> {
        match self {
            SourceLanguage::Auto => None,
            SourceLanguage::En => Some("en"),
            SourceLanguage::Zh => Some("zh"),
        }
    }
}

#[derive(Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StartSessionOptions {
    #[serde(default)]
    pub(crate) audio_source: AudioSource,
    #[serde(default)]
    pub(crate) source_language: SourceLanguage,
    #[serde(default)]
    pub(crate) translate: bool,
}

#[derive(Default)]
pub(crate) struct SessionState {
    worker: Option<SessionWorker>,
}

struct SessionWorker {
    audio_source: AudioSource,
    running: Arc<AtomicBool>,
    handle: JoinHandle<()>,
}

pub(crate) fn start_subtitle_session(
    app: tauri::AppHandle,
    state: State<'_, Mutex<SessionState>>,
    options: StartSessionOptions,
) -> Result<(), String> {
    join_finished_worker(&state)?;

    let mut session = state
        .lock()
        .map_err(|_| "Session state is unavailable.".to_string())?;

    if session.worker.is_some() {
        return Ok(());
    }

    let running = Arc::new(AtomicBool::new(true));
    let worker_running = Arc::clone(&running);
    let audio_source = options.audio_source;
    let (ready_sender, ready_receiver) = mpsc::channel();
    let (audio_sender, audio_receiver) = mpsc::sync_channel::<AudioChunk>(32);
    let api_key = load_openai_api_key().inspect_err(|error| {
        log_session_error(error);
    })?;

    let ready_sender_for_panic = ready_sender.clone();
    let handle = thread::spawn(move || {
        let worker_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let (stream, input_sample_rate) = match match audio_source {
                AudioSource::Microphone => start_microphone_capture(app.clone(), audio_sender),
                AudioSource::System => start_system_audio_capture(app.clone(), audio_sender),
            } {
                Ok(capture) => capture,
                Err(error) => {
                    send_ready_error(&ready_sender, error);
                    return;
                }
            };

            let runtime = match tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                Ok(runtime) => runtime,
                Err(error) => {
                    send_ready_error(
                        &ready_sender,
                        format!("Could not start transcription runtime: {error}"),
                    );
                    return;
                }
            };

            if let Err(error) = runtime.block_on(run_realtime_transcription(
                app.clone(),
                Arc::clone(&worker_running),
                audio_receiver,
                input_sample_rate,
                options,
                api_key,
                ready_sender,
            )) {
                if worker_running.load(Ordering::SeqCst) {
                    emit_session_error_message(&app, error);
                }
            }

            drop(stream);
        }));

        if let Err(payload) = worker_result {
            let message = format!(
                "{} worker crashed: {}",
                audio_source.capture_name(),
                panic_message(payload.as_ref())
            );
            send_ready_error(&ready_sender_for_panic, message.clone());

            if worker_running.load(Ordering::SeqCst) {
                emit_session_error_message(&app, message);
            }
        }
    });

    session.worker = Some(SessionWorker {
        audio_source,
        running: Arc::clone(&running),
        handle,
    });
    drop(session);

    match ready_receiver.recv() {
        Ok(Ok(())) => {
            if !running.load(Ordering::SeqCst) {
                return Err("Session start was cancelled.".to_string());
            }
        }
        Ok(Err(error)) => {
            join_starting_worker(&state, &running);
            return Err(error);
        }
        Err(_) => {
            join_starting_worker(&state, &running);
            return Err("Microphone capture worker stopped unexpectedly.".to_string());
        }
    }

    Ok(())
}

pub(crate) fn stop_subtitle_session(state: State<'_, Mutex<SessionState>>) -> Result<(), String> {
    let worker = {
        let mut session = state
            .lock()
            .map_err(|_| "Session state is unavailable.".to_string())?;
        session.worker.take()
    };

    if let Some(worker) = worker {
        worker.running.store(false, Ordering::SeqCst);
        worker.handle.join().map_err(|_| {
            let message = format!(
                "{} worker stopped unexpectedly.",
                worker.audio_source.capture_name()
            );
            log_session_error(&message);
            message
        })?;
    }

    Ok(())
}

pub(crate) fn send_ready_error(sender: &Sender<Result<(), String>>, message: String) {
    log_session_error(&message);
    let _ = sender.send(Err(message));
}

fn join_finished_worker(state: &State<'_, Mutex<SessionState>>) -> Result<(), String> {
    let worker = {
        let mut session = state
            .lock()
            .map_err(|_| "Session state is unavailable.".to_string())?;

        if session
            .worker
            .as_ref()
            .is_some_and(|worker| worker.handle.is_finished())
        {
            session.worker.take()
        } else {
            None
        }
    };

    if let Some(worker) = worker {
        worker.running.store(false, Ordering::SeqCst);
        worker.handle.join().map_err(|_| {
            let message = format!(
                "{} worker stopped unexpectedly.",
                worker.audio_source.capture_name()
            );
            log_session_error(&message);
            message
        })?;
    }

    Ok(())
}

fn join_starting_worker(state: &State<'_, Mutex<SessionState>>, running: &Arc<AtomicBool>) {
    let worker = {
        let Ok(mut session) = state.lock() else {
            return;
        };

        let is_starting_worker = session
            .worker
            .as_ref()
            .is_some_and(|worker| Arc::ptr_eq(&worker.running, running));

        if is_starting_worker {
            session.worker.take()
        } else {
            None
        }
    };

    if let Some(worker) = worker {
        worker.running.store(false, Ordering::SeqCst);
        let _ = worker.handle.join();
    }
}

fn panic_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }

    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }

    "unknown panic payload".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panic_message_reads_string_payloads() {
        let message = "worker failed".to_string();

        assert_eq!(panic_message(&message), "worker failed");
    }

    #[test]
    fn panic_message_reads_str_payloads() {
        let message = "worker failed";

        assert_eq!(panic_message(&message), "worker failed");
    }
}
