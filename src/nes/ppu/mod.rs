use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};
use errors::*;
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
    /// [PPU registers](http://wiki.nesdev.com/w/index.php/PPU_rendering#Preface)
    tile_register_high: u16,
    tile_register_low: u16,
    attr_value_register: u8,

    v_ram: Arc<Mutex<VRam>>,
}

impl Ppu {
    const HEIGHT: u16 = 240;
    const WIDTH: u16 = 256;
    const CLOCKS_PER_LINE: u32 = 341;
    const SCAN_LINES_COUNT: u32 = 262;
    pub fn new(v_ram: Arc<Mutex<VRam>>) -> Ppu {
        Ppu {
            tile_register_high: 0,
            tile_register_low: 0,
            attr_value_register: 0,
            v_ram,
        }
    }

    pub fn run(mut self) {
        let opengl = OpenGL::V3_2;
        let mut window: Sdl2Window = WindowSettings::new("nes", [Self::WIDTH as u32, Self::HEIGHT as u32])
            .opengl(opengl)
            .exit_on_esc(true)
            .build()
            .unwrap();
        let mut gl = GlGraphics::new(opengl);
        let mut events = Events::new(EventSettings::new());
        let image = Image::new().rect([0.0, 0.0, Self::WIDTH as f64, Self::HEIGHT as f64]);
        let mut image_buffer = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(Self::WIDTH as u32, Self::HEIGHT as u32);
        while let Some(e) = events.next(&mut window) {
            if let Some(args) = e.render_args() {
                let texture = Texture::from_image(&image_buffer, &TextureSettings::new());
                gl.draw(args.viewport(), |c, g| {
                    use graphics::clear;
                    image.draw(&texture, &c.draw_state, c.transform, g);
                    for line in -1..((Self::SCAN_LINES_COUNT as i64) - 1) {
                        for cycle in 0..(Self::CLOCKS_PER_LINE) {
                            match line {
                                // pre-render scanline
                                -1 => {}
                                // visual scanline
                                0...239 => {
                                    match cycle {
                                        // idle cycle
                                        0 => {}
                                        // render each pixcel
                                        1...256 => {
                                            let color = self.color_value((cycle - 1) as u16, line as u16);
                                            image_buffer.put_pixel((cycle - 1) as u32, line as u32, Rgba(color));
                                            self.shift_tile_registers();
                                            if (cycle % 8) == 0 {
                                                self.load_tile_registers(cycle as u16 - 1, line as u16);
                                                self.load_attr_value_register(cycle as u16 - 1, line as u16);
                                            }
                                        }
                                        // prepare tile data for the sprites on the next scanline
                                        257...320 => {}
                                        // the first two tiles for the next scanline are fetched, and loaded into the shift registers
                                        321...336 => {
                                            let high_lower: u8 = self.get_v_ram_value(self.pattern_table_idx_high(0, line as u16));
                                            let high_upper: u8 = self.get_v_ram_value(self.pattern_table_idx_high(8, line as u16));
                                            let low_lower: u8 = self.get_v_ram_value(self.pattern_table_idx_low(0, line as u16));
                                            let low_upper: u8 = self.get_v_ram_value(self.pattern_table_idx_low(8, line as u16));
                                            self.set_tile_register_high_lower(high_lower);
                                            self.set_tile_register_high_upper(high_upper);
                                            self.set_tile_register_low_lower(low_lower);
                                            self.set_tile_register_low_upper(low_upper);
                                        }
                                        // two byte(name table byte) are fetched
                                        337...340 => {}
                                        _ => panic!("cycle overflow: {}", cycle),
                                    }
                                }
                                // post-render scanline
                                240 => {}
                                // Vertical blanking lines
                                241...260 => {}
                                // pre-render scanline
                                261 => {}
                                _ => panic!("scanline overflow: {}", line),
                            }
                        }
                    }
                });
            }
        }
    }
    fn color_value(&self, pixel_x: u16, pixel_y: u16) -> NesColor {
        if (pixel_y >= 110 && pixel_y <= 130) {
            // println!("x:{}, y:{}, palette: {}, idx: {}", pixel_x, pixel_y, self.palette_num(pixel_x, pixel_y), self.idx_in_palette());
            // println!("high: {:0b}", self.tile_register_high_lower_value());
            // println!("low: {:0b}", self.tile_register_low_lower_value());
        }
        let color_idx: u8 =
            self.get_v_ram_value(VRam::IMAGE_PALETTE + (self.palette_num(pixel_x, pixel_y) as u16 * 4) + self.idx_in_palette() as u16);
        self.v_ram.lock().unwrap().get_color(color_idx)
    }

