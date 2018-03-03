use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use graphics::{Rectangle, Image};
use graphics::types::Color;
use sdl2_window::Sdl2Window;
use piston::event_loop::*;
use piston::input::*;
use piston::window::WindowSettings;
use piston_window::Key;
use image::{ImageBuffer, Rgba};

/// [PPU](http://wiki.nesdev.com/w/index.php/PPU) is short for Picture Processing Unit
pub struct Ppu {
    v_ram: Arc<Mutex<VRam>>,
}

impl Ppu {
    const HEIGHT: u16 = 240;
    const WIDTH: u16 = 256;
    pub fn new(v_ram: Arc<Mutex<VRam>>) -> Ppu {
        Ppu { v_ram }
    }

    pub fn run(self) {
        let opengl = OpenGL::V3_2;
        let mut window: Sdl2Window = WindowSettings::new("nes", [Self::WIDTH as u32, Self::HEIGHT as u32])
            .opengl(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();
        let mut gl = GlGraphics::new(opengl);
        let mut events = Events::new(EventSettings::new());
        let image = Image::new().rect([0.0, 0.0, Self::WIDTH as f64, Self::HEIGHT as f64]);
        let mut imgage_buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(Self::WIDTH as u32, Self::HEIGHT as u32);
        while let Some(e) = events.next(&mut window) {
            if let Some(args) = e.render_args() {
                gl.draw(args.viewport(), |c, g| {});
            }
        }
    }

    #[allow(dead_code)]
    pub fn dump(&self) {
        self.v_ram.lock().unwrap().dump();
    }
}

/// [PPU memory](http://wiki.nesdev.com/w/index.php/PPU_memory_map)
/// RAM is 2KB.
pub struct VRam(Box<[u8; 0x10000]>);

impl VRam {
    pub fn new(memory: Box<[u8; 0x10000]>) -> VRam {
        VRam(memory)
    }

    fn dump(&self) {
        let dump_file = "v_ram.dump";
        let mut f = BufWriter::new(File::create(dump_file).unwrap());
        for v in self.0.iter() {
            f.write(&[*v]).unwrap();
        }
    }
}
