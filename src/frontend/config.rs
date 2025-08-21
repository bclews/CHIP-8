//! Configuration management for the Chip-8 emulator.
//!
//! This module provides configuration loading, saving, and management
//! for the emulator settings.

use serde::{Deserialize, Serialize};
use std::path::Path;

use super::cli::DebugConfig;
use crate::audio::BuzzerConfig;
use crate::error::{ConfigError, EmulatorError};
use crate::graphics::GraphicsConfig;
use crate::input::KeyboardConfig;

/// Emulator behavior configuration for compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorBehaviorConfig {
    /// CPU speed in instructions per second.
    pub cpu_speed: u32,

    /// Enable memory wraparound for out-of-bounds access.
    pub memory_wraparound: bool,

    /// Strict bounds checking (disable for classic compatibility).
    pub strict_bounds: bool,

    /// Timer frequency in Hz (classic CHIP-8 uses 60Hz).
    pub timer_frequency: u32,
}

impl Default for EmulatorBehaviorConfig {
    fn default() -> Self {
        Self {
            cpu_speed: 700,
            memory_wraparound: false,
            strict_bounds: true,
            timer_frequency: 60,
        }
    }
}

impl EmulatorBehaviorConfig {
    /// Creates a classic CHIP-8 compatible configuration.
    pub fn classic() -> Self {
        Self {
            cpu_speed: 500,
            memory_wraparound: true,
            strict_bounds: false,
            timer_frequency: 60,
        }
    }

    /// Creates a modern interpretation configuration.
    pub fn modern() -> Self {
        Self {
            cpu_speed: 700,
            memory_wraparound: false,
            strict_bounds: true,
            timer_frequency: 60,
        }
    }
}

