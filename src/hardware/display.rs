//! Display interface for the Chip-8 emulator.
//!
//! This module defines the display abstraction and provides implementations
//! for the 64x32 monochrome Chip-8 display system.

use crate::error::EmulatorError;
use thiserror::Error;

/// Display width in pixels.
pub const DISPLAY_WIDTH: usize = 64;

/// Display height in pixels.
pub const DISPLAY_HEIGHT: usize = 32;

/// Total number of pixels.
pub const DISPLAY_PIXELS: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT;

/// Display-specific error types.
#[derive(Error, Debug)]
pub enum DisplayError {
    #[error("Invalid pixel coordinates: ({x}, {y})")]
    InvalidCoordinates { x: u8, y: u8 },

    #[error("Display not initialized")]
    NotInitialized,

    #[error("Render error: {0}")]
    RenderError(String),

    #[error("Invalid sprite data")]
    InvalidSpriteData,
}

/// Result type for display operations.
pub type DisplayResult<T> = Result<T, DisplayError>;

/// Display abstraction for the Chip-8 screen.
///
/// The Chip-8 has a 64x32 monochrome display where each pixel can be
/// either on (white) or off (black). Graphics are drawn using sprites
/// that are XORed with the existing pixel data.
pub trait Display {
    /// Clears the entire display (sets all pixels to off).
    fn clear(&mut self);

    /// Draws a sprite at the specified coordinates.
    ///
    /// Sprites are XORed with existing pixels. Returns true if any
    /// pixels were turned off (collision detection).
    ///
    /// # Arguments
    /// * `x` - X coordinate (0-63)
    /// * `y` - Y coordinate (0-31)
    /// * `sprite` - Sprite data (each byte represents 8 horizontal pixels)
    ///
    /// # Returns
    /// True if collision occurred (any pixel turned off), false otherwise.
    fn draw_sprite(&mut self, x: u8, y: u8, sprite: &[u8]) -> DisplayResult<bool>;

    /// Gets the state of a pixel.
    ///
    /// # Arguments
    /// * `x` - X coordinate (0-63)
    /// * `y` - Y coordinate (0-31)
    ///
    /// # Returns
    /// True if pixel is on, false if off.
    fn get_pixel(&self, x: u8, y: u8) -> DisplayResult<bool>;

    /// Sets the state of a pixel.
    ///
    /// # Arguments
    /// * `x` - X coordinate (0-63)
    /// * `y` - Y coordinate (0-31)
    /// * `on` - Whether the pixel should be on
    fn set_pixel(&mut self, x: u8, y: u8, on: bool) -> DisplayResult<()>;

    /// Renders the display to the screen.
    ///
    /// This should be called once per frame to update the visual output.
    fn render(&mut self) -> Result<(), EmulatorError>;

    /// Checks if the display has been modified since last render.
    fn is_dirty(&self) -> bool;

    /// Marks the display as clean (typically called after rendering).
    fn mark_clean(&mut self);

    /// Gets the raw pixel buffer for advanced operations.
    fn get_buffer(&self) -> &[bool];

    /// Gets a mutable reference to the pixel buffer.
    fn get_buffer_mut(&mut self) -> &mut [bool];
}

/// A basic software display implementation.
///
/// This implementation maintains the display state in memory and
/// provides the core Chip-8 display functionality without any
/// actual rendering backend.
pub struct SoftwareDisplay {
    /// Pixel buffer (true = on, false = off).
    pixels: [bool; DISPLAY_PIXELS],

    /// Whether the display has been modified.
    dirty: bool,
}

impl SoftwareDisplay {
    /// Creates a new software display.
    pub fn new() -> Self {
        Self {
            pixels: [false; DISPLAY_PIXELS],
            dirty: false,
        }
    }

    /// Converts coordinates to buffer index.
    fn coord_to_index(&self, x: u8, y: u8) -> DisplayResult<usize> {
        if x as usize >= DISPLAY_WIDTH || y as usize >= DISPLAY_HEIGHT {
            return Err(DisplayError::InvalidCoordinates { x, y });
        }
        Ok(y as usize * DISPLAY_WIDTH + x as usize)
    }

}

impl Default for SoftwareDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for SoftwareDisplay {
    fn clear(&mut self) {
        self.pixels.fill(false);
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
                    let index = self.coord_to_index(pixel_x, pixel_y)?;
                    let old_pixel = self.pixels[index];
                    self.pixels[index] ^= true;

                    // Collision if pixel was turned off
                    if old_pixel && !self.pixels[index] {
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
        let index = self.coord_to_index(x, y)?;
        Ok(self.pixels[index])
    }

    fn set_pixel(&mut self, x: u8, y: u8, on: bool) -> DisplayResult<()> {
        let index = self.coord_to_index(x, y)?;
        self.pixels[index] = on;
        self.dirty = true;
        Ok(())
    }

    fn render(&mut self) -> Result<(), EmulatorError> {
        // Software display doesn't actually render anything
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
        &self.pixels
    }

    fn get_buffer_mut(&mut self) -> &mut [bool] {
        self.dirty = true;
        &mut self.pixels
    }
}

/// A null display implementation for testing and headless operation.
pub struct NullDisplay {
    display: SoftwareDisplay,
}

impl NullDisplay {
    pub fn new() -> Self {
        Self {
            display: SoftwareDisplay::new(),
        }
    }
}

impl Default for NullDisplay {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for NullDisplay {
    fn clear(&mut self) {
        self.display.clear();
    }

