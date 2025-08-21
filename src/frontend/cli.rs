//! Command-line interface for the Chip-8 emulator.
//!
//! This module provides a comprehensive CLI using clap for running
//! and configuring the Chip-8 emulator.

use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

use super::{EmulatorConfig, FrontendResult, SimpleEmulator};

/// Debug configuration for CLI.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct DebugConfig {
    /// Enable debug mode.
    pub enabled: bool,

    /// Break execution on errors.
    pub break_on_error: bool,

    /// Log CPU instructions.
    pub log_instructions: bool,
}

/// Modern Chip-8 emulator written in Rust.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct CliApp {
    /// ROM file to load and run
    #[arg(value_name = "ROM_FILE")]
    pub rom_file: Option<PathBuf>,

    /// Display scale factor (1-20)
    #[arg(short, long, default_value_t = 10)]
    pub scale: u32,

    /// Audio volume (0.0-1.0)
    #[arg(short, long, default_value_t = 0.3)]
    pub volume: f32,

    /// CPU cycles per second (100-2000)
    #[arg(short, long, default_value_t = 700)]
    pub cps: u32,

    /// Enable debug mode
    #[arg(short, long)]
    pub debug: bool,

    /// Disable audio
    #[arg(long)]
    pub mute: bool,

    /// Enable verbose output
    #[arg(long)]
    pub verbose: bool,

    /// Enable graphical user interface
    #[arg(long)]
    pub gui: bool,

    /// Use classic CHIP-8 compatibility mode
    #[arg(long)]
    pub classic: bool,

    /// Enable memory wraparound (classic behavior)
    #[arg(long)]
    pub wraparound: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

/// Available CLI commands.
#[derive(Subcommand)]
pub enum Commands {
    /// Run a ROM file
    Run {
        /// ROM file to run
        rom_file: PathBuf,

        /// Number of cycles to execute
        #[arg(short, long, default_value_t = 1000)]
        cycles: u32,

        /// Enable debug output
        #[arg(short, long)]
        debug: bool,
    },

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

    /// List available options
    List {
        /// What to list
        #[arg(value_enum)]
        item: ListItem,
    },

    /// Benchmark emulator performance
    Benchmark {
        /// ROM file for benchmarking
        rom_file: Option<PathBuf>,

        /// Duration in seconds
        #[arg(short, long, default_value_t = 10)]
        duration: u32,
    },
}

/// Items that can be listed.
#[derive(ValueEnum, Clone)]
pub enum ListItem {
    /// List key mappings
    Keys,
    /// List audio options
    Audio,
    /// List available commands
    Commands,
}

/// Creates a configuration from CLI arguments.
fn create_config_from_args(args: &CliApp) -> EmulatorConfig {
    if args.classic {
        // Use classic preset as base
        let mut config = EmulatorConfig::classic();

        // Apply any CLI overrides
        if args.wraparound {
            config.behavior.memory_wraparound = true;
        }
        config.behavior.cpu_speed = args.cps;
        config.audio.volume = args.volume;
        config.graphics.scale_factor = args.scale;

        config
    } else {
        // Use modern preset as base
        let mut config = EmulatorConfig::modern();

        // Apply CLI overrides
        if args.wraparound {
            config.behavior.memory_wraparound = true;
        }
        config.behavior.cpu_speed = args.cps;
        config.audio.volume = args.volume;
        config.graphics.scale_factor = args.scale;

        config
    }
}

