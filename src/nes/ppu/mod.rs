extern crate piston;
extern crate rand;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate piston_window;
extern crate sdl2_window;
extern crate image as im;

use std::fs::File;
use std::io::{BufWriter, Write};
use nes::ppu::piston_window::Context;
use nes::piston_window::Graphics;
use nes::ppu::piston::event_loop::*;
use nes::ppu::piston::input::*;
use nes::ppu::piston::window::WindowSettings;
use nes::ppu::piston_window::Button::Keyboard;
use nes::ppu::piston_window::Key;
use nes::ppu::glutin_window::GlutinWindow as Window;
use self::opengl_graphics::{GlGraphics, OpenGL, Texture, TextureSettings};
use self::graphics::{Rectangle, Image};
use self::graphics::types::Color;
use nes::cpu::Cpu;
use nes::cpu::PrgRam;
use self::sdl2_window::Sdl2Window;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{Sender, Receiver};
use chrono::prelude::*;

pub struct Ppu2 {
    //sprite

    //background
    v_ram: Arc<Mutex<VRam>>,
    v_ram_address_register: VRamAddressRegister,
    temporary_v_ram_address: Arc<Mutex<VRamAddressRegister>>, // yyy, NN, YYYYY, XXXXX
    fine_x_scroll: Arc<Mutex<u8>>,
    first_or_second_write_toggle: Arc<Mutex<bool>>,
    pattern_high_value_register: (u8, u8),
    pattern_low_value_register: (u8, u8),
    attr_value_register: u8,

    // vram_index: u16,
    prg_ram: Arc<Mutex<PrgRam>>,

    cycle: u16,
    line: u16,
}

impl Ppu2 {
    pub fn dump(&self) {
        self.v_ram.lock().unwrap().dump();
    }
    pub fn new(
        prg_ram: Arc<Mutex<PrgRam>>,
        v_ram: Arc<Mutex<VRam>>,
        temporary_v_ram_address: Arc<Mutex<VRamAddressRegister>>,
        fine_x_scroll: Arc<Mutex<u8>>,
        first_or_second_write_toggle: Arc<Mutex<bool>>,
    ) -> Ppu2 {
        Ppu2 {
            v_ram,
            v_ram_address_register: VRamAddressRegister::new(),
            temporary_v_ram_address,
            fine_x_scroll,
            first_or_second_write_toggle,
            pattern_high_value_register: (0, 0),
            pattern_low_value_register: (0, 0),
            attr_value_register: 0,
            prg_ram,
            cycle: 0,
            line: 0,
        }
    }
    pub fn run<G: Graphics>(&mut self, c: &Context, g: &mut G) {
        // use self::graphics::Rectangle;
        loop {
            // Rectangle::new([0.0, 1.0, 0.0, 1.0]).draw([0.0, 0.0, 85.0, 80.0], &c.draw_state, c.transform, g);
        }
    }
    fn get_pattern_value(&self) -> u8 {
        // println!(
        //     "high: {}, low: {}",
        //     self.pattern_high_value_register.0,
        //     self.pattern_low_value_register.0
        // );
        (self.pattern_high_value_register.1 >> 7) + (self.pattern_low_value_register.1 >> 7) * 2
    }
    fn get_palette_num(&self, cycle: u16, line: u16) -> u8 {
        self.attr_value_register;
        match (cycle - 1) % 4 {
            0 | 1 => {
                match self.line % 4 {
                    0 | 1 => self.attr_value_register & 0b00000011,
                    _ => (self.attr_value_register >> 4) & 0b0011,
                }
            }
            _ => {
                match self.line % 4 {
                    0 | 1 => (self.attr_value_register >> 2) & 0b000011,
                    _ => (self.attr_value_register >> 6) & 0b11,
                }
            }
        }

    }
    fn shift_and_fetch_pixel(&mut self, cycle: u32, line: u32) -> [u8; 4] {
        self.shift_to_pixel2(cycle as u16, line as u16)
    }
    fn shift_and_draw_pixel<G: Graphics>(&mut self, c: &Context, g: &mut G, cycle: u16, line: u16) {
        let (color, rectangle) = self.shift_to_pixel(cycle, line);
        println!("{:?}, {:?}", color, rectangle);
        // Rectangle::new(color).draw(rectangle, &c.draw_state, c.transform, g);
        // Rectangle::new([0.5, 0.5, 0.5, 1.0]).draw(rectangle, &c.draw_state, c.transform, g);
    }

