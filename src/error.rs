//! Error types for the Chip-8 emulator.
//!
//! This module defines all the error types used throughout the emulator,
//! providing clear error messages and proper error propagation.

use pixels::Error as PixelsError;
use thiserror::Error;

/// Main emulator error type that encompasses all possible errors.
#[derive(Error, Debug)]
pub enum EmulatorError {
    #[error("Invalid memory access at address {address:#04x}")]
    InvalidMemoryAccess { address: u16 },

    #[error("Unknown instruction {opcode:#04x}")]
    UnknownInstruction { opcode: u16 },

    #[error("ROM file too large: {size} bytes (max {max_size})")]
    RomTooLarge { size: usize, max_size: usize },

    #[error("ROM file is empty")]
    RomEmpty,

    #[error("Stack overflow")]
    StackOverflow,

    #[error("Stack underflow")]
    StackUnderflow,

    #[error("Invalid register index: {index}")]
    InvalidRegister { index: u8 },

    #[error("Audio system error: {0}")]
    AudioError(#[from] AudioError),

    #[error("Graphics system error: {0}")]
    Graphics(#[from] GraphicsError),

    #[error("Input system error: {0}")]
    InputError(#[from] InputError),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    ConfigError(#[from] ConfigError),

    #[error("TOML parsing error: {0}")]
    TomlParseError(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerializeError(#[from] toml::ser::Error),

    #[error("Pixels error: {0}")]
    PixelsError(String),
}

impl From<PixelsError> for EmulatorError {
    fn from(err: PixelsError) -> Self {
        EmulatorError::PixelsError(err.to_string())
    }
}

/// Audio-specific error types.
#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Failed to initialize audio device")]
    InitializationFailed,

    #[error("Audio stream error: {0}")]
    StreamError(String),

    #[error("Unsupported audio format")]
    UnsupportedFormat,

    #[error("Audio device not available")]
    DeviceNotAvailable,
}

/// Graphics-specific error types.
#[derive(Error, Debug)]
pub enum GraphicsError {
    #[error("Failed to initialize graphics system")]
    InitializationFailed,

    #[error("Render error: {0}")]
    RenderError(String),

    #[error("Invalid display coordinates: ({x}, {y})")]
    InvalidCoordinates { x: u8, y: u8 },

    #[error("Invalid sprite data")]
    InvalidSpriteData,

    #[error("Window creation failed: {0}")]
    WindowCreationFailed(String),

    #[error("Event loop creation failed: {0}")]
    EventLoopCreationFailed(String),

    #[error("Event loop run failed: {0}")]
    EventLoopRunFailed(String),

    #[error("Pixels initialization failed: {0}")]
    PixelsInitializationFailed(String),

    #[error("Window resize failed: {0}")]
    ResizeFailed(String),

    #[error("Render failed: {0}")]
    RenderFailed(String),

    #[error("Window closed")]
    WindowClosed,

    #[error("Invalid buffer size: expected {expected}, got {actual}")]
    InvalidBufferSize { expected: usize, actual: usize },
}

/// Configuration-specific error types.
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid configuration value for '{key}': {value}")]
    InvalidValue { key: String, value: String },

    #[error("Missing required configuration key: {key}")]
    MissingKey { key: String },

    #[error("Configuration file format error: {0}")]
    FormatError(#[from] toml::de::Error),

    #[error("Configuration serialization error: {0}")]
    SerializationError(#[from] toml::ser::Error),
}

/// Input-specific error types.
#[derive(Error, Debug)]
pub enum InputError {
    #[error("Invalid key code: {key}")]
    InvalidKey { key: u8 },

    #[error("Key mapping error: {0}")]
    MappingError(String),

    #[error("Input device not available")]
    DeviceNotAvailable,
}

/// Alias for Result with EmulatorError.
pub type Result<T> = std::result::Result<T, EmulatorError>;

/// Alias for Result with AudioError.
pub type AudioResult<T> = std::result::Result<T, AudioError>;

/// Alias for Result with GraphicsError.
pub type GraphicsResult<T> = std::result::Result<T, GraphicsError>;

/// Alias for Result with ConfigError.
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;

/// Alias for Result with InputError.
pub type InputResult<T> = std::result::Result<T, InputError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = EmulatorError::InvalidMemoryAccess { address: 0x1234 };
        assert_eq!(error.to_string(), "Invalid memory access at address 0x1234");
    }

    #[test]
    fn test_rom_too_large_error() {
        let error = EmulatorError::RomTooLarge {
            size: 5000,
            max_size: 3584,
        };
        assert_eq!(
            error.to_string(),
            "ROM file too large: 5000 bytes (max 3584)"
        );
    }

    #[test]
    fn test_unknown_instruction_error() {
        let error = EmulatorError::UnknownInstruction { opcode: 0xABCD };
        assert_eq!(error.to_string(), "Unknown instruction 0xabcd");
    }

    #[test]
    fn test_audio_error_conversion() {
        let audio_err = AudioError::InitializationFailed;
        let emulator_err: EmulatorError = audio_err.into();

        assert!(matches!(
            emulator_err,
            EmulatorError::AudioError(AudioError::InitializationFailed)
        ));
    }

    #[test]
    fn test_graphics_error_conversion() {
        let graphics_err = GraphicsError::InvalidCoordinates { x: 100, y: 200 };
        let emulator_err: EmulatorError = graphics_err.into();

        assert!(matches!(
            emulator_err,
            EmulatorError::Graphics(GraphicsError::InvalidCoordinates { x: 100, y: 200 })
        ));
    }
}
