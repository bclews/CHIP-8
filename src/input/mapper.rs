//! Key mapping for the Chip-8 emulator.
//!
//! This module provides key mapping functionality to translate
//! physical keyboard input to Chip-8 hexadecimal keys.

use crate::hardware::ChipKey;
use super::keyboard::{KeyboardEvent, PhysicalKey, LogicalKey};
use std::collections::HashMap;
use winit::event::VirtualKeyCode;

/// Trait for mapping keyboard events to Chip-8 keys.
pub trait KeyMapper: Send + Sync {
    /// Maps a keyboard event to a Chip-8 key.
    fn map_key_event(&self, event: &KeyboardEvent) -> Option<ChipKey>;
    
    /// Maps a physical key to a Chip-8 key.
    fn map_physical_key(&self, key: PhysicalKey) -> Option<ChipKey>;
    
    /// Maps a logical key to a Chip-8 key.
    fn map_logical_key(&self, key: &LogicalKey) -> Option<ChipKey>;

    /// Maps a winit VirtualKeyCode to a Chip-8 key.
    fn map_virtual_keycode(&self, key: VirtualKeyCode) -> Option<ChipKey>;
    
    /// Gets all key mappings as (PhysicalKey, ChipKey) pairs.
    fn get_physical_mappings(&self) -> Vec<(PhysicalKey, ChipKey)>;
    
    /// Gets the name of this key mapper.
    fn name(&self) -> &str;
    
    /// Gets a description of the key layout.
    fn description(&self) -> &str;
}

/// QWERTY keyboard mapper for Chip-8 keys.
/// 
/// Maps the Chip-8 keypad to QWERTY keys as follows:
/// ```text
/// Chip-8:    QWERTY:
/// 1 2 3 C    1 2 3 4
/// 4 5 6 D    Q W E R
/// 7 8 9 E    A S D F
/// A 0 B F    Z X C V
/// ```
pub struct QwertyMapper {
    physical_map: HashMap<PhysicalKey, ChipKey>,
    logical_map: HashMap<LogicalKey, ChipKey>,
    virtual_keycode_map: HashMap<VirtualKeyCode, ChipKey>,
}