    fn shift_to_pixel2(&mut self, cycle: u16, line: u16) -> [u8; 4] {
        let idx = self.v_ram.lock().unwrap().fetch8(
            VRam::IMAGE_PALETTE + ((self.get_palette_num(cycle, line) as u16) * 4) +
                (self.get_pattern_value() as u16),
        );
        let color = self.v_ram.lock().unwrap().get_color2(idx);
        self.shift_registers();
        color
    }
    fn shift_to_pixel(&mut self, cycle: u16, line: u16) -> (Color, [f64; 4]) {
        let color: Color = self.v_ram.lock().unwrap().get_color(
            self.v_ram
                .lock()
                .unwrap()
                .fetch8(
                    VRam::IMAGE_PALETTE + ((self.get_palette_num(cycle, line) as u16) * 4) + (self.get_pattern_value() as u16),
                ),
        );
        self.shift_registers();
        // println!("{}", self.cycle);
        (color, [cycle as f64 - 1.0, line as f64, 1.0, 1.0])
        // (color, [self.cycle as f64 - 1.0, self.line as f64, 5.0, 5.0])
    }
    fn shift_registers(&mut self) {
        self.pattern_high_value_register.1 = self.pattern_high_value_register.1 << 1;
        self.pattern_low_value_register.1 = self.pattern_low_value_register.1 << 1;
    }
    pub fn run3(&mut self, txk: Sender<Option<Key>>, rx: Receiver<u8>) {
        let opengl = OpenGL::V3_2;
        let mut window: Sdl2Window = WindowSettings::new("nes", [256, 240])
            .opengl(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();
        let texture_height = 240;
        let texture_width = 256;
        let mut gl = GlGraphics::new(opengl);
        let mut events = Events::new(EventSettings::new());
        let image = Image::new().rect([0.0, 0.0, 256.0, 240.0]);
        let mut img = im::ImageBuffer::<im::Rgba<u8>, Vec<u8>>::new(texture_width, texture_height);
        while let Some(e) = events.next(&mut window) {
            let texture = Texture::from_image(&img, &TextureSettings::new());
            // rx.recv().unwrap();
            let mut key = None;
            if let Some(button) = e.press_args() {
                match button {
                    Keyboard(input) => {
                        key = Some(input);
                        println!("{:?}", key);
                    }
                    _ => {}
                }
            }
            if let Some(args) = e.render_args() {
                gl.draw(args.viewport(), |c, g| {
                    use self::graphics::clear;
                    // clear([1.0; 4], g);
                    image.draw(&texture, &c.draw_state, c.transform, g);
                    for line in 0..261 {
                        for cycle in 0..321 {
                            if 1 <= cycle && cycle <= 256 && line <= 239 {
                                // img.put_pixel(
                                //     cycle - 1,
                                //     line,
                                //     im::Rgba([rand::random(), rand::random(), rand::random(), 255]),
                                // );
                                // println!("{},{}", cycle, line);
                                let color = self.shift_and_fetch_pixel(cycle, line);
                                img.put_pixel(
                                    cycle - 1,
                                    line,
                                    // im::Rgba([rand::random(), rand::random(), rand::random(), 255]),
                                    im::Rgba(color),
                                );
                                if (cycle % 8) == 0 {
                                    // load tile bit
                                    let pattern_num = self.v_ram.lock().unwrap().fetch8(0x2000 + cycle as u16);
                                    // println!(
                                    //     "cycle: {}, line: {}, pattern_num: {}, vram: {}",
                                    //     cycle,
                                    //     line,
                                    //     pattern_num,
                                    //     self.v_ram.fetch8(
                                    //         0x0000 + 16 * pattern_num as u16 +
                                    //             (line % 8) as u16,
                                    //     )
                                    // );
                                    self.pattern_high_value_register.1 = self.pattern_high_value_register.0;
                                    self.pattern_low_value_register.1 = self.pattern_low_value_register.0;
                                    self.pattern_high_value_register.0 = self.v_ram.lock().unwrap().fetch8(
                                        0x0000 + 16 * pattern_num as u16 +
                                            (line % 8) as u16,
                                    );
                                    self.pattern_high_value_register.1 = self.v_ram.lock().unwrap().fetch8(
                                        0x0000 + 16 * pattern_num as u16 + (line % 8) as u16 +
                                            8,
                                    );
                                }
                            }
                            if line >= 240 {}
                        }
                    }
                });
            }
            // txk.send(key);
        }
    }
    pub fn run2(&mut self, txk: Sender<Option<Key>>, rx: Receiver<u8>) {
        use self::graphics::Rectangle;
        let opengl = OpenGL::V3_2;
        let mut window: Window = WindowSettings::new("nes", [256, 240])
            .opengl(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();
        let mut gl = GlGraphics::new(opengl);
        let mut events = Events::new(EventSettings::new());
        while let Some(e) = events.next(&mut window) {
            rx.recv().unwrap();
            let mut key = None;
            if let Some(button) = e.press_args() {
                match button {
                    Keyboard(input) => {
                        key = Some(input);
                        println!("{:?}", key);
                    }
                    _ => {}
                }
            }
            if let Some(args) = e.render_args() {
                gl.draw(args.viewport(), |c, g| {
                    use self::graphics::clear;
                    // clear([1.0; 4], g);
                    for line in 0..261 {
                        for cycle in 0..321 {
                            if 1 <= cycle && cycle <= 256 {
                                // self.shift_and_draw_pixel(&c, g, cycle, line);
                            }
                        }
                    }
                });
                // gl.draw(args.viewport(), |c, g| {
                //     use self::graphics::clear;
                //     // clear([1.0; 4], g);
                //     // loop
                //     // cycle 0
                //     // cycle 1-256
                //     if 1 <= self.cycle && self.cycle <= 256 {
                //         self.shift_and_draw_pixel(&c, g);
                //     }
                //     // cycle 257-320
                //     // tv.draw(&nes_controller, &c, g);
                // });
                self.cycle += 1;
                if self.cycle >= 341 {
                    self.cycle = 0;
                    self.line += 1;
                    if self.line >= 261 {
                        self.line = 0;
                    }
                }
            } else {
            }
            txk.send(key);
        }
    }
}

pub struct Ppu {
    //sprite
    spr_ram: Box<SprRam>,
    secondary_spr_ram: Box<SecondarySprRam>,
    spr_ram_index: u8,
    spr_bitmap_register_high0: u8,
    spr_bitmap_register_high1: u8,
    spr_bitmap_register_high2: u8,
    spr_bitmap_register_high3: u8,
    spr_bitmap_register_high4: u8,
    spr_bitmap_register_high5: u8,
    spr_bitmap_register_high6: u8,
    spr_bitmap_register_high7: u8,
    spr_bitmap_register_low0: u8,
    spr_bitmap_register_low1: u8,
    spr_bitmap_register_low2: u8,
    spr_bitmap_register_low3: u8,
    spr_bitmap_register_low4: u8,
    spr_bitmap_register_low5: u8,
    spr_bitmap_register_low6: u8,
    spr_bitmap_register_low7: u8,
    spr_buffer_register0: u8,
    spr_buffer_register1: u8,
    spr_buffer_register2: u8,
    spr_buffer_register3: u8,
    spr_buffer_register4: u8,
    spr_buffer_register5: u8,
    spr_buffer_register6: u8,
    spr_buffer_register7: u8,
    spr_latch0: u8,
    spr_latch1: u8,
    spr_latch2: u8,
    spr_latch3: u8,
    spr_latch4: u8,
    spr_latch5: u8,
    spr_latch6: u8,
    spr_latch7: u8,
    spr_counter0: u8,
    spr_counter1: u8,
    spr_counter2: u8,
    spr_counter3: u8,
    spr_counter4: u8,
    spr_counter5: u8,
    spr_counter6: u8,
    spr_counter7: u8,

    //background
    vram_address_register: VRamAddressRegister,
    temporary_vram_address: VRamAddressRegister, // yyy, NN, YYYYY, XXXXX
    fine_x_scroll: u8,
    first_or_second_write_toggle: bool,
    // first/second write toggle
    // 16-bit shift registers * 2
    background_shift_register16_0: (u8, u8),
    background_shift_register16_1: (u8, u8),
    background_shift_register8_0: u8,
    background_shift_register8_1: u8,
    cycle: u16,
    line: u8,

    shift_register0_16: u16,
    shift_register1_16: u16,
    shift_register0_8: u8,
    shift_register1_8: u8,

    name_table_num: u8,
    name_table_idx: u16,
    name_table_value: u8,

    attr_table_num: u8,
    attr_table_idx: u16,
    attr_table_value: u8,

    pattern_table_idx0: u16,
    pattern_table_idx1: u16,
    pattern_table0_value0: u8,
    pattern_table0_value1: u8,
    pattern_table1_value0: u8,
    pattern_table1_value1: u8,

    ram: Box<VRam>,
    // vram_index: u16,
    cpu: Arc<Mutex<Cpu>>,
}

impl Ppu {
    // fn new() -> Self {
    //     Ppu {
    //         cycle: 0,
    //         line: 0,
    //         name_table_num: 0,
    //         name_table_idx: 0,
    //         name_table_value: 0,
    //     }
    // }
    fn fetch_vram_address(&mut self) -> (u8, u8, u8, u8) {
        self.vram_address_register.fetch_vram_address(
            self.cpu
                .lock()
                .unwrap()
                .vram_offset_flag(),
        )
    }
    fn pixel_bit(register: u8) -> u8 {
        (register >> 7) & 0b00000001
    }

    fn shift_pixel_bit(&mut self) {
        self.background_shift_register16_0.0 = self.background_shift_register16_0.0 << 1;
        self.background_shift_register16_1.0 = self.background_shift_register16_1.0 << 1;
    }

    fn load_bit_shift(&mut self, line0: u8, line1: u8) {
        self.background_shift_register16_0.1 = self.background_shift_register16_0.0;
        self.background_shift_register16_1.1 = self.background_shift_register16_0.0;
        self.background_shift_register16_0.0 = line0;
        self.background_shift_register16_1.0 = line1;
    }
    fn fetch_attr_table_value(&self) -> u8 {
        self.vram_address_register.name_table_num
    }


    fn run2<G: Graphics>(&mut self, c: &Context, g: &mut G) {
        use self::graphics::Rectangle;
        loop {

            if 1 <= self.cycle && self.cycle <= 256 {
                //generate pixel
                let background1 = Self::pixel_bit(self.background_shift_register16_0.1);
                let background2 = Self::pixel_bit(self.background_shift_register16_1.1) * 2;
                let background = background1 + background2;
                // Rectangle::new([0.0, 1.0, 0.0, 1.0]).draw([0.0, 0.0, 85.0, 80.0], &c.draw_state, c.transform, g);
                // shift
                self.shift_pixel_bit();

                if self.cycle % 8 == 0 {
                    // new data load
                    let (fine_y_scroll, name_table_num, y_panel_pos, x_panel_pos) = self.fetch_vram_address();

                    let (tile_line0, tile_line1) = self.ram.fetch_tile_lines(
                        name_table_num,
                        y_panel_pos,
                        x_panel_pos,
                    );
                    self.load_bit_shift(tile_line0, tile_line1);
                }
            }
            self.cycle += 1;
        }
    }
    fn run(&mut self) {
        loop {
            if self.cycle == 0 {}
            if self.cycle >= 1 && self.cycle <= 256 {
                if (self.cycle % 8) == 1 {
                    self.name_table_num = 0;
                    self.name_table_idx = self.cycle / 8;
                }
                if (self.cycle % 8) == 2 {
                    self.name_table_value = self.ram.get_name_table_value(
                        self.name_table_num,
                        self.name_table_idx,
                    );
                }

                if (self.cycle % 8) == 3 {
                    self.attr_table_num = 0;
                    self.attr_table_idx = (self.line as u16 / 2) * 8 + ((self.cycle - 1) / 2)
                }
                if (self.cycle % 8) == 4 {
                    self.attr_table_value = self.ram.get_attr_table_value(
                        self.attr_table_num,
                        self.attr_table_idx,
                        self.line as u16,
                        self.cycle - 1,
                    );
                }

                if (self.cycle % 8) == 5 {
                    self.pattern_table_idx0 = 0x0000 + (16 * (self.name_table_value as u16)) + (self.line as u16 % 8);
                    self.pattern_table_idx1 = 0x0000 + 16 * (self.name_table_value as u16) + ((self.line as u16) % 8) + 8;
                }
                if (self.cycle % 8) == 6 {
                    self.pattern_table0_value0 = self.ram.get_pattern_table_value(0, self.pattern_table_idx0);
                    self.pattern_table0_value1 = self.ram.get_pattern_table_value(0, self.pattern_table_idx1);
                }

                if (self.cycle % 8) == 7 {
                    self.pattern_table_idx0 = 0x1000 + (16 * (self.name_table_value as u16)) + (self.line as u16 % 8);
                    self.pattern_table_idx1 = 0x1000 + 16 * (self.name_table_value as u16) + ((self.line as u16) % 8) + 8;

                }
                if (self.cycle % 8) == 0 {
                    self.pattern_table1_value0 = self.ram.get_pattern_table_value(0, self.pattern_table_idx0);
                    self.pattern_table1_value1 = self.ram.get_pattern_table_value(0, self.pattern_table_idx1);
                }
            }
            if self.cycle >= 257 && self.cycle <= 320 {}
            if self.cycle >= 321 && self.cycle <= 336 {}
            if self.cycle >= 337 && self.cycle <= 340 {}
            if self.cycle >= 341 {
                self.cycle -= 341;
                continue;
            }
            self.cycle += 1;
        }
    }
}
struct MemoryMap {}

// ObjectAttributeMemory
pub struct SprRam([u8; 0xFF]);
pub struct SecondarySprRam([u8; 0x1F]);

pub struct VRam([u8; 0xFFFF]);
impl VRam {
    const PATTERN_TABLE0: u16 = 0x0000;
    const PATTERN_TABLE1: u16 = 0x1000;

    const NAME_TABLE0: u16 = 0x2000;
    const NAME_TABLE1: u16 = 0x2400;
    const NAME_TABLE2: u16 = 0x2800;
    const NAME_TABLE3: u16 = 0x2C00;

    const ATTR_TABLE0: u16 = 0x23C0;
    const ATTR_TABLE1: u16 = 0x27C0;
    const ATTR_TABLE2: u16 = 0x2BC0;
    const ATTR_TABLE3: u16 = 0x2FC0;

    const IMAGE_PALETTE: u16 = 0x3F00;
    const SPRITE_PALETTE: u16 = 0x3F10;
    pub fn new() -> Self {
        VRam([0; 0xFFFF])
    }

    pub fn dump(&self) {
        let dt = Local::now().format("%Y-%m-%d_%H:%M:%S").to_string();
        let dt = "ppu_ram";
        // println!("{:?}", dt);
        let mut f = BufWriter::new(File::create(dt).unwrap());
        for v in self.0.iter() {
            f.write(&[*v]).unwrap();
            // println!("{:?}", &[*v]);
        }
    }
    fn get_name_table_value(&self, table_num: u8, index: u16) -> u8 {
        match table_num {
            0 => self.0[(Self::NAME_TABLE0 + index) as usize],
            1 => self.0[(Self::NAME_TABLE1 + index) as usize],
            2 => self.0[(Self::NAME_TABLE2 + index) as usize],
            3 => self.0[(Self::NAME_TABLE3 + index) as usize],
            _ => {
                panic!("name table{} doesn't exist", {
                    table_num
                })
            }
        }
    }

    fn get_attr_table_value(&self, table_num: u8, table_index: u16, line: u16, row: u16) -> u8 {
        let table_value = match table_num {
            0 => self.0[(Self::ATTR_TABLE0 + table_index) as usize],
            1 => self.0[(Self::ATTR_TABLE1 + table_index) as usize],
            2 => self.0[(Self::ATTR_TABLE2 + table_index) as usize],
            3 => self.0[(Self::ATTR_TABLE3 + table_index) as usize],
            _ => panic!("attr table{} doesn't exist", table_num),
        };
        match (line % 2) + (row % 2) {
            0 => table_value % 0b100,
            1 => (table_value >> 0b10) % 4,
            2 => (table_value >> 0b100) % 0b100,
            3 => table_value >> 0b110,
            _ => panic!("line {} and row {} doesn't exist", line, row),
        }

    }
    fn get_pattern_table_value(&self, table_num: u8, table_index: u16) -> u8 {
        self.0[table_index as usize]
    }
    fn fetch8(&self, address: u16) -> u8 {
        self.0[address as usize]
    }
    fn fetch_tile_lines(&self, name_table_num: u8, y_panel_pos: u8, x_panel_pos: u8) -> (u8, u8) {
        let pattern_table_idx: u16 = (name_table_num * 0x0400) as u16 + Self::NAME_TABLE0 + (y_panel_pos as u16);
        let pattern0 = self.0[pattern_table_idx as usize];
        let pattern1 = self.0[pattern_table_idx as usize + 8];
        (pattern0, pattern1)
    }
    fn get_color(&self, idx: u8) -> Color {
        Colors::COLORS[idx as usize]
    }
    fn get_color2(&self, idx: u8) -> [u8; 4] {
        Colors::COLORS2[idx as usize]
    }
}

pub struct VRamAddressRegister {
    y_offset_from_scanline: u8,
    name_table_num: u8,
    y_idx: u8,
    x_idx: u8,
}

impl VRamAddressRegister {
    pub fn new() -> VRamAddressRegister {
        VRamAddressRegister {
            y_offset_from_scanline: 0,
            name_table_num: 0,
            y_idx: 0,
            x_idx: 0,
        }
    }
    fn fetch_vram_address(&mut self, vram_offset_flag: bool) -> (u8, u8, u8, u8) {
        let fine_y_scroll = self.y_offset_from_scanline;
        let name_table_num = self.name_table_num;
        let y_panel_pos = self.y_idx;
        let x_panel_pos = self.x_idx;
        if vram_offset_flag {
            self.y_idx += 1;
        } else {
            self.x_idx += 1;
        }
        (fine_y_scroll, name_table_num, y_panel_pos, x_panel_pos)
    }
    pub fn set_y_offset_from_scanline(&mut self, offset: u8) {
        self.y_offset_from_scanline = offset;
    }
    pub fn set_name_table(&mut self, name_table: u8) {
        self.name_table_num = name_table;
    }
    pub fn set_y_idx(&mut self, idx: u8) {
        self.y_idx = idx;
    }
    pub fn set_x_idx(&mut self, idx: u8) {
        self.x_idx = idx;
    }
}

struct Colors;
impl Colors {
    const COLORS2: [[u8; 4]; 64] = [
        [0x80, 0x80, 0x80, 0xFF],
        [0x00, 0x3D, 0xA6, 0xFF],
        [0x00, 0x12, 0xB0, 0xFF],
        [0x44, 0x00, 0x96, 0xFF],
        [0xA1, 0x00, 0x5E, 0xFF],
        [0xC7, 0x00, 0x28, 0xFF],
        [0xBA, 0x06, 0x00, 0xFF],
        [0x8C, 0x17, 0x00, 0xFF],
        [0x5C, 0x2F, 0x00, 0xFF],
        [0x10, 0x45, 0x00, 0xFF],
        [0x05, 0x4A, 0x00, 0xFF],
        [0x00, 0x47, 0x2E, 0xFF],
        [0x00, 0x41, 0x66, 0xFF],
        [0x00, 0x00, 0x00, 0xFF],
        [0x05, 0x05, 0x05, 0xFF],
        [0x05, 0x05, 0x05, 0xFF],
        [0xC7, 0xC7, 0xC7, 0xFF],
        [0x00, 0x77, 0xFF, 0xFF],
        [0x21, 0x55, 0xFF, 0xFF],
        [0x82, 0x37, 0xFA, 0xFF],
        [0xEB, 0x2F, 0xB5, 0xFF],
        [0xFF, 0x29, 0x50, 0xFF],
        [0xFF, 0x22, 0x00, 0xFF],
        [0xD6, 0x32, 0x00, 0xFF],
        [0xC4, 0x62, 0x00, 0xFF],
        [0x35, 0x80, 0x00, 0xFF],
        [0x05, 0x8F, 0x00, 0xFF],
        [0x00, 0x8A, 0x55, 0xFF],
        [0x00, 0x99, 0xCC, 0xFF],
        [0x21, 0x21, 0x21, 0xFF],
        [0x09, 0x09, 0x09, 0xFF],
        [0x09, 0x09, 0x09, 0xFF],
        [0xFF, 0xFF, 0xFF, 0xFF],
        [0x0F, 0xD7, 0xFF, 0xFF],
        [0x69, 0xA2, 0xFF, 0xFF],
        [0xD4, 0x80, 0xFF, 0xFF],
        [0xFF, 0x45, 0xF3, 0xFF],
        [0xFF, 0x61, 0x8B, 0xFF],
        [0xFF, 0x88, 0x33, 0xFF],
        [0xFF, 0x9C, 0x12, 0xFF],
        [0xFA, 0xBC, 0x20, 0xFF],
        [0x9F, 0xE3, 0x0E, 0xFF],
        [0x2B, 0xF0, 0x35, 0xFF],
        [0x0C, 0xF0, 0xA4, 0xFF],
        [0x05, 0xFB, 0xFF, 0xFF],
        [0x5E, 0x5E, 0x5E, 0xFF],
        [0x0D, 0x0D, 0x0D, 0xFF],
        [0x0D, 0x0D, 0x0D, 0xFF],
        [0xFF, 0xFF, 0xFF, 0xFF],
        [0xA6, 0xFC, 0xFF, 0xFF],
        [0xB3, 0xEC, 0xFF, 0xFF],
        [0xDA, 0xAB, 0xEB, 0xFF],
        [0xFF, 0xA8, 0xF9, 0xFF],
        [0xFF, 0xAB, 0xB3, 0xFF],
        [0xFF, 0xD2, 0xB0, 0xFF],
        [0xFF, 0xEF, 0xA6, 0xFF],
        [0xFF, 0xF7, 0x9C, 0xFF],
        [0xD7, 0xE8, 0x95, 0xFF],
        [0xA6, 0xED, 0xAF, 0xFF],
        [0xA2, 0xF2, 0xDA, 0xFF],
        [0x99, 0xFF, 0xFC, 0xFF],
        [0xDD, 0xDD, 0xDD, 0xFF],
        [0x11, 0x11, 0x11, 0xFF],
        [0x11, 0x11, 0x11, 0xFF],
    ];
    const COLORS: [[f32; 4]; 64] = [
        [
            0x80 as f32 / 255.0,
            0x80 as f32 / 255.0,
            0x80 as f32 / 255.0,
            1.0,
        ],
        [
            0x00 as f32 / 255.0,
            0x3D as f32 / 255.0,
            0xA6 as f32 / 255.0,
            1.0,
        ],
        [
            0x00 as f32 / 255.0,
            0x12 as f32 / 255.0,
            0xB0 as f32 / 255.0,
            1.0,
        ],
        [
            0x44 as f32 / 255.0,
            0x00 as f32 / 255.0,
            0x96 as f32 / 255.0,
            1.0,
        ],
        [
            0xA1 as f32 / 255.0,
            0x00 as f32 / 255.0,
            0x5E as f32 / 255.0,
            1.0,
        ],
        [
            0xC7 as f32 / 255.0,
            0x00 as f32 / 255.0,
            0x28 as f32 / 255.0,
            1.0,
        ],
        [
            0xBA as f32 / 255.0,
            0x06 as f32 / 255.0,
            0x00 as f32 / 255.0,
            1.0,
        ],
        [
            0x8C as f32 / 255.0,
            0x17 as f32 / 255.0,
            0x00 as f32 / 255.0,
            1.0,
        ],
        [
            0x5C as f32 / 255.0,
            0x2F as f32 / 255.0,
            0x00 as f32 / 255.0,
            1.0,
        ],
        [
            0x10 as f32 / 255.0,
            0x45 as f32 / 255.0,
            0x00 as f32 / 255.0,
            1.0,
        ],
        [
            0x05 as f32 / 255.0,
            0x4A as f32 / 255.0,
            0x00 as f32 / 255.0,
            1.0,
        ],
        [
            0x00 as f32 / 255.0,
            0x47 as f32 / 255.0,
            0x2E as f32 / 255.0,
            1.0,
        ],
        [
            0x00 as f32 / 255.0,
            0x41 as f32 / 255.0,
            0x66 as f32 / 255.0,
            1.0,
        ],
        [
            0x00 as f32 / 255.0,
            0x00 as f32 / 255.0,
            0x00 as f32 / 255.0,
            1.0,
        ],
        [
            0x05 as f32 / 255.0,
            0x05 as f32 / 255.0,
            0x05 as f32 / 255.0,
            1.0,
        ],
        [
            0x05 as f32 / 255.0,
            0x05 as f32 / 255.0,
            0x05 as f32 / 255.0,
            1.0,
        ],
        [
            0xC7 as f32 / 255.0,
            0xC7 as f32 / 255.0,
            0xC7 as f32 / 255.0,
            1.0,
        ],
        [
            0x00 as f32 / 255.0,
            0x77 as f32 / 255.0,
            0xFF as f32 / 255.0,
            1.0,
        ],
        [
            0x21 as f32 / 255.0,
            0x55 as f32 / 255.0,
            0xFF as f32 / 255.0,
            1.0,
        ],
        [
            0x82 as f32 / 255.0,
            0x37 as f32 / 255.0,
            0xFA as f32 / 255.0,
            1.0,
        ],
        [
            0xEB as f32 / 255.0,
            0x2F as f32 / 255.0,
            0xB5 as f32 / 255.0,
            1.0,
        ],
        [
            0xFF as f32 / 255.0,
            0x29 as f32 / 255.0,
            0x50 as f32 / 255.0,
            1.0,
        ],
        [
            0xFF as f32 / 255.0,
            0x22 as f32 / 255.0,
            0x00 as f32 / 255.0,
            1.0,
        ],
        [
            0xD6 as f32 / 255.0,
            0x32 as f32 / 255.0,
            0x00 as f32 / 255.0,
            1.0,
        ],
        [
            0xC4 as f32 / 255.0,
            0x62 as f32 / 255.0,
            0x00 as f32 / 255.0,
            1.0,
        ],
        [
            0x35 as f32 / 255.0,
            0x80 as f32 / 255.0,
            0x00 as f32 / 255.0,
            1.0,
        ],
        [
            0x05 as f32 / 255.0,
            0x8F as f32 / 255.0,
            0x00 as f32 / 255.0,
            1.0,
        ],
        [
            0x00 as f32 / 255.0,
            0x8A as f32 / 255.0,
            0x55 as f32 / 255.0,
            1.0,
        ],
        [
            0x00 as f32 / 255.0,
            0x99 as f32 / 255.0,
            0xCC as f32 / 255.0,
            1.0,
        ],
        [
            0x21 as f32 / 255.0,
            0x21 as f32 / 255.0,
            0x21 as f32 / 255.0,
            1.0,
        ],
        [
            0x09 as f32 / 255.0,
            0x09 as f32 / 255.0,
            0x09 as f32 / 255.0,
            1.0,
        ],
        [
            0x09 as f32 / 255.0,
            0x09 as f32 / 255.0,
            0x09 as f32 / 255.0,
            1.0,
        ],
        [
            0xFF as f32 / 255.0,
            0xFF as f32 / 255.0,
            0xFF as f32 / 255.0,
            1.0,
        ],
        [
            0x0F as f32 / 255.0,
            0xD7 as f32 / 255.0,
            0xFF as f32 / 255.0,
            1.0,
        ],
        [
            0x69 as f32 / 255.0,
            0xA2 as f32 / 255.0,
            0xFF as f32 / 255.0,
            1.0,
        ],
        [
            0xD4 as f32 / 255.0,
            0x80 as f32 / 255.0,
            0xFF as f32 / 255.0,
            1.0,
        ],
        [
            0xFF as f32 / 255.0,
            0x45 as f32 / 255.0,
            0xF3 as f32 / 255.0,
            1.0,
        ],
        [
            0xFF as f32 / 255.0,
            0x61 as f32 / 255.0,
            0x8B as f32 / 255.0,
            1.0,
        ],
        [
            0xFF as f32 / 255.0,
            0x88 as f32 / 255.0,
            0x33 as f32 / 255.0,
            1.0,
        ],
        [
            0xFF as f32 / 255.0,
            0x9C as f32 / 255.0,
            0x12 as f32 / 255.0,
            1.0,
        ],
        [
            0xFA as f32 / 255.0,
            0xBC as f32 / 255.0,
            0x20 as f32 / 255.0,
            1.0,
        ],
        [
            0x9F as f32 / 255.0,
            0xE3 as f32 / 255.0,
            0x0E as f32 / 255.0,
            1.0,
        ],
        [
            0x2B as f32 / 255.0,
            0xF0 as f32 / 255.0,
            0x35 as f32 / 255.0,
            1.0,
        ],
        [
            0x0C as f32 / 255.0,
            0xF0 as f32 / 255.0,
            0xA4 as f32 / 255.0,
            1.0,
        ],
        [
            0x05 as f32 / 255.0,
            0xFB as f32 / 255.0,
            0xFF as f32 / 255.0,
            1.0,
        ],
        [
            0x5E as f32 / 255.0,
            0x5E as f32 / 255.0,
            0x5E as f32 / 255.0,
            1.0,
        ],
        [
            0x0D as f32 / 255.0,
            0x0D as f32 / 255.0,
            0x0D as f32 / 255.0,
            1.0,
        ],
        [
            0x0D as f32 / 255.0,
            0x0D as f32 / 255.0,
            0x0D as f32 / 255.0,
            1.0,
        ],
        [
            0xFF as f32 / 255.0,
            0xFF as f32 / 255.0,
            0xFF as f32 / 255.0,
            1.0,
        ],
        [
            0xA6 as f32 / 255.0,
            0xFC as f32 / 255.0,
            0xFF as f32 / 255.0,
            1.0,
        ],
        [
            0xB3 as f32 / 255.0,
            0xEC as f32 / 255.0,
            0xFF as f32 / 255.0,
            1.0,
        ],
        [
            0xDA as f32 / 255.0,
            0xAB as f32 / 255.0,
            0xEB as f32 / 255.0,
            1.0,
        ],
        [
            0xFF as f32 / 255.0,
            0xA8 as f32 / 255.0,
            0xF9 as f32 / 255.0,
            1.0,
        ],
        [
            0xFF as f32 / 255.0,
            0xAB as f32 / 255.0,
            0xB3 as f32 / 255.0,
            1.0,
        ],
        [
            0xFF as f32 / 255.0,
            0xD2 as f32 / 255.0,
            0xB0 as f32 / 255.0,
            1.0,
        ],
        [
            0xFF as f32 / 255.0,
            0xEF as f32 / 255.0,
            0xA6 as f32 / 255.0,
            1.0,
        ],
        [
            0xFF as f32 / 255.0,
            0xF7 as f32 / 255.0,
            0x9C as f32 / 255.0,
            1.0,
        ],
        [
            0xD7 as f32 / 255.0,
            0xE8 as f32 / 255.0,
            0x95 as f32 / 255.0,
            1.0,
        ],
        [
            0xA6 as f32 / 255.0,
            0xED as f32 / 255.0,
            0xAF as f32 / 255.0,
            1.0,
        ],
        [
            0xA2 as f32 / 255.0,
            0xF2 as f32 / 255.0,
            0xDA as f32 / 255.0,
            1.0,
        ],
        [
            0x99 as f32 / 255.0,
            0xFF as f32 / 255.0,
            0xFC as f32 / 255.0,
            1.0,
        ],
        [
            0xDD as f32 / 255.0,
            0xDD as f32 / 255.0,
            0xDD as f32 / 255.0,
            1.0,
        ],
        [
            0x11 as f32 / 255.0,
            0x11 as f32 / 255.0,
            0x11 as f32 / 255.0,
            1.0,
        ],
        [
            0x11 as f32 / 255.0,
            0x11 as f32 / 255.0,
            0x11 as f32 / 255.0,
            1.0,
        ],
    ];
}
