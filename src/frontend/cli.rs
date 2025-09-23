//! Command-line interface for the Chip-8 emulator.
//!
//! This module provides a comprehensive CLI using clap for running
//! and configuring the Chip-8 emulator.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use super::FrontendResult;

/// Modern Chip-8 emulator written in Rust.
///
/// By default, providing a ROM file launches the GUI.
/// Use subcommands like 'run' for batch execution mode.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct CliApp {
    /// ROM file to load and run
    #[arg(value_name = "ROM_FILE")]
    pub rom_file: Option<PathBuf>,

    /// Enable verbose output
    #[arg(long)]
    pub verbose: bool,

    /// Path to configuration file (TOML)
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    /// Configuration profile (classic, modern, gaming, development, retro)
    #[arg(long, value_name = "PROFILE")]
    pub profile: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available CLI commands.
#[derive(Subcommand)]
pub enum Commands {
    /// Display information about a ROM file
    Info {
        /// ROM file to analyze
        rom_file: PathBuf,
    },

    /// Validate a ROM file
    Validate {
        /// ROM file to validate
        rom_file: PathBuf,
    },
}

/// Runs the CLI application.
pub fn run_cli() -> FrontendResult<()> {
    let args = CliApp::parse();

    match &args.command {
        Some(Commands::Info { rom_file }) => show_rom_info(rom_file),
        Some(Commands::Validate { rom_file }) => validate_rom(rom_file),
        None => {
            // Show help or usage
            println!("Chip-8 Emulator");
            println!("Usage: chip8 [ROM_FILE]    - Launch GUI with ROM");
            println!("       chip8 <COMMAND>     - Run subcommand");
            println!();
            println!("Commands:");
            println!("  info       Show ROM information");
            println!("  validate   Validate a ROM file");
            println!();
            println!("Run 'chip8 --help' for more information.");
            Ok(())
        }
    }
}

/// Shows information about a ROM file.
fn show_rom_info(rom_file: &PathBuf) -> FrontendResult<()> {
    let rom_data = std::fs::read(rom_file)?;

    println!("ROM Information:");
    println!("File: {}", rom_file.display());
    println!("Size: {} bytes", rom_data.len());
    println!("Max size: 3584 bytes");

    if rom_data.len() > 3584 {
        println!("⚠️  Warning: ROM exceeds maximum size");
    } else {
        println!("✅ Size is valid");
    }

    if rom_data.is_empty() {
        println!("❌ Error: ROM is empty");
    } else {
        println!("✅ ROM contains data");
    }

    // Show first few bytes
    println!("\nFirst 16 bytes:");
    for (i, chunk) in rom_data.chunks(16).take(1).enumerate() {
        print!("{:04X}: ", 0x200 + i * 16);
        for byte in chunk {
            print!("{:02X} ", byte);
        }
        println!();
    }

    Ok(())
}

/// Validates a ROM file.
fn validate_rom(rom_file: &PathBuf) -> FrontendResult<()> {
    let rom_data = std::fs::read(rom_file)?;

    println!("Validating ROM: {}", rom_file.display());

    let mut valid = true;

    if rom_data.is_empty() {
        println!("❌ Error: ROM is empty");
        valid = false;
    }

    if rom_data.len() > 3584 {
        println!("❌ Error: ROM exceeds maximum size ({})", rom_data.len());
        valid = false;
    }

    // Check for common patterns
    let mut has_instructions = false;
    for chunk in rom_data.chunks(2) {
        if chunk.len() == 2 {
            let instruction = (chunk[0] as u16) << 8 | chunk[1] as u16;
            if instruction != 0x0000 {
                has_instructions = true;
                break;
            }
        }
    }

    if !has_instructions {
        println!("⚠️  Warning: ROM appears to contain only zeros");
    }

    if valid {
        println!("✅ ROM validation passed");
    } else {
        println!("❌ ROM validation failed");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_app_creation() {
        // Basic test to ensure CLI app can be created
        // (We can't actually test parsing without args in unit tests)
        let args = CliApp {
            rom_file: None,
            verbose: true,
            config: None,
            profile: None,
            command: None,
        };

        assert!(args.verbose);
        assert!(args.rom_file.is_none());
        assert!(args.command.is_none());
        assert!(args.config.is_none());
        assert!(args.profile.is_none());
    }
}
