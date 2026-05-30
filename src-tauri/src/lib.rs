use audioadapter_buffers::direct::InterleavedSlice;
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use futures_util::{SinkExt, StreamExt};
use rubato::{
    calculate_cutoff, Async, FixedAsync, Resampler, SincInterpolationParameters,
    SincInterpolationType, WindowFunction,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::{
    any::Any,
    collections::HashMap,
    env, fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender, SyncSender, TryRecvError},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::{Emitter, Manager, State};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue, protocol::Message},
};
use tracing::error;

const OPENAI_REALTIME_TRANSCRIPTION_URL: &str =
    "wss://api.openai.com/v1/realtime?intent=transcription";
const OPENAI_TRANSCRIPTION_MODEL: &str = "gpt-realtime-whisper";
const OPENAI_TARGET_SAMPLE_RATE: u32 = 24_000;
const OPENAI_AUDIO_CHUNK_FRAMES: usize = 1024;
const OPENAI_SPEECH_RMS_THRESHOLD: f32 = 0.006;
const OPENAI_SILENCE_COMMIT_DELAY: Duration = Duration::from_millis(350);
const OPENAI_MAX_COMMIT_DELAY: Duration = Duration::from_millis(2_500);

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

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct TranscriptSegment {
    id: String,
    audio_source: &'static str,
    text: String,
    source_language: String,
    is_final: bool,
    timestamp: u128,
}

struct AudioChunk {
    samples: Vec<f32>,
}

struct AudioNormalizer {
    input_sample_rate: u32,
    pending_mono: Vec<f64>,
    resampler: Option<Async<f64>>,
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

fn init_tracing() {
    let subscriber = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_max_level(tracing::Level::INFO)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);
}

fn log_session_error(message: &str) {
    error!(target: "subis::session", "{message}");
}

fn emit_session_error_message(app: &tauri::AppHandle, message: String) {
    log_session_error(&message);
    let _ = app.emit("session-error", SessionError { message });
}

fn send_ready_error(sender: &Sender<Result<(), String>>, message: String) {
    log_session_error(&message);
    let _ = sender.send(Err(message));
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
    emit_session_error_message(app, format!("Microphone capture failed: {error}"));
}

fn load_openai_api_key() -> Result<String, String> {
    if let Ok(dotenv_path) = env::current_dir().map(root_dotenv_path_from) {
        if fs::metadata(&dotenv_path).is_ok() {
            dotenvy::from_path_override(&dotenv_path)
                .map_err(|error| format!("Could not load .env: {error}"))?;
        }
    }

    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| "Missing OpenAI API key. Set OPENAI_API_KEY.".to_string())?;

    if api_key.trim().is_empty() {
        return Err("Missing OpenAI API key. Set OPENAI_API_KEY.".to_string());
    }

    Ok(api_key)
}

fn root_dotenv_path_from(current_dir: PathBuf) -> PathBuf {
    for path in current_dir.ancestors() {
        if path.join("package.json").is_file() && path.join("src-tauri").is_dir() {
            return path.join(".env");
        }
    }

    current_dir.join(".env")
}

fn chunk_from_interleaved_f32(samples: &[f32], channels: usize) -> AudioChunk {
    if channels <= 1 {
        return AudioChunk {
            samples: samples.to_vec(),
        };
    }

    let mut mono = Vec::with_capacity(samples.len() / channels);

    for frame in samples.chunks_exact(channels) {
        let sum: f32 = frame.iter().copied().sum();
        mono.push(sum / channels as f32);
    }

    AudioChunk { samples: mono }
}

fn chunk_from_interleaved_i16(samples: &[i16], channels: usize) -> AudioChunk {
    let converted: Vec<f32> = samples
        .iter()
        .map(|sample| *sample as f32 / i16::MAX as f32)
        .collect();
    chunk_from_interleaved_f32(&converted, channels)
}

