struct Ppu {
    cycle: u16,
    line: u8,
    name_table_num: u8,
    name_table_idx: u16,
    name_table_value: u8,
    ram: Box<VRam>,
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

pub struct VRam([u8; 0xFFFF]);
impl VRam {
    const NAME_TABLE0: u16 = 0x2000;
    const NAME_TABLE1: u16 = 0x2400;
    const NAME_TABLE2: u16 = 0x2800;
    const NAME_TABLE3: u16 = 0x2C00;

    fn get_name_table_value(&self, table_num: u8, index: u16) -> u8 {
        match table_num {
            0 => self.0[(Self::NAME_TABLE0 + index) as usize],
            1 => self.0[(Self::NAME_TABLE1 + index) as usize],
            2 => self.0[(Self::NAME_TABLE2 + index) as usize],
            3 => self.0[(Self::NAME_TABLE3 + index) as usize],
            _ => {
                panic!("table{} doesn't exist", {
                    table_num
                })
            }
        }
    }
}
