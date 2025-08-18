//! Chip-8 Emulator
//!
//! A modern Rust implementation of the classic Chip-8 virtual machine.

use chip8::frontend::cli::{CliApp, run_cli};
use chip8::frontend::gui::run_gui;
use color_eyre::eyre::Result;
use clap::Parser;

fn main() -> Result<()> {
    // Install color-eyre for better error handling
    color_eyre::install()?;

    let args = CliApp::parse();

    // Set up logging if verbose
    if args.verbose {
        env_logger::init();
    }

    if args.gui {
        if let Some(rom_file) = args.rom_file {
            run_gui(rom_file)?;
        } else {
            eprintln!("Error: --gui requires a ROM file to be specified.");
            std::process::exit(1);
        }
    } else {
        // Run the CLI application
        run_cli()?;
    }
    
    Ok(())
}