fn chunk_from_interleaved_u16(samples: &[u16], channels: usize) -> AudioChunk {
    let converted: Vec<f32> = samples
        .iter()
        .map(|sample| (*sample as f32 - 32768.0) / 32768.0)
        .collect();
    chunk_from_interleaved_f32(&converted, channels)
}

fn samples_contain_speech(samples: &[f32]) -> bool {
    if samples.is_empty() {
        return false;
    }

    let mut sum = 0.0f32;

    for sample in samples {
        let value = sample.clamp(-1.0, 1.0).abs();
        sum += value * value;
    }

    let rms = (sum / samples.len() as f32).sqrt();

    rms >= OPENAI_SPEECH_RMS_THRESHOLD
}

impl AudioNormalizer {
    fn new(input_sample_rate: u32) -> Result<Self, String> {
        let resampler = if input_sample_rate == OPENAI_TARGET_SAMPLE_RATE {
            None
        } else {
            let window = WindowFunction::Blackman2;
            let sinc_len = 128;
            let params = SincInterpolationParameters {
                sinc_len,
                f_cutoff: calculate_cutoff(sinc_len, window),
                interpolation: SincInterpolationType::Quadratic,
                oversampling_factor: 256,
                window,
            };
            Some(
                Async::<f64>::new_sinc(
                    OPENAI_TARGET_SAMPLE_RATE as f64 / input_sample_rate as f64,
                    1.1,
                    &params,
                    OPENAI_AUDIO_CHUNK_FRAMES,
                    1,
                    FixedAsync::Input,
                )
                .map_err(|error| format!("Could not create audio resampler: {error}"))?,
            )
        };

        Ok(Self {
            input_sample_rate,
            pending_mono: Vec::new(),
            resampler,
        })
    }

    fn push(&mut self, chunk: AudioChunk) -> Result<Vec<Vec<u8>>, String> {
        self.pending_mono.extend(
            chunk
                .samples
                .into_iter()
                .map(|sample| f64::from(sample.clamp(-1.0, 1.0))),
        );

        if self.input_sample_rate == OPENAI_TARGET_SAMPLE_RATE {
            let samples = std::mem::take(&mut self.pending_mono);
            return Ok(vec![pcm16_bytes_from_f64(&samples)]);
        }

        let Some(resampler) = self.resampler.as_mut() else {
            return Ok(Vec::new());
        };

        let mut output_chunks = Vec::new();

        while self.pending_mono.len() >= resampler.input_frames_next() {
            let input_frames = resampler.input_frames_next();
            let input_data: Vec<f64> = self.pending_mono.drain(..input_frames).collect();
            let input = InterleavedSlice::new(&input_data, 1, input_frames)
                .map_err(|error| format!("Could not prepare resampler input: {error}"))?;
            let output = resampler
                .process(&input, 0, None)
                .map_err(|error| format!("Could not resample microphone audio: {error}"))?;
            output_chunks.push(pcm16_bytes_from_f64(&output.take_data()));
        }

        Ok(output_chunks)
    }
}

fn pcm16_bytes_from_f64(samples: &[f64]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(samples.len() * 2);

    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f64).round() as i16;
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    bytes
}