impl QwertyMapper {
    /// Creates a new QWERTY mapper.
    pub fn new() -> Self {
        let mut physical_map = HashMap::new();
        let mut logical_map = HashMap::new();
        let mut virtual_keycode_map = HashMap::new();
        
        // Physical key mappings
        physical_map.insert(PhysicalKey::Key1, ChipKey::Key1);
        physical_map.insert(PhysicalKey::Key2, ChipKey::Key2);
        physical_map.insert(PhysicalKey::Key3, ChipKey::Key3);
        physical_map.insert(PhysicalKey::Key4, ChipKey::KeyC);
        physical_map.insert(PhysicalKey::KeyQ, ChipKey::Key4);
        physical_map.insert(PhysicalKey::KeyW, ChipKey::Key5);
        physical_map.insert(PhysicalKey::KeyE, ChipKey::Key6);
        physical_map.insert(PhysicalKey::KeyR, ChipKey::KeyD);
        physical_map.insert(PhysicalKey::KeyA, ChipKey::Key7);
        physical_map.insert(PhysicalKey::KeyS, ChipKey::Key8);
        physical_map.insert(PhysicalKey::KeyD, ChipKey::Key9);
        physical_map.insert(PhysicalKey::KeyF, ChipKey::KeyE);
        physical_map.insert(PhysicalKey::KeyZ, ChipKey::KeyA);
        physical_map.insert(PhysicalKey::KeyX, ChipKey::Key0);
        physical_map.insert(PhysicalKey::KeyC, ChipKey::KeyB);
        physical_map.insert(PhysicalKey::KeyV, ChipKey::KeyF);
        
        // Logical key mappings (for character keys)
        logical_map.insert(LogicalKey::Character('1'), ChipKey::Key1);
        logical_map.insert(LogicalKey::Character('2'), ChipKey::Key2);
        logical_map.insert(LogicalKey::Character('3'), ChipKey::Key3);
        logical_map.insert(LogicalKey::Character('4'), ChipKey::KeyC);
        logical_map.insert(LogicalKey::Character('q'), ChipKey::Key4);
        logical_map.insert(LogicalKey::Character('Q'), ChipKey::Key4);
        logical_map.insert(LogicalKey::Character('w'), ChipKey::Key5);
        logical_map.insert(LogicalKey::Character('W'), ChipKey::Key5);
        logical_map.insert(LogicalKey::Character('e'), ChipKey::Key6);
        logical_map.insert(LogicalKey::Character('E'), ChipKey::Key6);
        logical_map.insert(LogicalKey::Character('r'), ChipKey::KeyD);
        logical_map.insert(LogicalKey::Character('R'), ChipKey::KeyD);
        logical_map.insert(LogicalKey::Character('a'), ChipKey::Key7);
        logical_map.insert(LogicalKey::Character('A'), ChipKey::Key7);
        logical_map.insert(LogicalKey::Character('s'), ChipKey::Key8);
        logical_map.insert(LogicalKey::Character('S'), ChipKey::Key8);
        logical_map.insert(LogicalKey::Character('d'), ChipKey::Key9);
        logical_map.insert(LogicalKey::Character('D'), ChipKey::Key9);
        logical_map.insert(LogicalKey::Character('f'), ChipKey::KeyE);
        logical_map.insert(LogicalKey::Character('F'), ChipKey::KeyE);
        logical_map.insert(LogicalKey::Character('z'), ChipKey::KeyA);
        logical_map.insert(LogicalKey::Character('Z'), ChipKey::KeyA);
        logical_map.insert(LogicalKey::Character('x'), ChipKey::Key0);
        logical_map.insert(LogicalKey::Character('X'), ChipKey::Key0);
        logical_map.insert(LogicalKey::Character('c'), ChipKey::KeyB);
        logical_map.insert(LogicalKey::Character('C'), ChipKey::KeyB);
        logical_map.insert(LogicalKey::Character('v'), ChipKey::KeyF);
        logical_map.insert(LogicalKey::Character('V'), ChipKey::KeyF);

        // VirtualKeyCode mappings
        virtual_keycode_map.insert(VirtualKeyCode::Key1, ChipKey::Key1);
        virtual_keycode_map.insert(VirtualKeyCode::Key2, ChipKey::Key2);
        virtual_keycode_map.insert(VirtualKeyCode::Key3, ChipKey::Key3);
        virtual_keycode_map.insert(VirtualKeyCode::Key4, ChipKey::KeyC);
        virtual_keycode_map.insert(VirtualKeyCode::Q, ChipKey::Key4);
        virtual_keycode_map.insert(VirtualKeyCode::W, ChipKey::Key5);
        virtual_keycode_map.insert(VirtualKeyCode::E, ChipKey::Key6);
        virtual_keycode_map.insert(VirtualKeyCode::R, ChipKey::KeyD);
        virtual_keycode_map.insert(VirtualKeyCode::A, ChipKey::Key7);
        virtual_keycode_map.insert(VirtualKeyCode::S, ChipKey::Key8);
        virtual_keycode_map.insert(VirtualKeyCode::D, ChipKey::Key9);
        virtual_keycode_map.insert(VirtualKeyCode::F, ChipKey::KeyE);
        virtual_keycode_map.insert(VirtualKeyCode::Z, ChipKey::KeyA);
        virtual_keycode_map.insert(VirtualKeyCode::X, ChipKey::Key0);
        virtual_keycode_map.insert(VirtualKeyCode::C, ChipKey::KeyB);
        virtual_keycode_map.insert(VirtualKeyCode::V, ChipKey::KeyF);
        
        Self {
            physical_map,
            logical_map,
            virtual_keycode_map,
        }
    }
}

impl Default for QwertyMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyMapper for QwertyMapper {
    fn map_key_event(&self, event: &KeyboardEvent) -> Option<ChipKey> {
        // Try logical key first, then physical key
        self.map_logical_key(&event.logical_key)
            .or_else(|| self.map_physical_key(event.physical_key))
    }
    
    fn map_physical_key(&self, key: PhysicalKey) -> Option<ChipKey> {
        self.physical_map.get(&key).copied()
    }
    
    fn map_logical_key(&self, key: &LogicalKey) -> Option<ChipKey> {
        self.logical_map.get(key).copied()
    }

    fn map_virtual_keycode(&self, key: VirtualKeyCode) -> Option<ChipKey> {
        self.virtual_keycode_map.get(&key).copied()
    }
    
    fn get_physical_mappings(&self) -> Vec<(PhysicalKey, ChipKey)> {
        self.physical_map.iter().map(|(&k, &v)| (k, v)).collect()
    }
    
    fn name(&self) -> &str {
        "QWERTY"
    }
    
    fn description(&self) -> &str {
        r"Standard QWERTY keyboard layout:
1 2 3 4 -> 1 2 3 C
Q W E R -> 4 5 6 D
A S D F -> 7 8 9 E
Z X C V -> A 0 B F"
    }
}

