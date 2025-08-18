//! Configuration management for the Chip-8 emulator.
//!
//! This module provides configuration loading, saving, and management
//! for the emulator settings.

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::error::{ConfigError, EmulatorError};
use crate::graphics::GraphicsConfig;
use crate::audio::BuzzerConfig;
use crate::input::KeyboardConfig;
use super::cli::DebugConfig;

/// Complete emulator configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorConfig {
    /// Graphics configuration.
    pub graphics: GraphicsConfig,
    
    /// Audio configuration.
    pub audio: BuzzerConfig,
    
    /// Keyboard configuration.
    pub keyboard: KeyboardConfig,
    
    /// Debug configuration.
    pub debug: DebugConfig,
}

impl Default for EmulatorConfig {
    fn default() -> Self {
        Self {
            graphics: GraphicsConfig::classic_green(),
            audio: BuzzerConfig::classic(),
            keyboard: KeyboardConfig::desktop(),
            debug: DebugConfig::default(),
        }
    }
}

impl EmulatorConfig {
    /// Creates a new configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Creates a gaming-optimized configuration.
    pub fn gaming() -> Self {
        Self {
            graphics: GraphicsConfig::high_contrast()
                .with_scale_factor(12)
                .with_smooth_scaling(false),
            audio: BuzzerConfig::new()
                .with_volume(0.4)
                .with_frequency(800.0),
            keyboard: KeyboardConfig::gaming(),
            debug: DebugConfig {
                enabled: false,
                break_on_error: false,
                log_instructions: false,
            },
        }
    }
    
    /// Creates a development-friendly configuration.
    pub fn development() -> Self {
        Self {
            graphics: GraphicsConfig::classic_green()
                .with_scale_factor(8)
                .with_smooth_scaling(true),
            audio: BuzzerConfig::new()
                .with_volume(0.2)
                .with_frequency(440.0),
            keyboard: KeyboardConfig::desktop(),
            debug: DebugConfig {
                enabled: true,
                break_on_error: true,
                log_instructions: true,
            },
        }
    }
    
    /// Creates a classic retro configuration.
    pub fn retro() -> Self {
        Self {
            graphics: GraphicsConfig::classic_amber()
                .with_scale_factor(10)
                .with_smooth_scaling(false),
            audio: BuzzerConfig::classic()
                .with_volume(0.5),
            keyboard: KeyboardConfig::desktop(),
            debug: DebugConfig::default(),
        }
    }
    
    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate graphics settings
        if self.graphics.scale_factor == 0 || self.graphics.scale_factor > 20 {
            return Err(ConfigError::InvalidValue {
                key: "graphics.scale_factor".to_string(),
                value: self.graphics.scale_factor.to_string(),
            });
        }
        
        // Validate audio settings
        if self.audio.volume < 0.0 || self.audio.volume > 1.0 {
            return Err(ConfigError::InvalidValue {
                key: "audio.volume".to_string(),
                value: self.audio.volume.to_string(),
            });
        }
        
        if self.audio.frequency < 0.0 {
            return Err(ConfigError::InvalidValue {
                key: "audio.frequency".to_string(),
                value: self.audio.frequency.to_string(),
            });
        }
        
        // Validate keyboard settings
        if self.keyboard.polling_rate == 0 || self.keyboard.polling_rate > 1000 {
            return Err(ConfigError::InvalidValue {
                key: "keyboard.polling_rate".to_string(),
                value: self.keyboard.polling_rate.to_string(),
            });
        }
        
        Ok(())
    }
    
    /// Merges another configuration into this one.
    pub fn merge(&mut self, other: &Self) {
        // Replace configuration values with those from other config
        self.graphics = other.graphics.clone();
        self.audio = other.audio.clone();
        self.keyboard = other.keyboard.clone();
        self.debug = other.debug.clone();
    }
}

/// Loads configuration from a TOML file.
pub fn load_config<P: AsRef<Path>>(path: P) -> Result<EmulatorConfig, EmulatorError> {
    let content = std::fs::read_to_string(path.as_ref())?;
    let config: EmulatorConfig = toml::from_str(&content)?;
    config.validate()?;
    Ok(config)
}

