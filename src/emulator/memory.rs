//! Memory management for the Chip-8 emulator.
//!
//! This module implements the 4KB memory system with proper bounds checking,
//! font data initialization, and ROM loading functionality.

use crate::error::{EmulatorError, Result};

/// Total memory size for Chip-8 system (4KB).
pub const MEMORY_SIZE: usize = 4096;

/// Starting address for most programs (512 bytes).
pub const PROGRAM_START: u16 = 0x200;

/// Starting address for ETI 660 programs.
pub const ETI_PROGRAM_START: u16 = 0x600;

/// Starting address for font data.
pub const FONT_START: u16 = 0x50;

/// Size of the font data in bytes.
pub const FONT_SIZE: usize = 80;

/// Maximum ROM size (MEMORY_SIZE - PROGRAM_START).
pub const MAX_ROM_SIZE: usize = MEMORY_SIZE - PROGRAM_START as usize;

/// Built-in hexadecimal font set (0-F).
/// Each character is 4 pixels wide and 5 pixels tall.
const FONT_SET: [u8; FONT_SIZE] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

/// Chip-8 memory system.
/// 
/// The memory layout is:
/// - 0x000-0x1FF: Reserved for interpreter (font data at 0x50-0x9F)
/// - 0x200-0xFFF: Program area (most ROMs start here)
/// - 0x600-0xFFF: ETI 660 program area (some ROMs start here)
pub struct Memory {
    /// Raw memory data.
    data: [u8; MEMORY_SIZE],
}

impl Memory {
    /// Creates a new memory instance with font data pre-loaded.
    pub fn new() -> Self {
        let mut memory = Self {
            data: [0; MEMORY_SIZE],
        };
        
        // Load font data into memory
        memory.load_font_data();
        
        memory
    }

    /// Loads the built-in font data into memory at the standard location.
    fn load_font_data(&mut self) {
        let start = FONT_START as usize;
        let end = start + FONT_SIZE;
        self.data[start..end].copy_from_slice(&FONT_SET);
    }

    /// Reads a byte from memory at the specified address.
    /// 
    /// # Arguments
    /// * `address` - The memory address to read from (0x000-0xFFF)
    /// 
    /// # Returns
    /// The byte value at the specified address, or an error if the address is invalid.
    pub fn read_byte(&self, address: u16) -> Result<u8> {
        let addr = address as usize;
        
        if addr >= MEMORY_SIZE {
            return Err(EmulatorError::InvalidMemoryAccess { address });
        }
        
        Ok(self.data[addr])
    }

    /// Writes a byte to memory at the specified address.
    /// 
    /// # Arguments
    /// * `address` - The memory address to write to (0x000-0xFFF)
    /// * `value` - The byte value to write
    /// 
    /// # Returns
    /// Ok(()) on success, or an error if the address is invalid.
    pub fn write_byte(&mut self, address: u16, value: u8) -> Result<()> {
        let addr = address as usize;
        
        if addr >= MEMORY_SIZE {
            return Err(EmulatorError::InvalidMemoryAccess { address });
        }
        
        self.data[addr] = value;
        Ok(())
    }

    /// Reads a 16-bit word from memory at the specified address.
    /// 
    /// Chip-8 uses big-endian byte order (most significant byte first).
    /// 
    /// # Arguments
    /// * `address` - The memory address to read from (0x000-0xFFE)
    /// 
    /// # Returns
    /// The 16-bit word value, or an error if the address is invalid.
    pub fn read_word(&self, address: u16) -> Result<u16> {
        let high_byte = self.read_byte(address)?;
        let low_byte = self.read_byte(address + 1)?;
        
        Ok((high_byte as u16) << 8 | low_byte as u16)
    }

    /// Writes a 16-bit word to memory at the specified address.
    /// 
    /// Chip-8 uses big-endian byte order (most significant byte first).
    /// 
    /// # Arguments
    /// * `address` - The memory address to write to (0x000-0xFFE)
    /// * `value` - The 16-bit word value to write
    /// 
    /// # Returns
    /// Ok(()) on success, or an error if the address is invalid.
    pub fn write_word(&mut self, address: u16, value: u16) -> Result<()> {
        let high_byte = (value >> 8) as u8;
        let low_byte = (value & 0xFF) as u8;
        
        self.write_byte(address, high_byte)?;
        self.write_byte(address + 1, low_byte)?;
        
        Ok(())
    }

