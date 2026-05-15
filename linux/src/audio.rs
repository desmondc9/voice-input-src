/// Compute RMS over `samples` and normalize the resulting dBFS value to
/// the [0, 1] range using the same mapping as the macOS version:
/// `normalized = clamp((dB + 50) / 40, 0, 1)`.
/// dB is computed from `max(rms, 1e-6)` to avoid `log10(0)`.
pub fn rms_normalized(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    let rms = (sum_sq / samples.len() as f32).sqrt().max(1e-6);
    let db = 20.0 * rms.log10();
    ((db + 50.0) / 40.0).clamp(0.0, 1.0)
}

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat, Stream, StreamConfig};
use crossbeam_channel::Sender;

use crate::error::{AppError, AppResult};

/// Audio buffer + RMS level published by the capture callback.
pub struct AudioChunk {
    /// Samples in the device's native format (interleaved if multi-channel),
    /// converted to f32 in the range [-1, 1].
    pub samples: Vec<f32>,
    /// Native sample rate as reported by cpal.
    pub sample_rate: u32,
    /// Number of channels (1 or 2 typically).
    pub channels: u16,
    /// RMS level of this buffer, normalized to [0, 1] per `rms_normalized`.
    pub level: f32,
}

/// Live audio capture wrapper. Drop to stop the stream.
pub struct Capture {
    _stream: Stream,
    pub sample_rate: u32,
    pub channels: u16,
}

impl Capture {
    /// Open the default input device and start streaming.
    /// Each buffer is sent through `tx` along with the RMS level.
    /// The cpal callback returns immediately after sending; if the channel
    /// is full (downstream backed up), the buffer is silently dropped —
    /// this is the right behavior for real-time audio: never block the
    /// audio thread.
    pub fn start(tx: Sender<AudioChunk>) -> AppResult<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| AppError::NoMicrophone("no default input device".into()))?;

        let supported = device
            .default_input_config()
            .map_err(|e| AppError::NoMicrophone(format!("query default config: {}", e)))?;

        let config: StreamConfig = supported.clone().into();
        let sample_rate = config.sample_rate.0;
        let channels = config.channels;

        tracing::info!(
            device = %device.name().unwrap_or_else(|_| "<unknown>".into()),
            sample_rate,
            channels,
            format = ?supported.sample_format(),
            "opening input stream"
        );

        let stream = match supported.sample_format() {
            SampleFormat::F32 => build_stream::<f32>(&device, &config, tx, sample_rate, channels)?,
            SampleFormat::I16 => build_stream::<i16>(&device, &config, tx, sample_rate, channels)?,
            SampleFormat::U16 => build_stream::<u16>(&device, &config, tx, sample_rate, channels)?,
            other => {
                return Err(AppError::NoMicrophone(format!(
                    "unsupported sample format: {:?}",
                    other
                )));
            }
        };

        stream
            .play()
            .map_err(|e| AppError::NoMicrophone(format!("starting stream: {}", e)))?;

        Ok(Self {
            _stream: stream,
            sample_rate,
            channels,
        })
    }
}

fn build_stream<T>(
    device: &cpal::Device,
    config: &StreamConfig,
    tx: Sender<AudioChunk>,
    sample_rate: u32,
    channels: u16,
) -> AppResult<Stream>
where
    T: Sample + cpal::SizedSample,
    f32: cpal::FromSample<T>,
{
    let err_fn = |e| tracing::warn!(error = ?e, "cpal stream error");
    let data_fn = move |data: &[T], _: &cpal::InputCallbackInfo| {
        let samples: Vec<f32> = data
            .iter()
            .map(|&s| <f32 as cpal::FromSample<T>>::from_sample_(s))
            .collect();
        let level = rms_normalized(&samples);
        let chunk = AudioChunk {
            samples,
            sample_rate,
            channels,
            level,
        };
        // try_send: drop if downstream is backed up; never block audio thread
        let _ = tx.try_send(chunk);
    };

    device
        .build_input_stream(config, data_fn, err_fn, None)
        .map_err(|e| AppError::NoMicrophone(format!("build_input_stream: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_yields_zero() {
        let silence = vec![0.0_f32; 1024];
        let level = rms_normalized(&silence);
        assert!(level < 0.05, "expected near zero, got {}", level);
    }

    #[test]
    fn full_scale_sine_yields_one() {
        // Full-scale 1 kHz sine at 16 kHz sample rate
        let samples: Vec<f32> = (0..16_000)
            .map(|i| (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / 16_000.0).sin())
            .collect();
        let level = rms_normalized(&samples);
        assert!(level > 0.9, "expected near one, got {}", level);
    }

    #[test]
    fn empty_returns_zero() {
        assert_eq!(rms_normalized(&[]), 0.0);
    }

    #[test]
    fn quiet_noise_maps_to_low_range() {
        let quiet: Vec<f32> = (0..1024).map(|i| ((i as f32 * 0.1).sin()) * 0.001).collect();
        let level = rms_normalized(&quiet);
        assert!(level >= 0.0 && level < 0.2, "expected low range, got {}", level);
    }
}
