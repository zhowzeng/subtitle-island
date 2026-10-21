#[cfg(any(target_os = "windows", target_os = "linux"))]
use crate::events::emit_audio_activity;
#[cfg(target_os = "windows")]
use crate::{
    audio::{
        activity_from_f32, activity_from_i16, activity_from_u16, chunk_from_interleaved_f32,
        chunk_from_interleaved_i16, chunk_from_interleaved_u16, AudioCapture, AudioChunk,
    },
    events::emit_session_error_message,
};
#[cfg(target_os = "windows")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
#[cfg(target_os = "windows")]
use std::sync::mpsc::SyncSender;
#[cfg(any(target_os = "windows", target_os = "linux"))]
use std::time::{Duration, Instant};

#[cfg(target_os = "windows")]
pub(crate) fn start_system_audio_capture(
    app: tauri::AppHandle,
    audio_sender: SyncSender<AudioChunk>,
) -> Result<(AudioCapture, u32), String> {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| "No default system audio output device was found.".to_string())?;
    let supported_config = device
        .default_output_config()
        .map_err(|error| format!("Could not read default system audio config: {error}"))?;
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
                "Unsupported system audio sample format: {sample_format:?}"
            ));
        }
    }
    .map_err(|error| format!("Could not start system audio stream: {error}"))?;

    stream
        .play()
        .map_err(|error| format!("Could not play system audio stream: {error}"))?;

    Ok((AudioCapture::Cpal { _stream: stream }, sample_rate))
}

#[cfg(target_os = "linux")]
pub(crate) use linux::{start_system_audio_capture, PulseCapture};

#[cfg(target_os = "linux")]
mod linux {
    use super::emit_activity;
    use crate::{
        audio::{activity_from_f32, chunk_from_interleaved_f32, AudioCapture, AudioChunk},
        events::emit_session_error_message,
    };
    use libpulse_binding::{
        def::BufferAttr,
        sample::{Format, Spec},
        stream::Direction,
        time::MicroSeconds,
    };
    use libpulse_simple_binding::Simple;
    use std::{
        sync::{
            atomic::{AtomicBool, Ordering},
            mpsc::SyncSender,
            Arc,
        },
        thread::{self, JoinHandle},
        time::{Duration, Instant},
    };

    // Monitor of the default sink; supported by PulseAudio and pipewire-pulse.
    const DEFAULT_MONITOR_SOURCE: &str = "@DEFAULT_MONITOR@";
    const CAPTURE_SAMPLE_RATE: u32 = 48_000;
    const CAPTURE_CHANNELS: u8 = 2;
    const READ_FRAGMENT_MICROS: u64 = 20_000;

    /// Stops the PulseAudio read loop and joins its thread on drop.
    pub(crate) struct PulseCapture {
        running: Arc<AtomicBool>,
        handle: Option<JoinHandle<()>>,
    }

    impl Drop for PulseCapture {
        fn drop(&mut self) {
            self.running.store(false, Ordering::SeqCst);
            if let Some(handle) = self.handle.take() {
                let _ = handle.join();
            }
        }
    }

    pub(crate) fn start_system_audio_capture(
        app: tauri::AppHandle,
        audio_sender: SyncSender<AudioChunk>,
    ) -> Result<(AudioCapture, u32), String> {
        let spec = Spec {
            format: Format::F32le,
            channels: CAPTURE_CHANNELS,
            rate: CAPTURE_SAMPLE_RATE,
        };
        let fragment_bytes = spec.usec_to_bytes(MicroSeconds(READ_FRAGMENT_MICROS));
        let buffer_attr = BufferAttr {
            maxlength: u32::MAX,
            tlength: u32::MAX,
            prebuf: u32::MAX,
            minreq: u32::MAX,
            fragsize: fragment_bytes as u32,
        };
        let simple = Simple::new(
            None,
            "SubIs",
            Direction::Record,
            Some(DEFAULT_MONITOR_SOURCE),
            "System Audio Capture",
            &spec,
            None,
            Some(&buffer_attr),
        )
        .map_err(|error| format!("Could not open system audio monitor: {error}"))?;

        let running = Arc::new(AtomicBool::new(true));
        let thread_running = Arc::clone(&running);
        let channels = usize::from(CAPTURE_CHANNELS);
        let handle = thread::spawn(move || {
            let mut bytes = vec![0u8; fragment_bytes];
            let mut last_emit = Instant::now() - Duration::from_millis(100);

            while thread_running.load(Ordering::SeqCst) {
                if let Err(error) = simple.read(&mut bytes) {
                    if thread_running.load(Ordering::SeqCst) {
                        emit_session_error_message(
                            &app,
                            format!("System audio capture failed: {error}"),
                        );
                    }
                    return;
                }

                let samples: Vec<f32> = bytes
                    .chunks_exact(4)
                    .map(|sample| f32::from_le_bytes([sample[0], sample[1], sample[2], sample[3]]))
                    .collect();
                emit_activity(&app, &mut last_emit, activity_from_f32(&samples));
                let _ = audio_sender.try_send(chunk_from_interleaved_f32(&samples, channels));
            }
        });

        Ok((
            AudioCapture::Pulse {
                _capture: PulseCapture {
                    running,
                    handle: Some(handle),
                },
            },
            CAPTURE_SAMPLE_RATE,
        ))
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub(crate) fn start_system_audio_capture(
    _app: tauri::AppHandle,
    _audio_sender: std::sync::mpsc::SyncSender<crate::audio::AudioChunk>,
) -> Result<(crate::audio::AudioCapture, u32), String> {
    Err("System Audio Capture is only available on Windows and Linux in this version.".to_string())
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
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

#[cfg(target_os = "windows")]
fn emit_session_error(app: &tauri::AppHandle, error: cpal::StreamError) {
    emit_session_error_message(app, format!("System audio capture failed: {error}"));
}
