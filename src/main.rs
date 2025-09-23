//! Chip-8 Emulator
//!
//! A modern Rust implementation of the classic Chip-8 virtual machine.

use chip8::frontend::cli::{run_cli, CliApp};
use chip8::frontend::gui::run_gui;
use clap::Parser;
use color_eyre::eyre::Result;

fn main() -> Result<()> {
    // Install color-eyre for better error handling
    color_eyre::install()?;

    let args = CliApp::parse();

    // Set up logging if verbose
    if args.verbose {
        env_logger::init();
    }

    if let Some(rom_file) = args.rom_file.clone() {
        // Direct ROM execution defaults to GUI
        if args.command.is_none() {
            run_gui(rom_file, args.config.as_ref(), args.profile.as_ref())?;
        } else {
            // Has subcommand, use CLI
            run_cli()?;
        }
    } else {
        // No ROM file, run CLI (for subcommands or help)
        run_cli()?;
    }

    Ok(())
}
