extern crate graphics;
use nes::piston_window::Context;
use nes::piston_window::Graphics;
use nes::cpu::Cpu;
use nes::cpu::PrgRam;
use std::sync::{Arc, Mutex};
pub struct Ppu2 {
    //sprite

    //background
    v_ram: Box<VRam>,
    vram_address_register: VramAddressRegister,
    temporary_vram_address: VramAddressRegister, // yyy, NN, YYYYY, XXXXX
    fine_x_scroll: u8,
    first_or_second_write_toggle: bool,
    background_shift_register16_0: (u8, u8),
    background_shift_register16_1: (u8, u8),
    background_shift_register8_0: u8,
    background_shift_register8_1: u8,

    // vram_index: u16,
    prg_ram: Arc<Mutex<PrgRam>>,
}

impl Ppu2 {
    pub fn new(prg_ram: Arc<Mutex<PrgRam>>) -> Ppu2 {
        Ppu2 {
            v_ram: Box::new(VRam([0; 0xFFFF])),
            vram_address_register: VramAddressRegister::new(),
            temporary_vram_address: VramAddressRegister::new(),
            fine_x_scroll: 0,
            first_or_second_write_toggle: false,
            background_shift_register16_0: (0, 0),
            background_shift_register16_1: (0, 0),
            background_shift_register8_0: 0,
            background_shift_register8_1: 0,
            prg_ram,
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
    vram_address_register: VramAddressRegister,
    temporary_vram_address: VramAddressRegister, // yyy, NN, YYYYY, XXXXX
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
                Rectangle::new([0.0, 1.0, 0.0, 1.0]).draw([0.0, 0.0, 85.0, 80.0], &c.draw_state, c.transform, g);
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
    const NAME_TABLE0: u16 = 0x2000;
    const NAME_TABLE1: u16 = 0x2400;
    const NAME_TABLE2: u16 = 0x2800;
    const NAME_TABLE3: u16 = 0x2C00;

    const ATTR_TABLE0: u16 = 0x23C0;
    const ATTR_TABLE1: u16 = 0x27C0;
    const ATTR_TABLE2: u16 = 0x2BC0;
    const ATTR_TABLE3: u16 = 0x2FC0;

    const PATTERN_TABLE0: u16 = 0x0000;
    const PATTERN_TABLE1: u16 = 0x1000;

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
}

struct VramAddressRegister {
    y_offset_from_scanline: u8,
    name_table_num: u8,
    y_idx: u8,
    x_idx: u8,
}

impl VramAddressRegister {
    fn new() -> VramAddressRegister {
        VramAddressRegister {
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
}
