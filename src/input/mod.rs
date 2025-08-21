//! Input handling system for the Chip-8 emulator.
//!
//! This module provides real keyboard input handling for the Chip-8
//! hexadecimal keypad using various input backends.

pub mod keyboard;
pub mod mapper;

// Re-export commonly used types
pub use keyboard::{KeyboardConfig, KeyboardInput};
pub use mapper::{CustomMapper, KeyMapper, QwertyMapper};

use crate::error::InputError;
use crate::hardware::{ChipKey, Input, InputResult};

/// Result type for input operations.
pub type InputSystemResult<T> = Result<T, InputError>;

/// Complete input system that implements the Input trait with real keyboard handling.
pub struct InputSystem {
    /// The keyboard input handler.
    keyboard: KeyboardInput,

    /// Key mapper for translating physical keys to Chip-8 keys.
    mapper: Box<dyn KeyMapper>,

    /// Current state of pressed keys.
    pressed_keys: std::collections::HashSet<ChipKey>,

    /// Keys pressed this frame.
    keys_pressed_this_frame: std::collections::HashSet<ChipKey>,

    /// Keys released this frame.
    keys_released_this_frame: std::collections::HashSet<ChipKey>,

    /// Whether to capture keyboard events.
    capture_enabled: bool,
}

impl InputSystem {
    /// Creates a new input system.
    pub fn new() -> InputSystemResult<Self> {
        let keyboard = KeyboardInput::new(KeyboardConfig::default())?;
        let mapper = Box::new(QwertyMapper::new());

        Ok(Self {
            keyboard,
            mapper,
            pressed_keys: std::collections::HashSet::new(),
            keys_pressed_this_frame: std::collections::HashSet::new(),
            keys_released_this_frame: std::collections::HashSet::new(),
            capture_enabled: true,
        })
    }

    /// Creates an input system with custom configuration.
    pub fn with_config(
        keyboard_config: KeyboardConfig,
        mapper: Box<dyn KeyMapper>,
    ) -> InputSystemResult<Self> {
        let keyboard = KeyboardInput::new(keyboard_config)?;

        Ok(Self {
            keyboard,
            mapper,
            pressed_keys: std::collections::HashSet::new(),
            keys_pressed_this_frame: std::collections::HashSet::new(),
            keys_released_this_frame: std::collections::HashSet::new(),
            capture_enabled: true,
        })
    }

    /// Sets the key mapper.
    pub fn set_mapper(&mut self, mapper: Box<dyn KeyMapper>) {
        self.mapper = mapper;
    }

    /// Gets a reference to the current key mapper.
    pub fn mapper(&self) -> &dyn KeyMapper {
        self.mapper.as_ref()
    }

    /// Enables or disables keyboard capture.
    pub fn set_capture_enabled(&mut self, enabled: bool) {
        self.capture_enabled = enabled;
    }

    /// Checks if keyboard capture is enabled.
    pub fn is_capture_enabled(&self) -> bool {
        self.capture_enabled
    }

    /// Processes a keyboard event (for integration with window systems).
    pub fn process_keyboard_event(
        &mut self,
        event: &keyboard::KeyboardEvent,
    ) -> InputSystemResult<()> {
        if !self.capture_enabled {
            return Ok(());
        }

        if let Some(chip_key) = self.mapper.map_key_event(event) {
            match event.state {
                keyboard::KeyState::Pressed => {
                    if !self.pressed_keys.contains(&chip_key) {
                        self.keys_pressed_this_frame.insert(chip_key);
                    }
                    self.pressed_keys.insert(chip_key);
                }
                keyboard::KeyState::Released => {
                    if self.pressed_keys.contains(&chip_key) {
                        self.keys_released_this_frame.insert(chip_key);
                    }
                    self.pressed_keys.remove(&chip_key);
                }
            }
        }

        Ok(())
    }

    /// Simulates a key press (for testing or external control).
    pub fn simulate_key_press(&mut self, key: ChipKey) {
        if !self.pressed_keys.contains(&key) {
            self.keys_pressed_this_frame.insert(key);
        }
        self.pressed_keys.insert(key);
    }

    /// Simulates a key release (for testing or external control).
    pub fn simulate_key_release(&mut self, key: ChipKey) {
        if self.pressed_keys.contains(&key) {
            self.keys_released_this_frame.insert(key);
        }
        self.pressed_keys.remove(&key);
    }

    /// Gets all keys pressed this frame.
    pub fn get_keys_pressed_this_frame(&self) -> Vec<ChipKey> {
        self.keys_pressed_this_frame.iter().copied().collect()
    }

    /// Gets all keys released this frame.
    pub fn get_keys_released_this_frame(&self) -> Vec<ChipKey> {
        self.keys_released_this_frame.iter().copied().collect()
    }