/// Complete emulator configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmulatorConfig {
    /// Emulator behavior configuration.
    pub behavior: EmulatorBehaviorConfig,

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
            behavior: EmulatorBehaviorConfig::default(),
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

    /// Creates a classic CHIP-8 compatible configuration.
    pub fn classic() -> Self {
        Self {
            behavior: EmulatorBehaviorConfig::classic(),
            graphics: GraphicsConfig::classic_green().with_scale_factor(10),
            audio: BuzzerConfig::classic().with_volume(0.3),
            keyboard: KeyboardConfig::desktop(),
            debug: DebugConfig::default(),
        }
    }

    /// Creates a modern interpretation configuration.
    pub fn modern() -> Self {
        Self {
            behavior: EmulatorBehaviorConfig::modern(),
            graphics: GraphicsConfig::high_contrast().with_scale_factor(12),
            audio: BuzzerConfig::new().with_volume(0.4).with_frequency(440.0),
            keyboard: KeyboardConfig::gaming(),
            debug: DebugConfig::default(),
        }
    }

    /// Creates a gaming-optimized configuration.
    pub fn gaming() -> Self {
        Self {
            behavior: EmulatorBehaviorConfig::modern(),
            graphics: GraphicsConfig::high_contrast()
                .with_scale_factor(12)
                .with_smooth_scaling(false),
            audio: BuzzerConfig::new().with_volume(0.4).with_frequency(800.0),
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
            behavior: EmulatorBehaviorConfig::modern(),
            graphics: GraphicsConfig::classic_green()
                .with_scale_factor(8)
                .with_smooth_scaling(true),
            audio: BuzzerConfig::new().with_volume(0.2).with_frequency(440.0),
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
            behavior: EmulatorBehaviorConfig::classic(),
            graphics: GraphicsConfig::classic_amber()
                .with_scale_factor(10)
                .with_smooth_scaling(false),
            audio: BuzzerConfig::classic().with_volume(0.5),
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
    let config_paths = ["chip8.toml", "config/chip8.toml", ".config/chip8.toml"];

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
"#
    .to_string()
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
        assert_eq!(
            config.graphics.scale_factor,
            deserialized.graphics.scale_factor
        );
    }

    #[test]
    fn test_config_file_operations() {
        let config = EmulatorConfig::gaming();
        let temp_file = NamedTempFile::new().unwrap();

        // Test saving
        save_config(&config, temp_file.path()).unwrap();

        // Test loading
        let loaded_config = load_config(temp_file.path()).unwrap();
        assert_eq!(
            config.graphics.scale_factor,
            loaded_config.graphics.scale_factor
        );
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

    #[test]
    fn test_behavior_config_creation() {
        let config = EmulatorBehaviorConfig::default();
        assert_eq!(config.cpu_speed, 700);
        assert!(!config.memory_wraparound);
        assert!(config.strict_bounds);
        assert_eq!(config.timer_frequency, 60);
    }

    #[test]
    fn test_behavior_config_presets() {
        let classic = EmulatorBehaviorConfig::classic();
        assert_eq!(classic.cpu_speed, 500);
        assert!(classic.memory_wraparound);
        assert!(!classic.strict_bounds);
        assert_eq!(classic.timer_frequency, 60);

        let modern = EmulatorBehaviorConfig::modern();
        assert_eq!(modern.cpu_speed, 700);
        assert!(!modern.memory_wraparound);
        assert!(modern.strict_bounds);
        assert_eq!(modern.timer_frequency, 60);
    }

    #[test]
    fn test_emulator_config_presets() {
        let classic = EmulatorConfig::classic();
        assert!(classic.behavior.memory_wraparound);
        assert_eq!(classic.behavior.cpu_speed, 500);

        let modern = EmulatorConfig::modern();
        assert!(!modern.behavior.memory_wraparound);
        assert_eq!(modern.behavior.cpu_speed, 700);

        let retro = EmulatorConfig::retro();
        assert!(retro.behavior.memory_wraparound);
        assert_eq!(retro.behavior.cpu_speed, 500);
    }

    #[test]
    fn test_all_configuration_presets_comprehensive() {
        // Test classic configuration
        let classic = EmulatorConfig::classic();
        assert!(classic.behavior.memory_wraparound);
        assert_eq!(classic.behavior.cpu_speed, 500);
        assert!(!classic.behavior.strict_bounds);
        assert_eq!(classic.behavior.timer_frequency, 60);
        assert_eq!(classic.graphics.scale_factor, 10);
        assert_eq!(classic.audio.volume, 0.3);
        assert!(classic.validate().is_ok());
        
        // Test modern configuration
        let modern = EmulatorConfig::modern();
        assert!(!modern.behavior.memory_wraparound);
        assert_eq!(modern.behavior.cpu_speed, 700);
        assert!(modern.behavior.strict_bounds);
        assert_eq!(modern.graphics.scale_factor, 12);
        assert_eq!(modern.audio.frequency, 440.0);
        assert!(modern.validate().is_ok());
        
        // Test gaming configuration  
        let gaming = EmulatorConfig::gaming();
        assert_eq!(gaming.behavior.cpu_speed, 700); // Uses modern behavior
        assert_eq!(gaming.graphics.scale_factor, 12);
        assert!(!gaming.graphics.smooth_scaling); // Performance optimized
        assert!(!gaming.debug.enabled); // Debug disabled for performance
        assert_eq!(gaming.audio.frequency, 800.0);
        assert!(gaming.validate().is_ok());
        
        // Test development configuration
        let development = EmulatorConfig::development();
        assert!(development.debug.enabled); // Debug-friendly
        assert!(development.debug.break_on_error);
        assert!(development.debug.log_instructions);
        assert_eq!(development.graphics.scale_factor, 8);
        assert!(development.graphics.smooth_scaling);
        assert_eq!(development.audio.volume, 0.2); // Quieter for development
        assert!(development.validate().is_ok());

        // Test retro configuration
        let retro = EmulatorConfig::retro();
        assert!(retro.behavior.memory_wraparound); // Classic behavior
        assert_eq!(retro.behavior.cpu_speed, 500);
        assert_eq!(retro.graphics.scale_factor, 10);
        assert!(!retro.graphics.smooth_scaling); // Retro pixelated look
        assert_eq!(retro.audio.volume, 0.5);
        assert!(retro.validate().is_ok());
    }

    #[test]
    fn test_configuration_validation_edge_cases() {
        // Test boundary values for scale factor
        let mut config = EmulatorConfig::default();
        
        config.graphics.scale_factor = 1; // Valid minimum
        assert!(config.validate().is_ok());
        
        config.graphics.scale_factor = 20; // Valid maximum
        assert!(config.validate().is_ok());
        
        config.graphics.scale_factor = 0; // Invalid
        assert!(config.validate().is_err());
        
        config.graphics.scale_factor = 21; // Invalid  
        assert!(config.validate().is_err());
        
        // Test boundary values for audio volume
        config = EmulatorConfig::default();
        
        config.audio.volume = 0.0; // Valid minimum
        assert!(config.validate().is_ok());
        
        config.audio.volume = 1.0; // Valid maximum
        assert!(config.validate().is_ok());
        
        config.audio.volume = -0.1; // Invalid
        assert!(config.validate().is_err());
        
        config.audio.volume = 1.1; // Invalid
        assert!(config.validate().is_err());
        
        // Test frequency validation
        config = EmulatorConfig::default();
        
        config.audio.frequency = 0.0; // Valid minimum
        assert!(config.validate().is_ok());
        
        config.audio.frequency = 20000.0; // High but valid
        assert!(config.validate().is_ok());
        
        config.audio.frequency = -1.0; // Invalid
        assert!(config.validate().is_err());
        
        // Test keyboard polling rate
        config = EmulatorConfig::default();
        
        config.keyboard.polling_rate = 1; // Valid minimum
        assert!(config.validate().is_ok());
        
        config.keyboard.polling_rate = 1000; // Valid maximum
        assert!(config.validate().is_ok());
        
        config.keyboard.polling_rate = 0; // Invalid
        assert!(config.validate().is_err());
        
        config.keyboard.polling_rate = 1001; // Invalid
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_configuration_serialization_all_presets() {
        let presets = [
            EmulatorConfig::classic(),
            EmulatorConfig::modern(), 
            EmulatorConfig::gaming(),
            EmulatorConfig::development(),
            EmulatorConfig::retro(),
        ];
        
        for preset in presets {
            // Test TOML serialization
            let toml_str = toml::to_string(&preset).unwrap();
            assert!(!toml_str.is_empty());
            assert!(toml_str.contains("[behavior]"));
            assert!(toml_str.contains("[graphics]"));
            assert!(toml_str.contains("[audio]"));
            
            // Test deserialization
            let deserialized: EmulatorConfig = toml::from_str(&toml_str).unwrap();
            
            // Verify key properties are preserved
            assert_eq!(preset.behavior.cpu_speed, deserialized.behavior.cpu_speed);
            assert_eq!(preset.behavior.memory_wraparound, deserialized.behavior.memory_wraparound);
            assert_eq!(preset.graphics.scale_factor, deserialized.graphics.scale_factor);
            assert_eq!(preset.audio.volume, deserialized.audio.volume);
            assert_eq!(preset.debug.enabled, deserialized.debug.enabled);
            
            // Validate deserialized config
            assert!(deserialized.validate().is_ok());
        }
    }

    #[test]
    fn test_config_merge_functionality() {
        let mut base_config = EmulatorConfig::classic();
        let modern_config = EmulatorConfig::modern();
        
        // Verify initial state
        assert_eq!(base_config.behavior.cpu_speed, 500);
        assert_eq!(base_config.graphics.scale_factor, 10);
        
        // Merge modern into classic
        base_config.merge(&modern_config);
        
        // Should have modern's graphics and audio settings
        assert_eq!(base_config.graphics.scale_factor, 12);
        assert_eq!(base_config.audio.frequency, 440.0);
        
        // But behavior config isn't touched by merge (it's not included)
        // This tests the current implementation
        assert_eq!(base_config.behavior.cpu_speed, 500); // Still classic value
    }

    #[test]
    fn test_profile_creation_comprehensive() {
        let profile_names = ["default", "gaming", "development", "dev", "retro", "classic"];
        
        for name in profile_names {
            let config = ConfigProfiles::from_name(name).unwrap();
            assert!(config.validate().is_ok());
            
            // Verify each profile has expected characteristics
            match name {
                "gaming" => {
                    assert!(!config.debug.enabled);
                    assert!(!config.graphics.smooth_scaling);
                }
                "development" | "dev" => {
                    assert!(config.debug.enabled);
                    assert!(config.debug.break_on_error);
                }
                "retro" | "classic" => {
                    assert!(config.behavior.memory_wraparound);
                }
                "default" => {
                    assert!(!config.behavior.memory_wraparound);
                    assert!(config.behavior.strict_bounds);
                }
                _ => {}
            }
        }
        
        // Test invalid profile names
        let invalid_names = ["invalid", "unknown", ""];
        for name in invalid_names {
            assert!(ConfigProfiles::from_name(name).is_err());
        }
        
        // Test that case-insensitive matching works
        assert!(ConfigProfiles::from_name("GAMING").is_ok());
        assert!(ConfigProfiles::from_name("Development").is_ok());
    }

    #[test]
    fn test_environment_variable_overrides() {
        use std::env;
        
        // Save original values
        let original_scale = env::var("CHIP8_SCALE").ok();
        let original_volume = env::var("CHIP8_VOLUME").ok();
        
        // Set test values
        env::set_var("CHIP8_SCALE", "15");
        env::set_var("CHIP8_VOLUME", "0.8");
        env::set_var("CHIP8_DEBUG", "1");
        
        let mut config = EmulatorConfig::default();
        EnvConfig::apply_env_overrides(&mut config);
        
        // Check overrides were applied
        assert_eq!(config.graphics.scale_factor, 15);
        assert_eq!(config.audio.volume, 0.8);
        assert!(config.debug.enabled);
        
        // Test invalid values are ignored
        env::set_var("CHIP8_SCALE", "0"); // Invalid
        env::set_var("CHIP8_VOLUME", "2.0"); // Invalid
        
        let mut config2 = EmulatorConfig::default();
        EnvConfig::apply_env_overrides(&mut config2);
        
        // Should keep default values for invalid overrides
        assert_eq!(config2.graphics.scale_factor, 10); // Default value
        assert_eq!(config2.audio.volume, 0.4); // Default value
        
        // Cleanup
        env::remove_var("CHIP8_SCALE");
        env::remove_var("CHIP8_VOLUME");
        env::remove_var("CHIP8_DEBUG");
        
        // Restore original values if they existed
        if let Some(scale) = original_scale {
            env::set_var("CHIP8_SCALE", scale);
        }
        if let Some(volume) = original_volume {
            env::set_var("CHIP8_VOLUME", volume);
        }
    }
}
