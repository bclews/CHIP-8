//! Audio interface for the Chip-8 emulator.
//!
//! This module defines the audio abstraction and provides implementations
//! for the Chip-8 buzzer sound system.

use crate::error::AudioError;

/// Audio-specific result type.
pub type AudioResult<T> = Result<T, AudioError>;

/// Audio abstraction for the Chip-8 buzzer.
///
/// The Chip-8 has a simple buzzer that plays a single tone when the
/// sound timer is non-zero. The frequency and waveform are implementation-defined.
pub trait Audio {
    /// Starts playing the buzzer sound.
    fn play_beep(&mut self) -> AudioResult<()>;

    /// Stops playing the buzzer sound.
    fn stop_beep(&mut self) -> AudioResult<()>;

    /// Checks if the buzzer is currently playing.
    fn is_playing(&self) -> bool;

    /// Sets the volume (0.0 = silent, 1.0 = full volume).
    fn set_volume(&mut self, volume: f32) -> AudioResult<()>;

    /// Gets the current volume.
    fn get_volume(&self) -> f32;

    /// Sets the buzzer frequency in Hz.
    fn set_frequency(&mut self, frequency: f32) -> AudioResult<()>;

    /// Gets the current frequency.
    fn get_frequency(&self) -> f32;
}

/// A software audio implementation that tracks state without actual audio output.
pub struct SoftwareAudio {
    /// Whether the buzzer is currently playing.
    playing: bool,

    /// Current volume (0.0 to 1.0).
    volume: f32,

    /// Current frequency in Hz.
    frequency: f32,
}

impl SoftwareAudio {
    /// Creates a new software audio system.
    pub fn new() -> Self {
        Self {
            playing: false,
            volume: 0.5,
            frequency: 440.0, // Default to A4
        }
    }

    /// Creates a new software audio system with custom settings.
    pub fn with_settings(volume: f32, frequency: f32) -> Self {
        Self {
            playing: false,
            volume: volume.clamp(0.0, 1.0),
            frequency: frequency.max(0.0),
        }
    }
}

impl Default for SoftwareAudio {
    fn default() -> Self {
        Self::new()
    }
}

impl Audio for SoftwareAudio {
    fn play_beep(&mut self) -> AudioResult<()> {
        self.playing = true;
        Ok(())
    }

    fn stop_beep(&mut self) -> AudioResult<()> {
        self.playing = false;
        Ok(())
    }

    fn is_playing(&self) -> bool {
        self.playing
    }

    fn set_volume(&mut self, volume: f32) -> AudioResult<()> {
        self.volume = volume.clamp(0.0, 1.0);
        Ok(())
    }

    fn get_volume(&self) -> f32 {
        self.volume
    }

    fn set_frequency(&mut self, frequency: f32) -> AudioResult<()> {
        if frequency < 0.0 {
            return Err(AudioError::InitializationFailed);
        }
        self.frequency = frequency;
        Ok(())
    }

    fn get_frequency(&self) -> f32 {
        self.frequency
    }
}

/// A null audio implementation for testing and silent operation.
pub struct NullAudio {
    audio: SoftwareAudio,
}

impl NullAudio {
    pub fn new() -> Self {
        Self {
            audio: SoftwareAudio::new(),
        }
    }
}

impl Default for NullAudio {
    fn default() -> Self {
        Self::new()
    }
}

impl Audio for NullAudio {
    fn play_beep(&mut self) -> AudioResult<()> {
        self.audio.play_beep()
    }

    fn stop_beep(&mut self) -> AudioResult<()> {
        self.audio.stop_beep()
    }

    fn is_playing(&self) -> bool {
        self.audio.is_playing()
    }

    fn set_volume(&mut self, volume: f32) -> AudioResult<()> {
        self.audio.set_volume(volume)
    }

    fn get_volume(&self) -> f32 {
        self.audio.get_volume()
    }

    fn set_frequency(&mut self, frequency: f32) -> AudioResult<()> {
        self.audio.set_frequency(frequency)
    }

    fn get_frequency(&self) -> f32 {
        self.audio.get_frequency()
    }
}

/// Audio configuration for creating audio systems.
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// Volume level (0.0 to 1.0).
    pub volume: f32,

    /// Buzzer frequency in Hz.
    pub frequency: f32,

    /// Sample rate for audio generation.
    pub sample_rate: u32,

    /// Buffer size for audio streaming.
    pub buffer_size: usize,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            volume: 0.3,
            frequency: 440.0,
            sample_rate: 44100,
            buffer_size: 1024,
        }
    }
}

impl AudioConfig {
    /// Creates a new audio configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the volume (clamped to 0.0-1.0).
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Sets the frequency.
    pub fn with_frequency(mut self, frequency: f32) -> Self {
        self.frequency = frequency.max(0.0);
        self
    }

    /// Sets the sample rate.
    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = sample_rate;
        self
    }

    /// Sets the buffer size.
    pub fn with_buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;
        self
    }
}

