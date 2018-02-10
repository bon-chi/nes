use std::mem;
use std::ops::Range;
use std::path::Path;
use std::io::Read;
use std::fs::File;

/// [CPU](http://wiki.nesdev.com/w/index.php/CPU_registers)
pub struct Cpu {
    /// Accumulator
    a: Register8,
    /// Indexe Register
    x: Register8,
    y: Register8,
    /// Program Counter
    pc: Register16,
    sp: Register8,
    p: Register8,
    ram: Box<PrgRam>,
}

enum StatusFlag {
    CarryFlag,
    ZeroFlag,
    InterruptDisable,
    DecimalMode,
    BreakCommand,
    OverflowFlag,
    NegativeFlag,
}

impl Cpu {
    pub fn new(path: &Path) -> Cpu {
        println!("hoge2");
        // println!("{}", mem::size_of_val(&ram));
        Cpu {
            a: b'0',
            x: b'0',
            y: b'0',
            pc: 0x8000,
            sp: b'0',
            p: b'0',
            ram: PrgRam::load(path),
        }
    }

    pub fn run(&self) {
        self.fetch();
        self.exec();
    }
    fn fetch(&self) {
        println!("{}", self.ram.0[self.pc as usize]);
    }
    fn fetch_instruction_to_ir(&self) {}
    fn increment_pc(&self) {}
    fn decode_instruction(&self) {}
    // exec instruction
    fn fetch_store_address(&self) {}
    fn fetch_operand(&self) {}

    // jump instruction
    fn check_condition(&self) {}
    fn fetch_jump_address(&self) {}

    fn exec(&self) {}
    // exec instruction
    fn do_exec(&self) {}
    fn store_resutl(&self) {}
    // jump instruction
    fn load_address_to_pc(&self) {}

    fn set_pc(&mut self, value: Register16) {
        self.pc = value;
    }

    fn set_flag(&mut self, status_flag: StatusFlag) {
        match status_flag {
            StatusFlag::CarryFlag => {
                self.pc = self.pc | 0b00000001;
            }
            StatusFlag::ZeroFlag => {
                self.pc = self.pc | 0b00000010;
            }
            StatusFlag::InterruptDisable => {
                self.pc = self.pc | 0b00000100;
            }
            StatusFlag::DecimalMode => {
                self.pc = self.pc | 0b00001000;
            }
            StatusFlag::BreakCommand => {
                self.pc = self.pc | 0b00010000;
            }
            StatusFlag::OverflowFlag => {
                self.pc = self.pc | 0b01000000;
            }
            StatusFlag::NegativeFlag => {
                self.pc = self.pc | 0b10000000;
            }
        }
    }

    fn set_p(&mut self, value: Register8) {
        self.p = value;
    }


    fn reset(&mut self, ram: &mut PrgRam) {
        // ram.set8(MemoryMap::STACk_ADDRESSES.index(0), self.sp);
        // 3
        ram.set8(MemoryMap::STACk_ADDRESS + (self.sp as u16) - 2, self.p);
        ram.set16(
            (
                MemoryMap::STACk_ADDRESS + (self.sp as u16) - 1,
                MemoryMap::STACk_ADDRESS + (self.sp as u16),
            ),
            self.pc,
        );
        // 4
        self.set_flag(StatusFlag::InterruptDisable);

        // 5
        self.set_pc(ram.fetch16(MemoryMap::RESET_ADDRESSES));
    }
    fn nmi(&self) {}
    fn irq(&self) {}
}

struct MemoryMap {
    map: [u8; 2 ^ 16],
}

impl MemoryMap {
    const STACk_ADDRESSES: Range<u16> = 0x0100..0x0200;
    const STACk_ADDRESS: u16 = 0x0100;
    const NMI_ADDRESSES: (u16, u16) = (0xFFFA, 0xFFFB);
    const RESET_ADDRESSES: (u16, u16) = (0xFFFC, 0xFFFD);
    const IRQ_ADDRESSES: (u16, u16) = (0xFFFE, 0xFFFF);
}
// trait ControlBus {
//     fn fetch8(&self) -> u8;
// }
// trait AddressBus {
//     fn fetch16(&self) -> u16;
// }

type Register8 = u8;
type Register16 = u16;

// struct PrgRam {
//     memory: [u8; 2 ^ 16],
//     // ppu_memory: [u8; 2 ^ 16],
// }
// #[derive(Debug)]
pub struct PrgRam([u8; 0xFFFF]);

