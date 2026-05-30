use crate::events::{now_millis, AudioActivity};
use audioadapter_buffers::direct::InterleavedSlice;
use rubato::{
    calculate_cutoff, Async, FixedAsync, Resampler, SincInterpolationParameters,
    SincInterpolationType, WindowFunction,
};

pub(crate) const OPENAI_TARGET_SAMPLE_RATE: u32 = 24_000;
const OPENAI_AUDIO_CHUNK_FRAMES: usize = 1024;
const OPENAI_SPEECH_RMS_THRESHOLD: f32 = 0.006;

pub(crate) struct AudioChunk {
    pub(crate) samples: Vec<f32>,
}

pub(crate) struct AudioNormalizer {
    input_sample_rate: u32,
    pending_mono: Vec<f64>,
    resampler: Option<Async<f64>>,
}

pub(crate) fn activity_from_f32(samples: &[f32]) -> AudioActivity {
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

pub(crate) fn activity_from_i16(samples: &[i16]) -> AudioActivity {
    let converted: Vec<f32> = samples
        .iter()
        .map(|sample| *sample as f32 / i16::MAX as f32)
        .collect();
    activity_from_f32(&converted)
}

pub(crate) fn activity_from_u16(samples: &[u16]) -> AudioActivity {
    let converted: Vec<f32> = samples
        .iter()
        .map(|sample| (*sample as f32 - 32768.0) / 32768.0)
        .collect();
    activity_from_f32(&converted)
}

pub(crate) fn chunk_from_interleaved_f32(samples: &[f32], channels: usize) -> AudioChunk {
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

pub(crate) fn chunk_from_interleaved_i16(samples: &[i16], channels: usize) -> AudioChunk {
    let converted: Vec<f32> = samples
        .iter()
        .map(|sample| *sample as f32 / i16::MAX as f32)
        .collect();
    chunk_from_interleaved_f32(&converted, channels)
}

pub(crate) fn chunk_from_interleaved_u16(samples: &[u16], channels: usize) -> AudioChunk {
    let converted: Vec<f32> = samples
        .iter()
        .map(|sample| (*sample as f32 - 32768.0) / 32768.0)
        .collect();
    chunk_from_interleaved_f32(&converted, channels)
}

pub(crate) fn samples_contain_speech(samples: &[f32]) -> bool {
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
    pub(crate) fn new(input_sample_rate: u32) -> Result<Self, String> {
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

    pub(crate) fn push(&mut self, chunk: AudioChunk) -> Result<Vec<Vec<u8>>, String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_contain_speech_detects_loud_audio() {
        assert!(samples_contain_speech(&[0.0, 0.02, -0.03, 0.01]));
    }

    #[test]
    fn samples_contain_speech_ignores_quiet_audio() {
        assert!(!samples_contain_speech(&[0.0, 0.002, -0.003, 0.001]));
    }
}
