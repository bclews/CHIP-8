//! Keyboard input handling for the Chip-8 emulator.
//!
//! This module provides keyboard event handling and configuration
//! for capturing real keyboard input.

use super::InputSystemResult;
use std::collections::HashMap;

/// Configuration for keyboard input handling.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KeyboardConfig {
    /// Whether to capture keyboard events.
    pub capture_enabled: bool,
    
    /// Polling rate in Hz for keyboard state updates.
    pub polling_rate: u32,
    
    /// Whether to handle key repeat events.
    pub handle_repeats: bool,
    
    /// Minimum time between key repeat events (in milliseconds).
    pub repeat_delay: u32,
    
    /// Whether to use raw keyboard input (if available).
    pub use_raw_input: bool,
}

impl Default for KeyboardConfig {
    fn default() -> Self {
        Self {
            capture_enabled: true,
            polling_rate: 60, // 60 Hz polling
            handle_repeats: false, // Don't handle repeats for Chip-8
            repeat_delay: 500, // 500ms repeat delay
            use_raw_input: false, // Use standard input for compatibility
        }
    }
}

impl KeyboardConfig {
    /// Creates a new keyboard configuration.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Sets whether to capture keyboard events.
    pub fn with_capture_enabled(mut self, enabled: bool) -> Self {
        self.capture_enabled = enabled;
        self
    }
    
    /// Sets the polling rate.
    pub fn with_polling_rate(mut self, rate: u32) -> Self {
        self.polling_rate = rate;
        self
    }
    
    /// Sets whether to handle key repeats.
    pub fn with_handle_repeats(mut self, handle: bool) -> Self {
        self.handle_repeats = handle;
        self
    }
    
    /// Sets the repeat delay.
    pub fn with_repeat_delay(mut self, delay: u32) -> Self {
        self.repeat_delay = delay;
        self
    }
    
    /// Sets whether to use raw input.
    pub fn with_raw_input(mut self, raw: bool) -> Self {
        self.use_raw_input = raw;
        self
    }
    
    /// Creates a gaming-focused configuration.
    pub fn gaming() -> Self {
        Self::new()
            .with_polling_rate(120) // High polling rate
            .with_handle_repeats(false) // No repeats for gaming
            .with_raw_input(true) // Raw input for minimal latency
    }
    
    /// Creates a standard desktop configuration.
    pub fn desktop() -> Self {
        Self::new()
            .with_polling_rate(60) // Standard polling
            .with_handle_repeats(true) // Handle repeats for text input
            .with_raw_input(false) // Standard input for compatibility
    }
}

/// Keyboard event representing a key press or release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyboardEvent {
    /// The physical key that was pressed.
    pub physical_key: PhysicalKey,
    
    /// The logical key (after layout processing).
    pub logical_key: LogicalKey,
    
    /// The state of the key (pressed or released).
    pub state: KeyState,
    
    /// Whether this is a repeat event.
    pub is_repeat: bool,
    
    /// Timestamp of the event.
    pub timestamp: std::time::Instant,
}

/// Physical key codes (scan codes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhysicalKey {
    // Numbers
    Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9, Key0,
    
    // Letters (QWERTY layout)
    KeyQ, KeyW, KeyE, KeyR, KeyT, KeyY, KeyU, KeyI, KeyO, KeyP,
    KeyA, KeyS, KeyD, KeyF, KeyG, KeyH, KeyJ, KeyK, KeyL,
    KeyZ, KeyX, KeyC, KeyV, KeyB, KeyN, KeyM,
    
    // Special keys
    Space, Enter, Escape, Backspace, Tab,
    LeftShift, RightShift, LeftCtrl, RightCtrl, LeftAlt, RightAlt,
    
    // Arrow keys
    ArrowUp, ArrowDown, ArrowLeft, ArrowRight,
    
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    
    // Unknown key
    Unknown(u32),
}

/// Logical key representations (after keyboard layout processing).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogicalKey {
    /// Character key.
    Character(char),
    
    /// Named key.
    Named(NamedKey),
    
    /// Unidentified key.
    Unidentified,
}

/// Named logical keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamedKey {
    Space, Enter, Escape, Backspace, Tab,
    ArrowUp, ArrowDown, ArrowLeft, ArrowRight,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    Shift, Control, Alt,
}

