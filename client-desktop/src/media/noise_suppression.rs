use std::collections::VecDeque;
use nnnoiseless::DenoiseState;

/// Real-time AI Noise Suppressor powered by the RNNoise recurrent neural network.
///
/// Features:
/// - In-place 480-sample chunk processing at 48 kHz (10 ms frames)
/// - Voice Activity Detection (VAD) confidence score output
/// - FIFO ring-buffering for arbitrary input chunk sizes (e.g., 64, 128, 256, 512, 1024)
/// - Support for normalized `[-1.0, 1.0]` f32 and standard 16-bit PCM formats
pub struct AiNoiseSuppressor {
    denoiser: Box<DenoiseState<'static>>,
    fifo_in: VecDeque<f32>,
    vad_confidence: f32,
}

impl Default for AiNoiseSuppressor {
    fn default() -> Self {
        Self::new()
    }
}

impl AiNoiseSuppressor {
    /// 480 samples per RNNoise frame (10 ms at 48 kHz).
    pub const FRAME_SIZE: usize = DenoiseState::FRAME_SIZE;
    /// Supported audio sample rate (48 kHz).
    pub const SAMPLE_RATE: u32 = 48_000;
    /// PCM 16-bit float scaling constant.
    const SCALE_FACTOR: f32 = 32767.0;

    /// Creates a new `AiNoiseSuppressor` with fresh recurrent neural network state.
    pub fn new() -> Self {
        Self {
            denoiser: DenoiseState::new(),
            fifo_in: VecDeque::with_capacity(Self::FRAME_SIZE * 4),
            vad_confidence: 0.0,
        }
    }

    /// Returns the latest Voice Activity Detection (VAD) confidence score in `[0.0, 1.0]`.
    pub fn latest_vad_confidence(&self) -> f32 {
        self.vad_confidence
    }

    /// Number of audio samples pending in the internal FIFO queue.
    pub fn pending_samples(&self) -> usize {
        self.fifo_in.len()
    }

    /// Resets the internal neural network state and clears the FIFO queue.
    pub fn reset(&mut self) {
        self.denoiser = DenoiseState::new();
        self.fifo_in.clear();
        self.vad_confidence = 0.0;
    }

    /// Processes a single 480-sample frame in-place.
    /// Input is expected to be scaled to 16-bit PCM float range `[-32768.0, 32767.0]`.
    /// Returns the VAD probability `[0.0, 1.0]`.
    pub fn process_frame_inplace(&mut self, frame: &mut [f32; Self::FRAME_SIZE]) -> f32 {
        let input = *frame;
        let vad = self.denoiser.process_frame(frame, &input);
        self.vad_confidence = vad;
        vad
    }

    /// Processes a single 480-sample frame from `input` into `output`.
    /// Expects slices of length at least `Self::FRAME_SIZE` (480).
    /// Returns the VAD probability `[0.0, 1.0]`.
    pub fn process_frame(&mut self, output: &mut [f32], input: &[f32]) -> f32 {
        if input.len() < Self::FRAME_SIZE || output.len() < Self::FRAME_SIZE {
            tracing::warn!(
                "process_frame: buffer too short (input: {}, output: {}, need: {})",
                input.len(),
                output.len(),
                Self::FRAME_SIZE
            );
            return 0.0;
        }

        let vad = self.denoiser.process_frame(&mut output[..Self::FRAME_SIZE], &input[..Self::FRAME_SIZE]);
        self.vad_confidence = vad;
        vad
    }

    /// Processes a single 480-sample frame of normalized float samples `[-1.0, 1.0]` in-place.
    /// Automatically applies 16-bit PCM scaling required by RNNoise.
    /// Returns the VAD probability `[0.0, 1.0]`.
    pub fn process_frame_normalized_inplace(&mut self, frame: &mut [f32; Self::FRAME_SIZE]) -> f32 {
        let mut scaled_in = [0.0f32; Self::FRAME_SIZE];
        for (dst, &src) in scaled_in.iter_mut().zip(frame.iter()) {
            *dst = (src * Self::SCALE_FACTOR).clamp(-32768.0, 32767.0);
        }

        let mut scaled_out = [0.0f32; Self::FRAME_SIZE];
        let vad = self.denoiser.process_frame(&mut scaled_out, &scaled_in);
        self.vad_confidence = vad;

        for (dst, &src) in frame.iter_mut().zip(scaled_out.iter()) {
            *dst = (src / Self::SCALE_FACTOR).clamp(-1.0, 1.0);
        }

        vad
    }

