//! Register management for the Chip-8 emulator.
//!
//! This module implements the Chip-8 register system including general-purpose
//! registers V0-VF, the index register I, program counter, and stack pointer.

use crate::error::{EmulatorError, Result};

/// Number of general-purpose registers (V0-VF).
pub const NUM_REGISTERS: usize = 16;

/// Index of the flag register (VF).
pub const FLAG_REGISTER: usize = 0xF;

/// Chip-8 register set.
///
/// The Chip-8 has:
/// - 16 general-purpose 8-bit registers (V0-VF)
/// - VF is used as a flag register for arithmetic and graphics operations
/// - I is a 16-bit index register used for memory operations
/// - PC is the program counter (16-bit)
/// - SP is the stack pointer (8-bit)
pub struct Registers {
    /// General-purpose registers V0-VF.
    v: [u8; NUM_REGISTERS],

    /// Index register (I) - used for memory addresses.
    i: u16,

    /// Program counter - points to the current instruction.
    pc: u16,

    /// Stack pointer - points to the current stack level.
    sp: u8,
}

impl Registers {
    /// Creates a new register set with default values.
    pub fn new() -> Self {
        Self {
            v: [0; NUM_REGISTERS],
            i: 0,
            pc: 0x200, // Standard program start address
            sp: 0,
        }
    }

    /// Resets all registers to their initial state.
    pub fn reset(&mut self) {
        self.v.fill(0);
        self.i = 0;
        self.pc = 0x200;
        self.sp = 0;
    }

    /// Gets the value of a general-purpose register.
    ///
    /// # Arguments
    /// * `index` - Register index (0x0-0xF)
    ///
    /// # Returns
    /// The register value, or an error if the index is invalid.
    pub fn get_v(&self, index: u8) -> Result<u8> {
        if index as usize >= NUM_REGISTERS {
            return Err(EmulatorError::InvalidRegister { index });
        }

        Ok(self.v[index as usize])
    }

    /// Sets the value of a general-purpose register.
    ///
    /// # Arguments
    /// * `index` - Register index (0x0-0xF)
    /// * `value` - Value to set
    ///
    /// # Returns
    /// Ok(()) on success, or an error if the index is invalid.
    pub fn set_v(&mut self, index: u8, value: u8) -> Result<()> {
        if index as usize >= NUM_REGISTERS {
            return Err(EmulatorError::InvalidRegister { index });
        }

        self.v[index as usize] = value;
        Ok(())
    }

    /// Gets the flag register (VF) value.
    pub fn get_flag(&self) -> u8 {
        self.v[FLAG_REGISTER]
    }

    /// Sets the flag register (VF) value.
    ///
    /// # Arguments
    /// * `value` - Flag value (typically 0 or 1)
    pub fn set_flag(&mut self, value: u8) {
        self.v[FLAG_REGISTER] = value;
    }

    /// Gets the index register (I) value.
    pub fn get_i(&self) -> u16 {
        self.i
    }

    /// Sets the index register (I) value.
    ///
    /// # Arguments
    /// * `value` - Index register value
    pub fn set_i(&mut self, value: u16) {
        self.i = value;
    }

    /// Gets the program counter (PC) value.
    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    /// Sets the program counter (PC) value.
    ///
    /// # Arguments
    /// * `value` - Program counter value
    pub fn set_pc(&mut self, value: u16) {
        self.pc = value;
    }

    /// Increments the program counter by 2 (standard instruction size).
    pub fn increment_pc(&mut self) {
        self.pc = self.pc.wrapping_add(2);
    }

    /// Skips the next instruction by incrementing PC by 2.
    pub fn skip_instruction(&mut self) {
        self.pc = self.pc.wrapping_add(2);
    }

    /// Gets the stack pointer (SP) value.
    pub fn get_sp(&self) -> u8 {
        self.sp
    }

    /// Sets the stack pointer (SP) value.
    ///
    /// # Arguments
    /// * `value` - Stack pointer value
    pub fn set_sp(&mut self, value: u8) {
        self.sp = value;
    }

