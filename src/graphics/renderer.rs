//! Pixel renderer for the Chip-8 emulator.
//!
//! This module handles the rendering of the Chip-8 display buffer
//! to a pixel buffer that can be displayed on screen.

use super::GraphicsResult;
use crate::error::GraphicsError;
use crate::hardware::{DISPLAY_HEIGHT, DISPLAY_WIDTH};

/// Color representation for pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Creates a new color.
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Creates a color from RGB values (alpha = 255).
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    /// Creates a grayscale color.
    pub const fn gray(value: u8) -> Self {
        Self::rgb(value, value, value)
    }

    /// White color.
    pub const WHITE: Color = Color::rgb(255, 255, 255);

    /// Black color.
    pub const BLACK: Color = Color::rgb(0, 0, 0);

    /// Green color (classic Chip-8).
    pub const GREEN: Color = Color::rgb(0, 255, 0);

    /// Amber color (classic terminal).
    pub const AMBER: Color = Color::rgb(255, 191, 0);

    /// Converts to RGBA array.
    pub const fn to_rgba(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

/// Graphics configuration for rendering.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphicsConfig {
    /// Color for "on" pixels.
    pub foreground_color: Color,

    /// Color for "off" pixels.
    pub background_color: Color,

    /// Pixel scale factor.
    pub scale_factor: u32,

    /// Whether to use smooth scaling.
    pub smooth_scaling: bool,

    /// Whether to maintain aspect ratio.
    pub maintain_aspect_ratio: bool,
}

impl Default for GraphicsConfig {
    fn default() -> Self {
        Self {
            foreground_color: Color::WHITE,
            background_color: Color::BLACK,
            scale_factor: 10,
            smooth_scaling: false,
            maintain_aspect_ratio: true,
        }
    }
}

impl GraphicsConfig {
    /// Creates a new graphics configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the foreground color.
    pub fn with_foreground_color(mut self, color: Color) -> Self {
        self.foreground_color = color;
        self
    }

    /// Sets the background color.
    pub fn with_background_color(mut self, color: Color) -> Self {
        self.background_color = color;
        self
    }

    /// Sets the scale factor.
    pub fn with_scale_factor(mut self, scale: u32) -> Self {
        self.scale_factor = scale;
        self
    }

    /// Sets smooth scaling.
    pub fn with_smooth_scaling(mut self, smooth: bool) -> Self {
        self.smooth_scaling = smooth;
        self
    }

    /// Sets aspect ratio maintenance.
    pub fn with_maintain_aspect_ratio(mut self, maintain: bool) -> Self {
        self.maintain_aspect_ratio = maintain;
        self
    }

    /// Creates a classic green monochrome configuration.
    pub fn classic_green() -> Self {
        Self::new()
            .with_foreground_color(Color::GREEN)
            .with_background_color(Color::BLACK)
    }

    /// Creates a classic amber terminal configuration.
    pub fn classic_amber() -> Self {
        Self::new()
            .with_foreground_color(Color::AMBER)
            .with_background_color(Color::BLACK)
    }

    /// Creates a high-contrast configuration.
    pub fn high_contrast() -> Self {
        Self::new()
            .with_foreground_color(Color::WHITE)
            .with_background_color(Color::BLACK)
    }
}

/// Pixel renderer for converting Chip-8 display to RGBA pixels.
pub struct PixelRenderer {
    /// Configuration for rendering.
    config: GraphicsConfig,

    /// Internal frame buffer (RGBA format).
    frame_buffer: Vec<u8>,

    /// Width of the frame buffer.
    frame_width: u32,

    /// Height of the frame buffer.
    frame_height: u32,
}

impl PixelRenderer {
    /// Creates a new pixel renderer.
    pub fn new(config: GraphicsConfig) -> GraphicsResult<Self> {
        let frame_width = DISPLAY_WIDTH as u32 * config.scale_factor;
        let frame_height = DISPLAY_HEIGHT as u32 * config.scale_factor;
        let frame_buffer = vec![0u8; (frame_width * frame_height * 4) as usize];

        Ok(Self {
            config,
            frame_buffer,
            frame_width,
            frame_height,
        })
    }

