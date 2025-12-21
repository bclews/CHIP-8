//! Command-line interface for the Chip-8 emulator.
//!
//! This module provides a comprehensive CLI using clap for running
//! and configuring the Chip-8 emulator.

use clap::{Parser, Subcommand, builder::RangedU64ValueParser};
use std::path::{Path, PathBuf};

use super::{FrontendResult, SimpleEmulator};
use crate::error::EmulatorError;
use crate::graphics::{Color, GraphicsConfig, PixelRenderer};
use crate::hardware::display::SoftwareDisplay;

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

    /// Capture a screenshot after running a ROM for N cycles
    Screenshot {
        /// ROM file to run
        rom_file: PathBuf,

        /// Output PNG file path
        #[arg(short, long, default_value = "screenshot.png")]
        output: PathBuf,

        /// Number of CPU cycles to execute before capture
        #[arg(short, long, default_value_t = 1000)]
        cycles: u32,

        /// Scale factor for output image (1-20)
        #[arg(short, long, default_value_t = 10, value_parser = RangedU64ValueParser::<u32>::new().range(1..=20))]
        scale: u32,

        /// Foreground color as hex (e.g., "00FF00" for green)
        #[arg(long, default_value = "FFFFFF")]
        foreground: String,

        /// Background color as hex (e.g., "000000" for black)
        #[arg(long, default_value = "000000")]
        background: String,
    },
}

/// Runs the CLI application.
pub fn run_cli() -> FrontendResult<()> {
    let args = CliApp::parse();

    match &args.command {
        Some(Commands::Info { rom_file }) => show_rom_info(rom_file),
        Some(Commands::Validate { rom_file }) => validate_rom(rom_file),
        Some(Commands::Screenshot {
            rom_file,
            output,
            cycles,
            scale,
            foreground,
            background,
        }) => capture_screenshot(rom_file, output, *cycles, *scale, foreground, background),
        None => {
            // Show help or usage
            println!("Chip-8 Emulator");
            println!("Usage: chip8 [ROM_FILE]    - Launch GUI with ROM");
            println!("       chip8 <COMMAND>     - Run subcommand");
            println!();
            println!("Commands:");
            println!("  info       Show ROM information");
            println!("  validate   Validate a ROM file");
            println!("  screenshot Capture a screenshot after running ROM");
            println!();
            println!("Run 'chip8 --help' for more information.");
            Ok(())
        }
    }
}

/// Shows information about a ROM file.
fn show_rom_info(rom_file: &Path) -> FrontendResult<()> {
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
fn validate_rom(rom_file: &Path) -> FrontendResult<()> {
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

/// Parses a hex color string (e.g., "FF0000" or "#FF0000") into a Color.
fn parse_hex_color(hex: &str) -> Result<Color, EmulatorError> {
    use crate::error::ConfigError;

    let hex = hex.trim_start_matches('#');

    let make_error = || {
        EmulatorError::ConfigError(ConfigError::InvalidValue {
            key: "color".to_string(),
            value: hex.to_string(),
        })
    };

    if hex.len() != 6 {
        return Err(make_error());
    }

    let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| make_error())?;
    let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| make_error())?;
    let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| make_error())?;

    Ok(Color::rgb(r, g, b))
}

/// Captures a screenshot of the emulator display after running for N cycles.
fn capture_screenshot(
    rom_file: &Path,
    output: &Path,
    cycles: u32,
    scale: u32,
    foreground: &str,
    background: &str,
) -> FrontendResult<()> {
    let fg_color = parse_hex_color(foreground)?;
    let bg_color = parse_hex_color(background)?;

    println!("Loading ROM: {}", rom_file.display());

    let mut emulator = SimpleEmulator::new();
    emulator
        .cpu_mut()
        .set_display(Box::new(SoftwareDisplay::new()));
    emulator.load_rom(rom_file)?;

    println!("Running for {} cycles...", cycles);
    emulator.run_cycles(cycles)?;

    let display_buffer = emulator.get_display_buffer();

    let graphics_config = GraphicsConfig::new()
        .with_foreground_color(fg_color)
        .with_background_color(bg_color)
        .with_scale_factor(scale);

    let mut renderer = PixelRenderer::new(graphics_config).map_err(EmulatorError::Graphics)?;
    renderer
        .render(display_buffer)
        .map_err(EmulatorError::Graphics)?;

    let (width, height) = renderer.frame_size();
    image::save_buffer(
        output,
        renderer.frame_buffer(),
        width,
        height,
        image::ColorType::Rgba8,
    )
    .map_err(|e| {
        EmulatorError::IoError(std::io::Error::other(format!("Failed to save PNG: {}", e)))
    })?;

    println!("Screenshot saved to: {}", output.display());
    println!(
        "Image size: {}x{} pixels (scale factor: {})",
        width, height, scale
    );

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

    #[test]
    fn test_parse_hex_color_valid() {
        let white = parse_hex_color("FFFFFF").unwrap();
        assert_eq!(white.r, 255);
        assert_eq!(white.g, 255);
        assert_eq!(white.b, 255);

        let green = parse_hex_color("00FF00").unwrap();
        assert_eq!(green.r, 0);
        assert_eq!(green.g, 255);
        assert_eq!(green.b, 0);

        // With # prefix
        let red = parse_hex_color("#FF0000").unwrap();
        assert_eq!(red.r, 255);
        assert_eq!(red.g, 0);
        assert_eq!(red.b, 0);

        // Lowercase
        let blue = parse_hex_color("0000ff").unwrap();
        assert_eq!(blue.r, 0);
        assert_eq!(blue.g, 0);
        assert_eq!(blue.b, 255);
    }

    #[test]
    fn test_parse_hex_color_invalid() {
        // Too short
        assert!(parse_hex_color("FFF").is_err());

        // Too long
        assert!(parse_hex_color("FFFFFFFF").is_err());

        // Invalid characters
        assert!(parse_hex_color("GGGGGG").is_err());

        // Empty
        assert!(parse_hex_color("").is_err());
    }

    #[test]
    fn test_screenshot_command_parsing() {
        // Test that the command structure can be created correctly
        let args = CliApp {
            rom_file: None,
            verbose: false,
            config: None,
            profile: None,
            command: Some(Commands::Screenshot {
                rom_file: PathBuf::from("test.ch8"),
                output: PathBuf::from("output.png"),
                cycles: 2000,
                scale: 5,
                foreground: "00FF00".to_string(),
                background: "000000".to_string(),
            }),
        };

        if let Some(Commands::Screenshot { cycles, scale, .. }) = args.command {
            assert_eq!(cycles, 2000);
            assert_eq!(scale, 5);
        } else {
            panic!("Expected Screenshot command");
        }
    }
}