/// Alternative QWERTY mapper that uses different key positions.
pub struct AlternativeQwertyMapper {
    physical_map: HashMap<PhysicalKey, ChipKey>,
    logical_map: HashMap<LogicalKey, ChipKey>,
    virtual_keycode_map: HashMap<VirtualKeyCode, ChipKey>,
}

impl Default for AlternativeQwertyMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl AlternativeQwertyMapper {
    /// Creates a new alternative QWERTY mapper.
    /// 
    /// Uses the numpad-like layout on the left side of QWERTY:
    /// ```text
    /// Chip-8:    QWERTY:
    /// 1 2 3 C    1 2 3 4
    /// 4 5 6 D    Q W E R  
    /// 7 8 9 E    A S D F
    /// A 0 B F    Z X C V
    /// ```
    pub fn new() -> Self {
        let mut physical_map = HashMap::new();
        let mut logical_map = HashMap::new();
        let mut virtual_keycode_map = HashMap::new();
        
        // Use the same mappings as QwertyMapper for now
        // This could be extended with different layouts
        physical_map.insert(PhysicalKey::Key1, ChipKey::Key1);
        physical_map.insert(PhysicalKey::Key2, ChipKey::Key2);
        physical_map.insert(PhysicalKey::Key3, ChipKey::Key3);
        physical_map.insert(PhysicalKey::Key4, ChipKey::KeyC);
        physical_map.insert(PhysicalKey::KeyQ, ChipKey::Key4);
        physical_map.insert(PhysicalKey::KeyW, ChipKey::Key5);
        physical_map.insert(PhysicalKey::KeyE, ChipKey::Key6);
        physical_map.insert(PhysicalKey::KeyR, ChipKey::KeyD);
        physical_map.insert(PhysicalKey::KeyA, ChipKey::Key7);
        physical_map.insert(PhysicalKey::KeyS, ChipKey::Key8);
        physical_map.insert(PhysicalKey::KeyD, ChipKey::Key9);
        physical_map.insert(PhysicalKey::KeyF, ChipKey::KeyE);
        physical_map.insert(PhysicalKey::KeyZ, ChipKey::KeyA);
        physical_map.insert(PhysicalKey::KeyX, ChipKey::Key0);
        physical_map.insert(PhysicalKey::KeyC, ChipKey::KeyB);
        physical_map.insert(PhysicalKey::KeyV, ChipKey::KeyF);
        
        // Add logical mappings
        for (&physical, &chip) in &physical_map {
            let logical = super::keyboard::KeyboardInput::physical_to_logical(physical);
            logical_map.insert(logical, chip);
        }

        // VirtualKeyCode mappings (same as QwertyMapper for now)
        virtual_keycode_map.insert(VirtualKeyCode::Key1, ChipKey::Key1);
        virtual_keycode_map.insert(VirtualKeyCode::Key2, ChipKey::Key2);
        virtual_keycode_map.insert(VirtualKeyCode::Key3, ChipKey::Key3);
        virtual_keycode_map.insert(VirtualKeyCode::Key4, ChipKey::KeyC);
        virtual_keycode_map.insert(VirtualKeyCode::Q, ChipKey::Key4);
        virtual_keycode_map.insert(VirtualKeyCode::W, ChipKey::Key5);
        virtual_keycode_map.insert(VirtualKeyCode::E, ChipKey::Key6);
        virtual_keycode_map.insert(VirtualKeyCode::R, ChipKey::KeyD);
        virtual_keycode_map.insert(VirtualKeyCode::A, ChipKey::Key7);
        virtual_keycode_map.insert(VirtualKeyCode::S, ChipKey::Key8);
        virtual_keycode_map.insert(VirtualKeyCode::D, ChipKey::Key9);
        virtual_keycode_map.insert(VirtualKeyCode::F, ChipKey::KeyE);
        virtual_keycode_map.insert(VirtualKeyCode::Z, ChipKey::KeyA);
        virtual_keycode_map.insert(VirtualKeyCode::X, ChipKey::Key0);
        virtual_keycode_map.insert(VirtualKeyCode::C, ChipKey::KeyB);
        virtual_keycode_map.insert(VirtualKeyCode::V, ChipKey::KeyF);
        
        Self {
            physical_map,
            logical_map,
            virtual_keycode_map,
        }
    }
}

impl KeyMapper for AlternativeQwertyMapper {
    fn map_key_event(&self, event: &KeyboardEvent) -> Option<ChipKey> {
        self.map_logical_key(&event.logical_key)
            .or_else(|| self.map_physical_key(event.physical_key))
    }
    
    fn map_physical_key(&self, key: PhysicalKey) -> Option<ChipKey> {
        self.physical_map.get(&key).copied()
    }
    
