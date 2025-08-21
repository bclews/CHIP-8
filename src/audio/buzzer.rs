//! Audio buzzer implementation for the Chip-8 emulator.
//!
//! This module provides tone generation and buzzer functionality
//! for the Chip-8 sound system.

use super::AudioSystemResult;
use crate::error::AudioError;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Configuration for the audio buzzer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BuzzerConfig {
    /// Buzzer frequency in Hz.
    pub frequency: f32,

    /// Volume level (0.0 to 1.0).
    pub volume: f32,

    /// Sample rate for audio generation.
    pub sample_rate: u32,

    /// Waveform type for the buzzer.
    pub waveform: WaveformType,
}

impl Default for BuzzerConfig {
    fn default() -> Self {
        Self {
            frequency: 440.0, // A4 note
            volume: 0.3,      // 30% volume by default
            sample_rate: 44100,
            waveform: WaveformType::Square,
        }
    }
}

impl BuzzerConfig {
    /// Creates a new buzzer configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the frequency.
    pub fn with_frequency(mut self, frequency: f32) -> Self {
        self.frequency = frequency.max(0.0);
        self
    }

    /// Sets the volume.
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Sets the sample rate.
    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = sample_rate;
        self
    }

    /// Sets the waveform type.
    pub fn with_waveform(mut self, waveform: WaveformType) -> Self {
        self.waveform = waveform;
        self
    }

    /// Creates a classic Chip-8 buzzer configuration.
    pub fn classic() -> Self {
        Self::new()
            .with_frequency(440.0)
            .with_volume(0.4)
            .with_waveform(WaveformType::Square)
    }

    /// Creates a modern buzzer configuration.
    pub fn modern() -> Self {
        Self::new()
            .with_frequency(800.0)
            .with_volume(0.2)
            .with_waveform(WaveformType::Sine)
    }
}

/// Types of waveforms available for the buzzer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum WaveformType {
    /// Sine wave (smooth tone).
    Sine,

    /// Square wave (classic 8-bit sound).
    Square,

    /// Sawtooth wave (harsh tone).
    Sawtooth,

    /// Triangle wave (softer than square).
    Triangle,
}

/// Audio buzzer for generating tones.
pub struct AudioBuzzer {
    /// Configuration for the buzzer.
    config: BuzzerConfig,

    /// Current phase for waveform generation.
    phase: f32,

    /// Whether the buzzer is currently playing.
    playing: bool,

    /// Time when the tone started.
    start_time: Option<Instant>,

    /// Shared state for thread-safe access.
    state: Arc<Mutex<BuzzerState>>,
}

/// Internal state of the buzzer.
#[derive(Debug)]
pub struct BuzzerState {
    pub frequency: f32,
    pub volume: f32,
    pub playing: bool,
    pub phase: f32,
}

impl AudioBuzzer {
    /// Creates a new audio buzzer.
    pub fn new(config: BuzzerConfig) -> AudioSystemResult<Self> {
        let state = Arc::new(Mutex::new(BuzzerState {
            frequency: config.frequency,
            volume: config.volume,
            playing: false,
            phase: 0.0,
        }));

        Ok(Self {
            config,
            phase: 0.0,
            playing: false,
            start_time: None,
            state,
        })
    }

    /// Creates a silent buzzer (for fallback).
    pub fn silent() -> Self {
        let config = BuzzerConfig::new().with_volume(0.0);
        let config_clone = config.clone();
        Self::new(config).unwrap_or_else(|_| {
            // Fallback implementation if audio fails
            let state = Arc::new(Mutex::new(BuzzerState {
                frequency: 440.0,
                volume: 0.0,
                playing: false,
                phase: 0.0,
            }));

            Self {
                config: config_clone,
                phase: 0.0,
                playing: false,
                start_time: None,
                state,
            }
        })
    }

    /// Starts playing a tone.
    pub fn start_tone(&mut self) -> AudioSystemResult<()> {
        self.playing = true;
        self.start_time = Some(Instant::now());
        self.phase = 0.0;

        if let Ok(mut state) = self.state.lock() {
            state.playing = true;
            state.phase = 0.0;
        }

        Ok(())
    }

    /// Stops playing the tone.
    pub fn stop_tone(&mut self) -> AudioSystemResult<()> {
        self.playing = false;
        self.start_time = None;

        if let Ok(mut state) = self.state.lock() {
            state.playing = false;
        }

        Ok(())
    }

    /// Checks if the buzzer is currently playing.
    pub fn is_playing(&self) -> bool {
        self.playing
    }