    fn draw_sprite(&mut self, x: u8, y: u8, sprite: &[u8]) -> DisplayResult<bool> {
        self.display.draw_sprite(x, y, sprite)
    }

    fn get_pixel(&self, x: u8, y: u8) -> DisplayResult<bool> {
        self.display.get_pixel(x, y)
    }

    fn set_pixel(&mut self, x: u8, y: u8, on: bool) -> DisplayResult<()> {
        self.display.set_pixel(x, y, on)
    }

    fn render(&mut self) -> Result<(), EmulatorError> {
        // Null display doesn't render anything
        self.mark_clean();
        Ok(())
    }

    fn is_dirty(&self) -> bool {
        self.display.is_dirty()
    }

    fn mark_clean(&mut self) {
        self.display.mark_clean();
    }

    fn get_buffer(&self) -> &[bool] {
        self.display.get_buffer()
    }

    fn get_buffer_mut(&mut self) -> &mut [bool] {
        self.display.get_buffer_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_creation() {
        let display = SoftwareDisplay::new();

        assert!(!display.is_dirty());
        assert_eq!(display.get_buffer().len(), DISPLAY_PIXELS);
        assert!(display.get_buffer().iter().all(|&pixel| !pixel));
    }

    #[test]
    fn test_clear_display() {
        let mut display = SoftwareDisplay::new();

        // Set some pixels
        display.set_pixel(10, 10, true).unwrap();
        display.set_pixel(20, 15, true).unwrap();
        assert!(display.is_dirty());

        // Clear display
        display.clear();
        assert!(display.is_dirty());
        assert!(!display.get_pixel(10, 10).unwrap());
        assert!(!display.get_pixel(20, 15).unwrap());
    }

    #[test]
    fn test_pixel_operations() {
        let mut display = SoftwareDisplay::new();

        // Test setting pixels
        display.set_pixel(0, 0, true).unwrap();
        display.set_pixel(63, 31, true).unwrap();

        assert!(display.get_pixel(0, 0).unwrap());
        assert!(display.get_pixel(63, 31).unwrap());
        assert!(!display.get_pixel(1, 1).unwrap());

        // Test invalid coordinates
        assert!(display.get_pixel(64, 0).is_err());
        assert!(display.get_pixel(0, 32).is_err());
        assert!(display.set_pixel(64, 0, true).is_err());
    }

    #[test]
    fn test_sprite_drawing() {
        let mut display = SoftwareDisplay::new();

        // Draw a simple sprite (3x3 cross)
        let sprite = [
            0b01000000, // .#......
            0b11100000, // ###.....
            0b01000000, // .#......
        ];

        let collision = display.draw_sprite(0, 0, &sprite).unwrap();
        assert!(!collision); // No collision on empty display

        // Check pixels were set correctly
        assert!(!display.get_pixel(0, 0).unwrap());
        assert!(display.get_pixel(1, 0).unwrap());
        assert!(!display.get_pixel(2, 0).unwrap());

        assert!(display.get_pixel(0, 1).unwrap());
        assert!(display.get_pixel(1, 1).unwrap());
        assert!(display.get_pixel(2, 1).unwrap());

        assert!(!display.get_pixel(0, 2).unwrap());
        assert!(display.get_pixel(1, 2).unwrap());
        assert!(!display.get_pixel(2, 2).unwrap());
    }

    #[test]
    fn test_sprite_collision() {
        let mut display = SoftwareDisplay::new();

        // Set a pixel
        display.set_pixel(1, 1, true).unwrap();

        // Draw sprite that overlaps with the set pixel
        let sprite = [0b01000000]; // .#......
        let collision = display.draw_sprite(0, 1, &sprite).unwrap();

        assert!(collision); // Should detect collision
        assert!(!display.get_pixel(1, 1).unwrap()); // Pixel should be turned off
    }

    #[test]
    fn test_sprite_wrapping() {
        let mut display = SoftwareDisplay::new();

        // Draw sprite at edge to test wrapping
        let sprite = [0b11000000]; // ##......
        display.draw_sprite(63, 0, &sprite).unwrap();

        // Should wrap around
        assert!(display.get_pixel(63, 0).unwrap());
        assert!(display.get_pixel(0, 0).unwrap());
    }

    #[test]
    fn test_dirty_flag() {
        let mut display = SoftwareDisplay::new();

        assert!(!display.is_dirty());

        display.set_pixel(0, 0, true).unwrap();
        assert!(display.is_dirty());

        display.mark_clean();
        assert!(!display.is_dirty());

        display.clear();
        assert!(display.is_dirty());
    }

    #[test]
    fn test_null_display() {
        let mut display = NullDisplay::new();

        display.clear();
        display.set_pixel(10, 10, true).unwrap();
        assert!(display.get_pixel(10, 10).unwrap());

        let sprite = [0b11110000];
        let collision = display.draw_sprite(0, 0, &sprite).unwrap();
        assert!(!collision);

        display.render().unwrap();
    }
}

