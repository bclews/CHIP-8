//! CPU implementation for the Chip-8 emulator.
//!
//! This module implements the central processing unit that executes Chip-8
//! instructions and manages the system state.

use crate::emulator::{Memory, Registers, Stack, Timers};
use crate::error::{EmulatorError, Result};
use crate::hardware::{Audio, ChipKey, Display, Input};
use rand::{rngs::ThreadRng, Rng};
use std::cell::RefCell;
use std::rc::Rc;

/// CPU state for debugging and serialization.
#[derive(Debug, Clone, PartialEq)]
pub struct CpuState {
    pub pc: u16,
    pub sp: u8,
    pub i: u16,
    pub v: [u8; 16],
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub stack_contents: Vec<u16>,
    pub instruction_count: u64,
}

/// Chip-8 CPU implementation.
///
/// The CPU manages instruction execution, system state, and coordinates
/// with memory, registers, stack, and timers.
pub struct Cpu {
    /// Register set (V0-VF, I, PC, SP).
    registers: Registers,

    /// System memory (4KB).
    memory: Memory,

    /// Call stack (16 levels).
    stack: Stack,

    /// Delay and sound timers.
    timers: Timers,

    /// Random number generator for RND instruction.
    rng: ThreadRng,

    /// Total number of instructions executed.
    instruction_count: u64,

    /// Whether the CPU is waiting for a key press.
    waiting_for_key: bool,

    /// Register to store key press when waiting.
    key_wait_register: u8,

    /// The key being waited for release (None if not waiting for release).
    waiting_for_key_release: Option<u8>,

    /// Display system for rendering.
    display: Option<Box<dyn Display>>,

    /// Audio system for sound.
    audio: Option<Box<dyn Audio>>,

    /// Input system for keyboard handling.
    input: Option<Rc<RefCell<dyn Input>>>,
}