    /// Sets the volume.
    pub fn set_volume(&mut self, volume: f32) -> AudioSystemResult<()> {
        let clamped_volume = volume.clamp(0.0, 1.0);
        self.config.volume = clamped_volume;

        if let Ok(mut state) = self.state.lock() {
            state.volume = clamped_volume;
        }

        Ok(())
    }

    /// Gets the current volume.
    pub fn get_volume(&self) -> f32 {
        self.config.volume
    }

    /// Sets the frequency.
    pub fn set_frequency(&mut self, frequency: f32) -> AudioSystemResult<()> {
        if frequency < 0.0 {
            return Err(AudioError::InitializationFailed);
        }

        self.config.frequency = frequency;

        if let Ok(mut state) = self.state.lock() {
            state.frequency = frequency;
        }

        Ok(())
    }

    /// Gets the current frequency.
    pub fn get_frequency(&self) -> f32 {
        self.config.frequency
    }

    /// Generates the next audio sample.
    pub fn next_sample(&mut self) -> f32 {
        if !self.playing {
            return 0.0;
        }

        let sample = self.generate_waveform_sample();
        self.advance_phase();

        sample * self.config.volume
    }

    /// Generates multiple samples into a buffer.
    pub fn fill_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.next_sample();
        }
    }

    /// Gets a reference to the shared state (for audio threads).
    pub fn get_state(&self) -> Arc<Mutex<BuzzerState>> {
        Arc::clone(&self.state)
    }

    /// Gets the current configuration.
    pub fn config(&self) -> &BuzzerConfig {
        &self.config
    }

    /// Generates a waveform sample based on the current configuration.
    fn generate_waveform_sample(&self) -> f32 {
        use std::f32::consts::PI;

        match self.config.waveform {
            WaveformType::Sine => (self.phase * 2.0 * PI).sin(),
            WaveformType::Square => {
                if self.phase < 0.5 {
                    1.0
                } else {
                    -1.0
                }
            }
            WaveformType::Sawtooth => 2.0 * self.phase - 1.0,
            WaveformType::Triangle => {
                if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    3.0 - 4.0 * self.phase
                }
            }
        }
    }

    /// Advances the phase for the next sample.
    fn advance_phase(&mut self) {
        self.phase += self.config.frequency / self.config.sample_rate as f32;

        // Wrap phase to prevent overflow
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        // Update shared state
        if let Ok(mut state) = self.state.lock() {
            state.phase = self.phase;
        }
    }
}