    /// [Shift 1bit per cycle](http://wiki.nesdev.com/w/index.php/PPU_rendering#Preface)
    fn shift_tile_registers(&mut self) {
        let high_upper: u8 = self.tile_register_high_upper_value();
        let high_lower: u8 = self.tile_register_high_lower_value();
        let low_upper: u8 = self.tile_register_low_upper_value();
        let low_lower: u8 = self.tile_register_low_lower_value();
        self.set_tile_register_high(high_upper, high_lower << 1);
        self.set_tile_register_low(low_upper, low_lower << 1);
    }

    fn load_tile_registers(&mut self, pixel_x: u16, pixel_y: u16) {
        // let name_table_idx: u16 = VRam::NAME_TABLE0 + ((pixel_x) / 8) + ((pixel_x / 8) * 32);
        let pattern_table_idx_high: u16 = self.pattern_table_idx_high(pixel_x, pixel_y);
        let pattern_table_idx_low: u16 = self.pattern_table_idx_low(pixel_x, pixel_y);

        let high_lower = self.tile_register_high_upper_value();
        self.set_tile_register_high_lower(high_lower);
        let low_lower = self.tile_register_low_upper_value();
        self.set_tile_register_low_lower(low_lower);

        let high_upper = self.get_v_ram_value(pattern_table_idx_high);
        self.set_tile_register_high_upper(high_upper);
        let low_upper = self.get_v_ram_value(pattern_table_idx_low);
        self.set_tile_register_high_upper(low_upper);
        if (pixel_y >= 110 && pixel_y <= 130) {
            // println!(
            //     "load high: {:0x}, low: {:0x}, idx_high: {:0x}, idx_low: {:0x}",
            //     high_upper,
            //     low_upper,
            //     pattern_table_idx_high,
            //     pattern_table_idx_low
            // );
        }
    }

    fn load_attr_value_register(&mut self, pixel_x: u16, pixel_y: u16) {
        let attr_idx = pixel_x / 32 + ((pixel_y / 32) * 8);
        if (pixel_y >= 110 && pixel_y <= 130) {
            // println!("v_ram_addr: {:0x}", VRam::ATTR_TABLE0 + attr_idx);
        }
        self.attr_value_register = self.get_v_ram_value(VRam::ATTR_TABLE0 + attr_idx);
    }

    fn name_table_idx(&self, pixel_x: u16, pixel_y: u16) -> u16 {
        ((pixel_x) / 8) + ((pixel_y / 8) * 32)
    }

    fn pattern_table_idx_high(&self, pixel_x: u16, pixel_y: u16) -> u16 {
        let name_table_idx = self.name_table_idx(pixel_x, pixel_y);
        if (pixel_y >= 110 && pixel_y <= 130) {
            // println!(
            //     "x: {}, y: {}, name_table_idx: {:0x}, v: {:0x}",
            //     pixel_x,
            //     pixel_y,
            //     name_table_idx + VRam::NAME_TABLE0,
            //     self.get_v_ram_value(name_table_idx + VRam::NAME_TABLE0)
            // );
        }
        VRam::PATTERN_TABLE0 + ((self.get_v_ram_value(name_table_idx + VRam::NAME_TABLE0) as u16) * 16) + pixel_y % 8
    }

    fn pattern_table_idx_low(&self, pixel_x: u16, pixel_y: u16) -> u16 {
        self.pattern_table_idx_high(pixel_x, pixel_y) + 8
    }

    /// [Get palette number from attribute table(attr value register)](http://wiki.nesdev.com/w/index.php/PPU_attribute_tables)
    fn palette_num(&self, pixel_x: u16, pixel_y: u16) -> u8 {
        if (pixel_y >= 110 && pixel_y <= 130) {
            // println!("attr: {}", self.attr_value_register);
        }
        match (pixel_x / 16) % 2 {
            0 => {
                match (pixel_y / 16) % 2 {
                    // top-left
                    0 => self.attr_value_register & 0b00000011,
                    // bottom-left
                    1 => (self.attr_value_register >> 4) & 0b00000011,
                    _ => panic!("palette num overflow  pixel_x: {}, pixel_y: {}", pixel_x, pixel_y),
                }
            }
            1 => {
                match (pixel_y / 16) % 2 {
                    // top-right
                    0 => (self.attr_value_register >> 2) & 0b00000011,
                    // bottom-right
                    1 => (self.attr_value_register >> 6) & 0b00000011,
                    _ => panic!("palette num overflow  pixel_x: {}, pixel_y: {}", pixel_x, pixel_y),
                }
            }
            _ => panic!("palette num overflow  pixel_x: {}, pixel_y: {}", pixel_x, pixel_y),
        }
    }

