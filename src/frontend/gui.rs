//! GUI implementation for the Chip-8 emulator.
//!
//! This module provides a graphical user interface using `pixels` and `winit`.

use log::debug;
use pixels::{Pixels, SurfaceTexture};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};
use winit::{
    event::{ElementState, Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::audio::stream::StreamConfig;
use crate::audio::AudioSystem;
use crate::error::EmulatorError;
use crate::frontend::config::{
    load_config, load_default_config, ConfigProfiles, EmulatorConfig, EnvConfig,
};
use crate::frontend::SimpleEmulator;
use crate::graphics::GraphicsDisplay;
use crate::hardware::input::Input;
use crate::hardware::input::SoftwareInput;
use crate::hardware::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::input::mapper::{KeyMapper, QwertyMapper};

/// Loads configuration from CLI arguments.
fn load_configuration(
    config_path: Option<&PathBuf>,
    profile_name: Option<&String>,
) -> Result<EmulatorConfig, EmulatorError> {
    let mut config = if let Some(path) = config_path {
        // Load from specific file
        load_config(path)?
    } else if let Some(profile) = profile_name {
        // Load from profile
        ConfigProfiles::from_name(profile)?
    } else {
        // Load default config (searches standard locations)
        load_default_config()
    };

    // Apply environment variable overrides
    EnvConfig::apply_env_overrides(&mut config);

    // Validate configuration
    config.validate()?;

    Ok(config)
}

/// Runs the GUI application.
pub fn run_gui(
    rom_file: PathBuf,
    config_path: Option<&PathBuf>,
    profile_name: Option<&String>,
) -> Result<(), EmulatorError> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Chip-8 Emulator")
        .build(&event_loop)
        .map_err(|e| {
            EmulatorError::Graphics(crate::error::GraphicsError::WindowCreationFailed(
                e.to_string(),
            ))
        })?;

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32, surface_texture)?
    };

    // Load configuration
    let config = load_configuration(config_path, profile_name)?;

    // Initialize emulator with configuration
    let mut emulator = SimpleEmulator::new_with_config(&config);

    // Initialize graphics with configuration
    let graphics_display =
        GraphicsDisplay::with_config(config.graphics.clone()).map_err(EmulatorError::Graphics)?;
    emulator.cpu_mut().set_display(Box::new(graphics_display));

    // Initialize audio with configuration
    let mut audio_system = AudioSystem::with_config(config.audio.clone(), StreamConfig::default())?;
    audio_system.initialize_with_defaults()?;
    emulator.cpu_mut().set_audio(Box::new(audio_system));

    // Initialize input (QwertyMapper doesn't need config)
    let software_input = Rc::new(RefCell::new(SoftwareInput::new()));
    let qwerty_mapper = QwertyMapper::new();
    emulator.cpu_mut().set_input(software_input.clone());

    emulator.load_rom(&rom_file)?;

    let mut last_frame_time = Instant::now();
    let mut last_timer_update = Instant::now();
    let timer_update_interval = Duration::from_secs_f64(1.0 / 60.0);

    // Store colors for rendering
    let foreground_color = config.graphics.foreground_color;
    let background_color = config.graphics.background_color;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                debug!("KeyboardInput event: {:?}", input);
                if let Some(virtual_keycode) = input.virtual_keycode {
                    debug!("VirtualKeyCode: {:?}", virtual_keycode);
                    if let Some(chip_key) = qwerty_mapper.map_virtual_keycode(virtual_keycode) {
                        debug!("Mapped ChipKey: {:?}", chip_key);
                        match input.state {
                            ElementState::Pressed => {
                                debug!("Pressing ChipKey: {:?}", chip_key);
                                software_input.borrow_mut().press_key(chip_key);
                            }
                            ElementState::Released => {
                                debug!("Releasing ChipKey: {:?}", chip_key);
                                software_input.borrow_mut().release_key(chip_key);
                            }
                        }
                    }
                }
            }
            Event::MainEventsCleared => {
                let now = Instant::now();
                let delta_time = now.duration_since(last_frame_time);
                last_frame_time = now;

                let cycles_to_execute =
                    (delta_time.as_secs_f64() * emulator.target_cps() as f64) as usize;

                // Update emulator state
                for _ in 0..cycles_to_execute {
                    if let Err(e) = emulator.step() {
                        log::error!("Emulator error: {}", e);
                        *control_flow = ControlFlow::Exit;
                        return;
                    }
                }

                // Update Chip-8 timers at 60Hz
                if now.duration_since(last_timer_update) >= timer_update_interval {
                    emulator.cpu_mut().timers_mut().update();
                    last_timer_update = now;
                }

                if let Err(e) = software_input.borrow_mut().update() {
                    log::warn!("Input update error: {}", e);
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                // Draw the screen
                let frame = emulator.cpu().get_display_buffer();
                draw_frame(
                    frame,
                    pixels.frame_mut(),
                    foreground_color,
                    background_color,
                );
                if pixels.render().is_err() {
                    *control_flow = ControlFlow::Exit;
                }
                window.request_redraw();
            }
            _ => (),
        }
    });
}

/// Draws the frame to the pixel buffer.
fn draw_frame(
    frame: &[bool],
    buffer: &mut [u8],
    foreground: crate::graphics::Color,
    background: crate::graphics::Color,
) {
    for (i, pixel) in buffer.chunks_exact_mut(4).enumerate() {
        let x = i % DISPLAY_WIDTH;
        let y = i / DISPLAY_WIDTH;

        let index = y * DISPLAY_WIDTH + x;
        let color = if frame[index] {
            [foreground.r, foreground.g, foreground.b, foreground.a]
        } else {
            [background.r, background.g, background.b, background.a]
        };

        pixel.copy_from_slice(&color);
    }
}
