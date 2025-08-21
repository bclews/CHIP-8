//! Graphics system for the Chip-8 emulator.
//!
//! This module provides the graphics implementation for rendering
//! the Chip-8 display.

pub mod renderer;

// Re-export commonly used types
pub use renderer::{Color, GraphicsConfig, PixelRenderer};

use crate::error::{EmulatorError, GraphicsError};
use crate::hardware::{Display, DisplayResult, DISPLAY_HEIGHT, DISPLAY_PIXELS, DISPLAY_WIDTH};

/// Result type for graphics operations.
pub type GraphicsResult<T> = Result<T, GraphicsError>;

/// Hardware display implementation for testing without actual graphics.
pub struct GraphicsDisplay {
    /// Internal display buffer.
    buffer: [bool; DISPLAY_PIXELS],

    /// Dirty flag for tracking changes.
    dirty: bool,

    /// Software renderer for converting to pixel buffer.
    renderer: PixelRenderer,
}

impl GraphicsDisplay {
    /// Creates a new graphics display.
    pub fn new() -> GraphicsResult<Self> {
        Ok(Self {
            buffer: [false; DISPLAY_PIXELS],
            dirty: false,
            renderer: PixelRenderer::with_defaults()?,
        })
    }

    /// Creates a graphics display with custom configuration.
    pub fn with_config(graphics_config: GraphicsConfig) -> GraphicsResult<Self> {
        Ok(Self {
            buffer: [false; DISPLAY_PIXELS],
            dirty: false,
            renderer: PixelRenderer::new(graphics_config)?,
        })
    }

    /// Renders the display buffer and returns the pixel data.
    pub fn get_pixel_data(&mut self) -> GraphicsResult<&[u8]> {
        self.renderer.render(&self.buffer)?;
        Ok(self.renderer.frame_buffer())
    }

    /// Gets the frame size.
    pub fn frame_size(&self) -> (u32, u32) {
        self.renderer.frame_size()
    }

    /// Updates the graphics configuration.
    pub fn set_graphics_config(&mut self, config: GraphicsConfig) -> GraphicsResult<()> {
        self.renderer.set_config(config)
    }

    /// Gets the current graphics configuration.
    pub fn graphics_config(&self) -> &GraphicsConfig {
        self.renderer.config()
    }
}

impl Default for GraphicsDisplay {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl Display for GraphicsDisplay {
    fn clear(&mut self) {
        self.buffer.fill(false);
        self.dirty = true;
    }

    fn draw_sprite(&mut self, x: u8, y: u8, sprite: &[u8]) -> DisplayResult<bool> {
        let mut collision = false;

        for (row, &sprite_byte) in sprite.iter().enumerate() {
            let pixel_y = (y + row as u8) % DISPLAY_HEIGHT as u8;

            for col in 0..8 {
                let pixel_x = (x + col) % DISPLAY_WIDTH as u8;
                let sprite_pixel = (sprite_byte >> (7 - col)) & 1;

                if sprite_pixel == 1 {
                    let index = pixel_y as usize * DISPLAY_WIDTH + pixel_x as usize;
                    let old_pixel = self.buffer[index];
                    self.buffer[index] ^= true;

                    // Collision if pixel was turned off
                    if old_pixel && !self.buffer[index] {
                        collision = true;
                    }
                }
            }
        }

        if !sprite.is_empty() {
            self.dirty = true;
        }

        Ok(collision)
    }

    fn get_pixel(&self, x: u8, y: u8) -> DisplayResult<bool> {
        use crate::hardware::display::DisplayError;

        if x as usize >= DISPLAY_WIDTH || y as usize >= DISPLAY_HEIGHT {
            return Err(DisplayError::InvalidCoordinates { x, y });
        }

        let index = y as usize * DISPLAY_WIDTH + x as usize;
        Ok(self.buffer[index])
    }

