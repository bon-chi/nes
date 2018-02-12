extern crate nes;
extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

pub use tv::Tv;
pub use nes_controller::NesController;

mod tv;
mod nes_controller;

use nes::nes::Nes;
use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};


fn main() {
    let mut nes = Nes::new("sample1.nes");
    // nes.run();
    // let nes_controller = NesController::new(nes);
    let tv = Tv::new();

    let opengl = OpenGL::V3_2;
    let mut window: Window = WindowSettings::new("nes", [256, 240])
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();
    let mut gl = GlGraphics::new(opengl);
    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
                use graphics::clear;
                clear([1.0; 4], g);
                // tv.draw(&nes_controller, &c, g);
            });
        }
    }
}
