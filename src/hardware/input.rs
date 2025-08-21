//! Input interface for the Chip-8 emulator.
//!
//! This module defines the input abstraction and provides implementations
//! for the 16-key hexadecimal keypad used by Chip-8 systems.

use crate::error::InputError;
use std::collections::HashSet;

/// Input-specific result type.
pub type InputResult<T> = Result<T, InputError>;

/// Chip-8 hexadecimal keys (0-F).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChipKey {
    Key0 = 0x0,
    Key1 = 0x1,
    Key2 = 0x2,
    Key3 = 0x3,
    Key4 = 0x4,
    Key5 = 0x5,
    Key6 = 0x6,
    Key7 = 0x7,
    Key8 = 0x8,
    Key9 = 0x9,
    KeyA = 0xA,
    KeyB = 0xB,
    KeyC = 0xC,
    KeyD = 0xD,
    KeyE = 0xE,
    KeyF = 0xF,
}

impl ChipKey {
    /// Converts a u8 value to a ChipKey.
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x0 => Some(ChipKey::Key0),
            0x1 => Some(ChipKey::Key1),
            0x2 => Some(ChipKey::Key2),
            0x3 => Some(ChipKey::Key3),
            0x4 => Some(ChipKey::Key4),
            0x5 => Some(ChipKey::Key5),
            0x6 => Some(ChipKey::Key6),
            0x7 => Some(ChipKey::Key7),
            0x8 => Some(ChipKey::Key8),
            0x9 => Some(ChipKey::Key9),
            0xA => Some(ChipKey::KeyA),
            0xB => Some(ChipKey::KeyB),
            0xC => Some(ChipKey::KeyC),
            0xD => Some(ChipKey::KeyD),
            0xE => Some(ChipKey::KeyE),
            0xF => Some(ChipKey::KeyF),
            _ => None,
        }
    }

    /// Converts the ChipKey to its u8 value.
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    /// Returns all valid chip keys.
    pub fn all_keys() -> [ChipKey; 16] {
        [
            ChipKey::Key0,
            ChipKey::Key1,
            ChipKey::Key2,
            ChipKey::Key3,
            ChipKey::Key4,
            ChipKey::Key5,
            ChipKey::Key6,
            ChipKey::Key7,
            ChipKey::Key8,
            ChipKey::Key9,
            ChipKey::KeyA,
            ChipKey::KeyB,
            ChipKey::KeyC,
            ChipKey::KeyD,
            ChipKey::KeyE,
            ChipKey::KeyF,
        ]
    }
}

impl std::fmt::Display for ChipKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.to_u8())
    }
}

/// Input abstraction for the Chip-8 keypad.
///
/// The Chip-8 has a 16-key hexadecimal keypad arranged as:
/// ```text
/// 1 2 3 C
/// 4 5 6 D  
/// 7 8 9 E
/// A 0 B F
/// ```
pub trait Input {
    /// Checks if a specific key is currently pressed.
    fn is_key_pressed(&self, key: ChipKey) -> bool;

    /// Waits for any key press and returns the key.
    /// Returns None if no key is pressed.
    fn wait_for_key(&self) -> Option<ChipKey>;

    /// Gets all currently pressed keys.
    fn get_pressed_keys(&self) -> Vec<ChipKey>;

    /// Updates the input state (should be called each frame).
    fn update(&mut self) -> InputResult<()>;

    /// Checks if any key is currently pressed.
    fn any_key_pressed(&self) -> bool {
        ChipKey::all_keys()
            .iter()
            .any(|&key| self.is_key_pressed(key))
    }

    /// Gets the first pressed key (useful for single key operations).
    fn get_first_pressed_key(&self) -> Option<ChipKey> {
        ChipKey::all_keys()
            .iter()
            .find(|&&key| self.is_key_pressed(key))
            .copied()
    }
}

/// A software input implementation that maintains key state.
#[derive(Clone)]
pub struct SoftwareInput {
    /// Currently pressed keys.
    pressed_keys: HashSet<ChipKey>,