    fn set_pixel(&mut self, x: u8, y: u8, on: bool) -> DisplayResult<()> {
        use crate::hardware::display::DisplayError;

        if x as usize >= DISPLAY_WIDTH || y as usize >= DISPLAY_HEIGHT {
            return Err(DisplayError::InvalidCoordinates { x, y });
        }

        let index = y as usize * DISPLAY_WIDTH + x as usize;
        self.buffer[index] = on;
        self.dirty = true;
        Ok(())
    }

    fn render(&mut self) -> Result<(), EmulatorError> {
        // For this implementation, rendering just marks as clean
        // Real rendering would be handled by external graphics systems
        self.mark_clean();
        Ok(())
    }

    fn is_dirty(&self) -> bool {
        self.dirty
    }

    fn mark_clean(&mut self) {
        self.dirty = false;
    }

    fn get_buffer(&self) -> &[bool] {
        &self.buffer
    }

    fn get_buffer_mut(&mut self) -> &mut [bool] {
        self.dirty = true;
        &mut self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::Display;

    #[test]
    fn test_graphics_display_creation() {
        let display = GraphicsDisplay::new().unwrap();

        assert!(!display.is_dirty());
        assert_eq!(display.get_buffer().len(), DISPLAY_PIXELS);
        assert!(display.get_buffer().iter().all(|&pixel| !pixel));
    }

    #[test]
    fn test_graphics_display_pixel_operations() {
        let mut display = GraphicsDisplay::new().unwrap();

        // Test setting pixels
        display.set_pixel(0, 0, true).unwrap();
        display.set_pixel(63, 31, true).unwrap();

        assert!(display.get_pixel(0, 0).unwrap());
        assert!(display.get_pixel(63, 31).unwrap());
        assert!(!display.get_pixel(1, 1).unwrap());
        assert!(display.is_dirty());

        // Test invalid coordinates
        assert!(display.get_pixel(64, 0).is_err());
        assert!(display.get_pixel(0, 32).is_err());
        assert!(display.set_pixel(64, 0, true).is_err());
    }

    #[test]
    fn test_graphics_display_sprite_drawing() {
        let mut display = GraphicsDisplay::new().unwrap();

        // Draw a simple sprite (3x3 cross)
        let sprite = [
            0b01000000, // .#......
            0b11100000, // ###.....
            0b01000000, // .#......
        ];

        let collision = display.draw_sprite(0, 0, &sprite).unwrap();
        assert!(!collision); // No collision on empty display
        assert!(display.is_dirty());

        // Check pixels were set correctly
        assert!(!display.get_pixel(0, 0).unwrap());
        assert!(display.get_pixel(1, 0).unwrap());
        assert!(!display.get_pixel(2, 0).unwrap());

        assert!(display.get_pixel(0, 1).unwrap());
        assert!(display.get_pixel(1, 1).unwrap());
        assert!(display.get_pixel(2, 1).unwrap());
    }

    #[test]
    fn test_graphics_display_clear() {
        let mut display = GraphicsDisplay::new().unwrap();

        // Set some pixels
        display.set_pixel(10, 10, true).unwrap();
        display.set_pixel(20, 15, true).unwrap();

        // Clear display
        display.clear();
        assert!(display.is_dirty());
        assert!(!display.get_pixel(10, 10).unwrap());
        assert!(!display.get_pixel(20, 15).unwrap());
    }

    #[test]
    fn test_graphics_display_pixel_data() {
        let mut display = GraphicsDisplay::new().unwrap();

        // Set some pixels
        display.set_pixel(0, 0, true).unwrap();
        display.set_pixel(1, 0, true).unwrap();

        // Get pixel data
        let pixel_data = display.get_pixel_data().unwrap();
        assert!(!pixel_data.is_empty());

        // Check frame size
        let (width, height) = display.frame_size();
        assert!(width > 0 && height > 0);
    }

    #[test]
    fn test_graphics_display_config() {
        let mut display = GraphicsDisplay::new().unwrap();

        // Test setting a new config
        let new_config = GraphicsConfig::classic_green();
        display.set_graphics_config(new_config.clone()).unwrap();

        assert_eq!(display.graphics_config().foreground_color, Color::GREEN);
    }
}
