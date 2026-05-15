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