    /// Checks if a key was just pressed this frame.
    pub fn was_key_just_pressed(&self, key: ChipKey) -> bool {
        self.keys_pressed_this_frame.contains(&key)
    }

    /// Checks if a key was just released this frame.
    pub fn was_key_just_released(&self, key: ChipKey) -> bool {
        self.keys_released_this_frame.contains(&key)
    }

    /// Gets the keyboard configuration.
    pub fn keyboard_config(&self) -> &KeyboardConfig {
        self.keyboard.config()
    }
}

impl Default for InputSystem {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            // Fallback to a null input system if initialization fails
            let keyboard = KeyboardInput::null();
            let mapper = Box::new(QwertyMapper::new());

            Self {
                keyboard,
                mapper,
                pressed_keys: std::collections::HashSet::new(),
                keys_pressed_this_frame: std::collections::HashSet::new(),
                keys_released_this_frame: std::collections::HashSet::new(),
                capture_enabled: true,
            }
        })
    }
}

impl Input for InputSystem {
    fn is_key_pressed(&self, key: ChipKey) -> bool {
        self.pressed_keys.contains(&key)
    }

    fn wait_for_key(&self) -> Option<ChipKey> {
        self.pressed_keys.iter().next().copied()
    }

    fn get_pressed_keys(&self) -> Vec<ChipKey> {
        self.pressed_keys.iter().copied().collect()
    }

    fn update(&mut self) -> InputResult<()> {
        // Update keyboard state
        self.keyboard.update()?;

        // Clear frame-specific key events
        self.keys_pressed_this_frame.clear();
        self.keys_released_this_frame.clear();

        Ok(())
    }
}

/// Simple input system for headless operation and testing.
pub struct NullInputSystem {
    input: crate::hardware::input::NullInput,
}

impl NullInputSystem {
    pub fn new() -> Self {
        Self {
            input: crate::hardware::input::NullInput::new(),
        }
    }

    /// Simulates pressing a key (for testing).
    pub fn press_key(&mut self, key: ChipKey) {
        self.input.press_key(key);
    }

    /// Simulates releasing a key (for testing).
    pub fn release_key(&mut self, key: ChipKey) {
        self.input.release_key(key);
    }
}

impl Default for NullInputSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl Input for NullInputSystem {
    fn is_key_pressed(&self, key: ChipKey) -> bool {
        self.input.is_key_pressed(key)
    }

    fn wait_for_key(&self) -> Option<ChipKey> {
        self.input.wait_for_key()
    }

    fn get_pressed_keys(&self) -> Vec<ChipKey> {
        self.input.get_pressed_keys()
    }

    fn update(&mut self) -> InputResult<()> {
        self.input.update()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_system_creation() {
        let system = InputSystem::new();

        // It's okay if this fails in headless environments
        if let Ok(input) = system {
            assert!(!input.any_key_pressed());
            assert!(input.is_capture_enabled());
        }
    }

    #[test]
    fn test_input_system_default() {
        let system = InputSystem::default();
        assert!(!system.any_key_pressed());
        assert!(system.is_capture_enabled());
    }

    #[test]
    fn test_input_system_simulation() {
        let mut system = InputSystem::default();

        assert!(!system.is_key_pressed(ChipKey::Key1));

        system.simulate_key_press(ChipKey::Key1);
        assert!(system.is_key_pressed(ChipKey::Key1));
        assert!(system.was_key_just_pressed(ChipKey::Key1));

        system.simulate_key_release(ChipKey::Key1);
        assert!(!system.is_key_pressed(ChipKey::Key1));
        assert!(system.was_key_just_released(ChipKey::Key1));
    }

    #[test]
    fn test_input_system_frame_events() {
        let mut system = InputSystem::default();

        system.simulate_key_press(ChipKey::Key2);
        system.simulate_key_press(ChipKey::Key3);

        let pressed = system.get_keys_pressed_this_frame();
        assert_eq!(pressed.len(), 2);
        assert!(pressed.contains(&ChipKey::Key2));
        assert!(pressed.contains(&ChipKey::Key3));

        // Update should clear frame events
        system.update().unwrap();
        assert!(system.get_keys_pressed_this_frame().is_empty());
    }

    #[test]
    fn test_input_system_capture() {
        let mut system = InputSystem::default();

        system.set_capture_enabled(false);
        assert!(!system.is_capture_enabled());

        system.set_capture_enabled(true);
        assert!(system.is_capture_enabled());
    }

    #[test]
    fn test_null_input_system() {
        let mut system = NullInputSystem::new();

        assert!(!system.is_key_pressed(ChipKey::Key0));

        system.press_key(ChipKey::Key0);
        assert!(system.is_key_pressed(ChipKey::Key0));

        system.release_key(ChipKey::Key0);
        assert!(!system.is_key_pressed(ChipKey::Key0));

        system.update().unwrap();
    }
}
