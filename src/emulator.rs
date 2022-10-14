use crate::{
    cpu::{Cpu, KeyState},
    display::{FrameBuffer, Render},
    ram::Ram,
    timer::Timer,
};
use anyhow::Result;

pub const FONT_OFFSET: usize = 0x50;
pub type Font = [u8; 80];

/// Represents the current state of an [`Emulator`].
pub struct EmulatorState {
    pub ram: Ram,
    pub sound_timer: Timer,
    pub delay_timer: Timer,
    pub frame_buffer: FrameBuffer,
    pub key_state: KeyState,
}

/// A CHIP-8 emulator as a struct bundling all the components required.
pub struct Emulator<R: Render> {
    pub state: EmulatorState,
    pub cpu: Cpu,
    display: R,
    // The amount of cpu cycles since the last timer decrement
    ticks: u32,
    // How many cpu cycles there should be between every timer decrement
    timer_freq: u32,
}

impl<R: Render> Emulator<R> {
    /// Creates a new [`Emulator`] with the given [`Render`].
    /// `cycles` should be how often the step function is invoked per second.
    pub fn new(display: R, cycles: u32) -> Emulator<R> {
        Self {
            state: EmulatorState {
                ram: Ram::default(),
                delay_timer: Timer::default(),
                sound_timer: Timer::default(),
                frame_buffer: [[false; 32]; 64],
                key_state: [false; 16],
            },
            cpu: Cpu::default(),
            display,
            ticks: 0,
            timer_freq: cycles / 60,
        }
    }

    /// Loads the given font in the emulated RAM at the offset of 0x50 bytes.
    pub fn load_font(&mut self, font: &Font) -> Result<()> {
        self.load(FONT_OFFSET, font)?;
        Ok(())
    }

    /// Loads a ROM into the emulated RAM and jumps the pc to it.
    pub fn load_rom(&mut self, rom: &[u8]) -> Result<()> {
        self.load(0x200, rom)?;
        self.cpu.pc = 0x200;
        Ok(())
    }

    /// Copies the data that into the emulated RAM at a given offset.
    pub fn load(&mut self, offset: usize, data: &[u8]) -> Result<()> {
        for (i, byte) in data.iter().enumerate() {
            self.state.ram.set(offset + i, *byte)?;
        }
        Ok(())
    }

    /// Executes the next instruction and redraws the screen.
    /// An internal counter is kept that decrements the timers every 8th call
    /// of this function.
    pub fn step(&mut self) -> Result<()> {
        self.cpu.execute(&mut self.state)?;
        self.display.draw(self.state.frame_buffer)?;
        self.ticks += 1;
        if self.ticks >= self.timer_freq {
            self.state.sound_timer.decrement();
            self.state.delay_timer.decrement();
            self.ticks = 0;
        }
        Ok(())
    }
}
