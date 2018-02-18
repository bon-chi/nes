extern crate nes;
extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate piston_window;
// extern crate image;
extern crate image as im;
extern crate rand;
extern crate sdl2_window;

pub use tv::Tv;
pub use nes_controller::NesController;

mod tv;
mod nes_controller;

use nes::nes::Nes;
use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::time::Duration;
use piston_window::Button::Keyboard;
use piston_window::Key;
use im::{GenericImage, Pixel, RgbaImage, ImageBuffer};
use graphics::rectangle::square;
use graphics::Image;
use sdl2_window::Sdl2Window;
// use image::buffer::ImageBuffer;



// type GradientBuffer = image::ImageBuffer<image::Luma<u16>, Vec<u16>>;

fn main() {
    // let image_buffer = image::ImageBuffer::<image::Rgb<u8>>::new(100, 100);
    // let pixel = *im::Rgba::from_slice(&[100, 100, 100, 255]);
    // let mut img: RgbaImage = ImageBuffer::<im::Rgba<u8>, Vec<u8>>::new(256, 240);
    // img.put_pixel(10, 10, *im::Rgba::from_slice(&[100, 100, 100, 255]));
    // let texture = Texture::from_image(&img, &TextureSettings::new());
    // let texture_height = 240;
    // let texture_width = 256;
    //
    let opengl = OpenGL::V3_2;
    let texture_count = 1024;
    let frames = 200;
    let size = 32.0;
    let texture_width = 512;
    let texture_height = 512;
    //
    //
    // let texture = {
    //     let mut img = im::ImageBuffer::new(texture_width, texture_height);
    //     for y in 0..texture_height {
    //         for x in 0..texture_width {
    //             img.put_pixel(
    //                 x,
    //                 y,
    //                 im::Rgba([rand::random(), rand::random(), rand::random(), 255]),
    //             );
    //         }
    //     }
    //     Texture::from_image(&img, &TextureSettings::new())
    // };


    // let mut window: Sdl2Window = WindowSettings::new("nes", [1024; 2])
    //     .opengl(opengl)
    //     .build()
    //     .unwrap();

    // let mut window: Sdl2Window = WindowSettings::new("nes", [256, 240])
    //     .opengl(opengl)
    //     .exit_on_esc(true)
    //     .build()
    //     .unwrap();
    // let texture_height = 240;
    // let texture_width = 256;
    // let texture = {
    //     let mut img = im::ImageBuffer::new(texture_width, texture_height);
    //     for y in 0..texture_height {
    //         for x in 0..texture_width {
    //             img.put_pixel(
    //                 x,
    //                 y,
    //                 im::Rgba([rand::random(), rand::random(), rand::random(), 255]),
    //             );
    //         }
    //     }
    //     Texture::from_image(&img, &TextureSettings::new())
    // };
    // let mut gl = GlGraphics::new(opengl);
    // let mut events = Events::new(EventSettings::new());
    //
    // // let image = Image::new().rect(square(0.0, 30.0, 200.0));
    // let image = Image::new().rect([0.0, 0.0, 256.0, 240.0]);
    // let mut img = im::ImageBuffer::new(texture_width, texture_height);
    // while let Some(e) = events.next(&mut window) {
    //     let texture = {
    //         for y in 0..texture_height {
    //             for x in 0..texture_width {
    //                 img.put_pixel(x, y, im::Rgba([0, 0, 255, 255]));
    //             }
    //         }
    //         Texture::from_image(&img, &TextureSettings::new())
    //     };
    //     if let Some(args) = e.render_args() {
    //         gl.draw(args.viewport(), |c, g| {
    //             use graphics::clear;
    //             clear([0.0, 0.0, 0.0, 1.0], g);
    //             image.draw(&texture, &c.draw_state, c.transform, g);
    //         });
    //     }
    // }

    // let texture = Texture::from_image(&img, &TextureSettings::new());
    let (tx, rx) = mpsc::channel::<u8>();
    let (txk, rxk) = mpsc::channel::<Option<Key>>();


    let mut nes = Nes::new("sample1.nes");
    nes.run3();
    // let t = thread::spawn(|| {
    //     let mut nes = Nes::new("sample1.nes");
    //     nes.run(tx, rxk);
    // });
    // let _ = t.join();
    // nes.run();
    // let nes_controller = NesController::new(nes);
    // let tv = Tv::new();
    //
    // let opengl = OpenGL::V3_2;
    // let mut window: Window = WindowSettings::new("nes", [256, 240])
    //     .opengl(opengl)
    //     .exit_on_esc(true)
    //     .build()
    //     .unwrap();
    // let mut gl = GlGraphics::new(opengl);
    // let mut events = Events::new(EventSettings::new());
    // while let Some(e) = events.next(&mut window) {
    //     // println!("{}", rx.recv().unwrap());
    //     rx.recv().unwrap();
    //     let mut key = None;
    //     if let Some(button) = e.press_args() {
    //         match button {
    //             Keyboard(input) => {
    //                 key = Some(input);
    //                 println!("{:?}", key);
    //             }
    //             _ => {}
    //         }
    //     }
    //     if let Some(args) = e.render_args() {
    //         gl.draw(args.viewport(), |c, g| {
    //             use graphics::clear;
    //             clear([1.0; 4], g);
    //             // tv.draw(&nes_controller, &c, g);
    //         });
    //     }
    //     txk.send(key);
    // }
}