/// Saves configuration to a TOML file.
pub fn save_config<P: AsRef<Path>>(config: &EmulatorConfig, path: P) -> Result<(), EmulatorError> {
    config.validate()?;
    let content = toml::to_string_pretty(config)?;
    std::fs::write(path.as_ref(), content)?;
    Ok(())
}

/// Loads configuration from the default locations.
pub fn load_default_config() -> EmulatorConfig {
    let config_paths = [
        "chip8.toml",
        "config/chip8.toml",
        ".config/chip8.toml",
    ];
    
    for path in &config_paths {
        if Path::new(path).exists() {
            match load_config(path) {
                Ok(config) => return config,
                Err(e) => log::warn!("Failed to load config from {}: {}", path, e),
            }
        }
    }
    
    EmulatorConfig::default()
}

/// Creates a sample configuration file with comments.
pub fn create_sample_config() -> String {
    r#"# Chip-8 Emulator Configuration File
# This file contains all configurable options for the emulator

[graphics]
# Foreground color for "on" pixels (RGB values)
foreground_color = { r = 0, g = 255, b = 0, a = 255 }  # Green

# Background color for "off" pixels (RGB values)
background_color = { r = 0, g = 0, b = 0, a = 255 }    # Black

# Scale factor for the display (1-20)
scale_factor = 10

# Enable smooth scaling
smooth_scaling = false

# Maintain aspect ratio when resizing
maintain_aspect_ratio = true

[audio]
# Buzzer frequency in Hz
frequency = 440.0

# Volume level (0.0 = silent, 1.0 = full volume)
volume = 0.3

# Sample rate for audio generation
sample_rate = 44100

# Waveform type: "Sine", "Square", "Sawtooth", or "Triangle"
waveform = "Square"

[keyboard]
# Enable keyboard event capture
capture_enabled = true

# Polling rate in Hz for keyboard updates
polling_rate = 60

# Handle key repeat events
handle_repeats = false

# Minimum time between repeats in milliseconds
repeat_delay = 500

# Use raw keyboard input (may reduce latency)
use_raw_input = false

[debug]
# Enable debug mode
enabled = false

# Break execution on CPU errors
break_on_error = false

# Log CPU instructions to console
log_instructions = false
"#.to_string()
}

/// Configuration profiles for different use cases.
pub struct ConfigProfiles;

impl ConfigProfiles {
    /// Gets all available profile names.
    pub fn available_profiles() -> Vec<&'static str> {
        vec!["default", "gaming", "development", "retro"]
    }
    
    /// Creates a configuration from a profile name.
    pub fn from_name(name: &str) -> Result<EmulatorConfig, ConfigError> {
        match name.to_lowercase().as_str() {
            "default" => Ok(EmulatorConfig::default()),
            "gaming" => Ok(EmulatorConfig::gaming()),
            "development" | "dev" => Ok(EmulatorConfig::development()),
            "retro" | "classic" => Ok(EmulatorConfig::retro()),
            _ => Err(ConfigError::InvalidValue {
                key: "profile".to_string(),
                value: name.to_string(),
            }),
        }
    }
    
    /// Gets the description for a profile.
    pub fn profile_description(name: &str) -> Option<&'static str> {
        match name.to_lowercase().as_str() {
            "default" => Some("Default configuration with balanced settings"),
            "gaming" => Some("Optimized for gaming with high performance settings"),
            "development" => Some("Development-friendly with debugging enabled"),
            "retro" => Some("Classic retro styling with amber colors"),
            _ => None,
        }
    }
}

/// Environment variable configuration override.
pub struct EnvConfig;

impl EnvConfig {
    /// Loads configuration overrides from environment variables.
    pub fn apply_env_overrides(config: &mut EmulatorConfig) {
        // Graphics overrides
        if let Ok(scale) = std::env::var("CHIP8_SCALE") {
            if let Ok(scale_value) = scale.parse::<u32>() {
                if scale_value > 0 && scale_value <= 20 {
                    config.graphics.scale_factor = scale_value;
                }
            }
        }
        
        // Audio overrides
        if let Ok(volume) = std::env::var("CHIP8_VOLUME") {
            if let Ok(volume_value) = volume.parse::<f32>() {
                if (0.0..=1.0).contains(&volume_value) {
                    config.audio.volume = volume_value;
                }
            }
        }
        
        if let Ok(freq) = std::env::var("CHIP8_FREQUENCY") {
            if let Ok(freq_value) = freq.parse::<f32>() {
                if freq_value >= 0.0 {
                    config.audio.frequency = freq_value;
                }
            }
        }
        
        // Debug overrides
        if std::env::var("CHIP8_DEBUG").is_ok() {
            config.debug.enabled = true;
        }
        
        if std::env::var("CHIP8_VERBOSE").is_ok() {
            config.debug.log_instructions = true;
        }
    }
    