/// Key state (pressed or released).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyState {
    Pressed,
    Released,
}

/// Keyboard input handler.
pub struct KeyboardInput {
    /// Configuration for the keyboard.
    config: KeyboardConfig,
    
    /// Current state of physical keys.
    physical_state: HashMap<PhysicalKey, bool>,
    
    /// Last update timestamp.
    last_update: std::time::Instant,
    
    /// Event queue for processing.
    event_queue: Vec<KeyboardEvent>,
}

impl KeyboardInput {
    /// Creates a new keyboard input handler.
    pub fn new(config: KeyboardConfig) -> InputSystemResult<Self> {
        Ok(Self {
            config,
            physical_state: HashMap::new(),
            last_update: std::time::Instant::now(),
            event_queue: Vec::new(),
        })
    }
    
    /// Creates a null keyboard input (for headless operation).
    pub fn null() -> Self {
        Self {
            config: KeyboardConfig::new().with_capture_enabled(true), // Enable capture for testing
            physical_state: HashMap::new(),
            last_update: std::time::Instant::now(),
            event_queue: Vec::new(),
        }
    }
    
    /// Updates the keyboard state.
    pub fn update(&mut self) -> InputSystemResult<()> {
        let now = std::time::Instant::now();
        
        // Clear old events
        self.event_queue.clear();
        
        self.last_update = now;
        Ok(())
    }
    
    /// Processes a keyboard event (typically called from window event handling).
    pub fn process_event(&mut self, event: KeyboardEvent) -> InputSystemResult<()> {
        if !self.config.capture_enabled {
            return Ok(());
        }
        
        // Skip repeat events if not handling them
        if event.is_repeat && !self.config.handle_repeats {
            return Ok(());
        }
        
        // Update physical key state
        match event.state {
            KeyState::Pressed => {
                self.physical_state.insert(event.physical_key, true);
            }
            KeyState::Released => {
                self.physical_state.insert(event.physical_key, false);
            }
        }
        
        // Add to event queue
        self.event_queue.push(event);
        
        Ok(())
    }
    
    /// Checks if a physical key is currently pressed.
    pub fn is_physical_key_pressed(&self, key: PhysicalKey) -> bool {
        self.physical_state.get(&key).copied().unwrap_or(false)
    }
    
    /// Gets all currently pressed physical keys.
    pub fn get_pressed_physical_keys(&self) -> Vec<PhysicalKey> {
        self.physical_state
            .iter()
            .filter_map(|(&key, &pressed)| if pressed { Some(key) } else { None })
            .collect()
    }
    
    /// Gets the current event queue.
    pub fn get_events(&self) -> &[KeyboardEvent] {
        &self.event_queue
    }
    
    /// Simulates a key event (for testing).
    pub fn simulate_key_event(&mut self, physical_key: PhysicalKey, state: KeyState) {
        let logical_key = Self::physical_to_logical(physical_key);
        let event = KeyboardEvent {
            physical_key,
            logical_key,
            state,
            is_repeat: false,
            timestamp: std::time::Instant::now(),
        };
        
        let _ = self.process_event(event);
    }
    
    /// Gets the keyboard configuration.
    pub fn config(&self) -> &KeyboardConfig {
        &self.config
    }
    
