//! Hardware abstraction layer for the Chip-8 emulator.
//!
//! This module defines the traits and interfaces that abstract away
//! platform-specific hardware implementations for display, audio, and input.

pub mod audio;
pub mod display;
pub mod input;

// Re-export commonly used types
pub use audio::{Audio, AudioResult};
pub use display::{Display, DisplayResult, DISPLAY_WIDTH, DISPLAY_HEIGHT, DISPLAY_PIXELS};
pub use input::{Input, InputResult, ChipKey};

// Re-export error types from the main error module
pub use crate::error::{AudioError, GraphicsError, InputError};

use crate::error::EmulatorError;

/// Result type for hardware operations.
pub type HardwareResult<T> = Result<T, EmulatorError>;

/// Hardware abstraction for all Chip-8 I/O systems.
/// 
/// This trait combines all hardware interfaces into a single
/// abstraction that can be easily implemented and tested.
pub trait Hardware {
    type Display: Display;
    type Audio: Audio;
    type Input: Input;

    fn display(&mut self) -> &mut Self::Display;
    fn audio(&mut self) -> &mut Self::Audio;
    fn input(&mut self) -> &mut Self::Input;
    
    /// Updates all hardware systems.
    /// 
    /// This should be called once per frame to update input state,
    /// render graphics, and handle audio.
    fn update(&mut self) -> HardwareResult<()>;
}

/// A null hardware implementation for testing and headless operation.
pub struct NullHardware {
    display: display::NullDisplay,
    audio: audio::NullAudio,
    input: input::NullInput,
}

impl NullHardware {
    pub fn new() -> Self {
        Self {
            display: display::NullDisplay::new(),
            audio: audio::NullAudio::new(),
            input: input::NullInput::new(),
        }
    }
}

impl Default for NullHardware {
    fn default() -> Self {
        Self::new()
    }
}

impl Hardware for NullHardware {
    type Display = display::NullDisplay;
    type Audio = audio::NullAudio;
    type Input = input::NullInput;

    fn display(&mut self) -> &mut Self::Display {
        &mut self.display
    }

    fn audio(&mut self) -> &mut Self::Audio {
        &mut self.audio
    }

    fn input(&mut self) -> &mut Self::Input {
        &mut self.input
    }

    fn update(&mut self) -> HardwareResult<()> {
        self.input.update()?;
        self.display.render()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_hardware_creation() {
        let mut hardware = NullHardware::new();
        
        // Test that all systems are accessible
        assert!(!hardware.display().is_dirty());
        assert!(!hardware.audio().is_playing());
        assert!(!hardware.input().is_key_pressed(ChipKey::Key0));
    }

    #[test]
    fn test_hardware_update() {
        let mut hardware = NullHardware::new();
        
        // Should not fail with null implementations
        hardware.update().unwrap();
    }
}