    /// Gets all available environment variables.
    pub fn available_env_vars() -> Vec<(&'static str, &'static str)> {
        vec![
            ("CHIP8_SCALE", "Display scale factor (1-20)"),
            ("CHIP8_VOLUME", "Audio volume (0.0-1.0)"),
            ("CHIP8_FREQUENCY", "Buzzer frequency in Hz"),
            ("CHIP8_DEBUG", "Enable debug mode (any value)"),
            ("CHIP8_VERBOSE", "Enable verbose logging (any value)"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_config_creation() {
        let config = EmulatorConfig::new();
        assert_eq!(config.graphics.scale_factor, 10);
        assert_eq!(config.audio.volume, 0.4);
    }
    
    #[test]
    fn test_config_profiles() {
        let gaming = EmulatorConfig::gaming();
        assert_eq!(gaming.graphics.scale_factor, 12);
        assert!(!gaming.debug.enabled);
        
        let dev = EmulatorConfig::development();
        assert!(dev.debug.enabled);
        assert!(dev.debug.break_on_error);
        
        let retro = EmulatorConfig::retro();
        assert_eq!(retro.graphics.foreground_color.r, 255);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = EmulatorConfig::default();
        assert!(config.validate().is_ok());
        
        // Test invalid scale
        config.graphics = config.graphics.with_scale_factor(0);
        assert!(config.validate().is_err());
        
        // Test invalid volume (set directly to bypass clamping)
        config = EmulatorConfig::default();
        config.audio.volume = 2.0; // Set directly to bypass with_volume clamping
        assert!(config.validate().is_err());
    }
    
    #[test]
    fn test_config_serialization() {
        let config = EmulatorConfig::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(!toml_str.is_empty());
        
        let deserialized: EmulatorConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(config.graphics.scale_factor, deserialized.graphics.scale_factor);
    }
    
    #[test]
    fn test_config_file_operations() {
        let config = EmulatorConfig::gaming();
        let temp_file = NamedTempFile::new().unwrap();
        
        // Test saving
        save_config(&config, temp_file.path()).unwrap();
        
        // Test loading
        let loaded_config = load_config(temp_file.path()).unwrap();
        assert_eq!(config.graphics.scale_factor, loaded_config.graphics.scale_factor);
    }
    
    #[test]
    fn test_profile_creation() {
        let gaming = ConfigProfiles::from_name("gaming").unwrap();
        assert_eq!(gaming.graphics.scale_factor, 12);
        
        let dev = ConfigProfiles::from_name("development").unwrap();
        assert!(dev.debug.enabled);
        
        let invalid = ConfigProfiles::from_name("invalid");
        assert!(invalid.is_err());
    }
    
    #[test]
    fn test_profile_listings() {
        let profiles = ConfigProfiles::available_profiles();
        assert!(profiles.contains(&"gaming"));
        assert!(profiles.contains(&"development"));
        
        let desc = ConfigProfiles::profile_description("gaming");
        assert!(desc.is_some());
        assert!(desc.unwrap().contains("gaming"));
    }
    
    #[test]
    fn test_sample_config_generation() {
        let sample = create_sample_config();
        assert!(sample.contains("[graphics]"));
        assert!(sample.contains("[audio]"));
        assert!(sample.contains("[keyboard]"));
        assert!(sample.contains("[debug]"));
    }
    
    #[test]
    fn test_env_var_listings() {
        let vars = EnvConfig::available_env_vars();
        assert!(!vars.is_empty());
        assert!(vars.iter().any(|(name, _)| *name == "CHIP8_SCALE"));
        assert!(vars.iter().any(|(name, _)| *name == "CHIP8_VOLUME"));
    }
}