    fn idx_in_palette(&self) -> u8 {
        let high_lower: u8 = self.tile_register_high_lower_value();
        let low_lower: u8 = self.tile_register_low_lower_value();
        (high_lower >> 7) + (low_lower >> 7) * 2
    }

    fn get_v_ram_value(&self, address: u16) -> u8 {
        self.v_ram.lock().unwrap().get_value(address)
    }

    fn tile_register_high_upper_value(&self) -> u8 {
        (((self.tile_register_high & 0b1111111100000000) >> 8) as u8)
    }

    fn tile_register_high_lower_value(&self) -> u8 {
        ((self.tile_register_high & 0b111111111111) as u8)
    }

    fn tile_register_low_upper_value(&self) -> u8 {
        (((self.tile_register_low & 0b1111111100000000) >> 8) as u8)
    }

    fn tile_register_low_lower_value(&self) -> u8 {
        ((self.tile_register_low & 0b111111111111) as u8)
    }

    fn set_tile_register_high(&mut self, upper: u8, lower: u8) {
        self.tile_register_high = ((upper as u16) << 8) + (lower as u16);
    }

    fn set_tile_register_low(&mut self, upper: u8, lower: u8) {
        self.tile_register_low = ((upper as u16) << 8) + (lower as u16);
    }

    fn set_tile_register_high_upper(&mut self, value: u8) {
        let lower = self.tile_register_high_lower_value();
        self.set_tile_register_high(value, lower);
    }

    fn set_tile_register_high_lower(&mut self, value: u8) {
        let upper = self.tile_register_high_upper_value();
        self.set_tile_register_high(upper, value);
    }

    fn set_tile_register_low_upper(&mut self, value: u8) {
        let lower = self.tile_register_low_lower_value();
        self.set_tile_register_low(value, lower);
    }

    fn set_tile_register_low_lower(&mut self, value: u8) {
        let upper = self.tile_register_low_upper_value();
        self.set_tile_register_low(upper, value);
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
    const PATTERN_TABLE0: u16 = 0x0000;
    const NAME_TABLE0: u16 = 0x2000;
    const ATTR_TABLE0: u16 = 0x23C0;
    const IMAGE_PALETTE: u16 = 0x3F00;

    pub fn new(memory: Box<[u8; 0x10000]>) -> VRam {
        VRam(memory)
    }

    pub fn get_value(&self, address: u16) -> u8 {
        self.0[address as usize]
    }

    fn get_color(&self, idx: u8) -> NesColor {
        NesColors::COLORS[idx as usize]
    }

    pub fn set(&mut self, address: u16, data: u8) {
        self.0[address as usize] = data;
    }

    fn dump(&self) {
        let dump_file = "v_ram.dump";
        let mut f = BufWriter::new(File::create(dump_file).unwrap());
        for v in self.0.iter() {
            f.write(&[*v]).unwrap();
        }
    }
}

/// [PPU internal register](http://wiki.nesdev.com/w/index.php/PPU_scrolling#PPU_internal_registers)
/// yyy NN YYYYY XXXXX == VRam Index
/// ||| || ||||| +++++-- coarse X scroll(x index)
/// ||| || +++++-------- coarse Y scroll(y index)
/// ||| ++-------------- nametable select
/// +++----------------- fine Y scroll(y offsetr from scanline)
pub struct VRamAddressRegister(u16);
impl VRamAddressRegister {
    pub fn new() -> VRamAddressRegister {
        VRamAddressRegister(0)
    }

    pub fn value(&self) -> u16 {
        self.0
    }

    fn fine_y_scroll(&self) -> u8 {
        ((self.0 >> 12) & 0b0111) as u8
    }

    fn name_table(&self) -> u8 {
        ((self.0 >> 10) & 0b000011) as u8
    }

    fn coarse_y_scroll(&self) -> u8 {
        ((self.0 >> 5) & 0b00000011111) as u8
    }

    fn coarse_x_scroll(&self) -> u8 {
        (self.0 & 0b0000000000011111) as u8
    }

