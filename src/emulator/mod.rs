//! Emulator core module.
//!
//! This module contains the core emulation components for the Chip-8 system,
//! including CPU, memory, registers, stack, and timers.

pub mod cpu;
pub mod memory;
pub mod registers;
pub mod stack;
pub mod timers;

// Re-export commonly used types
pub use cpu::{Cpu, CpuState};
pub use memory::{Memory, MEMORY_SIZE, PROGRAM_START, FONT_START};
pub use registers::{Registers, NUM_REGISTERS, FLAG_REGISTER};
pub use stack::{Stack, STACK_SIZE};
pub use timers::{Timers, TIMER_FREQUENCY};