    /// Processes an arbitrary-length slice of normalized float samples `[-1.0, 1.0]`.
    ///
    /// Uses an internal FIFO queue to buffer incoming samples into 480-sample frames,
    /// denoises all complete frames, and returns the processed audio samples.
    pub fn process_stream_normalized(&mut self, input: &[f32]) -> Vec<f32> {
        for &s in input {
            self.fifo_in.push_back((s * Self::SCALE_FACTOR).clamp(-32768.0, 32767.0));
        }

        let mut output = Vec::with_capacity(input.len());
        let mut in_chunk = [0.0f32; Self::FRAME_SIZE];
        let mut out_chunk = [0.0f32; Self::FRAME_SIZE];

        while self.fifo_in.len() >= Self::FRAME_SIZE {
            for sample in in_chunk.iter_mut() {
                *sample = self.fifo_in.pop_front().unwrap_or(0.0);
            }

            self.vad_confidence = self.denoiser.process_frame(&mut out_chunk, &in_chunk);

            output.extend(
                out_chunk
                    .iter()
                    .map(|&s| (s / Self::SCALE_FACTOR).clamp(-1.0, 1.0)),
            );
        }

        output
    }

    /// Processes an arbitrary-length slice of 16-bit PCM integer samples (`i16`).
    ///
    /// Buffers incoming samples into the internal FIFO, runs RNNoise inference
    /// across full 480-sample frames, and returns denoised `i16` samples.
    pub fn process_stream_pcm16(&mut self, input: &[i16]) -> Vec<i16> {
        for &s in input {
            self.fifo_in.push_back(s as f32);
        }

        let mut output = Vec::with_capacity(input.len());
        let mut in_chunk = [0.0f32; Self::FRAME_SIZE];
        let mut out_chunk = [0.0f32; Self::FRAME_SIZE];

        while self.fifo_in.len() >= Self::FRAME_SIZE {
            for sample in in_chunk.iter_mut() {
                *sample = self.fifo_in.pop_front().unwrap_or(0.0);
            }

            self.vad_confidence = self.denoiser.process_frame(&mut out_chunk, &in_chunk);

            for &s in &out_chunk {
                output.push(s.clamp(-32768.0, 32767.0) as i16);
            }
        }

        output
    }

    /// Denoises full 480-sample chunks directly in-place within a mutable `i16` slice.
    /// Any tail samples shorter than 480 are buffered in the internal FIFO.
    pub fn process_inplace_pcm16(&mut self, samples: &mut [i16]) {
        let mut in_chunk = [0.0f32; Self::FRAME_SIZE];
        let mut out_chunk = [0.0f32; Self::FRAME_SIZE];

        let mut chunks = samples.chunks_exact_mut(Self::FRAME_SIZE);
        for chunk in &mut chunks {
            for (dst, &src) in in_chunk.iter_mut().zip(chunk.iter()) {
                *dst = src as f32;
            }

            self.vad_confidence = self.denoiser.process_frame(&mut out_chunk, &in_chunk);

            for (dst, &src) in chunk.iter_mut().zip(out_chunk.iter()) {
                *dst = src.clamp(-32768.0, 32767.0) as i16;
            }
        }

        for &s in chunks.into_remainder().iter() {
            self.fifo_in.push_back(s as f32);
        }
    }

