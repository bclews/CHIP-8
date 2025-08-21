//! Frontend interfaces for the Chip-8 emulator.
//!
//! This module provides command-line and user interface components
//! for interacting with the Chip-8 emulator.

pub mod cli;
pub mod config;
pub mod gui;

// Re-export commonly used types
pub use cli::{run_cli, CliApp, Commands};
pub use config::{load_config, save_config, EmulatorBehaviorConfig, EmulatorConfig};

use crate::error::EmulatorError;
use crate::Cpu;

/// Result type for frontend operations.
pub type FrontendResult<T> = Result<T, EmulatorError>;

/// Simple Chip-8 emulator for CLI use.
pub struct SimpleEmulator {
    /// The CPU core.
    cpu: Cpu,

    /// Whether the emulator is running.
    running: bool,

    /// Target cycles per second.
    target_cps: u32,
}

impl SimpleEmulator {
    /// Creates a new simple emulator.
    pub fn new() -> Self {
        let cpu = Cpu::new();

        Self {
            cpu,
            running: false,
            target_cps: 700,
        }
    }

    /// Creates a new simple emulator with configuration.
    pub fn new_with_config(config: &EmulatorConfig) -> Self {
        let cpu = Cpu::new_with_config(&config.behavior);

        Self {
            cpu,
            running: false,
            target_cps: config.behavior.cpu_speed,
        }
    }

    /// Configures the emulator with new settings.
    pub fn configure(&mut self, config: &EmulatorConfig) {
        self.cpu.configure(&config.behavior);
        self.target_cps = config.behavior.cpu_speed;
    }

    /// Loads a ROM file into the emulator.
    pub fn load_rom<P: AsRef<std::path::Path>>(&mut self, path: P) -> FrontendResult<()> {
        let rom_data = std::fs::read(path)?;
        self.cpu.load_rom(&rom_data)?;
        Ok(())
    }

    /// Executes a single CPU cycle.
    pub fn step(&mut self) -> FrontendResult<()> {
        self.running = true;
        self.cpu.cycle()?;
        Ok(())
    }

    /// Runs the emulator for a specified number of cycles.
    pub fn run_cycles(&mut self, cycles: u32) -> FrontendResult<()> {
        for _ in 0..cycles {
            self.step()?;
        }
        Ok(())
    }

    /// Gets the CPU.
    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    /// Gets mutable access to the CPU.
    pub fn cpu_mut(&mut self) -> &mut Cpu {
        &mut self.cpu
    }

    /// Sets the target cycles per second.
    pub fn set_target_cps(&mut self, cps: u32) {
        self.target_cps = cps;
    }

    /// Gets the target cycles per second.
    pub fn target_cps(&self) -> u32 {
        self.target_cps
    }

    /// Checks if the emulator is currently running.
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Stops the emulator.
    pub fn stop(&mut self) {
        self.running = false;
    }

    /// Gets the display buffer from the CPU.
    pub fn get_display_buffer(&self) -> &[bool] {
        self.cpu.get_display_buffer()
    }
}

impl Default for SimpleEmulator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_emulator_creation() {
        let emulator = SimpleEmulator::new();
        assert_eq!(emulator.target_cps(), 700);
    }

    #[test]
    fn test_simple_emulator_speed_settings() {
        let mut emulator = SimpleEmulator::new();
        emulator.set_target_cps(1000);
        assert_eq!(emulator.target_cps(), 1000);
    }

    #[test]
    fn test_simple_emulator_running_state() {
        let mut emulator = SimpleEmulator::new();

        // Initially not running
        assert!(!emulator.is_running());

        // Create a simple test ROM
        let test_rom = [0x60, 0x05]; // LD V0, 5
        let temp_file = std::env::temp_dir().join("running_test.ch8");
        std::fs::write(&temp_file, test_rom).unwrap();

        emulator.load_rom(&temp_file).unwrap();

        // Still not running until we step
        assert!(!emulator.is_running());

        // After stepping, should be running
        emulator.step().unwrap();
        assert!(emulator.is_running());

        // Can stop the emulator
        emulator.stop();
        assert!(!emulator.is_running());

        // Clean up
        std::fs::remove_file(&temp_file).unwrap();
    }
}