/// Runs the CLI application.
pub fn run_cli() -> FrontendResult<()> {
    let args = CliApp::parse();

    // Set up logging if verbose
    if args.verbose {
        env_logger::init();
    }

    match &args.command {
        Some(Commands::Run {
            rom_file,
            cycles,
            debug,
        }) => run_rom_command_with_args(rom_file, *cycles, *debug || args.debug, &args),
        Some(Commands::Info { rom_file }) => show_rom_info(rom_file),
        Some(Commands::Validate { rom_file }) => validate_rom(rom_file),
        Some(Commands::List { item }) => list_items(item),
        Some(Commands::Benchmark { rom_file, duration }) => {
            run_benchmark(rom_file.as_ref(), *duration)
        }
        None => {
            // Run ROM directly if provided
            if let Some(rom_file) = &args.rom_file {
                run_rom_command_with_args(rom_file, 1000, args.debug, &args)
            } else {
                // Show help if no ROM file provided
                println!("Chip-8 Emulator");
                println!("Usage: chip8 [OPTIONS] [ROM_FILE]");
                println!("       chip8 <COMMAND>");
                println!();
                println!("Commands:");
                println!("  run        Run a ROM file");
                println!("  info       Show ROM information");
                println!("  validate   Validate a ROM file");
                println!("  list       List available options");
                println!("  benchmark  Run performance benchmark");
                println!();
                println!("Run 'chip8 --help' for more information.");
                Ok(())
            }
        }
    }
}