    /// Flushes any pending samples remaining in the FIFO by zero-padding to 480 samples.
    pub fn flush_normalized(&mut self) -> Vec<f32> {
        if self.fifo_in.is_empty() {
            return Vec::new();
        }

        let remaining_count = self.fifo_in.len();
        let mut in_chunk = [0.0f32; Self::FRAME_SIZE];
        let mut out_chunk = [0.0f32; Self::FRAME_SIZE];

        for item in in_chunk.iter_mut().take(Self::FRAME_SIZE) {
            *item = self.fifo_in.pop_front().unwrap_or(0.0);
        }

        self.vad_confidence = self.denoiser.process_frame(&mut out_chunk, &in_chunk);

        let mut output = Vec::with_capacity(remaining_count);
        for &s in &out_chunk[..remaining_count] {
            output.push((s / Self::SCALE_FACTOR).clamp(-1.0, 1.0));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_suppressor_initialization_and_reset() {
        let mut suppressor = AiNoiseSuppressor::new();
        assert_eq!(suppressor.pending_samples(), 0);
        assert_eq!(suppressor.latest_vad_confidence(), 0.0);

        let input = vec![0.1f32; 100];
        let _ = suppressor.process_stream_normalized(&input);
        assert_eq!(suppressor.pending_samples(), 100);

        suppressor.reset();
        assert_eq!(suppressor.pending_samples(), 0);
        assert_eq!(suppressor.latest_vad_confidence(), 0.0);
    }

    #[test]
    fn test_process_frame_inplace_silence() {
        let mut suppressor = AiNoiseSuppressor::new();
        let mut frame = [0.0f32; AiNoiseSuppressor::FRAME_SIZE];

        let vad = suppressor.process_frame_inplace(&mut frame);
        // On pure silence, VAD probability should be low
        assert!((0.0..=1.0).contains(&vad));

        // Output silence should remain near zero
        for &sample in &frame {
            assert!(sample.abs() < 10.0, "Denoised silence sample {sample} was unexpectedly loud");
        }
    }

    #[test]
    fn test_process_frame_normalized_inplace() {
        let mut suppressor = AiNoiseSuppressor::new();
        let mut frame = [0.05f32; AiNoiseSuppressor::FRAME_SIZE];

        let vad = suppressor.process_frame_normalized_inplace(&mut frame);
        assert!((0.0..=1.0).contains(&vad));

        for &sample in &frame {
            assert!((-1.0..=1.0).contains(&sample));
        }
    }

    #[test]
    fn test_arbitrary_buffer_fifo_stream_streaming() {
        let mut suppressor = AiNoiseSuppressor::new();

        // Feed small 128-sample chunks (arbitrary buffer sizes)
        let chunk = vec![0.02f32; 128];
        let mut total_output_samples = 0;

        for _ in 0..10 {
            let out = suppressor.process_stream_normalized(&chunk);
            total_output_samples += out.len();
        }

        // 10 * 128 = 1280 samples.
        // 1280 / 480 = 2 frames (960 samples processed), 320 remaining in FIFO.
        assert_eq!(total_output_samples, 960);
        assert_eq!(suppressor.pending_samples(), 320);

        // Flush remaining samples
        let flushed = suppressor.flush_normalized();
        assert_eq!(flushed.len(), 320);
        assert_eq!(suppressor.pending_samples(), 0);
    }

    #[test]
    fn test_pcm16_noise_suppression_stream_and_inplace() {
        let mut suppressor = AiNoiseSuppressor::new();
        let pcm_input = vec![500i16; 960]; // Exactly 2 frames (20 ms at 48 kHz)

        let out = suppressor.process_stream_pcm16(&pcm_input);
        assert_eq!(out.len(), 960);
        assert_eq!(suppressor.pending_samples(), 0);

        // Test in-place processing
        let mut pcm_inplace = vec![800i16; 480];
        suppressor.process_inplace_pcm16(&mut pcm_inplace);
        // Processed in-place without panicking
        assert_eq!(pcm_inplace.len(), 480);
    }

    #[test]
    fn test_synthetic_noisy_speech_attenuation() {
        let mut suppressor = AiNoiseSuppressor::new();

        // Generate synthetic high-frequency noise frame (simulating fan/hum)
        let mut noisy_frame = [0.0f32; AiNoiseSuppressor::FRAME_SIZE];
        for (i, sample) in noisy_frame.iter_mut().enumerate() {
            let t = i as f32 / 48000.0;
            // 8 kHz high frequency hiss
            *sample = (t * 8000.0 * 2.0 * std::f32::consts::PI).sin() * 0.15;
        }

        // Measure input RMS
        let input_rms: f32 = (noisy_frame.iter().map(|s| s * s).sum::<f32>() / 480.0).sqrt();

        suppressor.process_frame_normalized_inplace(&mut noisy_frame);

        // Measure output RMS
        let output_rms: f32 = (noisy_frame.iter().map(|s| s * s).sum::<f32>() / 480.0).sqrt();

        // Noise should be significantly attenuated
        assert!(
            output_rms < input_rms,
            "Expected noise suppression (input RMS: {input_rms}, output RMS: {output_rms})"
        );
    }
}