fn start_microphone_capture(
    app: tauri::AppHandle,
    audio_sender: SyncSender<AudioChunk>,
) -> Result<(cpal::Stream, u32), String> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .ok_or_else(|| "No default microphone input device was found.".to_string())?;
    let supported_config = device
        .default_input_config()
        .map_err(|error| format!("Could not read default microphone config: {error}"))?;
    let config = supported_config.config();
    let channels = usize::from(config.channels);
    let sample_rate = config.sample_rate;

    let stream = match supported_config.sample_format() {
        cpal::SampleFormat::F32 => {
            let app_for_audio = app.clone();
            let app_for_error = app.clone();
            let sender = audio_sender.clone();
            let mut last_emit = Instant::now() - Duration::from_millis(100);
            device.build_input_stream(
                &config,
                move |data: &[f32], _| {
                    emit_activity(&app_for_audio, &mut last_emit, activity_from_f32(data));
                    let _ = sender.try_send(chunk_from_interleaved_f32(data, channels));
                },
                move |error| emit_session_error(&app_for_error, error),
                None,
            )
        }
        cpal::SampleFormat::I16 => {
            let app_for_audio = app.clone();
            let app_for_error = app.clone();
            let sender = audio_sender.clone();
            let mut last_emit = Instant::now() - Duration::from_millis(100);
            device.build_input_stream(
                &config,
                move |data: &[i16], _| {
                    emit_activity(&app_for_audio, &mut last_emit, activity_from_i16(data));
                    let _ = sender.try_send(chunk_from_interleaved_i16(data, channels));
                },
                move |error| emit_session_error(&app_for_error, error),
                None,
            )
        }
        cpal::SampleFormat::U16 => {
            let app_for_audio = app.clone();
            let app_for_error = app.clone();
            let sender = audio_sender.clone();
            let mut last_emit = Instant::now() - Duration::from_millis(100);
            device.build_input_stream(
                &config,
                move |data: &[u16], _| {
                    emit_activity(&app_for_audio, &mut last_emit, activity_from_u16(data));
                    let _ = sender.try_send(chunk_from_interleaved_u16(data, channels));
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

    Ok((stream, sample_rate))
}

async fn run_realtime_transcription(
    app: tauri::AppHandle,
    running: Arc<AtomicBool>,
    audio_receiver: Receiver<AudioChunk>,
    input_sample_rate: u32,
    api_key: String,
    ready_sender: Sender<Result<(), String>>,
) -> Result<(), String> {
    let mut request = match OPENAI_REALTIME_TRANSCRIPTION_URL.into_client_request() {
        Ok(request) => request,
        Err(error) => {
            let message = format!("Could not prepare Realtime request: {error}");
            send_ready_error(&ready_sender, message);
            return Ok(());
        }
    };
    request.headers_mut().insert(
        "Authorization",
        match HeaderValue::from_str(&format!("Bearer {api_key}")) {
            Ok(header) => header,
            Err(error) => {
                let message = format!("Could not prepare API authorization header: {error}");
                send_ready_error(&ready_sender, message);
                return Ok(());
            }
        },
    );

    let connect = connect_async(request);
    tokio::pin!(connect);
    let (socket, _) = loop {
        if !running.load(Ordering::SeqCst) {
            send_ready_error(&ready_sender, "Session start was cancelled.".to_string());
            return Ok(());
        }

        tokio::select! {
            connection = &mut connect => {
                match connection {
                    Ok(connection) => break connection,
                    Err(error) => {
                        let message = format!("Could not connect to OpenAI Realtime: {error}");
                        send_ready_error(&ready_sender, message);
                        return Ok(());
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(20)) => {}
        }
    };
    let (mut writer, mut reader) = socket.split();
    let mut normalizer = match AudioNormalizer::new(input_sample_rate) {
        Ok(normalizer) => normalizer,
        Err(error) => {
            send_ready_error(&ready_sender, error);
            return Ok(());
        }
    };

    let session_update = writer.send(Message::Text(
        json!({
            "type": "session.update",
            "session": {
                "type": "transcription",
                "audio": {
                    "input": {
                        "format": {
                            "type": "audio/pcm",
                            "rate": OPENAI_TARGET_SAMPLE_RATE
                        },
                        "transcription": {
                            "model": OPENAI_TRANSCRIPTION_MODEL
                        }
                    }
                }
            }
        })
        .to_string()
        .into(),
    ));
    tokio::pin!(session_update);
    loop {
        if !running.load(Ordering::SeqCst) {
            send_ready_error(&ready_sender, "Session start was cancelled.".to_string());
            return Ok(());
        }

        tokio::select! {
            result = &mut session_update => {
                if let Err(error) = result {
                    let message = format!("Could not configure Realtime transcription: {error}");
                    send_ready_error(&ready_sender, message);
                    return Ok(());
                }
                break;
            }
            _ = tokio::time::sleep(Duration::from_millis(20)) => {}
        }
    }

    let _ = ready_sender.send(Ok(()));
    let mut transcript_accumulator = HashMap::new();
    let mut has_uncommitted_audio = false;
    let mut buffer_started_at = None;
    let mut last_voice_at = None;

    while running.load(Ordering::SeqCst) {
        loop {
            match audio_receiver.try_recv() {
                Ok(chunk) => {
                    if samples_contain_speech(&chunk.samples) {
                        last_voice_at = Some(Instant::now());
                    }

                    for pcm_bytes in normalizer.push(chunk)? {
                        if pcm_bytes.is_empty() {
                            continue;
                        }

                        writer
                            .send(Message::Text(
                                json!({
                                    "type": "input_audio_buffer.append",
                                    "audio": BASE64_STANDARD.encode(pcm_bytes)
                                })
                                .to_string()
                                .into(),
                            ))
                            .await
                            .map_err(|error| {
                                format!("Could not stream microphone audio to OpenAI: {error}")
                            })?;
                        buffer_started_at.get_or_insert_with(Instant::now);
                        has_uncommitted_audio = true;
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    return Err("Microphone audio stream stopped unexpectedly.".to_string());
                }
            }
        }

        let should_commit = if has_uncommitted_audio {
            last_voice_at
                .is_some_and(|last_voice| last_voice.elapsed() >= OPENAI_SILENCE_COMMIT_DELAY)
                || buffer_started_at
                    .is_some_and(|buffer_start| buffer_start.elapsed() >= OPENAI_MAX_COMMIT_DELAY)
        } else {
            false
        };

        if should_commit {
            writer
                .send(Message::Text(
                    json!({
                        "type": "input_audio_buffer.commit"
                    })
                    .to_string()
                    .into(),
                ))
                .await
                .map_err(|error| format!("Could not commit microphone audio to OpenAI: {error}"))?;
            has_uncommitted_audio = false;
            buffer_started_at = None;
            last_voice_at = None;
        }

        tokio::select! {
            message = reader.next() => {
                match message {
                    Some(Ok(message)) => handle_realtime_message(
                        &app,
                        message,
                        &mut transcript_accumulator,
                    )?,
                    Some(Err(error)) => return Err(format!("OpenAI Realtime connection failed: {error}")),
                    None => return Err("OpenAI Realtime connection closed.".to_string()),
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(20)) => {}
        }
    }

    if has_uncommitted_audio {
        let _ = writer
            .send(Message::Text(
                json!({
                    "type": "input_audio_buffer.commit"
                })
                .to_string()
                .into(),
            ))
            .await;
    }
    let _ = writer.close().await;
    Ok(())
}

fn handle_realtime_message(
    app: &tauri::AppHandle,
    message: Message,
    transcript_accumulator: &mut HashMap<String, String>,
) -> Result<(), String> {
    let Message::Text(text) = message else {
        return Ok(());
    };

    let value: Value = serde_json::from_str(&text)
        .map_err(|error| format!("Could not read OpenAI Realtime event: {error}"))?;
    let event_type = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();

    match event_type {
        "conversation.item.input_audio_transcription.delta" => {
            let delta = value
                .get("delta")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if !delta.is_empty() {
                let transcript =
                    accumulated_transcript_delta(&value, delta, transcript_accumulator);
                emit_transcript_segment(app, &value, transcript, false);
            }
        }
        "conversation.item.input_audio_transcription.completed" => {
            let transcript = value
                .get("transcript")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if !transcript.is_empty() {
                emit_transcript_segment(app, &value, transcript, true);
            }
            transcript_accumulator.remove(&transcript_segment_id(&value));
        }
        "error" => {
            let message = value
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("OpenAI Realtime returned an error.");
            return Err(message.to_string());
        }
        _ => {}
    }

    Ok(())
}

fn accumulated_transcript_delta<'a>(
    event: &Value,
    delta: &str,
    transcript_accumulator: &'a mut HashMap<String, String>,
) -> &'a str {
    let id = transcript_segment_id(event);
    let transcript = transcript_accumulator.entry(id).or_default();
    transcript.push_str(delta);
    transcript
}

fn transcript_segment_id(event: &Value) -> String {
    event
        .get("item_id")
        .and_then(Value::as_str)
        .or_else(|| event.get("event_id").and_then(Value::as_str))
        .unwrap_or("transcript-segment")
        .to_string()
}

fn emit_transcript_segment(app: &tauri::AppHandle, event: &Value, text: &str, is_final: bool) {
    let _ = app.emit(
        "transcript-update",
        TranscriptSegment {
            id: transcript_segment_id(event),
            audio_source: "microphone",
            text: text.to_string(),
            source_language: "auto".to_string(),
            is_final,
            timestamp: now_millis(),
        },
    );
}

#[tauri::command]
fn start_session(
    app: tauri::AppHandle,
    state: State<'_, Mutex<SessionState>>,
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
    let (ready_sender, ready_receiver) = mpsc::channel();
    let (audio_sender, audio_receiver) = mpsc::sync_channel(32);
    let api_key = load_openai_api_key().inspect_err(|error| {
        log_session_error(error);
    })?;

    let ready_sender_for_panic = ready_sender.clone();
    let handle = thread::spawn(move || {
        let worker_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let (stream, input_sample_rate) =
                match start_microphone_capture(app.clone(), audio_sender) {
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
                "Microphone capture worker crashed: {}",
                panic_message(payload.as_ref())
            );
            send_ready_error(&ready_sender_for_panic, message.clone());

            if worker_running.load(Ordering::SeqCst) {
                emit_session_error_message(&app, message);
            }
        }
    });

    session.worker = Some(SessionWorker {
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
            let message = "Microphone capture worker stopped unexpectedly.".to_string();
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
        worker.handle.join().map_err(|_| {
            let message = "Microphone capture worker stopped unexpectedly.".to_string();
            log_session_error(&message);
            message
        })?;
    }

    Ok(())
}

#[tauri::command]
fn close_app(
    window: tauri::WebviewWindow,
    state: State<'_, Mutex<SessionState>>,
) -> Result<(), String> {
    stop_session(state)?;
    window.close().map_err(|error| error.to_string())
}

#[tauri::command]
fn begin_subtitle_island_drag(window: tauri::WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_tracing();

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
            close_app,
            begin_subtitle_island_drag
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn root_dotenv_path_uses_project_root_from_src_tauri() {
        let src_tauri_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root_dotenv_path_from(src_tauri_dir.clone());

        assert_eq!(
            path,
            src_tauri_dir
                .parent()
                .expect("src-tauri should have a project root parent")
                .join(".env")
        );
    }

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

    #[test]
    fn transcription_deltas_accumulate_by_item_id() {
        let event = json!({
            "type": "conversation.item.input_audio_transcription.delta",
            "item_id": "item-1"
        });
        let mut accumulator = HashMap::new();

        assert_eq!(
            accumulated_transcript_delta(&event, "hello", &mut accumulator),
            "hello"
        );
        assert_eq!(
            accumulated_transcript_delta(&event, " world", &mut accumulator),
            "hello world"
        );
    }

    #[test]
    fn samples_contain_speech_detects_loud_audio() {
        assert!(samples_contain_speech(&[0.0, 0.02, -0.03, 0.01]));
    }

    #[test]
    fn samples_contain_speech_ignores_quiet_audio() {
        assert!(!samples_contain_speech(&[0.0, 0.002, -0.003, 0.001]));
    }
}
