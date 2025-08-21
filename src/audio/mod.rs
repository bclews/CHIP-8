//! Audio system for the Chip-8 emulator.
//!
//! This module provides actual audio implementation using CPAL
//! for real-time audio output with buzzer functionality.

pub mod buzzer;
pub mod stream;

// Re-export commonly used types
pub use buzzer::{AudioBuzzer, BuzzerConfig};
pub use stream::{AudioStream, StreamConfig};

use crate::error::AudioError;
use crate::hardware::{Audio, AudioResult};

/// Result type for audio operations.
pub type AudioSystemResult<T> = Result<T, AudioError>;

/// Complete audio system that implements the Audio trait with real output.
pub struct AudioSystem {
    /// The audio buzzer for generating tones.
    buzzer: AudioBuzzer,

    /// The audio stream for output.
    stream: Option<AudioStream>,

    /// Current playing state.
    is_playing: bool,
}

impl AudioSystem {
    /// Creates a new audio system.
    pub fn new() -> AudioSystemResult<Self> {
        let buzzer = AudioBuzzer::new(BuzzerConfig::default())?;

        Ok(Self {
            buzzer,
            stream: None,
            is_playing: false,
        })
    }

    /// Creates an audio system with custom configuration.
    pub fn with_config(
        buzzer_config: BuzzerConfig,
        _stream_config: StreamConfig,
    ) -> AudioSystemResult<Self> {
        let buzzer = AudioBuzzer::new(buzzer_config)?;

        Ok(Self {
            buzzer,
            stream: None,
            is_playing: false,
        })
    }

    /// Initializes the audio stream for output.
    pub fn initialize_stream(&mut self, config: StreamConfig) -> AudioSystemResult<()> {
        let stream = AudioStream::new(config, &mut self.buzzer)?;
        self.stream = Some(stream);
        Ok(())
    }

    /// Initializes with default stream configuration.
    pub fn initialize_with_defaults(&mut self) -> AudioSystemResult<()> {
        self.initialize_stream(StreamConfig::default())
    }

    /// Checks if the audio stream is initialized.
    pub fn is_initialized(&self) -> bool {
        self.stream.is_some()
    }
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            buzzer: AudioBuzzer::silent(),
            stream: None,
            is_playing: false,
        })
    }
}

impl Audio for AudioSystem {
    fn play_beep(&mut self) -> AudioResult<()> {
        self.buzzer.start_tone()?;

        if let Some(ref mut stream) = self.stream {
            stream.start()?;
        }

        self.is_playing = true;
        Ok(())
    }

    fn stop_beep(&mut self) -> AudioResult<()> {
        self.buzzer.stop_tone()?;

        if let Some(ref mut stream) = self.stream {
            stream.pause()?;
        }

        self.is_playing = false;
        Ok(())
    }

    fn is_playing(&self) -> bool {
        self.is_playing && self.buzzer.is_playing()
    }

    fn set_volume(&mut self, volume: f32) -> AudioResult<()> {
        self.buzzer.set_volume(volume)
    }

    fn get_volume(&self) -> f32 {
        self.buzzer.get_volume()
    }

    fn set_frequency(&mut self, frequency: f32) -> AudioResult<()> {
        self.buzzer.set_frequency(frequency)
    }

    fn get_frequency(&self) -> f32 {
        self.buzzer.get_frequency()
    }
}

