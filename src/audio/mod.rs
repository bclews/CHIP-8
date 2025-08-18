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
    pub fn with_config(buzzer_config: BuzzerConfig, _stream_config: StreamConfig) -> AudioSystemResult<Self> {
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
}