    /// Increments the stack pointer.
    ///
    /// # Returns
    /// Ok(()) on success, or StackOverflow error if SP would exceed limits.
    pub fn increment_sp(&mut self) -> Result<()> {
        if self.sp >= 15 {
            return Err(EmulatorError::StackOverflow);
        }

        self.sp += 1;
        Ok(())
    }

    /// Decrements the stack pointer.
    ///
    /// # Returns
    /// Ok(()) on success, or StackUnderflow error if SP would go below 0.
    pub fn decrement_sp(&mut self) -> Result<()> {
        if self.sp == 0 {
            return Err(EmulatorError::StackUnderflow);
        }

        self.sp -= 1;
        Ok(())
    }

    /// Performs addition with carry flag setting.
    ///
    /// # Arguments
    /// * `vx` - First register index
    /// * `vy` - Second register index
    ///
    /// # Returns
    /// Ok(()) on success, or an error if register indices are invalid.
    pub fn add_with_carry(&mut self, vx: u8, vy: u8) -> Result<()> {
        let val_x = self.get_v(vx)?;
        let val_y = self.get_v(vy)?;

        let (result, carry) = val_x.overflowing_add(val_y);

        self.set_v(vx, result)?;
        self.set_flag(if carry { 1 } else { 0 });

        Ok(())
    }

    /// Performs subtraction with borrow flag setting.
    ///
    /// # Arguments
    /// * `vx` - First register index (minuend)
    /// * `vy` - Second register index (subtrahend)
    ///
    /// # Returns
    /// Ok(()) on success, or an error if register indices are invalid.
    pub fn sub_with_borrow(&mut self, vx: u8, vy: u8) -> Result<()> {
        let val_x = self.get_v(vx)?;
        let val_y = self.get_v(vy)?;

        let (result, borrow) = val_x.overflowing_sub(val_y);

        self.set_v(vx, result)?;
        self.set_flag(if borrow { 0 } else { 1 }); // Chip-8 uses inverted borrow flag

        Ok(())
    }

    /// Performs reverse subtraction with borrow flag setting (VY - VX).
    ///
    /// # Arguments
    /// * `vx` - First register index (result destination)
    /// * `vy` - Second register index (minuend)
    ///
    /// # Returns
    /// Ok(()) on success, or an error if register indices are invalid.
    pub fn sub_reverse_with_borrow(&mut self, vx: u8, vy: u8) -> Result<()> {
        let val_x = self.get_v(vx)?;
        let val_y = self.get_v(vy)?;

        let (result, borrow) = val_y.overflowing_sub(val_x);

        self.set_v(vx, result)?;
        self.set_flag(if borrow { 0 } else { 1 }); // Chip-8 uses inverted borrow flag

        Ok(())
    }

    /// Performs right shift with carry flag setting.
    ///
    /// # Arguments
    /// * `vx` - Register index to shift
    ///
    /// # Returns
    /// Ok(()) on success, or an error if register index is invalid.
    pub fn shift_right(&mut self, vx: u8) -> Result<()> {
        let val = self.get_v(vx)?;
        let lsb = val & 0x1;

        self.set_v(vx, val >> 1)?;
        self.set_flag(lsb);

        Ok(())
    }

    /// Performs left shift with carry flag setting.
    ///
    /// # Arguments
    /// * `vx` - Register index to shift
    ///
    /// # Returns
    /// Ok(()) on success, or an error if register index is invalid.
    pub fn shift_left(&mut self, vx: u8) -> Result<()> {
        let val = self.get_v(vx)?;
        let msb = (val & 0x80) >> 7;

        self.set_v(vx, val << 1)?;
        self.set_flag(msb);

        Ok(())
    }

    /// Gets all V register values as a slice.
    pub fn get_all_v(&self) -> &[u8] {
        &self.v
    }