    /// Loads ROM data into memory starting at the standard program address.
    /// 
    /// # Arguments
    /// * `rom_data` - The ROM data to load
    /// 
    /// # Returns
    /// Ok(()) on success, or an error if the ROM is too large or empty.
    pub fn load_rom(&mut self, rom_data: &[u8]) -> Result<()> {
        self.load_rom_at(rom_data, PROGRAM_START)
    }

    /// Loads ROM data into memory starting at the specified address.
    /// 
    /// # Arguments
    /// * `rom_data` - The ROM data to load
    /// * `start_address` - The address to start loading at
    /// 
    /// # Returns
    /// Ok(()) on success, or an error if the ROM is too large or empty.
    pub fn load_rom_at(&mut self, rom_data: &[u8], start_address: u16) -> Result<()> {
        if rom_data.is_empty() {
            return Err(EmulatorError::RomEmpty);
        }

        let start = start_address as usize;
        let available_space = MEMORY_SIZE - start;

        if rom_data.len() > available_space {
            return Err(EmulatorError::RomTooLarge { 
                size: rom_data.len(), 
                max_size: available_space 
            });
        }

        // Clear existing program area
        for addr in start..MEMORY_SIZE {
            self.data[addr] = 0;
        }

        // Load ROM data
        let end = start + rom_data.len();
        self.data[start..end].copy_from_slice(rom_data);

        Ok(())
    }

    /// Clears all memory except font data.
    pub fn clear(&mut self) {
        // Clear everything
        self.data.fill(0);
        
        // Reload font data
        self.load_font_data();
    }

    /// Gets the address of a font character.
    /// 
    /// # Arguments
    /// * `character` - The hexadecimal character (0x0-0xF)
    /// 
    /// # Returns
    /// The memory address of the font character, or an error if invalid.
    pub fn get_font_address(&self, character: u8) -> Result<u16> {
        if character > 0xF {
            return Err(EmulatorError::InvalidMemoryAccess { 
                address: character as u16 
            });
        }
        
        Ok(FONT_START + (character as u16 * 5))
    }

    /// Gets a slice of memory for reading.
    /// 
    /// # Arguments
    /// * `start` - Starting address
    /// * `length` - Number of bytes to read
    /// 
    /// # Returns
    /// A slice of the memory data, or an error if the range is invalid.
    pub fn get_slice(&self, start: u16, length: usize) -> Result<&[u8]> {
        let start_addr = start as usize;
        let end_addr = start_addr + length;
        
        if end_addr > MEMORY_SIZE {
            return Err(EmulatorError::InvalidMemoryAccess { address: start });
        }
        
        Ok(&self.data[start_addr..end_addr])
    }

    /// Copies data from one memory location to another.
    /// 
    /// # Arguments
    /// * `source` - Source address
    /// * `dest` - Destination address  
    /// * `length` - Number of bytes to copy
    /// 
    /// # Returns
    /// Ok(()) on success, or an error if addresses are invalid.
    pub fn copy(&mut self, source: u16, dest: u16, length: usize) -> Result<()> {
        let src_start = source as usize;
        let dst_start = dest as usize;
        let src_end = src_start + length;
        let dst_end = dst_start + length;
        
        if src_end > MEMORY_SIZE {
            return Err(EmulatorError::InvalidMemoryAccess { address: source });
        }
        
        if dst_end > MEMORY_SIZE {
            return Err(EmulatorError::InvalidMemoryAccess { address: dest });
        }
        
        // Use a temporary buffer to handle overlapping regions safely
        let temp: Vec<u8> = self.data[src_start..src_end].to_vec();
        self.data[dst_start..dst_end].copy_from_slice(&temp);
        
        Ok(())
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let memory = Memory::new();
        
        // Check that font data is loaded
        assert_eq!(memory.read_byte(FONT_START).unwrap(), 0xF0); // First byte of '0'
        assert_eq!(memory.read_byte(FONT_START + 5).unwrap(), 0x20); // First byte of '1'
    }

    #[test]
    fn test_byte_read_write() {
        let mut memory = Memory::new();
        
        // Test valid address
        memory.write_byte(0x200, 0xAB).unwrap();
        assert_eq!(memory.read_byte(0x200).unwrap(), 0xAB);
        
        // Test invalid address
        assert!(memory.read_byte(0x1000).is_err());
        assert!(memory.write_byte(0x1000, 0xFF).is_err());
    }

