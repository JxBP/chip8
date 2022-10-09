use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window};

const SCALE: u32 = 1;

pub type FrameBuffer = [[bool; 32]; 64];

/// A trait to render a [`FrameBuffer`].
pub trait Render {
    fn draw(&mut self, frame_buffer: FrameBuffer) -> anyhow::Result<()>;
}

/// The built-in renderer using SDL as graphics library.
pub struct SDLRenderer(Canvas<Window>);

impl SDLRenderer {
    /// Creates a new [`SDLRenderer`] from a [`sdl2::Sdl`] as context.
    pub fn new(ctx: &sdl2::Sdl) -> Self {
        let video_subsystem = ctx.video().unwrap();
        let window = video_subsystem
            .window("CHIP-8 Emulator", 64 * SCALE, 32 * SCALE)
            .position_centered()
            .build()
            .unwrap();
        Self(window.into_canvas().build().unwrap())
    }
}

impl Render for SDLRenderer {
    fn draw(&mut self, frame_buffer: FrameBuffer) -> anyhow::Result<()> {
        self.0.set_draw_color(Color::BLACK);
        self.0.clear();

        for (x, col) in frame_buffer.iter().enumerate() {
            for (y, state) in col.iter().enumerate() {
                // If this is true we shall render the pixel white
                if *state {
                    self.0
                        .fill_rect(Rect::new(
                            i32::try_from(x)? * SCALE as i32,
                            i32::try_from(y)? * SCALE as i32,
                            SCALE,
                            SCALE,
                        ))
                        .map_err(anyhow::Error::msg)?;
                }
            }
        }

        self.0.present();
        Ok(())
    }
}