/// Type alias for headless audio operation - use the hardware null implementation.
pub type NullAudioSystem = crate::hardware::audio::NullAudio;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_system_creation() {
        // Test creation without actual audio hardware (for CI)
        let system = AudioSystem::new();

        // It's okay if this fails in headless environments
        if let Ok(audio) = system {
            assert!(!audio.is_playing());
            assert!(!audio.is_initialized());
        }
    }

    #[test]
    fn test_null_audio_system() {
        let mut system = NullAudioSystem::new();

        assert!(!system.is_playing());

        system.play_beep().unwrap();
        assert!(system.is_playing());

        system.stop_beep().unwrap();
        assert!(!system.is_playing());

        system.set_volume(0.8).unwrap();
        assert_eq!(system.get_volume(), 0.8);

        system.set_frequency(880.0).unwrap();
        assert_eq!(system.get_frequency(), 880.0);
    }

    #[test]
    fn test_audio_system_default() {
        let system = AudioSystem::default();
        assert!(!system.is_playing());
    }

    #[test]
    fn test_audio_system_with_config() {
        let buzzer_config = BuzzerConfig::new().with_frequency(880.0);
        let stream_config = StreamConfig::new();
        
        // This might fail in headless environments, which is expected
        let result = AudioSystem::with_config(buzzer_config, stream_config);
        
        if let Ok(system) = result {
            assert!(!system.is_playing());
            assert!(!system.is_initialized());
            assert_eq!(system.get_frequency(), 880.0);
        }
        // Failure is acceptable in CI/headless environments
    }

    #[test]
    fn test_audio_system_stream_initialization() {
        let mut system = AudioSystem::default();
        
        // Test initialization with default config
        let result = system.initialize_with_defaults();
        
        // May fail without audio hardware - that's expected behavior
        match result {
            Ok(_) => {
                assert!(system.is_initialized());
                // Test that we can still control audio after initialization
                system.set_volume(0.7).unwrap();
                assert_eq!(system.get_volume(), 0.7);
            }
            Err(_) => {
                // Expected in headless environments
                // Ensure system is still functional for basic operations
                system.set_volume(0.5).unwrap();
                assert_eq!(system.get_volume(), 0.5);
            }
        }
    }

    #[test]
    fn test_audio_system_full_workflow() {
        let mut system = AudioSystem::default();
        
        // Test complete workflow - may fail, that's ok
        let _play_result = system.play_beep();
        
        // Volume and frequency should always work
        system.set_volume(0.5).unwrap();
        assert_eq!(system.get_volume(), 0.5);
        
        system.set_frequency(1000.0).unwrap();
        assert_eq!(system.get_frequency(), 1000.0);
        
        // Test that stop doesn't error even if play failed
        let _stop_result = system.stop_beep();
        
        // Test edge cases for volume and frequency
        system.set_volume(0.0).unwrap();
        assert_eq!(system.get_volume(), 0.0);
        
        system.set_volume(1.0).unwrap();
        assert_eq!(system.get_volume(), 1.0);
    }

    #[test]
    fn test_audio_system_state_management() {
        let mut system = AudioSystem::default();
        
        // Initial state
        assert!(!system.is_playing());
        assert!(!system.is_initialized());
        
        // Test state consistency after operations
        let _result = system.play_beep();
        // is_playing() should reflect actual buzzer state
        let playing_state = system.is_playing();
        
        let _result = system.stop_beep();
        if playing_state {
            // If play worked, stop should make it not playing
            assert!(!system.is_playing());
        }
        
        // Test initialization state tracking
        let init_result = system.initialize_with_defaults();
        match init_result {
            Ok(_) => assert!(system.is_initialized()),
            Err(_) => assert!(!system.is_initialized()),
        }
    }

    #[test]
    fn test_null_audio_system_comprehensive() {
        let mut system = NullAudioSystem::new();
        
        // Test all operations work perfectly with null system
        assert!(!system.is_playing());
        
        system.play_beep().unwrap();
        assert!(system.is_playing());
        
        // Test volume range
        system.set_volume(0.0).unwrap();
        assert_eq!(system.get_volume(), 0.0);
        
        system.set_volume(1.0).unwrap();
        assert_eq!(system.get_volume(), 1.0);
        
        // Test frequency range
        system.set_frequency(20.0).unwrap();
        assert_eq!(system.get_frequency(), 20.0);
        
        system.set_frequency(20000.0).unwrap();
        assert_eq!(system.get_frequency(), 20000.0);
        
        // Test state transitions
        system.stop_beep().unwrap();
        assert!(!system.is_playing());
        
        // Multiple play/stop cycles
        for _ in 0..5 {
            system.play_beep().unwrap();
            assert!(system.is_playing());
            system.stop_beep().unwrap();
            assert!(!system.is_playing());
        }
    }

    #[test]
    fn test_audio_system_error_edge_cases() {
        // Test with various invalid configurations
        use crate::audio::buzzer::WaveformType;
        
        let invalid_configs = [
            BuzzerConfig::new().with_frequency(0.0).with_volume(0.0),
            BuzzerConfig::new().with_frequency(f32::MAX).with_volume(1.0),
            BuzzerConfig::new().with_waveform(WaveformType::Triangle),
        ];
        
        for config in invalid_configs {
            let stream_config = StreamConfig::new();
            
            // These might fail or succeed depending on the config and environment
            let _result = AudioSystem::with_config(config.clone(), stream_config);
            // We just want to ensure no panics occur
        }
    }
}