    #[test]
    fn test_word_read_write() {
        let mut memory = Memory::new();
        
        // Test big-endian word storage
        memory.write_word(0x200, 0x1234).unwrap();
        assert_eq!(memory.read_byte(0x200).unwrap(), 0x12); // High byte
        assert_eq!(memory.read_byte(0x201).unwrap(), 0x34); // Low byte
        assert_eq!(memory.read_word(0x200).unwrap(), 0x1234);
    }

    #[test]
    fn test_rom_loading() {
        let mut memory = Memory::new();
        let rom_data = vec![0x12, 0x34, 0x56, 0x78];
        
        memory.load_rom(&rom_data).unwrap();
        
        assert_eq!(memory.read_byte(PROGRAM_START).unwrap(), 0x12);
        assert_eq!(memory.read_byte(PROGRAM_START + 1).unwrap(), 0x34);
        assert_eq!(memory.read_byte(PROGRAM_START + 2).unwrap(), 0x56);
        assert_eq!(memory.read_byte(PROGRAM_START + 3).unwrap(), 0x78);
    }

    #[test]
    fn test_rom_too_large() {
        let mut memory = Memory::new();
        let large_rom = vec![0xFF; MAX_ROM_SIZE + 1];
        
        let result = memory.load_rom(&large_rom);
        assert!(matches!(result, Err(EmulatorError::RomTooLarge { .. })));
    }

    #[test]
    fn test_empty_rom() {
        let mut memory = Memory::new();
        let empty_rom = vec![];
        
        let result = memory.load_rom(&empty_rom);
        assert!(matches!(result, Err(EmulatorError::RomEmpty)));
    }

    #[test]
    fn test_font_addresses() {
        let memory = Memory::new();
        
        // Test valid characters
        assert_eq!(memory.get_font_address(0x0).unwrap(), FONT_START);
        assert_eq!(memory.get_font_address(0xF).unwrap(), FONT_START + 0xF * 5);
        
        // Test invalid character
        assert!(memory.get_font_address(0x10).is_err());
    }

    #[test]
    fn test_memory_slice() {
        let memory = Memory::new();
        
        // Test valid slice
        let slice = memory.get_slice(FONT_START, 5).unwrap();
        assert_eq!(slice.len(), 5);
        assert_eq!(slice[0], 0xF0); // First byte of '0'
        
        // Test invalid slice (extends beyond memory)
        assert!(memory.get_slice(0xFFF, 2).is_err());
    }

    #[test]
    fn test_memory_copy() {
        let mut memory = Memory::new();
        
        // Set up source data
        memory.write_byte(0x300, 0xAA).unwrap();
        memory.write_byte(0x301, 0xBB).unwrap();
        memory.write_byte(0x302, 0xCC).unwrap();
        
        // Copy data
        memory.copy(0x300, 0x400, 3).unwrap();
        
        // Verify copy
        assert_eq!(memory.read_byte(0x400).unwrap(), 0xAA);
        assert_eq!(memory.read_byte(0x401).unwrap(), 0xBB);
        assert_eq!(memory.read_byte(0x402).unwrap(), 0xCC);
    }

    #[test]
    fn test_memory_clear() {
        let mut memory = Memory::new();
        
        // Write some data
        memory.write_byte(0x200, 0xFF).unwrap();
        memory.write_byte(0x300, 0xAA).unwrap();
        
        // Clear memory
        memory.clear();
        
        // Check that program area is cleared but font data remains
        assert_eq!(memory.read_byte(0x200).unwrap(), 0x00);
        assert_eq!(memory.read_byte(0x300).unwrap(), 0x00);
        assert_eq!(memory.read_byte(FONT_START).unwrap(), 0xF0); // Font data preserved
    }

    #[test]
    fn test_font_data_integrity() {
        let memory = Memory::new();
        
        // Verify all font characters are loaded correctly
        for i in 0..16 {
            let addr = memory.get_font_address(i).unwrap();
            let font_slice = memory.get_slice(addr, 5).unwrap();
            
            // Each character should have 5 bytes
            assert_eq!(font_slice.len(), 5);
            
            // Verify against original font set
            let start_idx = (i * 5) as usize;
            for j in 0..5 {
                assert_eq!(font_slice[j], FONT_SET[start_idx + j]);
            }
        }
    }
}