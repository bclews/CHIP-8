//! Test utilities for the Chip-8 emulator.
//!
//! This module provides common test setup and utility functions to reduce
//! code duplication across test modules.

#[cfg(test)]
pub mod helpers {
    use crate::emulator::{Cpu, Memory, Registers, Stack};
    use crate::error::Result;

    /// Creates a new CPU with default initialization for testing.
    pub fn create_test_cpu() -> Cpu {
        Cpu::new()
    }

    /// Creates a CPU and loads test ROM data.
    pub fn create_cpu_with_rom(rom_data: &[u8]) -> Result<Cpu> {
        let mut cpu = create_test_cpu();
        cpu.load_rom(rom_data)?;
        Ok(cpu)
    }

    /// Creates a simple test ROM with the given instructions.
    pub fn create_test_rom(instructions: &[u16]) -> Vec<u8> {
        let mut rom = Vec::new();
        for instruction in instructions {
            rom.push((instruction >> 8) as u8);
            rom.push((instruction & 0xFF) as u8);
        }
        rom
    }

    /// Creates a new memory instance for testing.
    pub fn create_test_memory() -> Memory {
        Memory::new()
    }

    /// Creates a new registers instance for testing.
    pub fn create_test_registers() -> Registers {
        Registers::new()
    }

    /// Creates a new stack instance for testing.
    pub fn create_test_stack() -> Stack {
        Stack::new()
    }

    /// Sets up a CPU with specific values in memory for testing.
    pub fn setup_cpu_with_memory_values(_addresses: &[(u16, u8)]) -> Result<Cpu> {
        let cpu = create_test_cpu();
        // Note: We would need access to mutable memory for this to work properly
        // For now, this is a placeholder that shows the intended API
        Ok(cpu)
    }

    /// Helper to run multiple CPU cycles.
    pub fn run_cycles(cpu: &mut Cpu, count: u32) -> Result<()> {
        for _ in 0..count {
            cpu.cycle()?;
        }
        Ok(())
    }

    /// Asserts that memory contains expected value.
    pub fn assert_memory_value(cpu: &Cpu, address: u16, expected: u8) {
        let actual = cpu.get_memory().read_byte(address).unwrap();
        assert_eq!(actual, expected, "Memory at 0x{:04X} should be 0x{:02X}, but was 0x{:02X}", address, expected, actual);
    }

    /// Asserts that CPU state matches expected values.
    pub fn assert_cpu_state(cpu: &Cpu, expected_pc: u16, expected_i: u16, expected_sp: u8) {
        let state = cpu.get_state();
        assert_eq!(state.pc, expected_pc, "PC should be 0x{:04X}, but was 0x{:04X}", expected_pc, state.pc);
        assert_eq!(state.i, expected_i, "I register should be 0x{:04X}, but was 0x{:04X}", expected_i, state.i);
        assert_eq!(state.sp, expected_sp, "SP should be {}, but was {}", expected_sp, state.sp);
    }

    /// Asserts that a V register has the expected value.
    pub fn assert_v_register(cpu: &Cpu, reg: u8, expected: u8) {
        let state = cpu.get_state();
        let actual = state.v[reg as usize];
        assert_eq!(actual, expected, "Register V{:X} should be 0x{:02X}, but was 0x{:02X}", reg, expected, actual);
    }

    /// Common assertions for CPU state after reset.
    pub fn assert_cpu_reset_state(cpu: &Cpu) {
        let state = cpu.get_state();
        assert_eq!(state.pc, 0x200);
        assert_eq!(state.i, 0);
        assert_eq!(state.sp, 0);
        for (i, &value) in state.v.iter().enumerate() {
            assert_eq!(value, 0, "Register V{:X} should be 0 after reset", i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::helpers::*;

    #[test]
    fn test_create_test_cpu() {
        let cpu = create_test_cpu();
        assert_cpu_reset_state(&cpu);
    }

    #[test]
    fn test_create_test_rom() {
        let instructions = [0x1234, 0x5678, 0xABCD];
        let rom = create_test_rom(&instructions);
        assert_eq!(rom, vec![0x12, 0x34, 0x56, 0x78, 0xAB, 0xCD]);
    }

    #[test]
    fn test_cpu_with_rom() {
        let rom = create_test_rom(&[0x6123]); // Load 0x23 into V1
        let cpu = create_cpu_with_rom(&rom).unwrap();
        assert_eq!(cpu.get_memory().read_byte(0x200).unwrap(), 0x61);
        assert_eq!(cpu.get_memory().read_byte(0x201).unwrap(), 0x23);
    }

    #[test]
    fn test_assertion_helpers() {
        let cpu = create_test_cpu();
        
        // Test initial state assertions
        assert_cpu_reset_state(&cpu);
        assert_v_register(&cpu, 0, 0);
        assert_cpu_state(&cpu, 0x200, 0, 0);
    }

    #[test] 
    fn test_run_cycles() {
        let mut cpu = create_test_cpu();
        
        // This should not error on a fresh CPU
        run_cycles(&mut cpu, 0).unwrap();
    }
}