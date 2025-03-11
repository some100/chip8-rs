use crate::{cpu::Cpu, EmuError};
use sdl2::{
    pixels::Color, rect::Rect, render::WindowCanvas, video::Window
};

pub struct Renderer {
    canvas: WindowCanvas,
}

impl Renderer {
    pub fn new(window: Window) -> Result<Renderer, EmuError> {
        let canvas = window.into_canvas().build()?;
        Ok(Renderer { canvas })
    }
    pub fn draw(&mut self, cpu: &mut Cpu) -> Result<(), EmuError> {
        self.canvas.set_draw_color(Color::BLACK);
        self.canvas.clear();
        self.canvas.set_draw_color(Color::WHITE);
        for (i, pixel) in cpu.display_buffer.iter().enumerate() {
            if *pixel {
                let coord = i32::try_from(i)?;
                let x = (coord % 64) * 16;
                let y = (coord / 64) * 16;
                self.canvas.fill_rect(Rect::new(x, y, 16, 16)).map_err(EmuError::Sdl)?;
            }
        }
        self.canvas.present();
        Ok(())
    }
}