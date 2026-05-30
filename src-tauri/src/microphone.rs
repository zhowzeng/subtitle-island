use crate::{
    audio::{
        activity_from_f32, activity_from_i16, activity_from_u16, chunk_from_interleaved_f32,
        chunk_from_interleaved_i16, chunk_from_interleaved_u16, AudioChunk,
    },
    events::{emit_audio_activity, emit_session_error_message},
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::{
    sync::mpsc::SyncSender,
    time::{Duration, Instant},
};

pub(crate) fn start_microphone_capture(
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

fn emit_activity(
    app: &tauri::AppHandle,
    last_emit: &mut Instant,
    activity: crate::events::AudioActivity,
) {
    if last_emit.elapsed() < Duration::from_millis(100) {
        return;
    }

    *last_emit = Instant::now();
    emit_audio_activity(app, activity);
}

fn emit_session_error(app: &tauri::AppHandle, error: cpal::StreamError) {
    emit_session_error_message(app, format!("Microphone capture failed: {error}"));
}
