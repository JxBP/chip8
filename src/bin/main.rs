use std::{fs::File, io::Read, time::Duration};

use chip_8::{cpu::KeyState, display::SDLRenderer, emulator::Emulator, ram::RAM_SIZE};
use clap::{command, Parser};
use sdl2::{event::Event, keyboard::Keycode};

const FONT: [u8; 80] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

#[derive(Parser)]
#[command(author, version, about = "A CHIP-8 emulator")]
struct Cli {
    /// The ROM file to run
    rom_file: String,

    /// How many cpu cycles per second
    #[arg(short, long, default_value_t = 500)]
    cycles: u32,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Arbitrary value really. I don't know how big ROMs for CHIP-8 usually are.
    let mut rom = Vec::with_capacity(RAM_SIZE / 2);
    File::open(cli.rom_file)?.read_to_end(&mut rom)?;

    let sdl2_ctx = sdl2::init().map_err(anyhow::Error::msg)?;
    let mut event_pump = sdl2_ctx.event_pump().map_err(anyhow::Error::msg)?;

    let display = SDLRenderer::new(&sdl2_ctx);

    let mut emulator = Emulator::new(display, cli.cycles);
    emulator.load_font(&FONT)?;
    // TODO: Remove this unnecessary copy and make the emulator directly load from the file.
    emulator.load_rom(rom.as_mut())?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => handle_keypress(keycode, true, &mut emulator.state.key_state),
                Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                } => handle_keypress(keycode, false, &mut emulator.state.key_state),
                _ => {}
            }
        }
        emulator.step()?;
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / cli.cycles));
    }

    Ok(())
}

fn handle_keypress(keycode: Keycode, is_up: bool, key_state: &mut KeyState) {
    let index = match keycode {
        Keycode::Num1 => 0x1,
        Keycode::Num2 => 0x2,
        Keycode::Num3 => 0x3,
        Keycode::Num4 => 0xC,
        Keycode::Q => 0x4,
        Keycode::W => 0x5,
        Keycode::E => 0x6,
        Keycode::R => 0x7,
        Keycode::A => 0x8,
        Keycode::S => 0x9,
        Keycode::D => 0xA,
        Keycode::F => 0xB,
        Keycode::Y => 0xC,
        Keycode::X => 0xD,
        Keycode::C => 0xE,
        Keycode::V => 0xF,
        _ => return,
    };

    key_state[index] = is_up;
}