/// Tone generator for creating audio waveforms.
pub struct ToneGenerator {
    /// Current frequency in Hz.
    frequency: f32,

    /// Sample rate in Hz.
    sample_rate: f32,

    /// Current phase (0.0 to 1.0).
    phase: f32,

    /// Volume multiplier.
    volume: f32,
}

impl ToneGenerator {
    /// Creates a new tone generator.
    pub fn new(frequency: f32, sample_rate: f32, volume: f32) -> Self {
        Self {
            frequency,
            sample_rate,
            phase: 0.0,
            volume: volume.clamp(0.0, 1.0),
        }
    }

    /// Generates the next audio sample.
    pub fn next_sample(&mut self) -> f32 {
        let sample = (self.phase * 2.0 * std::f32::consts::PI).sin() * self.volume;
        self.phase += self.frequency / self.sample_rate;

        // Wrap phase to prevent overflow
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        sample
    }

    /// Sets the frequency.
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency.max(0.0);
    }

    /// Sets the volume.
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    /// Resets the phase to zero.
    pub fn reset_phase(&mut self) {
        self.phase = 0.0;
    }

    /// Generates multiple samples into a buffer.
    pub fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.next_sample();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_software_audio_creation() {
        let audio = SoftwareAudio::new();

        assert!(!audio.is_playing());
        assert_eq!(audio.get_volume(), 0.5);
        assert_eq!(audio.get_frequency(), 440.0);
    }

    #[test]
    fn test_audio_playback_control() {
        let mut audio = SoftwareAudio::new();

        // Test starting playback
        audio.play_beep().unwrap();
        assert!(audio.is_playing());

        // Test stopping playback
        audio.stop_beep().unwrap();
        assert!(!audio.is_playing());
    }

    #[test]
    fn test_volume_control() {
        let mut audio = SoftwareAudio::new();

        // Test setting valid volume
        audio.set_volume(0.8).unwrap();
        assert_eq!(audio.get_volume(), 0.8);

        // Test clamping volume
        audio.set_volume(1.5).unwrap();
        assert_eq!(audio.get_volume(), 1.0);

        audio.set_volume(-0.1).unwrap();
        assert_eq!(audio.get_volume(), 0.0);
    }

    #[test]
    fn test_frequency_control() {
        let mut audio = SoftwareAudio::new();

        // Test setting valid frequency
        audio.set_frequency(880.0).unwrap();
        assert_eq!(audio.get_frequency(), 880.0);

        // Test invalid frequency
        assert!(audio.set_frequency(-100.0).is_err());
    }

    #[test]
    fn test_audio_config() {
        let config = AudioConfig::new()
            .with_volume(0.7)
            .with_frequency(880.0)
            .with_sample_rate(48000)
            .with_buffer_size(512);

        assert_eq!(config.volume, 0.7);
        assert_eq!(config.frequency, 880.0);
        assert_eq!(config.sample_rate, 48000);
        assert_eq!(config.buffer_size, 512);
    }

    #[test]
    fn test_tone_generator() {
        let mut generator = ToneGenerator::new(440.0, 44100.0, 0.5);

        // Generate some samples
        let sample1 = generator.next_sample();
        let sample2 = generator.next_sample();

        // Samples should be different (sine wave progression)
        assert_ne!(sample1, sample2);

        // Should be within expected range
        assert!(sample1.abs() <= 0.5);
        assert!(sample2.abs() <= 0.5);
    }

    #[test]
    fn test_tone_generator_frequency_change() {
        let mut generator = ToneGenerator::new(440.0, 44100.0, 0.5);

        let sample_before = generator.next_sample();

        generator.set_frequency(880.0);
        let sample_after = generator.next_sample();

        // Frequency change should affect subsequent samples
        // (though this is a basic check)
        assert!(sample_before != sample_after || generator.phase == 0.0);
    }

    #[test]
    fn test_tone_generator_buffer_fill() {
        let mut generator = ToneGenerator::new(440.0, 44100.0, 0.3);
        let mut buffer = [0.0f32; 100];

        generator.fill_buffer(&mut buffer);

        // All samples should be generated
        assert!(buffer.iter().any(|&x| x != 0.0));

        // Samples should be within volume range
        assert!(buffer.iter().all(|&x| x.abs() <= 0.3));
    }

    #[test]
    fn test_null_audio() {
        let mut audio = NullAudio::new();

        assert!(!audio.is_playing());

        audio.play_beep().unwrap();
        assert!(audio.is_playing());

        audio.stop_beep().unwrap();
        assert!(!audio.is_playing());

        audio.set_volume(0.8).unwrap();
        assert_eq!(audio.get_volume(), 0.8);
    }

    #[test]
    fn test_software_audio_with_settings() {
        let audio = SoftwareAudio::with_settings(0.8, 880.0);

        assert_eq!(audio.get_volume(), 0.8);
        assert_eq!(audio.get_frequency(), 880.0);
        assert!(!audio.is_playing());
    }
}
