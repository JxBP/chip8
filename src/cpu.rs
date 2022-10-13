use anyhow::{bail, Result};
use rand::Rng;

use crate::{
    emulator::{EmulatorState, FONT_OFFSET},
    instruction::Instruction,
};

pub type KeyState = [bool; 16];

/// This struct plays the role of a cpu and executes CHIP-8 instructions.
/// The fetching is done using [`Instruction::parse()`].
pub struct Cpu {
    registers: [u8; 16],
    pub pc: usize,
    pub i: u16,
    pub stack: Vec<u16>,
}

impl Cpu {
    /// Creates a new [`Cpu`].
    ///
    /// `stack_capacity` is used as **initial** capacity for the stack.
    pub fn new(stack_capacity: usize) -> Self {
        Self {
            registers: [0u8; 16],
            pc: 0,
            i: 0,
            stack: Vec::with_capacity(stack_capacity),
        }
    }

    /// Fetches and executes the next instruction.
    pub fn execute(&mut self, state: &mut EmulatorState) -> Result<()> {
        let instruction = Instruction::parse(state.ram.get(self.pc)?, state.ram.get(self.pc + 1)?);

        // Advance to the next instruction
        self.pc += 2;

        let vx = self.get_register(instruction.x)?;
        let vy = self.get_register(instruction.y)?;

        match (
            instruction.opcode,
            instruction.x,
            instruction.y,
            instruction.n,
        ) {
            // Clear screen
            (0, 0, 0xE, 0) => state.frame_buffer = [[false; 32]; 64],
            // Return
            (0, 0, 0xE, 0xE) => {
                self.pc = self
                    .stack
                    .pop()
                    .ok_or_else(|| anyhow::anyhow!("Popped from empty stack"))?
                    .into()
            }
            // Jump
            (0x1, _, _, _) => {
                self.pc = instruction.nnn as usize;
            }
            // Call
            (0x2, _, _, _) => {
                self.stack.push(self.pc as u16);
                self.pc = instruction.nnn.into();
            }
            // Skip if VX equal to NN
            (0x3, _, _, _) => {
                if vx == instruction.nn {
                    self.pc += 2
                }
            }
            // Skip if VX NOT equal to NN
            (0x4, _, _, _) => {
                if vx != instruction.nn {
                    self.pc += 2
                }
            }
            // Skip if VX equal to VY
            (0x5, _, _, 0x0) => {
                if vx == vy {
                    self.pc += 2
                }
            }
            // Skip if VX NOT equal to VY
            (0x9, _, _, 0x0) => {
                if vx != vy {
                    self.pc += 2
                }
            }
            // Set register VX to NN
            (0x6, _, _, _) => self.set_register(instruction.x, instruction.nn)?,
            // Add NN to VX
            (0x7, _, _, _) => self.set_register(instruction.x, vx.wrapping_add(instruction.nn))?,

            // TODO: Find out if I can write a macro for this to reduce code duplication

            // Set VX to VY
            (0x8, _, _, 0x0) => self.set_register(instruction.x, vy)?,
            // Set VX to VX | VY
            (0x8, _, _, 0x1) => self.set_register(instruction.x, vx | vy)?,
            // Set VX to VX & VY
            (0x8, _, _, 0x2) => self.set_register(instruction.x, vx & vy)?,
            // Set VX to VX ^ VY
            (0x8, _, _, 0x3) => self.set_register(instruction.x, vx ^ vy)?,
            // Set VX to VX + VY (overflow -> carryflag)
            (0x8, _, _, 0x4) => {
                let (result, did_overflow) = vx.overflowing_add(vy);
                self.set_register(instruction.x, result)?;
                self.set_register(0xF, if did_overflow { 1 } else { 0 })?;
            }
            // Set VX to VX - VY (overflow -> carryflag)
            (0x8, _, _, 0x5) => {
                let (result, did_overflow) = vx.overflowing_sub(vy);
                self.set_register(instruction.x, result)?;
                self.set_register(0xF, if did_overflow { 1 } else { 0 })?;
            }
            // Set VY to VY - VX (overflow -> carryflag)
            (0x8, _, _, 0x7) => {
                let (result, did_overflow) = vy.overflowing_sub(vx);
                self.set_register(instruction.x, result)?;
                self.set_register(0xF, if did_overflow { 1 } else { 0 })?;
            }
            // TODO: Make old vs. new behaviour configurable
            // Here we are using the new behaviour

            // Shift right
            (0x8, _, _, 0x6) => {
                self.set_register(0xF, vy & 0b00000001)?;
                self.set_register(instruction.x, vy >> 1)?;
            }
            // Shift left
            (0x8, _, _, 0xE) => {
                self.set_register(0xF, vy & 0b10000000)?;
                self.set_register(instruction.x, vy << 1)?;
            }

            // Set I to NNN
            (0xA, _, _, _) => self.i = instruction.nnn,

            // TODO: Make old vs. new behaviour configurable
            // Here we are using the old behaviour
            // Jump with offset
            (0xB, _, _, _) => self.pc = (instruction.nnn + self.get_register(0)? as u16) as usize,

            // RNG
            (0xC, _, _, _) => self.set_register(instruction.x, rand::thread_rng().gen())?,

            // Display
            (0xD, _, _, _) => {
                let pos_x = (vx % 64) as usize;
                let pos_y = (vy % 32) as usize;

                let mut did_change = false;

                for row in 0..instruction.n {
                    let y = pos_y + row as usize;
                    if y >= 32 {
                        break;
                    }

                    let sprite = state.ram.get(self.i as usize + row as usize)?;
                    for col in 0..8 {
                        let x = pos_x + col;
                        if x >= 64 {
                            break;
                        }

                        let screen_pixel = state.frame_buffer[x][y];
                        let sprite_pixel = (sprite & (1 << (7 - col))) != 0;
                        state.frame_buffer[x][y] = screen_pixel ^ sprite_pixel;
                        did_change = did_change || (screen_pixel && sprite_pixel);
                    }
                }

                self.set_register(0xF, if did_change { 1 } else { 0 })?;
            }

            // Skip if pressed
            (0xE, _, 0x9, 0xE) => {
                if state.key_state[vx as usize] {
                    self.pc += 2;
                }
            }
            // Skip if NOT pressed
            (0xE, _, 0xA, 0x1) => {
                if !state.key_state[vx as usize] {
                    self.pc += 2;
                }
            }

            // Set VX to delay timer
            (0xF, _, 0, 0x7) => self.set_register(instruction.x, state.delay_timer.get())?,
            // Set delay timer to VX
            (0xF, _, 0x1, 0x5) => state.delay_timer.set(vx),
            // Set sound timer to VX
            (0xF, _, 0x1, 0x8) => state.sound_timer.set(vx),

            // Add VX to I
            (0xF, _, 0x1, 0xE) => self.i = self.i.wrapping_add(vx.into()),

            // Wait for key
            (0xF, _, 0, 0xA) => {
                if let Some((i, _)) = state.key_state.iter().enumerate().find(|(_, key)| **key) {
                    self.set_register(instruction.x, i as u8)?;
                } else {
                    self.pc -= 2
                }
            }

            // Get font character
            (0xF, _, 0x2, 0x9) => self.i = FONT_OFFSET as u16 + (vx as u16 * 5),

            // Binary-coded decimal conversion
            (0xF, _, 0x3, 0x3) => {
                state.ram.set(self.i as usize, vx / 100)?;
                state.ram.set(self.i as usize + 1, (vx / 10) % 10)?;
                state.ram.set(self.i as usize + 2, vx % 10)?;
            }

            // Store memory
            (0xF, _, 0x5, 0x5) => {
                for i in 0..instruction.x + 1 {
                    state
                        .ram
                        .set((self.i + i as u16) as usize, self.get_register(i)?)?;
                }
            }

            // Load memory
            (0xF, _, 0x6, 0x5) => {
                for i in 0..instruction.x + 1 {
                    self.set_register(i, state.ram.get((self.i + i as u16) as usize)?)?;
                }
            }

            _ => panic!("Unknown instruction: {:?}", instruction),
        }
        Ok(())
    }

    /// Sets the value of a cpu register
    pub fn set_register(&mut self, register: u8, value: u8) -> Result<()> {
        is_valid_register(register)?;
        self.registers[register as usize] = value;
        Ok(())
    }

    /// Gets the value of a cpu register
    pub fn get_register(&self, register: u8) -> Result<u8> {
        is_valid_register(register)?;
        Ok(self.registers[register as usize])
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new(16)
    }
}

fn is_valid_register(register: u8) -> Result<()> {
    if register >= 16 {
        bail!("Invalid register: {}", register)
    } else {
        Ok(())
    }
}