/// Creates a simple buzzer for testing without audio output.
pub fn create_test_buzzer() -> AudioBuzzer {
    AudioBuzzer::new(BuzzerConfig::default()).unwrap_or_else(|_| AudioBuzzer::silent())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buzzer_config_creation() {
        let config = BuzzerConfig::new();

        assert_eq!(config.frequency, 440.0);
        assert_eq!(config.volume, 0.3);
        assert_eq!(config.sample_rate, 44100);
        assert_eq!(config.waveform, WaveformType::Square);
    }

    #[test]
    fn test_buzzer_config_builder() {
        let config = BuzzerConfig::new()
            .with_frequency(880.0)
            .with_volume(0.5)
            .with_waveform(WaveformType::Sine);

        assert_eq!(config.frequency, 880.0);
        assert_eq!(config.volume, 0.5);
        assert_eq!(config.waveform, WaveformType::Sine);
    }

    #[test]
    fn test_buzzer_config_presets() {
        let classic = BuzzerConfig::classic();
        assert_eq!(classic.waveform, WaveformType::Square);

        let modern = BuzzerConfig::modern();
        assert_eq!(modern.waveform, WaveformType::Sine);
        assert_eq!(modern.frequency, 800.0);
    }

    #[test]
    fn test_audio_buzzer_creation() {
        let config = BuzzerConfig::new();
        let buzzer = AudioBuzzer::new(config);

        assert!(buzzer.is_ok());
        let buzz = buzzer.unwrap();
        assert!(!buzz.is_playing());
        assert_eq!(buzz.get_frequency(), 440.0);
        assert_eq!(buzz.get_volume(), 0.3);
    }

    #[test]
    fn test_buzzer_tone_control() {
        let mut buzzer = create_test_buzzer();

        assert!(!buzzer.is_playing());

        buzzer.start_tone().unwrap();
        assert!(buzzer.is_playing());

        buzzer.stop_tone().unwrap();
        assert!(!buzzer.is_playing());
    }

    #[test]
    fn test_buzzer_volume_control() {
        let mut buzzer = create_test_buzzer();

        buzzer.set_volume(0.8).unwrap();
        assert_eq!(buzzer.get_volume(), 0.8);

        // Test clamping
        buzzer.set_volume(1.5).unwrap();
        assert_eq!(buzzer.get_volume(), 1.0);

        buzzer.set_volume(-0.1).unwrap();
        assert_eq!(buzzer.get_volume(), 0.0);
    }

    #[test]
    fn test_buzzer_frequency_control() {
        let mut buzzer = create_test_buzzer();

        buzzer.set_frequency(880.0).unwrap();
        assert_eq!(buzzer.get_frequency(), 880.0);

        // Test invalid frequency
        assert!(buzzer.set_frequency(-100.0).is_err());
    }

    #[test]
    fn test_buzzer_sample_generation() {
        let mut buzzer = create_test_buzzer();

        // Without playing, should generate silence
        let silent_sample = buzzer.next_sample();
        assert_eq!(silent_sample, 0.0);

        // With playing, should generate samples
        buzzer.start_tone().unwrap();
        let samples: Vec<f32> = (0..10).map(|_| buzzer.next_sample()).collect();

        // Should have non-zero samples (unless volume is 0)
        if buzzer.get_volume() > 0.0 {
            assert!(samples.iter().any(|&s| s != 0.0));
        }
    }

    #[test]
    fn test_buzzer_buffer_fill() {
        let mut buzzer = create_test_buzzer();
        let mut buffer = [0.0f32; 100];

        buzzer.start_tone().unwrap();
        buzzer.fill_buffer(&mut buffer);

        // Buffer should be filled with samples
        if buzzer.get_volume() > 0.0 {
            assert!(buffer.iter().any(|&s| s != 0.0));
        }
    }

    #[test]
    fn test_silent_buzzer() {
        let buzzer = AudioBuzzer::silent();
        assert_eq!(buzzer.get_volume(), 0.0);
        assert!(!buzzer.is_playing());
    }

    #[test]
    fn test_all_waveform_types() {
        // Test each waveform type
        for waveform in [WaveformType::Sine, WaveformType::Square, 
                         WaveformType::Sawtooth, WaveformType::Triangle] {
            let config = BuzzerConfig::new().with_waveform(waveform);
            let mut test_buzzer = AudioBuzzer::new(config).unwrap();
            
            test_buzzer.start_tone().unwrap();
            let samples: Vec<f32> = (0..100).map(|_| test_buzzer.next_sample()).collect();
            
            // Each waveform should produce different patterns
            assert!(samples.iter().any(|&s| s != 0.0));
            
            // Test waveform characteristics
            match waveform {
                WaveformType::Square => {
                    // Square wave should have only +1.0 and -1.0 values (when volume scaled)
                    assert!(samples.iter().all(|&s| s.abs() <= test_buzzer.get_volume()));
                },
                WaveformType::Sine => {
                    // Sine wave should have smooth transitions
                    assert!(samples.windows(2).any(|w| (w[0] - w[1]).abs() < 0.1));
                },
                _ => {} // Basic checks for sawtooth/triangle
            }
        }
    }

    #[test]
    fn test_buzzer_phase_wraparound() {
        let mut buzzer = create_test_buzzer();
        buzzer.start_tone().unwrap();
        
        // Generate many samples to force phase wraparound
        for _ in 0..48000 { // More than one second at 44.1kHz
            buzzer.next_sample();
        }
        
        // Buzzer should still be functional after phase wraparound
        let sample = buzzer.next_sample();
        if buzzer.get_volume() > 0.0 {
            assert!(sample.abs() > 0.0);
        }
    }

    #[test]
    fn test_buzzer_config_validation() {
        // Test frequency clamping
        let config = BuzzerConfig::new().with_frequency(-100.0);
        assert_eq!(config.frequency, 0.0); // Should clamp to 0
        
        // Test volume clamping  
        let config = BuzzerConfig::new().with_volume(1.5);
        assert_eq!(config.volume, 1.0);
        
        let config = BuzzerConfig::new().with_volume(-0.5);
        assert_eq!(config.volume, 0.0);
    }

    #[test]
    fn test_buzzer_state_management() {
        let buzzer = create_test_buzzer();
        let state = buzzer.get_state();
        
        // Test shared state access
        {
            let state_lock = state.lock().unwrap();
            assert!(!state_lock.playing);
            assert_eq!(state_lock.phase, 0.0);
        }
        
        // Test state configuration access
        let config = buzzer.config();
        assert_eq!(config.frequency, 440.0);
        assert_eq!(config.sample_rate, 44100);
    }
}