    /// Creates a renderer with default configuration.
    pub fn with_defaults() -> GraphicsResult<Self> {
        Self::new(GraphicsConfig::default())
    }

    /// Renders a display buffer to the frame buffer.
    pub fn render(&mut self, display_buffer: &[bool]) -> GraphicsResult<()> {
        if display_buffer.len() != DISPLAY_WIDTH * DISPLAY_HEIGHT {
            return Err(GraphicsError::InvalidBufferSize {
                expected: DISPLAY_WIDTH * DISPLAY_HEIGHT,
                actual: display_buffer.len(),
            });
        }

        let fg_color = self.config.foreground_color.to_rgba();
        let bg_color = self.config.background_color.to_rgba();
        let scale = self.config.scale_factor as usize;

        // Clear the frame buffer
        self.frame_buffer.fill(0);

        for (i, &pixel_on) in display_buffer.iter().enumerate() {
            let src_x = i % DISPLAY_WIDTH;
            let src_y = i / DISPLAY_WIDTH;

            let color = if pixel_on { fg_color } else { bg_color };

            // Scale the pixel
            for dy in 0..scale {
                for dx in 0..scale {
                    let dst_x = src_x * scale + dx;
                    let dst_y = src_y * scale + dy;

                    if dst_x < self.frame_width as usize && dst_y < self.frame_height as usize {
                        let dst_index = (dst_y * self.frame_width as usize + dst_x) * 4;

                        if dst_index + 3 < self.frame_buffer.len() {
                            self.frame_buffer[dst_index..dst_index + 4].copy_from_slice(&color);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Copies the frame buffer to an external buffer.
    pub fn copy_to_frame(&self, dest: &mut [u8]) -> GraphicsResult<()> {
        if dest.len() < self.frame_buffer.len() {
            return Err(GraphicsError::InvalidBufferSize {
                expected: self.frame_buffer.len(),
                actual: dest.len(),
            });
        }

        dest[..self.frame_buffer.len()].copy_from_slice(&self.frame_buffer);
        Ok(())
    }

    /// Gets the frame buffer.
    pub fn frame_buffer(&self) -> &[u8] {
        &self.frame_buffer
    }

    /// Gets the frame dimensions.
    pub fn frame_size(&self) -> (u32, u32) {
        (self.frame_width, self.frame_height)
    }

    /// Updates the graphics configuration.
    pub fn set_config(&mut self, config: GraphicsConfig) -> GraphicsResult<()> {
        // Check if we need to resize the frame buffer
        let new_width = DISPLAY_WIDTH as u32 * config.scale_factor;
        let new_height = DISPLAY_HEIGHT as u32 * config.scale_factor;

        if new_width != self.frame_width || new_height != self.frame_height {
            self.frame_width = new_width;
            self.frame_height = new_height;
            self.frame_buffer = vec![0u8; (new_width * new_height * 4) as usize];
        }

        self.config = config;
        Ok(())
    }

    /// Gets the current configuration.
    pub fn config(&self) -> &GraphicsConfig {
        &self.config
    }
}

/// Software-only renderer for testing and headless operation.
pub struct SoftwareRenderer {
    /// Internal pixel renderer.
    renderer: PixelRenderer,
}

impl SoftwareRenderer {
    /// Creates a new software renderer.
    pub fn new(config: GraphicsConfig) -> GraphicsResult<Self> {
        Ok(Self {
            renderer: PixelRenderer::new(config)?,
        })
    }

    /// Renders a display buffer and returns the frame buffer.
    pub fn render(&mut self, display_buffer: &[bool]) -> GraphicsResult<&[u8]> {
        self.renderer.render(display_buffer)?;
        Ok(self.renderer.frame_buffer())
    }

    /// Gets the frame size.
    pub fn frame_size(&self) -> (u32, u32) {
        self.renderer.frame_size()
    }

    /// Updates the configuration.
    pub fn set_config(&mut self, config: GraphicsConfig) -> GraphicsResult<()> {
        self.renderer.set_config(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_creation() {
        let color = Color::new(255, 128, 64, 255);
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
        assert_eq!(color.a, 255);
    }

    #[test]
    fn test_color_constants() {
        assert_eq!(Color::WHITE, Color::rgb(255, 255, 255));
        assert_eq!(Color::BLACK, Color::rgb(0, 0, 0));
        assert_eq!(Color::GREEN, Color::rgb(0, 255, 0));
    }

    #[test]
    fn test_color_conversions() {
        let color = Color::rgb(255, 128, 64);
        let rgba = color.to_rgba();
        assert_eq!(rgba, [255, 128, 64, 255]);
    }

    #[test]
    fn test_graphics_config_creation() {
        let config = GraphicsConfig::new();
        assert_eq!(config.foreground_color, Color::WHITE);
        assert_eq!(config.background_color, Color::BLACK);
        assert_eq!(config.scale_factor, 10);
    }

    #[test]
    fn test_graphics_config_builder() {
        let config = GraphicsConfig::new()
            .with_foreground_color(Color::GREEN)
            .with_background_color(Color::BLACK)
            .with_scale_factor(8);

        assert_eq!(config.foreground_color, Color::GREEN);
        assert_eq!(config.scale_factor, 8);
    }

    #[test]
    fn test_graphics_config_presets() {
        let green_config = GraphicsConfig::classic_green();
        assert_eq!(green_config.foreground_color, Color::GREEN);

        let amber_config = GraphicsConfig::classic_amber();
        assert_eq!(amber_config.foreground_color, Color::AMBER);
    }

    #[test]
    fn test_pixel_renderer_creation() {
        let config = GraphicsConfig::new();
        let renderer = PixelRenderer::new(config);

        assert!(renderer.is_ok());
        let r = renderer.unwrap();
        assert_eq!(r.frame_size(), (640, 320)); // 64*10, 32*10
    }

    #[test]
    fn test_pixel_renderer_render() {
        let config = GraphicsConfig::new().with_scale_factor(2);
        let mut renderer = PixelRenderer::new(config).unwrap();

        // Create a simple test pattern
        let mut display_buffer = vec![false; DISPLAY_WIDTH * DISPLAY_HEIGHT];
        display_buffer[0] = true; // Top-left pixel
        display_buffer[DISPLAY_WIDTH - 1] = true; // Top-right pixel

        let result = renderer.render(&display_buffer);
        assert!(result.is_ok());

        let frame = renderer.frame_buffer();
        assert!(!frame.is_empty());

        // Check that the frame buffer has the correct size
        let expected_size = (DISPLAY_WIDTH as u32 * 2) * (DISPLAY_HEIGHT as u32 * 2) * 4;
        assert_eq!(frame.len(), expected_size as usize);
    }

    #[test]
    fn test_pixel_renderer_invalid_buffer() {
        let config = GraphicsConfig::new();
        let mut renderer = PixelRenderer::new(config).unwrap();

        // Test with wrong buffer size
        let wrong_buffer = vec![false; 10];
        let result = renderer.render(&wrong_buffer);
        assert!(result.is_err());
    }

    #[test]
    fn test_software_renderer() {
        let config = GraphicsConfig::new().with_scale_factor(4);
        let mut renderer = SoftwareRenderer::new(config).unwrap();

        let display_buffer = vec![true; DISPLAY_WIDTH * DISPLAY_HEIGHT];
        let frame = renderer.render(&display_buffer);

        assert!(frame.is_ok());
        let frame_data = frame.unwrap();
        assert!(!frame_data.is_empty());
    }

    #[test]
    fn test_renderer_config_update() {
        let initial_config = GraphicsConfig::new().with_scale_factor(2);
        let mut renderer = PixelRenderer::new(initial_config).unwrap();

        assert_eq!(renderer.frame_size(), (128, 64)); // 64*2, 32*2

        let new_config = GraphicsConfig::new().with_scale_factor(4);
        let result = renderer.set_config(new_config);

        assert!(result.is_ok());
        assert_eq!(renderer.frame_size(), (256, 128)); // 64*4, 32*4
    }
}