    fn map_logical_key(&self, key: &LogicalKey) -> Option<ChipKey> {
        self.logical_map.get(key).copied()
    }

    fn map_virtual_keycode(&self, key: VirtualKeyCode) -> Option<ChipKey> {
        self.virtual_keycode_map.get(&key).copied()
    }
    
    fn get_physical_mappings(&self) -> Vec<(PhysicalKey, ChipKey)> {
        self.physical_map.iter().map(|(&k, &v)| (k, v)).collect()
    }
    
    fn name(&self) -> &str {
        "Alternative QWERTY"
    }
    
    fn description(&self) -> &str {
        "Alternative QWERTY keyboard layout"
    }
}

/// Custom key mapper that allows user-defined mappings.
pub struct CustomMapper {
    physical_map: HashMap<PhysicalKey, ChipKey>,
    logical_map: HashMap<LogicalKey, ChipKey>,
    virtual_keycode_map: HashMap<VirtualKeyCode, ChipKey>,
    name: String,
    description: String,
}

impl CustomMapper {
    /// Creates a new custom mapper.
    pub fn new(name: String, description: String) -> Self {
        Self {
            physical_map: HashMap::new(),
            logical_map: HashMap::new(),
            virtual_keycode_map: HashMap::new(),
            name,
            description,
        }
    }
    
    /// Adds a physical key mapping.
    pub fn add_physical_mapping(&mut self, physical: PhysicalKey, chip: ChipKey) {
        self.physical_map.insert(physical, chip);
    }
    
    /// Adds a logical key mapping.
    pub fn add_logical_mapping(&mut self, logical: LogicalKey, chip: ChipKey) {
        self.logical_map.insert(logical, chip);
    }

    /// Adds a virtual keycode mapping.
    pub fn add_virtual_keycode_mapping(&mut self, virtual_keycode: VirtualKeyCode, chip: ChipKey) {
        self.virtual_keycode_map.insert(virtual_keycode, chip);
    }
    
    /// Removes a physical key mapping.
    pub fn remove_physical_mapping(&mut self, physical: PhysicalKey) {
        self.physical_map.remove(&physical);
    }
    
    /// Removes a logical key mapping.
    pub fn remove_logical_mapping(&mut self, logical: &LogicalKey) {
        self.logical_map.remove(logical);
    }

    /// Removes a virtual keycode mapping.
    pub fn remove_virtual_keycode_mapping(&mut self, virtual_keycode: VirtualKeyCode) {
        self.virtual_keycode_map.remove(&virtual_keycode);
    }
    
    /// Clears all mappings.
    pub fn clear_mappings(&mut self) {
        self.physical_map.clear();
        self.logical_map.clear();
        self.virtual_keycode_map.clear();
    }
    
    /// Loads mappings from a QWERTY mapper.
    pub fn load_from_qwerty(&mut self) {
        let qwerty = QwertyMapper::new();
        self.physical_map = qwerty.physical_map;
        self.logical_map = qwerty.logical_map;
        self.virtual_keycode_map = qwerty.virtual_keycode_map;
    }
}

impl KeyMapper for CustomMapper {
    fn map_key_event(&self, event: &KeyboardEvent) -> Option<ChipKey> {
        self.map_logical_key(&event.logical_key)
            .or_else(|| self.map_physical_key(event.physical_key))
    }
    
    fn map_physical_key(&self, key: PhysicalKey) -> Option<ChipKey> {
        self.physical_map.get(&key).copied()
    }
    
    fn map_logical_key(&self, key: &LogicalKey) -> Option<ChipKey> {
        self.logical_map.get(key).copied()
    }

    fn map_virtual_keycode(&self, key: VirtualKeyCode) -> Option<ChipKey> {
        self.virtual_keycode_map.get(&key).copied()
    }
    
    fn get_physical_mappings(&self) -> Vec<(PhysicalKey, ChipKey)> {
        self.physical_map.iter().map(|(&k, &v)| (k, v)).collect()
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        &self.description
    }
}

/// Creates a mapper from a configuration string.
pub fn create_mapper_from_config(config: &str) -> Result<Box<dyn KeyMapper>, String> {
    match config.to_lowercase().as_str() {
        "qwerty" | "standard" => Ok(Box::new(QwertyMapper::new())),
        "alternative" | "alt" => Ok(Box::new(AlternativeQwertyMapper::new())),
        _ => Err(format!("Unknown mapper configuration: {}", config)),
    }
}

