//! GUI implementation for the Chip-8 emulator.
//!
//! This module provides a graphical user interface using `pixels` and `winit`.

use std::path::PathBuf;
use std::time::{Instant, Duration};
use winit::{
    event::{Event, WindowEvent, ElementState},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use pixels::{Pixels, SurfaceTexture};
use std::rc::Rc;
use std::cell::RefCell;
use log::debug;

use crate::error::EmulatorError;
use crate::frontend::SimpleEmulator;
use crate::hardware::{DISPLAY_WIDTH, DISPLAY_HEIGHT};
use crate::graphics::GraphicsDisplay;
use crate::hardware::input::SoftwareInput;
use crate::input::mapper::{QwertyMapper, KeyMapper};
use crate::hardware::input::Input;
use crate::audio::AudioSystem;

/// Runs the GUI application.
pub fn run_gui(rom_file: PathBuf) -> Result<(), EmulatorError> {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Chip-8 Emulator")
        .build(&event_loop)
        .map_err(|e| EmulatorError::Graphics(crate::error::GraphicsError::WindowCreationFailed(e.to_string())))?;

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(DISPLAY_WIDTH as u32, DISPLAY_HEIGHT as u32, surface_texture)?
    };

    let mut emulator = SimpleEmulator::new();
    let graphics_display = GraphicsDisplay::new()
        .map_err(EmulatorError::Graphics)?;
    emulator.cpu_mut().set_display(Box::new(graphics_display));

    // Initialize audio system
    let mut audio_system = AudioSystem::new()?;
    audio_system.initialize_with_defaults()?;
    emulator.cpu_mut().set_audio(Box::new(audio_system));

    let software_input = Rc::new(RefCell::new(SoftwareInput::new()));
    let qwerty_mapper = QwertyMapper::new();
    emulator.cpu_mut().set_input(software_input.clone());

    emulator.load_rom(&rom_file)?;

    let mut last_frame_time = Instant::now();
    let mut last_timer_update = Instant::now();
    let timer_update_interval = Duration::from_secs_f64(1.0 / 60.0);

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
                            },
                            ElementState::Released => {
                                debug!("Releasing ChipKey: {:?}", chip_key);
                                software_input.borrow_mut().release_key(chip_key);
                            },
                        }
                    }
                }
            },
            Event::MainEventsCleared => {
                let now = Instant::now();
                let delta_time = now.duration_since(last_frame_time);
                last_frame_time = now;

                let cycles_to_execute = (delta_time.as_secs_f64() * emulator.target_cps() as f64) as usize;
                
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
                draw_frame(frame, pixels.frame_mut());
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
fn draw_frame(frame: &[bool], buffer: &mut [u8]) {
    for (i, pixel) in buffer.chunks_exact_mut(4).enumerate() {
        let x = i % DISPLAY_WIDTH;
        let y = i / DISPLAY_WIDTH;

        let index = y * DISPLAY_WIDTH + x;
        let color = if frame[index] {
            [0x00, 0xFF, 0x00, 0xFF] // Green
        } else {
            [0x00, 0x00, 0x00, 0xFF] // Black
        };

        pixel.copy_from_slice(&color);
    }
}