/// Runs a ROM file with the emulator using CLI arguments.
fn run_rom_command_with_args(
    rom_file: &PathBuf,
    cycles: u32,
    debug: bool,
    args: &CliApp,
) -> FrontendResult<()> {
    println!("🎮 Chip-8 Emulator Starting...");
    println!("Loading ROM: {}", rom_file.display());

    // Create configuration based on CLI arguments
    let config = create_config_from_args(args);
    if args.classic {
        println!("🕹️  Running in classic CHIP-8 compatibility mode");
    }
    if args.wraparound {
        println!("🔄 Memory wraparound enabled");
    }

    let mut emulator = SimpleEmulator::new_with_config(&config);

    emulator.load_rom(rom_file)?;

    println!("✅ ROM loaded successfully!");

    if debug {
        println!("\n🐛 Initial CPU state: {:?}", emulator.cpu().get_state());
    }

    println!("\n🚀 Executing {} cycles...", cycles);

    let mut instruction_count = 0;
    let _draw_instructions = 0;
    let mut sound_instructions = 0;
    let mut jump_instructions = 0;

    for i in 0..cycles {
        let prev_state = emulator.cpu().get_state();

        match emulator.step() {
            Ok(()) => {
                let curr_state = emulator.cpu().get_state();
                instruction_count += 1;

                // Analyze what instruction was executed based on PC change
                let pc_diff = curr_state.pc.wrapping_sub(prev_state.pc);

                // Detect interesting instructions
                if curr_state.i != prev_state.i {
                    // I register changed, might be font/sprite related
                }
                if curr_state.sound_timer > prev_state.sound_timer {
                    sound_instructions += 1;
                    if !debug {
                        log::info!(
                            "🔊 Sound timer set to {} at cycle {}",
                            curr_state.sound_timer,
                            i + 1
                        );
                    }
                }
                if pc_diff != 2 {
                    jump_instructions += 1;
                    if !debug && jump_instructions <= 3 {
                        log::info!(
                            "🔄 Jump/Call instruction at cycle {}: PC 0x{:04X} -> 0x{:04X}",
                            i + 1,
                            prev_state.pc,
                            curr_state.pc
                        );
                    }
                }

                if debug && i < 10 {
                    log::debug!(
                        "Cycle {}: PC=0x{:04X}, I=0x{:04X}, SP={}",
                        i + 1,
                        curr_state.pc,
                        curr_state.i,
                        curr_state.sp
                    );
                }

                // Show progress every 100 cycles
                if !debug && i % 100 == 0 && i > 0 {
                    println!(
                        "⏱️  Progress: {}/{} cycles ({:.1}%)",
                        i,
                        cycles,
                        (i as f32 / cycles as f32) * 100.0
                    );
                }
            }
            Err(e) => {
                println!("⚠️  Execution stopped at cycle {}: {}", i + 1, e);
                break;
            }
        }
    }

    let final_state = emulator.cpu().get_state();

    println!("\n📊 Execution Summary:");
    println!("   🔢 Total cycles executed: {}", instruction_count);
    println!("   🔄 Jump/Call instructions: {}", jump_instructions);
    println!("   🔊 Sound instructions: {}", sound_instructions);
    println!("   📍 Final PC: 0x{:04X}", final_state.pc);
    println!("   📋 Final I register: 0x{:04X}", final_state.i);
    println!("   📚 Stack depth: {}", final_state.sp);

    if final_state.sound_timer > 0 {
        println!(
            "   🎵 Sound timer active: {} frames remaining",
            final_state.sound_timer
        );
    }
    if final_state.delay_timer > 0 {
        println!(
            "   ⏰ Delay timer active: {} frames remaining",
            final_state.delay_timer
        );
    }

    // Check if any registers were modified
    let modified_regs: Vec<usize> = final_state
        .v
        .iter()
        .enumerate()
        .filter(|(_, &val)| val != 0)
        .map(|(i, _)| i)
        .collect();

    if !modified_regs.is_empty() {
        println!("   📝 Modified registers: {:?}", modified_regs);
        for &reg in &modified_regs {
            println!(
                "      V{:X} = 0x{:02X} ({})",
                reg, final_state.v[reg], final_state.v[reg]
            );
        }
    }

    if debug {
        println!("\n🐛 Final CPU state: {:?}", final_state);
    }

    println!("\n✅ Execution completed successfully!");
    println!("💡 Try running with --debug for detailed instruction tracing");
    println!(
        "💡 Try: cargo run -- info {} for ROM analysis",
        rom_file.display()
    );

    Ok(())
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

/// Lists available items.
fn list_items(item: &ListItem) -> FrontendResult<()> {
    match item {
        ListItem::Keys => {
            println!("Key mappings (QWERTY keyboard to Chip-8 keypad):");
            println!("  Chip-8:    QWERTY:");
            println!("  1 2 3 C    1 2 3 4");
            println!("  4 5 6 D    Q W E R");
            println!("  7 8 9 E    A S D F");
            println!("  A 0 B F    Z X C V");
        }
        ListItem::Audio => {
            println!("Audio options:");
            println!("  --volume     Volume level (0.0-1.0)");
            println!("  --mute       Disable audio completely");
        }
        ListItem::Commands => {
            println!("Available commands:");
            println!("  run          Run a ROM file");
            println!("  info         Display ROM information");
            println!("  validate     Validate a ROM file");
            println!("  list         List available options");
            println!("  benchmark    Run performance benchmark");
        }
    }

    Ok(())
}

/// Runs a performance benchmark.
fn run_benchmark(rom_file: Option<&PathBuf>, duration: u32) -> FrontendResult<()> {
    println!("Running benchmark for {} seconds...", duration);

    let mut emulator = SimpleEmulator::new();

    if let Some(rom) = rom_file {
        emulator.load_rom(rom)?;
        println!("Loaded ROM: {}", rom.display());
    } else {
        println!("Running without ROM (CPU benchmark)");
    }

    let start_time = std::time::Instant::now();
    let mut cycles = 0u64;

    while start_time.elapsed().as_secs() < duration as u64 {
        if emulator.step().is_ok() {
            cycles += 1;
        }
    }

    let elapsed = start_time.elapsed();
    let cps = cycles as f64 / elapsed.as_secs_f64();

    println!("Benchmark results:");
    println!("  Duration: {:.2}s", elapsed.as_secs_f64());
    println!("  Cycles: {}", cycles);
    println!("  Average CPS: {:.0}", cps);
    println!("  Target CPS: {}", emulator.target_cps());

    if cps >= emulator.target_cps() as f64 {
        println!("✅ Performance is adequate");
    } else {
        println!("⚠️  Performance may be inadequate for real-time emulation");
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
            scale: 8,
            volume: 0.5,
            cps: 1000,
            debug: true,
            mute: false,
            verbose: false,
            gui: false,
            classic: false,
            wraparound: false,
            command: None,
        };

        assert_eq!(args.scale, 8);
        assert_eq!(args.volume, 0.5);
        assert_eq!(args.cps, 1000);
        assert!(args.debug);
    }
}