/// Gets all available mapper names.
pub fn get_available_mappers() -> Vec<&'static str> {
    vec!["qwerty", "alternative"]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hardware::ChipKey;
    
    #[test]
    fn test_qwerty_mapper_creation() {
        let mapper = QwertyMapper::new();
        
        assert_eq!(mapper.name(), "QWERTY");
        assert!(!mapper.description().is_empty());
    }
    
    #[test]
    fn test_qwerty_physical_mappings() {
        let mapper = QwertyMapper::new();
        
        assert_eq!(mapper.map_physical_key(PhysicalKey::Key1), Some(ChipKey::Key1));
        assert_eq!(mapper.map_physical_key(PhysicalKey::KeyQ), Some(ChipKey::Key4));
        assert_eq!(mapper.map_physical_key(PhysicalKey::KeyA), Some(ChipKey::Key7));
        assert_eq!(mapper.map_physical_key(PhysicalKey::KeyZ), Some(ChipKey::KeyA));
        assert_eq!(mapper.map_physical_key(PhysicalKey::KeyX), Some(ChipKey::Key0));
        assert_eq!(mapper.map_physical_key(PhysicalKey::KeyV), Some(ChipKey::KeyF));
    }
    
    #[test]
    fn test_qwerty_logical_mappings() {
        let mapper = QwertyMapper::new();
        
        assert_eq!(mapper.map_logical_key(&LogicalKey::Character('1')), Some(ChipKey::Key1));
        assert_eq!(mapper.map_logical_key(&LogicalKey::Character('q')), Some(ChipKey::Key4));
        assert_eq!(mapper.map_logical_key(&LogicalKey::Character('Q')), Some(ChipKey::Key4));
        assert_eq!(mapper.map_logical_key(&LogicalKey::Character('a')), Some(ChipKey::Key7));
        assert_eq!(mapper.map_logical_key(&LogicalKey::Character('x')), Some(ChipKey::Key0));
        assert_eq!(mapper.map_logical_key(&LogicalKey::Character('v')), Some(ChipKey::KeyF));
    }
    
    #[test]
    fn test_qwerty_key_event_mapping() {
        let mapper = QwertyMapper::new();
        
        let event = KeyboardEvent {
            physical_key: PhysicalKey::Key1,
            logical_key: LogicalKey::Character('1'),
            state: crate::input::keyboard::KeyState::Pressed,
            is_repeat: false,
            timestamp: std::time::Instant::now(),
        };
        
        assert_eq!(mapper.map_key_event(&event), Some(ChipKey::Key1));
    }
    
    #[test]
    fn test_qwerty_all_mappings() {
        let mapper = QwertyMapper::new();
        let mappings = mapper.get_physical_mappings();
        
        // Should have 16 mappings (one for each Chip-8 key)
        assert_eq!(mappings.len(), 16);
        
        // Check that all Chip-8 keys are mapped
        let chip_keys: std::collections::HashSet<_> = mappings.iter().map(|(_, chip)| *chip).collect();
        assert_eq!(chip_keys.len(), 16);
        
        for expected_key in ChipKey::all_keys() {
            assert!(chip_keys.contains(&expected_key));
        }
    }
    
    #[test]
    fn test_custom_mapper() {
        let mut mapper = CustomMapper::new(
            "Test Mapper".to_string(),
            "A test mapper".to_string(),
        );
        
        assert_eq!(mapper.name(), "Test Mapper");
        assert_eq!(mapper.description(), "A test mapper");
        
        // Initially no mappings
        assert_eq!(mapper.map_physical_key(PhysicalKey::Key1), None);
        
        // Add a mapping
        mapper.add_physical_mapping(PhysicalKey::Key1, ChipKey::Key1);
        assert_eq!(mapper.map_physical_key(PhysicalKey::Key1), Some(ChipKey::Key1));
        
        // Remove mapping
        mapper.remove_physical_mapping(PhysicalKey::Key1);
        assert_eq!(mapper.map_physical_key(PhysicalKey::Key1), None);
        
        // Load from QWERTY
        mapper.load_from_qwerty();
        assert_eq!(mapper.map_physical_key(PhysicalKey::Key1), Some(ChipKey::Key1));
    }
    
    #[test]
    fn test_mapper_creation_from_config() {
        let qwerty = create_mapper_from_config("qwerty").unwrap();
        assert_eq!(qwerty.name(), "QWERTY");
        
        let alt = create_mapper_from_config("alternative").unwrap();
        assert_eq!(alt.name(), "Alternative QWERTY");
        
        let invalid = create_mapper_from_config("invalid");
        assert!(invalid.is_err());
    }
    
    #[test]
    fn test_available_mappers() {
        let mappers = get_available_mappers();
        assert!(!mappers.is_empty());
        assert!(mappers.contains(&"qwerty"));
        assert!(mappers.contains(&"alternative"));
    }
}