    // fn fetch_vram_address(&mut self, vram_offset_flag: bool) -> (u8, u8, u8, u8) {
    //     let fine_y_scroll = self.y_offset_from_scanline;
    //     let name_table_num = self.name_table_num;
    //     let y_panel_pos = self.y_idx;
    //     let x_panel_pos = self.x_idx;
    //     if vram_offset_flag {
    //         self.y_idx += 1;
    //     } else {
    //         self.x_idx += 1;
    //     }
    //     (fine_y_scroll, name_table_num, y_panel_pos, x_panel_pos)
    // }
    // pub fn set_y_offset_from_scanline(&mut self, offset: u8) {
    //     self.y_offset_from_scanline = offset;
    // }
    pub fn set_fine_y_scroll(&mut self, value: u8) -> Result<()> {
        match value {
            0...0b111 => {
                self.0 = (self.0 & 0b0000111111111111) + ((value as u16) << 12);
                Ok(())
            }
            _ => Err(format!("fine_y_scroll overflow : {:0b}", value))?,
        }
    }

    pub fn set_name_table(&mut self, value: u8) -> Result<()> {
        match value {
            0...0b11 => {
                self.0 = (self.0 & 0b0111001111111111) + ((value as u16) << 10);
                Ok(())
            }
            _ => Err(format!("name_table overflow : {:0b}", value))?,
        }
    }

    pub fn set_coarse_y_scroll(&mut self, value: u8) -> Result<()> {
        match value {
            0...0b11111 => {
                self.0 = (self.0 & 0b0111110000011111) + ((value << 5) as u16);
                Ok(())
            }
            _ => Err(format!("corse_y_scroll overflow : {:0b}", value))?,
        }
    }

    pub fn set_coarse_x_scroll(&mut self, value: u8) -> Result<()> {
        match value {
            0...0b11111 => {
                self.0 = (self.0 & 0b0111111111100000) + (value as u16);
                Ok(())
            }
            _ => Err(format!("corse_x_scroll overflow : {:0b}", value))?,
        }
    }

    pub fn set_upper_6bits(&mut self, value: u8) -> Result<()> {
        match value {
            0...0b111111 => {
                self.0 = (self.0 & 0b0000000011111111) + ((value as u16) << 8);
                Ok(())
            }
            _ => Err(format!("upper 6bits overflow : {:0b}", value))?,
        }
    }

    pub fn set_lower_8bits(&mut self, value: u8) {
        self.0 = (self.0 & 0b1111111100000000) + (value as u16);
    }

    pub fn set_full_15bits(&mut self, value: u16) -> Result<()> {
        match value {
            0...0b111111111111111 => {
                self.0 = value;
                Ok(())
            }
            _ => Err(format!("full 15bits overflow : {:0b}", value))?,
        }
    }
    pub fn increment_x(&mut self) {
        self.0 += 1;
    }

    pub fn increment_y(&mut self) {
        self.0 += 0b000000000100000
    }
    // pub fn set_y_idx(&mut self, idx: u8) {
    //     self.y_idx = idx;
    // }
    // pub fn set_x_idx(&mut self, idx: u8) {
    //     self.x_idx = idx;
    // }
    // pub fn increment(&mut self, flag: bool) {
    //     if flag {
    //         self.y_idx += 1;
    //     } else {
    //         self.x_idx += 1;
    //     }
    // }
    // pub fn dump(&self) -> u16 {
    // let y_scroll = (self.y_offset_from_scanline as u16) << 12;
    // let name_table_num = (self.name_table_num as u16) << 10;
    // let y = (self.y_idx as u16) << 5;
    // y_scroll + name_table_num + y + (self.x_idx as u16)
    // }
}

pub struct FineXScroll(u8);
impl FineXScroll {
    pub fn new(value: u8) -> FineXScroll {
        FineXScroll(value)
    }

    pub fn set_value(&mut self, value: u8) -> Result<()> {
        match value {
            0...0b111 => {
                self.0 = value & 0b00000111;
                Ok(())
            }
            _ => Err(format!("fine_x_scroll overflow : {:0x}", value))?,
        }
    }
}
/// [PPU internal register](http://wiki.nesdev.com/w/index.php/PPU_scrolling#PPU_internal_registers)
pub struct FirstOrSecondWriteToggle(bool);
impl FirstOrSecondWriteToggle {
    pub fn new() -> FirstOrSecondWriteToggle {
        FirstOrSecondWriteToggle(false)
    }

    pub fn set(&mut self, flag: bool) {
        self.0 = flag;
    }

    pub fn toggle(&mut self) {
        match self.0 {
            true => self.0 = false,
            false => self.0 = true,
        }
    }

    pub fn is_true(&self) -> bool {
        self.0
    }
}


struct NesColors;
type NesColor = [u8; 4];
impl NesColors {
    const COLORS: [[u8; 4]; 64] = [
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
}