    /// Keys pressed this frame.
    keys_pressed_this_frame: HashSet<ChipKey>,

    /// Keys released this frame.
    keys_released_this_frame: HashSet<ChipKey>,
}

impl SoftwareInput {
    /// Creates a new software input system.
    pub fn new() -> Self {
        Self {
            pressed_keys: HashSet::new(),
            keys_pressed_this_frame: HashSet::new(),
            keys_released_this_frame: HashSet::new(),
        }
    }

    /// Simulates pressing a key.
    pub fn press_key(&mut self, key: ChipKey) {
        if !self.pressed_keys.contains(&key) {
            self.keys_pressed_this_frame.insert(key);
        }
        self.pressed_keys.insert(key);
    }

    /// Simulates releasing a key.
    pub fn release_key(&mut self, key: ChipKey) {
        if self.pressed_keys.contains(&key) {
            self.keys_released_this_frame.insert(key);
        }
        self.pressed_keys.remove(&key);
    }

    /// Releases all keys.
    pub fn release_all_keys(&mut self) {
        for &key in &self.pressed_keys {
            self.keys_released_this_frame.insert(key);
        }
        self.pressed_keys.clear();
    }

    /// Checks if a key was just pressed this frame.
    pub fn was_key_just_pressed(&self, key: ChipKey) -> bool {
        self.keys_pressed_this_frame.contains(&key)
    }

    /// Checks if a key was just released this frame.
    pub fn was_key_just_released(&self, key: ChipKey) -> bool {
        self.keys_released_this_frame.contains(&key)
    }

    /// Gets all keys pressed this frame.
    pub fn get_keys_pressed_this_frame(&self) -> Vec<ChipKey> {
        self.keys_pressed_this_frame.iter().copied().collect()
    }

    /// Gets all keys released this frame.
    pub fn get_keys_released_this_frame(&self) -> Vec<ChipKey> {
        self.keys_released_this_frame.iter().copied().collect()
    }
}

impl Default for SoftwareInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Input for SoftwareInput {
    fn is_key_pressed(&self, key: ChipKey) -> bool {
        self.pressed_keys.contains(&key)
    }

    fn wait_for_key(&self) -> Option<ChipKey> {
        self.get_first_pressed_key()
    }

    fn get_pressed_keys(&self) -> Vec<ChipKey> {
        self.pressed_keys.iter().copied().collect()
    }

    fn update(&mut self) -> InputResult<()> {
        // Clear frame-specific key states
        self.keys_pressed_this_frame.clear();
        self.keys_released_this_frame.clear();
        Ok(())
    }
}

/// A null input implementation for testing and automated operation.
pub struct NullInput {
    input: SoftwareInput,
}