    /// Sets multiple V register values from a slice.
    ///
    /// # Arguments
    /// * `start_index` - Starting register index
    /// * `values` - Values to set
    ///
    /// # Returns
    /// Ok(()) on success, or an error if indices would be invalid.
    pub fn set_v_range(&mut self, start_index: u8, values: &[u8]) -> Result<()> {
        let start = start_index as usize;

        if start + values.len() > NUM_REGISTERS {
            return Err(EmulatorError::InvalidRegister {
                index: (start + values.len() - 1) as u8,
            });
        }

        self.v[start..start + values.len()].copy_from_slice(values);
        Ok(())
    }

    /// Gets a range of V register values.
    ///
    /// # Arguments
    /// * `start_index` - Starting register index
    /// * `count` - Number of registers to get
    ///
    /// # Returns
    /// A slice of register values, or an error if indices would be invalid.
    pub fn get_v_range(&self, start_index: u8, count: usize) -> Result<&[u8]> {
        let start = start_index as usize;

        if start + count > NUM_REGISTERS {
            return Err(EmulatorError::InvalidRegister {
                index: (start + count - 1) as u8,
            });
        }

        Ok(&self.v[start..start + count])
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_creation() {
        let registers = Registers::new();

        // Check initial values
        assert_eq!(registers.get_pc(), 0x200);
        assert_eq!(registers.get_sp(), 0);
        assert_eq!(registers.get_i(), 0);
        assert_eq!(registers.get_flag(), 0);

        // Check all V registers are zero
        for i in 0..NUM_REGISTERS {
            assert_eq!(registers.get_v(i as u8).unwrap(), 0);
        }
    }

    #[test]
    fn test_v_register_operations() {
        let mut registers = Registers::new();

        // Test valid register access
        registers.set_v(0x5, 0xAB).unwrap();
        assert_eq!(registers.get_v(0x5).unwrap(), 0xAB);

        // Test invalid register access
        assert!(registers.get_v(0x10).is_err());
        assert!(registers.set_v(0x10, 0xFF).is_err());
    }

    #[test]
    fn test_flag_register() {
        let mut registers = Registers::new();

        registers.set_flag(1);
        assert_eq!(registers.get_flag(), 1);
        assert_eq!(registers.get_v(0xF).unwrap(), 1);

        registers.set_v(0xF, 0).unwrap();
        assert_eq!(registers.get_flag(), 0);
    }

    #[test]
    fn test_index_register() {
        let mut registers = Registers::new();

        registers.set_i(0x1234);
        assert_eq!(registers.get_i(), 0x1234);
    }

    #[test]
    fn test_program_counter() {
        let mut registers = Registers::new();

        assert_eq!(registers.get_pc(), 0x200);

        registers.increment_pc();
        assert_eq!(registers.get_pc(), 0x202);

        registers.skip_instruction();
        assert_eq!(registers.get_pc(), 0x204);

        registers.set_pc(0x300);
        assert_eq!(registers.get_pc(), 0x300);
    }

    #[test]
    fn test_stack_pointer() {
        let mut registers = Registers::new();

        assert_eq!(registers.get_sp(), 0);

        registers.increment_sp().unwrap();
        assert_eq!(registers.get_sp(), 1);

        registers.decrement_sp().unwrap();
        assert_eq!(registers.get_sp(), 0);

        // Test underflow
        assert!(registers.decrement_sp().is_err());

        // Test overflow
        registers.set_sp(15);
        assert!(registers.increment_sp().is_err());
    }

    #[test]
    fn test_add_with_carry() {
        let mut registers = Registers::new();

        // Test normal addition
        registers.set_v(0x1, 0x10).unwrap();
        registers.set_v(0x2, 0x20).unwrap();
        registers.add_with_carry(0x1, 0x2).unwrap();

        assert_eq!(registers.get_v(0x1).unwrap(), 0x30);
        assert_eq!(registers.get_flag(), 0); // No carry

        // Test addition with carry
        registers.set_v(0x1, 0xFF).unwrap();
        registers.set_v(0x2, 0x01).unwrap();
        registers.add_with_carry(0x1, 0x2).unwrap();

        assert_eq!(registers.get_v(0x1).unwrap(), 0x00);
        assert_eq!(registers.get_flag(), 1); // Carry occurred
    }

    #[test]
    fn test_sub_with_borrow() {
        let mut registers = Registers::new();

        // Test normal subtraction
        registers.set_v(0x1, 0x30).unwrap();
        registers.set_v(0x2, 0x10).unwrap();
        registers.sub_with_borrow(0x1, 0x2).unwrap();

        assert_eq!(registers.get_v(0x1).unwrap(), 0x20);
        assert_eq!(registers.get_flag(), 1); // No borrow

        // Test subtraction with borrow
        registers.set_v(0x1, 0x10).unwrap();
        registers.set_v(0x2, 0x20).unwrap();
        registers.sub_with_borrow(0x1, 0x2).unwrap();

        assert_eq!(registers.get_v(0x1).unwrap(), 0xF0);
        assert_eq!(registers.get_flag(), 0); // Borrow occurred
    }

    #[test]
    fn test_shift_operations() {
        let mut registers = Registers::new();

        // Test right shift
        registers.set_v(0x1, 0b10101010).unwrap();
        registers.shift_right(0x1).unwrap();

        assert_eq!(registers.get_v(0x1).unwrap(), 0b01010101);
        assert_eq!(registers.get_flag(), 0); // LSB was 0

        registers.set_v(0x1, 0b10101011).unwrap();
        registers.shift_right(0x1).unwrap();

        assert_eq!(registers.get_v(0x1).unwrap(), 0b01010101);
        assert_eq!(registers.get_flag(), 1); // LSB was 1

        // Test left shift
        registers.set_v(0x1, 0b01010101).unwrap();
        registers.shift_left(0x1).unwrap();

        assert_eq!(registers.get_v(0x1).unwrap(), 0b10101010);
        assert_eq!(registers.get_flag(), 0); // MSB was 0

        registers.set_v(0x1, 0b11010101).unwrap();
        registers.shift_left(0x1).unwrap();

        assert_eq!(registers.get_v(0x1).unwrap(), 0b10101010);
        assert_eq!(registers.get_flag(), 1); // MSB was 1
    }

    #[test]
    fn test_register_range_operations() {
        let mut registers = Registers::new();

        // Test setting range
        let values = [0x11, 0x22, 0x33, 0x44];
        registers.set_v_range(0x5, &values).unwrap();

        assert_eq!(registers.get_v(0x5).unwrap(), 0x11);
        assert_eq!(registers.get_v(0x6).unwrap(), 0x22);
        assert_eq!(registers.get_v(0x7).unwrap(), 0x33);
        assert_eq!(registers.get_v(0x8).unwrap(), 0x44);

        // Test getting range
        let range = registers.get_v_range(0x5, 4).unwrap();
        assert_eq!(range, &[0x11, 0x22, 0x33, 0x44]);

        // Test invalid range
        assert!(registers.set_v_range(0xE, &[1, 2, 3]).is_err());
        assert!(registers.get_v_range(0xE, 3).is_err());
    }

    #[test]
    fn test_reset() {
        let mut registers = Registers::new();

        // Modify registers
        registers.set_v(0x5, 0xFF).unwrap();
        registers.set_i(0x1234);
        registers.set_pc(0x500);
        registers.set_sp(5);

        // Reset
        registers.reset();

        // Check all values are back to initial state
        assert_eq!(registers.get_v(0x5).unwrap(), 0);
        assert_eq!(registers.get_i(), 0);
        assert_eq!(registers.get_pc(), 0x200);
        assert_eq!(registers.get_sp(), 0);
    }

    #[test]
    fn test_pc_wraparound() {
        let mut registers = Registers::new();

        // Test PC wraparound at 16-bit boundary
        registers.set_pc(0xFFFE);
        registers.increment_pc();
        assert_eq!(registers.get_pc(), 0x0000);

        // Test skip instruction wraparound
        registers.set_pc(0xFFFE);
        registers.skip_instruction();
        assert_eq!(registers.get_pc(), 0x0000);
    }
}
