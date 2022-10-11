use anyhow::{bail, Result};

use crate::{emulator::EmulatorState, instruction::Instruction};

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

        match (
            instruction.opcode,
            instruction.x,
            instruction.y,
            instruction.n,
        ) {
            // Clear screen
            (0, 0, 0xE, 0) => state.frame_buffer = [[false; 32]; 64],
            // Jump
            (0x1, _, _, _) => {
                self.pc = instruction.nnn as usize;
            }
            // Set register VX to NN
            (0x6, _, _, _) => self.set_register(instruction.x, instruction.nn)?,
            // Add NN to VX
            (0x7, _, _, _) => self.set_register(
                instruction.x,
                self.get_register(instruction.x)?
                    .wrapping_add(instruction.nn),
            )?,
            // Set I to NNN
            (0xA, _, _, _) => self.i = instruction.nnn,
            // Display
            (0xD, _, _, _) => {
                let pos_x = (self.get_register(instruction.x)? % 64) as usize;
                let pos_y = (self.get_register(instruction.y)? % 32) as usize;

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
            _ => {}
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