impl Cpu {
    /// Creates a new CPU instance.
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            memory: Memory::new(),
            stack: Stack::new(),
            timers: Timers::new(),
            rng: rand::thread_rng(),
            instruction_count: 0,
            waiting_for_key: false,
            key_wait_register: 0,
            waiting_for_key_release: None,
            display: None,
            audio: None,
            input: None,
        }
    }

    /// Creates a new CPU instance with specific behavior configuration.
    pub fn new_with_config(config: &crate::frontend::EmulatorBehaviorConfig) -> Self {
        // Configure timers if needed
        // Note: Timer frequency configuration would require timer system updates

        Self {
            registers: Registers::new(),
            memory: Memory::new_with_wraparound(config.memory_wraparound),
            stack: Stack::new(),
            timers: Timers::new(),
            rng: rand::thread_rng(),
            instruction_count: 0,
            waiting_for_key: false,
            key_wait_register: 0,
            waiting_for_key_release: None,
            display: None,
            audio: None,
            input: None,
        }
    }

    /// Configures the CPU with behavior settings.
    pub fn configure(&mut self, config: &crate::frontend::EmulatorBehaviorConfig) {
        self.memory.set_wraparound(config.memory_wraparound);
        // Additional configuration can be added here
    }

    /// Sets the display system.
    pub fn set_display(&mut self, display: Box<dyn Display>) {
        self.display = Some(display);
    }

    /// Sets the audio system.
    pub fn set_audio(&mut self, audio: Box<dyn Audio>) {
        self.audio = Some(audio);
    }

    /// Sets the input system.
    pub fn set_input(&mut self, input: Rc<RefCell<dyn Input>>) {
        self.input = Some(input);
    }

    /// Resets the CPU to initial state.
    pub fn reset(&mut self) {
        self.registers.reset();
        self.memory.clear();
        self.stack.reset();
        self.timers.reset();
        self.instruction_count = 0;
        self.waiting_for_key = false;
        self.key_wait_register = 0;
        self.waiting_for_key_release = None;
    }

    /// Loads a ROM into memory.
    ///
    /// # Arguments
    /// * `rom_data` - The ROM data to load
    ///
    /// # Returns
    /// Ok(()) on success, or an error if the ROM is invalid.
    pub fn load_rom(&mut self, rom_data: &[u8]) -> Result<()> {
        self.memory.load_rom(rom_data)?;
        self.registers
            .set_pc(crate::emulator::memory::PROGRAM_START);
        Ok(())
    }

    /// Loads a ROM at a specific address (for ETI 660 compatibility).
    ///
    /// # Arguments
    /// * `rom_data` - The ROM data to load
    /// * `start_address` - The address to load the ROM at
    ///
    /// # Returns
    /// Ok(()) on success, or an error if the ROM is invalid.
    pub fn load_rom_at(&mut self, rom_data: &[u8], start_address: u16) -> Result<()> {
        self.memory.load_rom_at(rom_data, start_address)?;
        self.registers.set_pc(start_address);
        Ok(())
    }

    /// Executes a single CPU cycle.
    ///
    /// This fetches, decodes, and executes one instruction, then updates timers.
    ///
    /// # Returns
    /// Ok(()) on successful execution, or an error if something goes wrong.
    pub fn cycle(&mut self) -> Result<()> {
        // Update timers first
        self.timers.update();

        // Handle audio based on sound timer
        if let Some(audio) = &mut self.audio {
            if self.timers.get_sound_timer() > 0 {
                if let Err(e) = audio.play_beep() {
                    log::warn!("Failed to play audio beep: {}", e);
                }
            } else if let Err(e) = audio.stop_beep() {
                log::warn!("Failed to stop audio beep: {}", e);
            }
        }

        // If waiting for key press or release, check for input but don't execute instructions
        if self.waiting_for_key {
            if let Some(input) = &self.input {
                if let Some(waiting_key) = self.waiting_for_key_release {
                    // We're waiting for a specific key to be released
                    let chip_key = crate::hardware::input::ChipKey::from_u8(waiting_key);
                    if let Some(key) = chip_key {
                        if !input.borrow().is_key_pressed(key) {
                            // Key has been released, store it and continue
                            self.registers.set_v(self.key_wait_register, waiting_key)?;
                            self.waiting_for_key = false;
                            self.waiting_for_key_release = None;
                        }
                    }
                } else {
                    // We're waiting for any key to be pressed
                    if let Some(pressed_key) = input.borrow().get_first_pressed_key() {
                        // Key pressed, now wait for it to be released
                        self.waiting_for_key_release = Some(pressed_key.to_u8());
                    }
                }
            }
            return Ok(());
        }

        // Fetch instruction
        let pc = self.registers.get_pc();
        let instruction = self.memory.read_word(pc)?;

        // Increment PC before execution (some instructions modify PC)
        self.registers.increment_pc();

        // Decode and execute instruction
        log::debug!("PC: {:#04x}, Instruction: {:#04x}", pc, instruction);
        self.execute_instruction(instruction)?;

        // Increment instruction counter
        self.instruction_count += 1;

        Ok(())
    }

    /// Executes a single instruction.
    ///
    /// # Arguments
    /// * `instruction` - The 16-bit instruction to execute
    ///
    /// # Returns
    /// Ok(()) on successful execution, or an error for unknown instructions.
    fn execute_instruction(&mut self, instruction: u16) -> Result<()> {
        let _initial_pc = self.registers.get_pc();
        let nibbles = (
            ((instruction & 0xF000) >> 12) as u8,
            ((instruction & 0x0F00) >> 8) as u8,
            ((instruction & 0x00F0) >> 4) as u8,
            (instruction & 0x000F) as u8,
        );

        let nnn = instruction & 0x0FFF;
        let nn = (instruction & 0x00FF) as u8;
        let x = nibbles.1;
        let y = nibbles.2;
        let n = nibbles.3;

        match nibbles {
            // 0NNN - System call (ignored in modern interpreters)
            (0x0, 0x0, 0xE, 0x0) => self.cls()?, // 00E0 - Clear screen
            (0x0, 0x0, 0xE, 0xE) => self.ret()?, // 00EE - Return from subroutine
            (0x0, _, _, _) => {}                 // 0NNN - System call (ignored)

            // 1NNN - Jump to address NNN
            (0x1, _, _, _) => self.jp(nnn)?,

            // 2NNN - Call subroutine at NNN
            (0x2, _, _, _) => {
                log::debug!(
                    "CALL {:#04x} (PC before push: {:#04x}, SP: {})",
                    nnn,
                    self.registers.get_pc(),
                    self.stack.depth()
                );
                self.call(nnn)?;
                log::debug!("Stack after CALL: {:?}", self.stack.get_contents());
            }

            // 3XNN - Skip next instruction if VX == NN
            (0x3, _, _, _) => {
                let vx_val = self.registers.get_v(x)?;
                log::debug!("SE V{:X}, {:#02x} (V{:X} = {:#02x})", x, nn, x, vx_val);
                self.se_vx_nn(x, nn)?;
                log::debug!("PC after SE: {:#04x}", self.registers.get_pc());
            }

            // 4XNN - Skip next instruction if VX != NN
            (0x4, _, _, _) => self.sne_vx_nn(x, nn)?,

            // 5XY0 - Skip next instruction if VX == VY
            (0x5, _, _, 0x0) => self.se_vx_vy(x, y)?,

            // 6XNN - Set VX = NN
            (0x6, _, _, _) => self.ld_vx_nn(x, nn)?,

            // 7XNN - Set VX = VX + NN
            (0x7, _, _, _) => self.add_vx_nn(x, nn)?,

            // 8XY0 - Set VX = VY
            (0x8, _, _, 0x0) => self.ld_vx_vy(x, y)?,

            // 8XY1 - Set VX = VX OR VY
            (0x8, _, _, 0x1) => self.or_vx_vy(x, y)?,

            // 8XY2 - Set VX = VX AND VY
            (0x8, _, _, 0x2) => self.and_vx_vy(x, y)?,

            // 8XY3 - Set VX = VX XOR VY
            (0x8, _, _, 0x3) => self.xor_vx_vy(x, y)?,

            // 8XY4 - Set VX = VX + VY, VF = carry
            (0x8, _, _, 0x4) => self.add_vx_vy(x, y)?,

            // 8XY5 - Set VX = VX - VY, VF = NOT borrow
            (0x8, _, _, 0x5) => self.sub_vx_vy(x, y)?,

            // 8XY6 - Set VX = VX >> 1, VF = LSB
            (0x8, _, _, 0x6) => self.shr_vx(x)?,

            // 8XY7 - Set VX = VY - VX, VF = NOT borrow
            (0x8, _, _, 0x7) => self.subn_vx_vy(x, y)?,

            // 8XYE - Set VX = VX << 1, VF = MSB
            (0x8, _, _, 0xE) => self.shl_vx(x)?,

            // 9XY0 - Skip next instruction if VX != VY
            (0x9, _, _, 0x0) => self.sne_vx_vy(x, y)?,

            // ANNN - Set I = NNN
            (0xA, _, _, _) => self.ld_i_nnn(nnn)?,

            // BNNN - Jump to NNN + V0
            (0xB, _, _, _) => self.jp_v0_nnn(nnn)?,

            // CXNN - Set VX = random byte AND NN
            (0xC, _, _, _) => self.rnd_vx_nn(x, nn)?,

            // DXYN - Draw sprite at (VX, VY) with height N
            (0xD, _, _, _) => self.drw(x, y, n)?,

            // EX9E - Skip next instruction if key VX is pressed
            (0xE, _, 0x9, 0xE) => self.skp_vx(x)?,

            // EXA1 - Skip next instruction if key VX is not pressed
            (0xE, _, 0xA, 0x1) => self.sknp_vx(x)?,

            // FX07 - Set VX = delay timer
            (0xF, _, 0x0, 0x7) => self.ld_vx_dt(x)?,

            // FX0A - Wait for key press, store in VX
            (0xF, _, 0x0, 0xA) => self.ld_vx_k(x)?,

            // FX15 - Set delay timer = VX
            (0xF, _, 0x1, 0x5) => self.ld_dt_vx(x)?,

            // FX18 - Set sound timer = VX
            (0xF, _, 0x1, 0x8) => self.ld_st_vx(x)?,

            // FX1E - Set I = I + VX
            (0xF, _, 0x1, 0xE) => self.add_i_vx(x)?,

            // FX29 - Set I = location of sprite for digit VX
            (0xF, _, 0x2, 0x9) => self.ld_f_vx(x)?,

            // FX33 - Store BCD representation of VX at I, I+1, I+2
            (0xF, _, 0x3, 0x3) => self.ld_b_vx(x)?,

            // FX55 - Store V0 through VX in memory starting at I
            (0xF, _, 0x5, 0x5) => self.ld_i_vx(x)?,

            // FX65 - Load V0 through VX from memory starting at I
            (0xF, _, 0x6, 0x5) => {
                let i_val = self.registers.get_i();
                log::debug!("LD V{:X}, [I] (I: {:#04x})", x, i_val);
                self.ld_vx_i(x)?;
                log::debug!(
                    "V0-V{:X} after LD: {:?}",
                    x,
                    self.registers.get_v_range(0, x as usize + 1)?
                );
                log::debug!("I after LD: {:#04x}", self.registers.get_i());
            }

            _ => {
                return Err(EmulatorError::UnknownInstruction {
                    opcode: instruction,
                })
            }
        }

        Ok(())
    }

    /// Handles key press when waiting for input (legacy method).
    ///
    /// # Arguments
    /// * `key` - The key that was pressed (0x0-0xF)
    ///
    /// # Returns
    /// Ok(()) if handled, or an error if not waiting for key.
    pub fn handle_key_press(&mut self, key: u8) -> Result<()> {
        if !self.waiting_for_key {
            return Ok(()); // Not waiting for key, ignore
        }

        if key <= 0xF && self.waiting_for_key_release.is_none() {
            // Only handle if we're in the initial waiting state, not waiting for release
            self.waiting_for_key_release = Some(key);
        }

        Ok(())
    }

    /// Checks if the CPU is waiting for a key press.
    pub fn is_waiting_for_key(&self) -> bool {
        self.waiting_for_key
    }

    /// Gets the current CPU state for debugging.
    pub fn get_state(&self) -> CpuState {
        CpuState {
            pc: self.registers.get_pc(),
            sp: self.registers.get_sp(),
            i: self.registers.get_i(),
            v: *self.registers.get_all_v().try_into().unwrap_or(&[0; 16]),
            delay_timer: self.timers.get_delay_timer(),
            sound_timer: self.timers.get_sound_timer(),
            stack_contents: self.stack.get_contents(),
            instruction_count: self.instruction_count,
        }
    }

    /// Gets the memory for external access (read-only).
    pub fn get_memory(&self) -> &Memory {
        &self.memory
    }

    /// Gets the timers for external access.
    pub fn get_timers(&self) -> &Timers {
        &self.timers
    }

    /// Gets mutable access to the timers.
    pub fn timers_mut(&mut self) -> &mut Timers {
        &mut self.timers
    }

    /// Gets whether sound should be playing.
    pub fn should_play_sound(&self) -> bool {
        self.timers.should_play_sound()
    }

    /// Gets the current display buffer.
    pub fn get_display_buffer(&self) -> &[bool] {
        self.display.as_ref().map_or(&[], |d| d.get_buffer())
    }

    // Instruction implementations

    fn cls(&mut self) -> Result<()> {
        if let Some(display) = &mut self.display {
            display.clear();
        }
        Ok(())
    }

    fn ret(&mut self) -> Result<()> {
        let addr = self.stack.pop()?;
        self.registers.set_pc(addr);
        Ok(())
    }

    fn jp(&mut self, addr: u16) -> Result<()> {
        self.registers.set_pc(addr);
        Ok(())
    }

    fn call(&mut self, addr: u16) -> Result<()> {
        self.stack.push(self.registers.get_pc())?;
        self.registers.set_pc(addr);
        Ok(())
    }

    fn se_vx_nn(&mut self, x: u8, nn: u8) -> Result<()> {
        if self.registers.get_v(x)? == nn {
            self.registers.skip_instruction();
        }
        Ok(())
    }

    fn sne_vx_nn(&mut self, x: u8, nn: u8) -> Result<()> {
        if self.registers.get_v(x)? != nn {
            self.registers.skip_instruction();
        }
        Ok(())
    }

    fn se_vx_vy(&mut self, x: u8, y: u8) -> Result<()> {
        if self.registers.get_v(x)? == self.registers.get_v(y)? {
            self.registers.skip_instruction();
        }
        Ok(())
    }

    fn ld_vx_nn(&mut self, x: u8, nn: u8) -> Result<()> {
        self.registers.set_v(x, nn)
    }

    fn add_vx_nn(&mut self, x: u8, nn: u8) -> Result<()> {
        let vx = self.registers.get_v(x)?;
        self.registers.set_v(x, vx.wrapping_add(nn))
    }

    fn ld_vx_vy(&mut self, x: u8, y: u8) -> Result<()> {
        let vy = self.registers.get_v(y)?;
        self.registers.set_v(x, vy)
    }

    fn or_vx_vy(&mut self, x: u8, y: u8) -> Result<()> {
        let vx = self.registers.get_v(x)?;
        let vy = self.registers.get_v(y)?;
        self.registers.set_v(x, vx | vy)
    }

    fn and_vx_vy(&mut self, x: u8, y: u8) -> Result<()> {
        let vx = self.registers.get_v(x)?;
        let vy = self.registers.get_v(y)?;
        self.registers.set_v(x, vx & vy)
    }

    fn xor_vx_vy(&mut self, x: u8, y: u8) -> Result<()> {
        let vx = self.registers.get_v(x)?;
        let vy = self.registers.get_v(y)?;
        self.registers.set_v(x, vx ^ vy)
    }

    fn add_vx_vy(&mut self, x: u8, y: u8) -> Result<()> {
        self.registers.add_with_carry(x, y)
    }

    fn sub_vx_vy(&mut self, x: u8, y: u8) -> Result<()> {
        self.registers.sub_with_borrow(x, y)
    }

    fn shr_vx(&mut self, x: u8) -> Result<()> {
        self.registers.shift_right(x)
    }

    fn subn_vx_vy(&mut self, x: u8, y: u8) -> Result<()> {
        self.registers.sub_reverse_with_borrow(x, y)
    }

    fn shl_vx(&mut self, x: u8) -> Result<()> {
        self.registers.shift_left(x)
    }

    fn sne_vx_vy(&mut self, x: u8, y: u8) -> Result<()> {
        if self.registers.get_v(x)? != self.registers.get_v(y)? {
            self.registers.skip_instruction();
        }
        Ok(())
    }

    fn ld_i_nnn(&mut self, nnn: u16) -> Result<()> {
        self.registers.set_i(nnn);
        Ok(())
    }

    fn jp_v0_nnn(&mut self, nnn: u16) -> Result<()> {
        let v0 = self.registers.get_v(0)? as u16;
        self.registers.set_pc(nnn + v0);
        Ok(())
    }

    fn rnd_vx_nn(&mut self, x: u8, nn: u8) -> Result<()> {
        let random_byte: u8 = self.rng.gen();
        self.registers.set_v(x, random_byte & nn)
    }

    fn drw(&mut self, x: u8, y: u8, n: u8) -> Result<()> {
        let x_pos = self.registers.get_v(x)? as usize;
        let y_pos = self.registers.get_v(y)? as usize;
        let sprite_addr = self.registers.get_i() as usize;

        // Default to no collision
        let mut collision = false;

        if let Some(display) = &mut self.display {
            // Read sprite data from memory
            let mut sprite_data = Vec::new();
            for i in 0..n {
                let addr = sprite_addr + i as usize;
                if addr < 4096 {
                    // MEMORY_SIZE
                    sprite_data.push(self.memory.read_byte(addr as u16)?);
                }
            }

            // Draw sprite and check for collision
            collision = display
                .draw_sprite(x_pos as u8, y_pos as u8, &sprite_data)
                .unwrap_or(false);
        }

        // Set VF flag based on collision
        self.registers.set_flag(if collision { 1 } else { 0 });
        Ok(())
    }

    fn skp_vx(&mut self, x: u8) -> Result<()> {
        let key_value = self.registers.get_v(x)?;

        if let Some(input) = &self.input {
            if let Some(chip_key) = ChipKey::from_u8(key_value) {
                let is_pressed = input.borrow().is_key_pressed(chip_key);
                log::debug!(
                    "SKP V{:X} (key {:#X}): is_pressed = {}",
                    x,
                    key_value,
                    is_pressed
                );
                if is_pressed {
                    self.registers.set_pc(self.registers.get_pc() + 2);
                }
            }
        }

        Ok(())
    }

    fn sknp_vx(&mut self, x: u8) -> Result<()> {
        let key_value = self.registers.get_v(x)?;

        if let Some(input) = &self.input {
            if let Some(chip_key) = ChipKey::from_u8(key_value) {
                let is_pressed = input.borrow().is_key_pressed(chip_key);
                log::debug!(
                    "SKNP V{:X} (key {:#X}): is_pressed = {}",
                    x,
                    key_value,
                    is_pressed
                );
                if !is_pressed {
                    self.registers.set_pc(self.registers.get_pc() + 2);
                }
            }
        } else {
            // If no input system, treat as key not pressed
            self.registers.set_pc(self.registers.get_pc() + 2);
        }

        Ok(())
    }

    fn ld_vx_dt(&mut self, x: u8) -> Result<()> {
        let dt = self.timers.get_delay_timer();
        self.registers.set_v(x, dt)
    }

    fn ld_vx_k(&mut self, x: u8) -> Result<()> {
        self.waiting_for_key = true;
        self.key_wait_register = x;
        Ok(())
    }

    fn ld_dt_vx(&mut self, x: u8) -> Result<()> {
        let vx = self.registers.get_v(x)?;
        self.timers.set_delay_timer(vx);
        Ok(())
    }

    fn ld_st_vx(&mut self, x: u8) -> Result<()> {
        let vx = self.registers.get_v(x)?;
        self.timers.set_sound_timer(vx);
        Ok(())
    }

    fn add_i_vx(&mut self, x: u8) -> Result<()> {
        let vx = self.registers.get_v(x)? as u16;
        let i = self.registers.get_i();
        let result = i.wrapping_add(vx);

        // Set VF flag if there's an overflow beyond 0x0FFF (12-bit address space)
        self.registers.set_flag(if result > 0x0FFF { 1 } else { 0 });

        self.registers.set_i(result);
        Ok(())
    }

    fn ld_f_vx(&mut self, x: u8) -> Result<()> {
        let vx = self.registers.get_v(x)?;
        let font_addr = self.memory.get_font_address(vx & 0xF)?;
        self.registers.set_i(font_addr);
        Ok(())
    }

    fn ld_b_vx(&mut self, x: u8) -> Result<()> {
        let vx = self.registers.get_v(x)?;
        let i = self.registers.get_i();

        let hundreds = vx / 100;
        let tens = (vx / 10) % 10;
        let ones = vx % 10;

        self.memory.write_byte(i, hundreds)?;
        self.memory.write_byte(i + 1, tens)?;
        self.memory.write_byte(i + 2, ones)?;

        Ok(())
    }

    fn ld_i_vx(&mut self, x: u8) -> Result<()> {
        let i = self.registers.get_i();

        for reg in 0..=x {
            let value = self.registers.get_v(reg)?;
            self.memory.write_byte(i + reg as u16, value)?;
        }

        Ok(())
    }

    fn ld_vx_i(&mut self, x: u8) -> Result<()> {
        let i = self.registers.get_i();

        for reg in 0..=x {
            let value = self.memory.read_byte(i + reg as u16)?;
            self.registers.set_v(reg, value)?;
        }

        // Original Chip-8 behavior: don't modify I register
        Ok(())
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_creation() {
        let cpu = Cpu::new();
        let state = cpu.get_state();

        assert_eq!(state.pc, 0x200);
        assert_eq!(state.sp, 0);
        assert_eq!(state.i, 0);
        assert_eq!(state.instruction_count, 0);
        assert!(!cpu.is_waiting_for_key());
    }

    #[test]
    fn test_rom_loading() {
        let mut cpu = Cpu::new();
        let rom_data = vec![0x12, 0x34, 0x56, 0x78];

        cpu.load_rom(&rom_data).unwrap();

        // Check ROM is loaded at correct address
        assert_eq!(cpu.memory.read_byte(0x200).unwrap(), 0x12);
        assert_eq!(cpu.memory.read_byte(0x201).unwrap(), 0x34);
        assert_eq!(cpu.memory.read_byte(0x202).unwrap(), 0x56);
        assert_eq!(cpu.memory.read_byte(0x203).unwrap(), 0x78);

        // Check PC is set correctly
        assert_eq!(cpu.get_state().pc, 0x200);
    }

    #[test]
    fn test_basic_instructions() {
        let mut cpu = Cpu::new();

        // Test 6XNN - Load immediate
        cpu.execute_instruction(0x6123).unwrap(); // Load 0x23 into V1
        assert_eq!(cpu.registers.get_v(1).unwrap(), 0x23);

        // Test 7XNN - Add immediate
        cpu.execute_instruction(0x7110).unwrap(); // Add 0x10 to V1
        assert_eq!(cpu.registers.get_v(1).unwrap(), 0x33);

        // Test ANNN - Set I
        cpu.execute_instruction(0xA456).unwrap(); // Set I to 0x456
        assert_eq!(cpu.registers.get_i(), 0x456);
    }

    #[test]
    fn test_arithmetic_instructions() {
        let mut cpu = Cpu::new();

        // Set up registers
        cpu.registers.set_v(1, 0x10).unwrap();
        cpu.registers.set_v(2, 0x05).unwrap();

        // Test 8XY4 - Add with carry
        cpu.execute_instruction(0x8124).unwrap(); // V1 = V1 + V2
        assert_eq!(cpu.registers.get_v(1).unwrap(), 0x15);
        assert_eq!(cpu.registers.get_flag(), 0); // No carry

        // Test overflow
        cpu.registers.set_v(1, 0xFF).unwrap();
        cpu.registers.set_v(2, 0x01).unwrap();
        cpu.execute_instruction(0x8124).unwrap(); // V1 = V1 + V2
        assert_eq!(cpu.registers.get_v(1).unwrap(), 0x00);
        assert_eq!(cpu.registers.get_flag(), 1); // Carry occurred
    }

    #[test]
    fn test_jump_and_call() {
        let mut cpu = Cpu::new();

        // Test 1NNN - Jump using direct instruction execution
        cpu.execute_instruction(0x1300).unwrap(); // Jump to 0x300
        assert_eq!(cpu.registers.get_pc(), 0x300);

        // Test 2NNN - Call subroutine using full cycle for proper PC handling
        let rom_data = vec![0x24, 0x00]; // Call 0x400
        cpu.load_rom(&rom_data).unwrap();

        // CPU starts at 0x200, execute one cycle (call instruction)
        cpu.cycle().unwrap();
        assert_eq!(cpu.registers.get_pc(), 0x400);
        assert_eq!(cpu.stack.depth(), 1);
        assert_eq!(cpu.stack.peek().unwrap(), 0x202); // Return address (PC after call instruction)

        // Test 00EE - Return using direct execution since we know the expected behavior
        cpu.execute_instruction(0x00EE).unwrap(); // Return
        assert_eq!(cpu.registers.get_pc(), 0x202);
        assert_eq!(cpu.stack.depth(), 0);
    }

    #[test]
    fn test_conditional_instructions() {
        let mut cpu = Cpu::new();

        cpu.registers.set_v(1, 0x42).unwrap();
        cpu.registers.set_pc(0x200);

        // Test 3XNN - Skip if equal
        // Simulate the PC increment that happens in cycle() before execute_instruction()
        cpu.registers.increment_pc(); // PC becomes 0x202
        cpu.execute_instruction(0x3142).unwrap(); // Skip if V1 == 0x42
        assert_eq!(cpu.registers.get_pc(), 0x204); // Should have skipped (0x202 + 2)

        cpu.registers.set_pc(0x200);
        cpu.registers.increment_pc(); // PC becomes 0x202
        cpu.execute_instruction(0x3143).unwrap(); // Skip if V1 == 0x43
        assert_eq!(cpu.registers.get_pc(), 0x202); // Should not have skipped
    }

    #[test]
    fn test_timer_instructions() {
        let mut cpu = Cpu::new();

        // Test FX15 - Set delay timer
        cpu.registers.set_v(1, 60).unwrap();
        cpu.execute_instruction(0xF115).unwrap(); // Set delay timer to V1
        assert_eq!(cpu.timers.get_delay_timer(), 60);

        // Test FX18 - Set sound timer
        cpu.registers.set_v(2, 30).unwrap();
        cpu.execute_instruction(0xF218).unwrap(); // Set sound timer to V2
        assert_eq!(cpu.timers.get_sound_timer(), 30);

        // Test FX07 - Read delay timer
        cpu.execute_instruction(0xF307).unwrap(); // V3 = delay timer
        assert_eq!(cpu.registers.get_v(3).unwrap(), 60);
    }

    #[test]
    fn test_bcd_instruction() {
        let mut cpu = Cpu::new();

        cpu.registers.set_v(1, 123).unwrap();
        cpu.registers.set_i(0x300);

        cpu.execute_instruction(0xF133).unwrap(); // Store BCD of V1 at I

        assert_eq!(cpu.memory.read_byte(0x300).unwrap(), 1); // Hundreds
        assert_eq!(cpu.memory.read_byte(0x301).unwrap(), 2); // Tens
        assert_eq!(cpu.memory.read_byte(0x302).unwrap(), 3); // Ones
    }

    #[test]
    fn test_memory_instructions() {
        let mut cpu = Cpu::new();

        // Set up registers
        for i in 0..5 {
            cpu.registers.set_v(i, (i + 1) * 10).unwrap();
        }
        cpu.registers.set_i(0x300);

        // Test FX55 - Store registers
        cpu.execute_instruction(0xF455).unwrap(); // Store V0-V4 at I

        for i in 0..5 {
            let expected = (i + 1) * 10;
            assert_eq!(cpu.memory.read_byte(0x300 + i as u16).unwrap(), expected);
        }

        // Clear registers
        for i in 0..5 {
            cpu.registers.set_v(i, 0).unwrap();
        }

        // Test FX65 - Load registers
        cpu.execute_instruction(0xF465).unwrap(); // Load V0-V4 from I

        for i in 0..5 {
            let expected = (i + 1) * 10;
            assert_eq!(cpu.registers.get_v(i).unwrap(), expected);
        }
    }

    #[test]
    fn test_key_wait() {
        use crate::hardware::input::{ChipKey, SoftwareInput};
        use std::cell::RefCell;
        use std::rc::Rc;

        let mut cpu = Cpu::new();
        let input = Rc::new(RefCell::new(SoftwareInput::new()));
        cpu.set_input(input.clone());

        // Test FX0A - Wait for key
        cpu.execute_instruction(0xF10A).unwrap(); // Wait for key, store in V1
        assert!(cpu.is_waiting_for_key());

        // Initially no keys pressed - should still be waiting
        cpu.cycle().unwrap();
        assert!(cpu.is_waiting_for_key());

        // Press a key - should transition to waiting for release
        input.borrow_mut().press_key(ChipKey::Key5);
        cpu.cycle().unwrap();
        assert!(cpu.is_waiting_for_key()); // Still waiting for release

        // Release the key - should complete the operation
        input.borrow_mut().release_key(ChipKey::Key5);
        cpu.cycle().unwrap();

        assert!(!cpu.is_waiting_for_key());
        assert_eq!(cpu.registers.get_v(1).unwrap(), 0x5);
    }

    #[test]
    fn test_font_instruction() {
        let mut cpu = Cpu::new();

        cpu.registers.set_v(1, 0xA).unwrap(); // Character 'A'
        cpu.execute_instruction(0xF129).unwrap(); // Set I to font address for V1

        let expected_addr = 0x50 + 0xA * 5; // Font start + character * 5 bytes
        assert_eq!(cpu.registers.get_i(), expected_addr);
    }

    #[test]
    fn test_unknown_instruction() {
        let mut cpu = Cpu::new();

        let result = cpu.execute_instruction(0xFFFF); // Invalid instruction
        assert!(matches!(
            result,
            Err(EmulatorError::UnknownInstruction { .. })
        ));
    }

    #[test]
    fn test_cpu_cycle() {
        let mut cpu = Cpu::new();
        let rom_data = vec![0x61, 0x23]; // Load 0x23 into V1

        cpu.load_rom(&rom_data).unwrap();

        let initial_pc = cpu.registers.get_pc();
        cpu.cycle().unwrap();

        // PC should have advanced
        assert_eq!(cpu.registers.get_pc(), initial_pc + 2);

        // Instruction should have executed
        assert_eq!(cpu.registers.get_v(1).unwrap(), 0x23);

        // Instruction count should have incremented
        assert_eq!(cpu.get_state().instruction_count, 1);
    }

    #[test]
    fn test_reset() {
        let mut cpu = Cpu::new();

        // Modify CPU state
        cpu.registers.set_v(5, 0xFF).unwrap();
        cpu.registers.set_i(0x1234);
        cpu.instruction_count = 1000;

        cpu.reset();

        let state = cpu.get_state();
        assert_eq!(state.pc, 0x200);
        assert_eq!(state.i, 0);
        assert_eq!(state.v[5], 0);
        assert_eq!(state.instruction_count, 0);
    }
}