    /// Converts a physical key to a logical key (basic mapping).
    pub fn physical_to_logical(physical: PhysicalKey) -> LogicalKey {
        match physical {
            PhysicalKey::Key1 => LogicalKey::Character('1'),
            PhysicalKey::Key2 => LogicalKey::Character('2'),
            PhysicalKey::Key3 => LogicalKey::Character('3'),
            PhysicalKey::Key4 => LogicalKey::Character('4'),
            PhysicalKey::Key5 => LogicalKey::Character('5'),
            PhysicalKey::Key6 => LogicalKey::Character('6'),
            PhysicalKey::Key7 => LogicalKey::Character('7'),
            PhysicalKey::Key8 => LogicalKey::Character('8'),
            PhysicalKey::Key9 => LogicalKey::Character('9'),
            PhysicalKey::Key0 => LogicalKey::Character('0'),
            
            PhysicalKey::KeyQ => LogicalKey::Character('q'),
            PhysicalKey::KeyW => LogicalKey::Character('w'),
            PhysicalKey::KeyE => LogicalKey::Character('e'),
            PhysicalKey::KeyR => LogicalKey::Character('r'),
            PhysicalKey::KeyT => LogicalKey::Character('t'),
            PhysicalKey::KeyY => LogicalKey::Character('y'),
            PhysicalKey::KeyU => LogicalKey::Character('u'),
            PhysicalKey::KeyI => LogicalKey::Character('i'),
            PhysicalKey::KeyO => LogicalKey::Character('o'),
            PhysicalKey::KeyP => LogicalKey::Character('p'),
            PhysicalKey::KeyA => LogicalKey::Character('a'),
            PhysicalKey::KeyS => LogicalKey::Character('s'),
            PhysicalKey::KeyD => LogicalKey::Character('d'),
            PhysicalKey::KeyF => LogicalKey::Character('f'),
            PhysicalKey::KeyG => LogicalKey::Character('g'),
            PhysicalKey::KeyH => LogicalKey::Character('h'),
            PhysicalKey::KeyJ => LogicalKey::Character('j'),
            PhysicalKey::KeyK => LogicalKey::Character('k'),
            PhysicalKey::KeyL => LogicalKey::Character('l'),
            PhysicalKey::KeyZ => LogicalKey::Character('z'),
            PhysicalKey::KeyX => LogicalKey::Character('x'),
            PhysicalKey::KeyC => LogicalKey::Character('c'),
            PhysicalKey::KeyV => LogicalKey::Character('v'),
            PhysicalKey::KeyB => LogicalKey::Character('b'),
            PhysicalKey::KeyN => LogicalKey::Character('n'),
            PhysicalKey::KeyM => LogicalKey::Character('m'),
            
            PhysicalKey::Space => LogicalKey::Named(NamedKey::Space),
            PhysicalKey::Enter => LogicalKey::Named(NamedKey::Enter),
            PhysicalKey::Escape => LogicalKey::Named(NamedKey::Escape),
            PhysicalKey::Backspace => LogicalKey::Named(NamedKey::Backspace),
            PhysicalKey::Tab => LogicalKey::Named(NamedKey::Tab),
            
            PhysicalKey::LeftShift | PhysicalKey::RightShift => LogicalKey::Named(NamedKey::Shift),
            PhysicalKey::LeftCtrl | PhysicalKey::RightCtrl => LogicalKey::Named(NamedKey::Control),
            PhysicalKey::LeftAlt | PhysicalKey::RightAlt => LogicalKey::Named(NamedKey::Alt),
            
            PhysicalKey::ArrowUp => LogicalKey::Named(NamedKey::ArrowUp),
            PhysicalKey::ArrowDown => LogicalKey::Named(NamedKey::ArrowDown),
            PhysicalKey::ArrowLeft => LogicalKey::Named(NamedKey::ArrowLeft),
            PhysicalKey::ArrowRight => LogicalKey::Named(NamedKey::ArrowRight),
            
            PhysicalKey::F1 => LogicalKey::Named(NamedKey::F1),
            PhysicalKey::F2 => LogicalKey::Named(NamedKey::F2),
            PhysicalKey::F3 => LogicalKey::Named(NamedKey::F3),
            PhysicalKey::F4 => LogicalKey::Named(NamedKey::F4),
            PhysicalKey::F5 => LogicalKey::Named(NamedKey::F5),
            PhysicalKey::F6 => LogicalKey::Named(NamedKey::F6),
            PhysicalKey::F7 => LogicalKey::Named(NamedKey::F7),
            PhysicalKey::F8 => LogicalKey::Named(NamedKey::F8),
            PhysicalKey::F9 => LogicalKey::Named(NamedKey::F9),
            PhysicalKey::F10 => LogicalKey::Named(NamedKey::F10),
            PhysicalKey::F11 => LogicalKey::Named(NamedKey::F11),
            PhysicalKey::F12 => LogicalKey::Named(NamedKey::F12),
            
            PhysicalKey::Unknown(_) => LogicalKey::Unidentified,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_keyboard_config_creation() {
        let config = KeyboardConfig::new();
        
        assert!(config.capture_enabled);
        assert_eq!(config.polling_rate, 60);
        assert!(!config.handle_repeats);
        assert_eq!(config.repeat_delay, 500);
        assert!(!config.use_raw_input);
    }
    
    #[test]
    fn test_keyboard_config_builder() {
        let config = KeyboardConfig::new()
            .with_capture_enabled(false)
            .with_polling_rate(120)
            .with_handle_repeats(true)
            .with_repeat_delay(250)
            .with_raw_input(true);
        
        assert!(!config.capture_enabled);
        assert_eq!(config.polling_rate, 120);
        assert!(config.handle_repeats);
        assert_eq!(config.repeat_delay, 250);
        assert!(config.use_raw_input);
    }
    
    #[test]
    fn test_keyboard_config_presets() {
        let gaming = KeyboardConfig::gaming();
        assert_eq!(gaming.polling_rate, 120);
        assert!(!gaming.handle_repeats);
        assert!(gaming.use_raw_input);
        
        let desktop = KeyboardConfig::desktop();
        assert_eq!(desktop.polling_rate, 60);
        assert!(desktop.handle_repeats);
        assert!(!desktop.use_raw_input);
    }
    
    #[test]
    fn test_keyboard_input_creation() {
        let config = KeyboardConfig::new();
        let keyboard = KeyboardInput::new(config);
        
        assert!(keyboard.is_ok());
        let kb = keyboard.unwrap();
        assert!(kb.config().capture_enabled);
    }
    
    #[test]
    fn test_keyboard_input_simulation() {
        let mut keyboard = KeyboardInput::null();
        
        assert!(!keyboard.is_physical_key_pressed(PhysicalKey::KeyA));
        
        keyboard.simulate_key_event(PhysicalKey::KeyA, KeyState::Pressed);
        assert!(keyboard.is_physical_key_pressed(PhysicalKey::KeyA));
        
        keyboard.simulate_key_event(PhysicalKey::KeyA, KeyState::Released);
        assert!(!keyboard.is_physical_key_pressed(PhysicalKey::KeyA));
    }
    
    #[test]
    fn test_keyboard_event_queue() {
        let mut keyboard = KeyboardInput::null();
        
        keyboard.simulate_key_event(PhysicalKey::Key1, KeyState::Pressed);
        keyboard.simulate_key_event(PhysicalKey::Key2, KeyState::Pressed);
        
        let events = keyboard.get_events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].physical_key, PhysicalKey::Key1);
        assert_eq!(events[1].physical_key, PhysicalKey::Key2);
        
        keyboard.update().unwrap();
        assert!(keyboard.get_events().is_empty());
    }
    
