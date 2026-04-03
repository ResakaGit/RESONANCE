//! D3: Ecosystem Music — sonification of evolved creatures.
//!
//! Maps OscillatorySignature frequencies to audio sine waves.
//! Interference between nearby entities = chords.
//! Pure math: genome data → PCM samples → WAV bytes. Zero external crates.

use crate::batch::bridge;
use crate::batch::genome::GenomeBlob;

/// Audio configuration for sonification.
pub struct SonificationConfig {
    /// Audio sample rate in Hz.
    pub sample_rate: u32,
    /// Duration in seconds.
    pub duration_secs: f32,
    /// Scale factor: sim frequency → audible frequency (divide by this).
    pub freq_divisor: f32,
    /// Master volume [0..1].
    pub master_volume: f32,
}

impl Default for SonificationConfig {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            duration_secs: 10.0,
            freq_divisor: 2.0, // 400 Hz sim → 200 Hz audio
            master_volume: 0.3,
        }
    }
}

/// Generate WAV file bytes from evolved genomes.
///
/// Each genome contributes a sine wave at its frequency (scaled to audible range).
/// Genomes with higher resilience have higher amplitude.
/// The mix creates chords from frequency interference (Axiom 8 made literal).
pub fn genomes_to_wav(genomes: &[GenomeBlob], config: &SonificationConfig) -> Vec<u8> {
    let total_samples = (config.sample_rate as f32 * config.duration_secs) as usize;
    let mut pcm = vec![0.0f32; total_samples];

    // Each genome = one oscillator
    let voices: Vec<(f32, f32)> = genomes
        .iter()
        .map(|g| {
            let sim_freq = bridge::genome_to_components(g).2.frequency_hz();
            let audio_freq = sim_freq / config.freq_divisor;
            let amplitude = (g.resilience * 0.5 + 0.1) * config.master_volume;
            (audio_freq, amplitude)
        })
        .collect();

    let n_voices = voices.len().max(1) as f32;
    let inv_sqrt_voices = 1.0 / n_voices.sqrt(); // normalize to prevent clipping

    for (i, sample) in pcm.iter_mut().enumerate() {
        let t = i as f32 / config.sample_rate as f32;
        let mut mix = 0.0f32;
        for &(freq, amp) in &voices {
            mix += (2.0 * std::f32::consts::PI * freq * t).sin() * amp;
        }
        *sample = (mix * inv_sqrt_voices).clamp(-1.0, 1.0);
    }

    pcm_to_wav(&pcm, config.sample_rate)
}

/// Encode raw f32 PCM samples as 16-bit WAV.
///
/// WAV format: 44-byte RIFF header + interleaved 16-bit LE samples.
/// Mono channel. No compression. No external crate needed.
fn pcm_to_wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let num_samples = samples.len() as u32;
    let bytes_per_sample: u16 = 2; // 16-bit
    let channels: u16 = 1; // mono
    let data_size = num_samples * bytes_per_sample as u32;
    let file_size = 36 + data_size;

    let mut wav = Vec::with_capacity(44 + data_size as usize);

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt subchunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // subchunk size
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    wav.extend_from_slice(&channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * channels as u32 * bytes_per_sample as u32;
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    let block_align = channels * bytes_per_sample;
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&(bytes_per_sample * 8).to_le_bytes()); // bits per sample

    // data subchunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    for &s in samples {
        let clamped = s.clamp(-1.0, 1.0);
        let i16_val = (clamped * i16::MAX as f32) as i16;
        wav.extend_from_slice(&i16_val.to_le_bytes());
    }

    wav
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wav_header_valid() {
        let wav = pcm_to_wav(&[0.0; 100], 44100);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
        assert_eq!(&wav[12..16], b"fmt ");
        assert_eq!(&wav[36..40], b"data");
    }

    #[test]
    fn wav_size_correct() {
        let samples = vec![0.0f32; 1000];
        let wav = pcm_to_wav(&samples, 44100);
        // 44 header + 1000 samples × 2 bytes = 2044
        assert_eq!(wav.len(), 44 + 1000 * 2);
    }

    #[test]
    fn pcm_clamps_to_range() {
        let wav = pcm_to_wav(&[2.0, -2.0, 0.5], 44100);
        // Check that extreme values are clamped (no overflow)
        assert_eq!(wav.len(), 44 + 3 * 2);
    }
}
