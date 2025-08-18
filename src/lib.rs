//! Chip-8 Emulator Library
//!
//! A modern Rust implementation of the Chip-8 virtual machine system.
//! This library provides a complete emulation core with clean APIs for
//! building frontends and tools.

pub mod audio;
pub mod emulator;
pub mod error;
pub mod frontend;
pub mod graphics;
pub mod hardware;
pub mod input;

#[cfg(test)]
pub mod test_utils;

// Re-export commonly used types
pub use audio::{AudioSystem, NullAudioSystem, AudioBuzzer, BuzzerConfig};
pub use emulator::{Cpu, CpuState, Memory, Registers, Stack, Timers};
pub use error::{EmulatorError, Result};
pub use frontend::{SimpleEmulator, EmulatorConfig, CliApp};
pub use graphics::{GraphicsDisplay, GraphicsConfig};
pub use hardware::{Hardware, NullHardware, Audio, Display, Input};
pub use input::{InputSystem, NullInputSystem, KeyboardInput, QwertyMapper};