    #[test]
    fn test_physical_to_logical_mapping() {
        assert_eq!(
            KeyboardInput::physical_to_logical(PhysicalKey::Key1),
            LogicalKey::Character('1')
        );
        assert_eq!(
            KeyboardInput::physical_to_logical(PhysicalKey::KeyA),
            LogicalKey::Character('a')
        );
        assert_eq!(
            KeyboardInput::physical_to_logical(PhysicalKey::Space),
            LogicalKey::Named(NamedKey::Space)
        );
        assert_eq!(
            KeyboardInput::physical_to_logical(PhysicalKey::F1),
            LogicalKey::Named(NamedKey::F1)
        );
    }
    
    #[test]
    fn test_pressed_keys_tracking() {
        let mut keyboard = KeyboardInput::null();
        
        keyboard.simulate_key_event(PhysicalKey::KeyA, KeyState::Pressed);
        keyboard.simulate_key_event(PhysicalKey::KeyB, KeyState::Pressed);
        
        let pressed = keyboard.get_pressed_physical_keys();
        assert_eq!(pressed.len(), 2);
        assert!(pressed.contains(&PhysicalKey::KeyA));
        assert!(pressed.contains(&PhysicalKey::KeyB));
        
        keyboard.simulate_key_event(PhysicalKey::KeyA, KeyState::Released);
        
        let pressed = keyboard.get_pressed_physical_keys();
        assert_eq!(pressed.len(), 1);
        assert!(pressed.contains(&PhysicalKey::KeyB));
    }
}