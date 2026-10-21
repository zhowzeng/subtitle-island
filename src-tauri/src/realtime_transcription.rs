use crate::{
    audio::{samples_contain_speech, AudioChunk, AudioNormalizer, OPENAI_TARGET_SAMPLE_RATE},
    events::{
        emit_transcript_segment, emit_translation_error_message, now_millis, TranscriptSegment,
    },
    session::{send_ready_error, SourceLanguage, StartSessionOptions},
    translation::{
        is_chinese_text, needs_translation, translate_to_traditional_chinese, translation_client,
    },
};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine};
use futures_util::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender, TryRecvError},
        Arc,
    },
    time::{Duration, Instant},
};
use tokio::task::JoinSet;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{client::IntoClientRequest, http::HeaderValue, protocol::Message},
};

const OPENAI_REALTIME_TRANSCRIPTION_URL: &str =
    "wss://api.openai.com/v1/realtime?intent=transcription";
const OPENAI_TRANSCRIPTION_MODEL: &str = "gpt-realtime-whisper";
const OPENAI_SILENCE_COMMIT_DELAY: Duration = Duration::from_millis(350);
const OPENAI_MAX_COMMIT_DELAY: Duration = Duration::from_millis(2_500);
const OPENAI_SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(1);

pub(crate) async fn run_realtime_transcription(
    app: tauri::AppHandle,
    running: Arc<AtomicBool>,
    audio_receiver: Receiver<AudioChunk>,
    input_sample_rate: u32,
    options: StartSessionOptions,
    api_key: String,
    ready_sender: Sender<Result<(), String>>,
) -> Result<(), String> {
    let StartSessionOptions {
        audio_source,
        source_language,
        translate,
    } = options;
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
    let translator = if translate {
        match translation_client() {
            Ok(client) => Some(Translator {
                client,
                api_key,
                running: Arc::clone(&running),
            }),
            Err(error) => {
                send_ready_error(&ready_sender, error);
                return Ok(());
            }
        }
    } else {
        None
    };
    // Dropping the JoinSet aborts in-flight Translation when the session ends.
    let mut translations = JoinSet::new();

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
                        "transcription": transcription_config(source_language)
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
        while translations.try_join_next().is_some() {}

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
                                format!(
                                    "Could not stream {} audio to OpenAI: {error}",
                                    audio_source.as_str()
                                )
                            })?;
                        buffer_started_at.get_or_insert_with(Instant::now);
                        has_uncommitted_audio = true;
                    }
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    return Err(format!(
                        "{} audio stream stopped unexpectedly.",
                        audio_source.as_str()
                    ));
                }
            }
        }

        if should_commit_audio(has_uncommitted_audio, buffer_started_at, last_voice_at) {
            writer
                .send(Message::Text(
                    json!({
                        "type": "input_audio_buffer.commit"
                    })
                    .to_string()
                    .into(),
                ))
                .await
                .map_err(|error| {
                    format!(
                        "Could not commit {} audio to OpenAI: {error}",
                        audio_source.as_str()
                    )
                })?;
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
                        options,
                        translator.as_ref(),
                        &mut translations,
                    )?,
                    Some(Err(error)) => return Err(format!("OpenAI Realtime connection failed: {error}")),
                    None => return Err("OpenAI Realtime connection closed.".to_string()),
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(20)) => {}
        }
    }

    // Bound the shutdown so a stalled connection cannot hang Stop.
    let _ = tokio::time::timeout(OPENAI_SHUTDOWN_TIMEOUT, async {
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
    })
    .await;
    Ok(())
}

fn transcription_config(source_language: SourceLanguage) -> Value {
    let mut config = json!({ "model": OPENAI_TRANSCRIPTION_MODEL });
    if let Some(code) = source_language.code() {
        config["language"] = json!(code);
    }
    config
}

fn segment_source_language(source_language: SourceLanguage, text: &str) -> &'static str {
    source_language
        .code()
        .unwrap_or_else(|| if is_chinese_text(text) { "zh" } else { "auto" })
}

fn should_commit_audio(
    has_uncommitted_audio: bool,
    buffer_started_at: Option<Instant>,
    last_voice_at: Option<Instant>,
) -> bool {
    has_uncommitted_audio
        && (last_voice_at
            .is_some_and(|last_voice| last_voice.elapsed() >= OPENAI_SILENCE_COMMIT_DELAY)
            || buffer_started_at
                .is_some_and(|buffer_start| buffer_start.elapsed() >= OPENAI_MAX_COMMIT_DELAY))
}

fn handle_realtime_message(
    app: &tauri::AppHandle,
    message: Message,
    transcript_accumulator: &mut HashMap<String, String>,
    options: StartSessionOptions,
    translator: Option<&Translator>,
    translations: &mut JoinSet<()>,
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
                emit_live_subtitle(app, &value, transcript, false, options);
            }
        }
        "conversation.item.input_audio_transcription.completed" => {
            let transcript = value
                .get("transcript")
                .and_then(Value::as_str)
                .unwrap_or_default();
            if !transcript.is_empty() {
                let segment = emit_live_subtitle(app, &value, transcript, true, options);
                if let Some(translator) = translator
                    .filter(|_| needs_translation(&segment.source_language, &segment.text))
                {
                    translations.spawn(translator.translate_segment(app.clone(), segment));
                }
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

fn emit_live_subtitle(
    app: &tauri::AppHandle,
    event: &Value,
    text: &str,
    is_final: bool,
    options: StartSessionOptions,
) -> TranscriptSegment {
    let segment = TranscriptSegment {
        id: transcript_segment_id(event),
        audio_source: options.audio_source.as_str(),
        text: text.to_string(),
        translation: None,
        source_language: segment_source_language(options.source_language, text).to_string(),
        is_final,
        timestamp: now_millis(),
    };
    emit_transcript_segment(app, segment.clone());
    segment
}

struct Translator {
    client: reqwest::Client,
    api_key: String,
    running: Arc<AtomicBool>,
}

impl Translator {
    fn translate_segment(
        &self,
        app: tauri::AppHandle,
        mut segment: TranscriptSegment,
    ) -> impl std::future::Future<Output = ()> + Send + 'static {
        let client = self.client.clone();
        let api_key = self.api_key.clone();
        let running = Arc::clone(&self.running);

        async move {
            let result = translate_to_traditional_chinese(&client, &api_key, &segment.text).await;
            if !running.load(Ordering::SeqCst) {
                return;
            }

            match result {
                Ok(translation) => {
                    segment.translation = Some(translation);
                    emit_transcript_segment(&app, segment);
                }
                Err(error) => emit_translation_error_message(&app, error),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transcription_config_sets_selected_language() {
        assert_eq!(
            transcription_config(SourceLanguage::En),
            json!({ "model": OPENAI_TRANSCRIPTION_MODEL, "language": "en" })
        );
        assert_eq!(
            transcription_config(SourceLanguage::Auto),
            json!({ "model": OPENAI_TRANSCRIPTION_MODEL })
        );
    }

    #[test]
    fn segment_source_language_prefers_selected_language() {
        assert_eq!(segment_source_language(SourceLanguage::Zh, "OK"), "zh");
        assert_eq!(segment_source_language(SourceLanguage::En, "你好"), "en");
        assert_eq!(segment_source_language(SourceLanguage::Auto, "你好"), "zh");
        assert_eq!(
            segment_source_language(SourceLanguage::Auto, "hello"),
            "auto"
        );
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
}