impl NullInput {
    pub fn new() -> Self {
        Self {
            input: SoftwareInput::new(),
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

impl Default for NullInput {
    fn default() -> Self {
        Self::new()
    }
}

impl Input for NullInput {
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

/// Default key mapping for QWERTY keyboards.
///
/// Maps the Chip-8 keypad to QWERTY keys as follows:
/// ```text
/// Chip-8:    QWERTY:
/// 1 2 3 C    1 2 3 4
/// 4 5 6 D    Q W E R
/// 7 8 9 E    A S D F
/// A 0 B F    Z X C V
/// ```
pub struct QwertyKeyMap;

impl QwertyKeyMap {
    /// Maps a character to a ChipKey.
    pub fn char_to_chip_key(c: char) -> Option<ChipKey> {
        match c.to_ascii_uppercase() {
            '1' => Some(ChipKey::Key1),
            '2' => Some(ChipKey::Key2),
            '3' => Some(ChipKey::Key3),
            '4' => Some(ChipKey::KeyC),
            'Q' => Some(ChipKey::Key4),
            'W' => Some(ChipKey::Key5),
            'E' => Some(ChipKey::Key6),
            'R' => Some(ChipKey::KeyD),
            'A' => Some(ChipKey::Key7),
            'S' => Some(ChipKey::Key8),
            'D' => Some(ChipKey::Key9),
            'F' => Some(ChipKey::KeyE),
            'Z' => Some(ChipKey::KeyA),
            'X' => Some(ChipKey::Key0),
            'C' => Some(ChipKey::KeyB),
            'V' => Some(ChipKey::KeyF),
            _ => None,
        }
    }

    /// Maps a ChipKey to its QWERTY character.
    pub fn chip_key_to_char(key: ChipKey) -> char {
        match key {
            ChipKey::Key1 => '1',
            ChipKey::Key2 => '2',
            ChipKey::Key3 => '3',
            ChipKey::KeyC => '4',
            ChipKey::Key4 => 'Q',
            ChipKey::Key5 => 'W',
            ChipKey::Key6 => 'E',
            ChipKey::KeyD => 'R',
            ChipKey::Key7 => 'A',
            ChipKey::Key8 => 'S',
            ChipKey::Key9 => 'D',
            ChipKey::KeyE => 'F',
            ChipKey::KeyA => 'Z',
            ChipKey::Key0 => 'X',
            ChipKey::KeyB => 'C',
            ChipKey::KeyF => 'V',
        }
    }

    /// Gets all key mappings as (ChipKey, char) pairs.
    pub fn get_all_mappings() -> [(ChipKey, char); 16] {
        [
            (ChipKey::Key0, 'X'),
            (ChipKey::Key1, '1'),
            (ChipKey::Key2, '2'),
            (ChipKey::Key3, '3'),
            (ChipKey::Key4, 'Q'),
            (ChipKey::Key5, 'W'),
            (ChipKey::Key6, 'E'),
            (ChipKey::Key7, 'A'),
            (ChipKey::Key8, 'S'),
            (ChipKey::Key9, 'D'),
            (ChipKey::KeyA, 'Z'),
            (ChipKey::KeyB, 'C'),
            (ChipKey::KeyC, '4'),
            (ChipKey::KeyD, 'R'),
            (ChipKey::KeyE, 'F'),
            (ChipKey::KeyF, 'V'),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chip_key_conversion() {
        // Test from_u8
        assert_eq!(ChipKey::from_u8(0x0), Some(ChipKey::Key0));
        assert_eq!(ChipKey::from_u8(0xF), Some(ChipKey::KeyF));
        assert_eq!(ChipKey::from_u8(0x10), None);

        // Test to_u8
        assert_eq!(ChipKey::Key0.to_u8(), 0x0);
        assert_eq!(ChipKey::KeyF.to_u8(), 0xF);
    }

    #[test]
    fn test_chip_key_display() {
        assert_eq!(format!("{}", ChipKey::Key0), "0");
        assert_eq!(format!("{}", ChipKey::KeyA), "A");
        assert_eq!(format!("{}", ChipKey::KeyF), "F");
    }

    #[test]
    fn test_chip_key_all_keys() {
        let all_keys = ChipKey::all_keys();
        assert_eq!(all_keys.len(), 16);

        // Verify all keys are unique
        let mut values = all_keys.iter().map(|k| k.to_u8()).collect::<Vec<_>>();
        values.sort();
        assert_eq!(values, (0..16).collect::<Vec<_>>());
    }

    #[test]
    fn test_software_input_creation() {
        let input = SoftwareInput::new();

        assert!(!input.any_key_pressed());
        assert_eq!(input.get_pressed_keys().len(), 0);
        assert_eq!(input.wait_for_key(), None);
    }

    #[test]
    fn test_software_input_key_operations() {
        let mut input = SoftwareInput::new();

        // Test pressing keys
        input.press_key(ChipKey::Key1);
        input.press_key(ChipKey::Key5);

        assert!(input.is_key_pressed(ChipKey::Key1));
        assert!(input.is_key_pressed(ChipKey::Key5));
        assert!(!input.is_key_pressed(ChipKey::Key0));
        assert!(input.any_key_pressed());

        let pressed = input.get_pressed_keys();
        assert_eq!(pressed.len(), 2);
        assert!(pressed.contains(&ChipKey::Key1));
        assert!(pressed.contains(&ChipKey::Key5));

        // Test releasing keys
        input.release_key(ChipKey::Key1);
        assert!(!input.is_key_pressed(ChipKey::Key1));
        assert!(input.is_key_pressed(ChipKey::Key5));

        // Test release all
        input.release_all_keys();
        assert!(!input.any_key_pressed());
    }

    #[test]
    fn test_software_input_frame_events() {
        let mut input = SoftwareInput::new();

        // Press a key
        input.press_key(ChipKey::Key2);
        assert!(input.was_key_just_pressed(ChipKey::Key2));
        assert!(!input.was_key_just_released(ChipKey::Key2));

        // Update frame
        input.update().unwrap();
        assert!(!input.was_key_just_pressed(ChipKey::Key2));
        assert!(!input.was_key_just_released(ChipKey::Key2));

        // Release the key
        input.release_key(ChipKey::Key2);
        assert!(!input.was_key_just_pressed(ChipKey::Key2));
        assert!(input.was_key_just_released(ChipKey::Key2));

        // Update frame
        input.update().unwrap();
        assert!(!input.was_key_just_pressed(ChipKey::Key2));
        assert!(!input.was_key_just_released(ChipKey::Key2));
    }

    #[test]
    fn test_software_input_wait_for_key() {
        let mut input = SoftwareInput::new();

        assert_eq!(input.wait_for_key(), None);

        input.press_key(ChipKey::Key7);
        assert_eq!(input.wait_for_key(), Some(ChipKey::Key7));
        assert_eq!(input.get_first_pressed_key(), Some(ChipKey::Key7));
    }

    #[test]
    fn test_null_input() {
        let mut input = NullInput::new();

        assert!(!input.is_key_pressed(ChipKey::Key0));

        input.press_key(ChipKey::Key3);
        assert!(input.is_key_pressed(ChipKey::Key3));

        input.release_key(ChipKey::Key3);
        assert!(!input.is_key_pressed(ChipKey::Key3));

        input.update().unwrap();
    }

    #[test]
    fn test_qwerty_key_mapping() {
        // Test char to chip key
        assert_eq!(QwertyKeyMap::char_to_chip_key('1'), Some(ChipKey::Key1));
        assert_eq!(QwertyKeyMap::char_to_chip_key('Q'), Some(ChipKey::Key4));
        assert_eq!(QwertyKeyMap::char_to_chip_key('q'), Some(ChipKey::Key4)); // Case insensitive
        assert_eq!(QwertyKeyMap::char_to_chip_key('X'), Some(ChipKey::Key0));
        assert_eq!(QwertyKeyMap::char_to_chip_key('V'), Some(ChipKey::KeyF));
        assert_eq!(QwertyKeyMap::char_to_chip_key('Y'), None); // Invalid key

        // Test chip key to char
        assert_eq!(QwertyKeyMap::chip_key_to_char(ChipKey::Key1), '1');
        assert_eq!(QwertyKeyMap::chip_key_to_char(ChipKey::Key4), 'Q');
        assert_eq!(QwertyKeyMap::chip_key_to_char(ChipKey::Key0), 'X');
        assert_eq!(QwertyKeyMap::chip_key_to_char(ChipKey::KeyF), 'V');
    }

    #[test]
    fn test_qwerty_mapping_completeness() {
        let mappings = QwertyKeyMap::get_all_mappings();
        assert_eq!(mappings.len(), 16);

        // Verify all chip keys are covered
        let mut chip_keys: Vec<_> = mappings.iter().map(|(key, _)| *key).collect();
        chip_keys.sort_by_key(|k| k.to_u8());

        let expected_keys = ChipKey::all_keys();
        assert_eq!(chip_keys, expected_keys);

        // Verify bidirectional mapping
        for (chip_key, qwerty_char) in mappings {
            assert_eq!(QwertyKeyMap::chip_key_to_char(chip_key), qwerty_char);
            assert_eq!(QwertyKeyMap::char_to_chip_key(qwerty_char), Some(chip_key));
        }
    }
}
