extern crate nes;
extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate piston_window;

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
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::time::Duration;
use piston_window::Button::Keyboard;
use piston_window::Key;


fn main() {
    let (tx, rx) = mpsc::channel::<u8>();
    let (txk, rxk) = mpsc::channel::<Option<Key>>();
    let t = thread::spawn(|| {
        let mut nes = Nes::new("sample1.nes");
        nes.run(tx, rxk);
        // nes.run();
    });
    // let _ = t.join();
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
        // println!("{}", rx.recv().unwrap());
        rx.recv().unwrap();
        let mut key = None;
        if let Some(button) = e.press_args() {
            match button {
                Keyboard(input) => {
                    key = Some(input);
                    println!("{:?}", key);
                }
                // Keyboard(Key::W) => println!("W"),
                // Keyboard(Key::A) => println!("A"),
                // Keyboard(Key::S) => println!("S"),
                // Keyboard(Key::D) => println!("D"),
                // Keyboard(Key::J) => println!("J"),
                // Keyboard(Key::K) => println!("K"),
                _ => {}
            }
            // println!("{:?}", button);
        }
        if let Some(args) = e.render_args() {
            gl.draw(args.viewport(), |c, g| {
                use graphics::clear;
                clear([1.0; 4], g);
                // tv.draw(&nes_controller, &c, g);
            });
        }
        txk.send(key);
    }
}
