use anyhow::Result;

use crate::{emulator::EmulatorState, instruction::Instruction};

pub type KeyState = [bool; 16];

/// This struct plays the role of a cpu and executes CHIP-8 instructions.
/// The fetching is done using [`Instruction::parse()`].
pub struct Cpu {
    pub registers: [u8; 16],
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
        Ok(())
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new(16)
    }
}