impl PrgRam {
    pub fn load(path: &Path) -> Box<PrgRam> {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(why) => panic!("{}: path is {:?}", why, path),
        };
        // let mut nes_buffer: [u8; 16] = [0; 16];
        let mut nes_buffer: Vec<u8> = Vec::new();
        let result = file.read_to_end(&mut nes_buffer).unwrap();
        let nes_header_size = 0x0010;

        let prg_rom_banks_num = nes_buffer[4];
        let prg_rom_start = nes_header_size;
        let prg_rom_end = prg_rom_start + prg_rom_banks_num as u64 * 0x4000 - 1;

        let chr_rom_banks_num = nes_buffer[5];
        let chr_rom_start = prg_rom_end + 1;
        let chr_rom_end = chr_rom_start + chr_rom_banks_num as u64 * 0x2000 - 1;

        println!(
            "{} - {} = {}: {} - {} = {}",
            prg_rom_end,
            prg_rom_start,
            prg_rom_end - prg_rom_start,
            chr_rom_end,
            chr_rom_start,
            chr_rom_end - chr_rom_start,
        );
        println!(
            "{:0x} - {:0x} = {:0x}: {:0x} - {:0x} = {:0x}",
            prg_rom_end,
            prg_rom_start,
            prg_rom_end - prg_rom_start,
            chr_rom_end,
            chr_rom_start,
            chr_rom_end - chr_rom_start,
        );

        let mut prg_rom: Vec<u8> = Vec::new();
        for i in prg_rom_start..(prg_rom_end + 1) {
            prg_rom.push(nes_buffer[i as usize]);
        }

        let mut chr_rom: Vec<u8> = Vec::new();
        for i in chr_rom_start..(chr_rom_end + 1) {
            chr_rom.push(nes_buffer[i as usize]);
        }

        let mut memory: [u8; 0xFFFF] = [0; 0xFFFF];

        let memory_idx_low = 0x8000;
        let memory_idx_high = 0xC000;
        let mut prg_rom_idx = 0;
        for memory_idx in (memory_idx_low..memory_idx_high) {
            memory[memory_idx] = prg_rom[prg_rom_idx];
            prg_rom_idx += 1;
        }


        match chr_rom_banks_num {
            1 => {
                prg_rom_idx = 0;
                for memory_idx in (memory_idx_low..memory_idx_high) {
                    memory[memory_idx] = prg_rom[prg_rom_idx];
                    prg_rom_idx += 1;
                }
            }
            2...255 => {
                let memory_idx_end = 0x10000;
                for memory_idx in (memory_idx_high..memory_idx_end) {
                    memory[memory_idx] = prg_rom[prg_rom_idx];
                    prg_rom_idx += 1;
                }
            }
            _ => {}
        }


        // let pattern_table0_idx = 0x0000;
        // let pattern_table1_idx = 0x1000;
        // let mut chr_rom_idx = 0;
        // for ppu_memory_idx in (pattern_table0_idx..pattern_table1_idx) {
        //     self.ppu_memory[ppu_memory_idx] = chr_rom[chr_rom_idx];
        //     chr_rom_idx += 1;
        // }
        //
        // let pattern_table_end = 0x2000;
        // for ppu_memory_idx in (pattern_table1_idx..pattern_table_end) {
        //     self.ppu_memory[ppu_memory_idx] = chr_rom[chr_rom_idx];
        //     chr_rom_idx += 1;
        // }
        //
        // let mut program_rom_up: [u8; 4];
        // let mut pattern_table0: [u8; 0x1000];
        // let mut pattern_table1: [u8; 0x1000];
        // println!("{:?}", memory[0x8000]);
        println!("{}", mem::size_of_val(&memory));
        // println!("hoge1");
        Box::new(PrgRam(memory))
        // prg
    }
    fn fetch8(&self, address: u16) -> Register8 {
        self.0[address as usize]
    }

    fn fetch16(&self, addresses: (u16, u16)) -> Register16 {
        (((self.0[addresses.1 as usize] as u16) << 0b1000) + (self.0[addresses.0 as usize]) as u16)
    }

    fn set8(&mut self, address: u16, data: u8) {
        self.0[address as usize] = data;
    }
    fn set16(&mut self, addressess: (u16, u16), data: u16) {
        self.0[addressess.0 as usize] = (data % 0x100) as u8;
        self.0[addressess.1 as usize] = (data >> 0b100) as u8;
    }
}
