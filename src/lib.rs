#![recursion_limit = "1024"]
#[macro_use]
extern crate error_chain;

extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate piston_window;
extern crate sdl2_window;
extern crate image;

mod errors {
    error_chain!{}
}

pub mod nes;
