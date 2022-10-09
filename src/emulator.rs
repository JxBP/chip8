use crate::{
    cpu::{Cpu, KeyState},
    display::{FrameBuffer, Render},
    ram::Ram,
    timer::Timer,
};
use anyhow::Result;

pub type Font = [u8; 80];

/// Represents the current state of an [`Emulator`].
pub struct EmulatorState<'a> {
    pub ram: Ram,
    pub sound_timer: Timer,
    pub delay_timer: Timer,
    pub frame_buffer: FrameBuffer,
    pub key_state: &'a mut KeyState,
}

/// A CHIP-8 emulator as a struct bundling all the components required.
pub struct Emulator<'a, R: Render> {
    pub state: EmulatorState<'a>,
    pub cpu: Cpu,
    display: R,
}

impl<'a, R: Render> Emulator<'a, R> {
    // Creates a new [`Emulator`] with the given [`Render`] and [`KeyState`].
    pub fn new(display: R, key_state: &'a mut KeyState) -> Emulator<R> {
        Self {
            state: EmulatorState {
                ram: Ram::default(),
                delay_timer: Timer::default(),
                sound_timer: Timer::default(),
                frame_buffer: [[false; 32]; 64],
                key_state,
            },
            cpu: Cpu::default(),
            display,
        }
    }

    /// Loads the given font in the emulated RAM at the offset of 0x50 bytes.
    pub fn load_font(&mut self, font: &Font) -> Result<()> {
        for (i, byte) in font.iter().enumerate() {
            self.state.ram.set(0x50 + i, *byte)?;
        }
        Ok(())
    }

    /// Executes the next instruction, updates sound and delay timer as well as redrawing the
    /// screen.
    pub fn step(&mut self) -> Result<()> {
        self.cpu.execute(&mut self.state)?;
        self.display.draw(self.state.frame_buffer)?;
        self.state.sound_timer.decrement();
        self.state.delay_timer.decrement();
        Ok(())
    }
}
