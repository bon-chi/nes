extern crate piston_window;
use std::fmt;
use std::ops::Range;
use std::path::Path;
use std::io::Read;
use std::fs::File;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::sync::{Arc, Mutex};
use self::piston_window::Key;
use nes::ppu::{Ppu, VRam, VRamAddressRegister, FirstOrSecondWriteToggle};

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
    ram: Arc<Mutex<PrgRam>>,
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
    pub fn new2() {}
    pub fn new(ram: Arc<Mutex<PrgRam>>) -> Cpu {
        // println!("{}", mem::size_of_val(&ram));
        Cpu {
            a: b'0',
            x: b'0',
            y: b'0',
            pc: 0x8000,
            sp: b'0',
            p: b'0',
            ram: ram,
        }
    }

    pub fn run(&mut self, tx: Sender<u8>, rxk: Receiver<Option<Key>>) {

        loop {
            // tx.send(1);
            let (op_code, addressing_mode, register) = self.fetch();
            let operand = self.get_operand(addressing_mode, register);
            self.exec(op_code, operand);
            // rxk.recv().unwrap();
            // println!("{:?}", rxk.recv().unwrap());
        }
    }
    fn fetch(&mut self) -> (OpCode, AddressingMode, Option<IndexRegister>) {
        let instruction = self.ram_value();
        let (op_code, addressing_mode, register): (OpCode, AddressingMode, Option<IndexRegister>) = match instruction {

            0x20 => (OpCode::JSR, AddressingMode::Absolute, None),
            0x21 => (OpCode::AND, AddressingMode::IndexedIndirect, None),
            0x4c => (OpCode::JMP, AddressingMode::Absolute, None),
            0x78 => (OpCode::SEI, AddressingMode::Implied, None),
            0x88 => (OpCode::DEY, AddressingMode::Implied, None),
            0x8d => (OpCode::STA, AddressingMode::Absolute, None),
            0x9a => (OpCode::TXS, AddressingMode::Implied, None),
            0xa0 => (OpCode::LDY, AddressingMode::Immediate, None),
            0xa2 => (OpCode::LDX, AddressingMode::Immediate, None),
            0xa9 => (OpCode::LDA, AddressingMode::Immediate, None),
            0xbd => {
                (
                    OpCode::LDA,
                    AddressingMode::Absolute,
                    Some(IndexRegister::X),
                )
            }
            0xd0 => (OpCode::BNE, AddressingMode::Relative, None),
            0xe8 => (OpCode::INX, AddressingMode::Implied, None),
            0x78 => (OpCode::SEI, AddressingMode::Implied, None),
            _ => {
                panic!(
                    "unknown instruction: {:0x} at: {:0x}, cpu_dump: {:?}",
                    instruction,
                    self.pc,
                    self
                );
            }
        };
        // #[cfg(feature="debug_log")]
        // match op_code {
        //     OpCode::JMP => {}
        //     _ => {
        //         println!(
        //     "{:0x} {:?} {:?} {:?} ",
        //     self.pc,
        //     op_code,
        //     addressing_mode,
        //     register,
        // );
        //     }
        // }


        // #[cfg(not(test))]
        // println!("debughoge");

        self.increment_pc();
        (op_code, addressing_mode, register)
    }
    fn get_operand(&mut self, addressing_mode: AddressingMode, register: Option<IndexRegister>) -> Option<Operand> {
        let operand: Option<Operand> = match addressing_mode {
            AddressingMode::Implied => None,
            AddressingMode::Immediate => {
                let operand = self.ram_value();
                self.increment_pc();
                Some(Operand::Value(operand))
            }
            AddressingMode::Absolute => {
                match register {
                    Some(IndexRegister::X) => {
                        let address0 = self.ram_value();
                        self.increment_pc();
                        let address1 = self.ram_value();
                        self.increment_pc();
                        let address = PrgRam::concat_addresses(address1, address0) + self.x as u16;
                        Some(Operand::Index(address))
                    }
                    None => {
                        let address0 = self.ram_value();
                        self.increment_pc();
                        let address1 = self.ram_value();
                        self.increment_pc();
                        let address = PrgRam::concat_addresses(address1, address0) as u16;
                        Some(Operand::Index(address))
                    }
                    _ => {
                        panic!(
                            "invalid register: {:?}, addressing_mode is {:?}",
                            register,
                            addressing_mode
                        );
                    }
                }
            }
            AddressingMode::Relative => {
                let index = match self.ram_value() as i8 {
                    i @ -128...0 => self.pc + 1 - ((-i).abs() as u16),
                    i @ 0...127 => self.pc + 1 + (i as u16),
                    _ => panic!("invalid"),
                };
                Some(Operand::Index(index))
            }
            AddressingMode::IndexedIndirect => {
                match register {
                    Some(IndexRegister::X) => {
                        let bb = PrgRam::concat_addresses(0x00 as u8, self.ram_value()) + (self.x as u16);
                        let xx: u8 = self.get_ram_value(bb);
                        let yy: u8 = self.get_ram_value(bb + 1);
                        self.increment_pc();
                        Some(Operand::Index(PrgRam::concat_addresses(yy, xx)))
                    }
                    _ => {
                        panic!(
                            "invalid register: {:?}, addressing_mode is {:?}",
                            register,
                            addressing_mode
                        );

                    }
                }
            }
            _ => panic!("invalid AddressingMode: {:?}", addressing_mode),
        };
        operand
    }

    fn ram_value(&self) -> u8 {
        self.ram.lock().unwrap().memory[self.pc as usize]
    }
    fn get_ram_value(&self, idx: u16) -> u8 {
        self.ram.lock().unwrap().memory[idx as usize]
    }
    fn set_ram_value(&mut self, value: u8, idx: u16) {
        self.ram.lock().unwrap().set8(idx, value);
    }
    fn fetch_instruction_to_ir(&self) {}
    fn increment_pc(&mut self) {
        self.pc = self.pc + 1;
    }
    fn decode_instruction(&self) {}
    // exec instruction
    fn fetch_store_address(&self) {}
    fn fetch_operand(&self) {}

    // jump instruction
    fn check_condition(&self) {}
    fn fetch_jump_address(&self) {}

    fn exec(&mut self, op_code: OpCode, operand: Option<Operand>) {
        match op_code {
            OpCode::ADC | OpCode::SBC | OpCode::ORA | OpCode::EOR | OpCode::INC | OpCode::DEC | OpCode::DEX | OpCode::INY => {
                // check zeroflag
                panic!("not inplemented");
            }
            OpCode::SEI => self.sei(),
            OpCode::LDA => {
                match operand {
                    Some(Operand::Index(idx)) => self.a = self.get_ram_value(idx),
                    Some(Operand::Value(value)) => self.a = value,
                    _ => panic!("invalid operand: {:?}", operand),
                }
            }
            OpCode::LDX => {
                match operand {
                    Some(Operand::Index(idx)) => self.x = self.get_ram_value(idx),
                    Some(Operand::Value(value)) => self.x = value,
                    _ => panic!("invalid operand: {:?}", operand),
                }
            }
            OpCode::LDY => {
                match operand {
                    Some(Operand::Index(idx)) => self.y = self.get_ram_value(idx),
                    Some(Operand::Value(value)) => self.y = value,
                    _ => panic!("invalid operand: {:?}", operand),
                }
            }
            OpCode::TXS => self.x = self.sp,
            OpCode::STA => {
                match operand {
                    Some(Operand::Index(idx)) => {
                        let a = self.a;
                        self.set_ram_value(a, idx)
                    }
                    _ => panic!("invalid operand: {:?}", operand),

                }
            }
            OpCode::INX => {
                self.x += 1;
                if self.x == 0 {
                    self.set_flag(StatusFlag::ZeroFlag);
                } else {
                    self.reset_flag(StatusFlag::ZeroFlag);
                }
            }
            OpCode::DEY => {
                self.y -= 1;
                if self.y == 0 {
                    self.set_flag(StatusFlag::ZeroFlag);
                } else {
                    self.reset_flag(StatusFlag::ZeroFlag);
                }
            }
            OpCode::BNE => {
                println!("zero_flag: {}", self.get_zero_flag());
                if !self.get_zero_flag() {
                    self.increment_pc();
                    let dest = self.ram.lock().unwrap().memory[(self.pc + 1) as usize] as u16;
                    match operand {
                        Some(Operand::Index(idx)) => self.pc = idx,
                        _ => panic!("BNE invalid operand: {:?}", operand),
                    }
                } else {
                    self.pc = self.pc + 1;
                }
            }
            OpCode::JMP => {
                match operand {
                    Some(Operand::Index(index)) => {
                        self.pc = index;
                    }
                    _ => panic!("JMP invalid operand: {:?}", operand),
                }
            }
            OpCode::JSR => {
                match operand {
                    Some(Operand::Index(index)) => {
                        self.pc -= 1;
                        let (high, low) = PrgRam::split_address(self.pc);
                        self.push_stack(high);
                        self.push_stack(low);
                        println!("jump:{:0x}", index);
                        self.pc = index;
                    }
                    _ => panic!("JSR invalid operand: {:?}", operand),
                }
            }
            OpCode::AND => {
                match operand {
                    Some(Operand::Index(idx)) => {
                        self.a = self.ram_value() & self.a;
                        if self.a == 0 {
                            self.set_flag(StatusFlag::ZeroFlag);
                        } else {
                            self.reset_flag(StatusFlag::ZeroFlag);
                        }
                    }
                    _ => panic!("invalid operand: {:?}", operand),
                }
            }
            _ => {
                panic!(
                    "invalid exec op_code: {:?}, operand is {:?}",
                    op_code,
                    operand
                )
            }
        }
    }
    // exec instruction
    fn do_exec(&self) {}
    fn store_resutl(&self) {}
    // jump instruction
    fn load_address_to_pc(&self) {}

    fn set_pc(&mut self, value: Register16) {
        self.pc = value;
    }

    fn get_zero_flag(&self) -> bool {
        println!("self.p: {:0b}, &: {:0b}", self.p, self.p & 0b00000010);
        (self.p & 0b00000010) == 0b00000010
    }
    fn reset_flag(&mut self, status_flag: StatusFlag) {
        match status_flag {
            StatusFlag::CarryFlag => {
                self.p = self.p & 0b11111110;
            }
            StatusFlag::ZeroFlag => {
                self.p = self.p & 0b11111101;
            }
            StatusFlag::InterruptDisable => {
                self.p = self.p & 0b11111011;
            }
            StatusFlag::DecimalMode => {
                self.p = self.p & 0b11110111;
            }
            StatusFlag::BreakCommand => {
                self.p = self.p & 0b11101111;
            }
            StatusFlag::OverflowFlag => {
                self.p = self.p & 0b10111111;
            }
            StatusFlag::NegativeFlag => {
                self.p = self.p & 0b01111111;
            }
        }
    }
    fn set_flag(&mut self, status_flag: StatusFlag) {
        match status_flag {
            StatusFlag::CarryFlag => {
                self.p = self.p | 0b00000001;
            }
            StatusFlag::ZeroFlag => {
                self.p = self.p | 0b00000010;
            }
            StatusFlag::InterruptDisable => {
                self.p = self.p | 0b00000100;
            }
            StatusFlag::DecimalMode => {
                self.p = self.p | 0b00001000;
            }
            StatusFlag::BreakCommand => {
                self.p = self.p | 0b00010000;
            }
            StatusFlag::OverflowFlag => {
                self.p = self.p | 0b01000000;
            }
            StatusFlag::NegativeFlag => {
                self.p = self.p | 0b10000000;
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

    fn sei(&mut self) {
        self.set_flag(StatusFlag::InterruptDisable);
    }
    fn push_stack(&mut self, value: u8) {
        let address = PrgRam::concat_addresses(0x01 as u8, self.sp);
        self.set_ram_value(value, address);
        self.sp -= 1;
    }

    fn pop_stack(&mut self) -> u8 {
        self.sp += 1;
        let address = PrgRam::concat_addresses(0x01 as u8, self.sp);
        self.get_ram_value(address)
    }

    // for vram methods
    pub fn vram_offset_flag(&self) -> bool {
        self.ram.lock().unwrap().vram_offset_flag()
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "A:{:0x}, X:{:0x}, Y:{:0x}, PC:{:0x}, SP:{:0x}, P:{:0x}",
            self.a,
            self.x,
            self.y,
            self.pc,
            self.sp,
            self.p,
        )
    }
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
pub struct PrgRam {
    memory: Box<[u8; 0xFFFF]>,
    v_ram_address_register: Arc<Mutex<VRamAddressRegister>>, // yyy, NN, YYYYY, XXXXX
    temporary_v_ram_address: Arc<Mutex<VRamAddressRegister>>, // yyy, NN, YYYYY, XXXXX
    fine_x_scroll: Arc<Mutex<u8>>,
    first_or_second_write_toggle: Arc<Mutex<FirstOrSecondWriteToggle>>,
    v_ram: Arc<Mutex<VRam>>,
}

impl PrgRam {
    pub fn load(
        path: &Path,
        v_ram_address_register: Arc<Mutex<VRamAddressRegister>>,
        temporary_v_ram_address: Arc<Mutex<VRamAddressRegister>>,
        fine_x_scroll: Arc<Mutex<u8>>,
        first_or_second_write_toggle: Arc<Mutex<FirstOrSecondWriteToggle>>,
        v_ram: Arc<Mutex<VRam>>,
    ) -> PrgRam {
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

        // println!(
        //     "{} - {} = {}: {} - {} = {}",
        //     prg_rom_end,
        //     prg_rom_start,
        //     prg_rom_end - prg_rom_start,
        //     chr_rom_end,
        //     chr_rom_start,
        //     chr_rom_end - chr_rom_start,
        // );
        // println!(
        //     "{:0x} - {:0x} = {:0x}: {:0x} - {:0x} = {:0x}",
        //     prg_rom_end,
        //     prg_rom_start,
        //     prg_rom_end - prg_rom_start,
        //     chr_rom_end,
        //     chr_rom_start,
        //     chr_rom_end - chr_rom_start,
        // );

        let mut prg_rom: Vec<u8> = Vec::new();
        for i in prg_rom_start..(prg_rom_end + 1) {
            prg_rom.push(nes_buffer[i as usize]);
        }

        let mut chr_rom: Vec<u8> = Vec::new();
        for i in chr_rom_start..(chr_rom_end + 1) {
            chr_rom.push(nes_buffer[i as usize]);
        }

        let mut memory: Box<[u8; 0xFFFF]> = Box::new([0; 0xFFFF]);

        let memory_idx_low = 0x8000;
        let memory_idx_high = 0xC000;
        let mut prg_rom_idx = 0;
        for memory_idx in memory_idx_low..memory_idx_high {
            memory[memory_idx] = prg_rom[prg_rom_idx];
            prg_rom_idx += 1;
        }


        match chr_rom_banks_num {
            1 => {
                prg_rom_idx = 0;
                for memory_idx in memory_idx_low..memory_idx_high {
                    memory[memory_idx] = prg_rom[prg_rom_idx];
                    prg_rom_idx += 1;
                }
            }
            2...255 => {
                let memory_idx_end = 0x10000;
                for memory_idx in memory_idx_high..memory_idx_end {
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
        // println!("hoge1");
        PrgRam {
            memory,
            v_ram_address_register,
            temporary_v_ram_address,
            fine_x_scroll,
            first_or_second_write_toggle,
            v_ram,
        }
        // prg
    }
    fn fetch8(&self, address: u16) -> Register8 {
        let data = self.memory[address as usize];
        if address == 0x2007 {
            self.v_ram_address_register.lock().unwrap().increment(
                self.vram_offset_flag(),
            )
        }
        data
    }

    fn fetch16(&self, addresses: (u16, u16)) -> Register16 {
        let data = (((self.memory[addresses.1 as usize] as u16) << 0b1000) + (self.memory[addresses.0 as usize]) as u16);
        if addresses.0 == 0x2007 || addresses.1 == 0x2007 {
            self.v_ram_address_register.lock().unwrap().increment(
                self.vram_offset_flag(),
            )
        }
        data
    }

    pub fn concat_addresses(address0: u8, address1: u8) -> u16 {
        ((address0 as u16) << 0b1000) + address1 as u16
    }
    fn split_address(address: u16) -> (u8, u8) {
        ((address >> 0b1000) as u8, (address % 0x100) as u8)
    }
    fn set8(&mut self, address: u16, data: u8) {
        println!("set PrgRam {:0x}, {:0x}", address, data);
        match address {
            0x2000 => {
                let name_table = data & 0b00000011;
                self.temporary_v_ram_address
                    .lock()
                    .unwrap()
                    .set_name_table(name_table);
            }
            0x2002 => {
                self.first_or_second_write_toggle.lock().unwrap().set(false);
            }
            0x2005 => {
                match self.first_or_second_write_toggle.lock().unwrap().is_true() {
                    false => {
                        *self.fine_x_scroll.lock().unwrap() = data & 0b00000111;
                        let x = (data >> 3) * 0b00011111;
                        self.temporary_v_ram_address.lock().unwrap().set_x_idx(x);
                    }
                    true => {
                        let fine_y_scroll = data & 0b00000111;
                        let coarse_y_scroll = (data >> 3) & 0b00011111;
                        self.temporary_v_ram_address
                            .lock()
                            .unwrap()
                            .set_y_offset_from_scanline(fine_y_scroll);
                        self.temporary_v_ram_address.lock().unwrap().set_y_idx(
                            coarse_y_scroll,
                        );
                    }
                }
                self.first_or_second_write_toggle.lock().unwrap().toggle();
            }
            0x2006 => {
                match self.first_or_second_write_toggle.lock().unwrap().is_true() {
                    false => {
                        let (_, _, y, _x) = self.temporary_v_ram_address
                            .lock()
                            .unwrap()
                            .get_vram_address();
                        println!("y: {},x:{}, data: {}", y, _x, data);
                        let y_scroll = (data >> 4) & 0b00000011;
                        let name_table = (data >> 2) & 0b00000011;
                        // let y = y | ((data & 0b00000011) << 3);
                        // let y = (((y >> 3) | (data & 0b00000011)) << 3) + (y & 0b00000111);
                        let y = ((data & 0b00000011) << 3) + (y & 0b00000111);
                        // println!(
                        //     "0x:2006_0: {:0x},{:0x},{:0x}",
                        //     y_scroll,
                        //     name_table,
                        //     y,
                        // );
                        self.temporary_v_ram_address
                            .lock()
                            .unwrap()
                            .set_y_offset_from_scanline(y_scroll);
                        self.temporary_v_ram_address
                            .lock()
                            .unwrap()
                            .set_name_table(name_table);
                        self.temporary_v_ram_address.lock().unwrap().set_y_idx(y);
                        // println!(
                        //     "0x:2006_0_temporary_dump: {:0x}",
                        //     self.temporary_v_ram_address.lock().unwrap().dump()
                        // );
                    }
                    true => {
                        let (_, _, y, _) = self.temporary_v_ram_address
                            .lock()
                            .unwrap()
                            .get_vram_address();
                        let x = data & 0b00011111;
                        // let y = y | ((data >> 5) & 0b00000111);
                        let y = (y & 0b11000) + ((data >> 5) & 0b00000111);
                        self.temporary_v_ram_address.lock().unwrap().set_y_idx(y);
                        self.temporary_v_ram_address.lock().unwrap().set_x_idx(x);
                        // println!(
                        //     "0x:2006_1_temporary_dump: {:0x}",
                        //     self.temporary_v_ram_address.lock().unwrap().dump()
                        // );
                        let (y_scroll, name_table, y, x) = self.temporary_v_ram_address
                            .lock()
                            .unwrap()
                            .get_vram_address();
                        // println!(
                        //     "0x:2006_1: {:0x},{:0x},{:0x},{:0x}",
                        //     y_scroll,
                        //     name_table,
                        //     y,
                        //     x
                        // );
                        self.v_ram_address_register
                            .lock()
                            .unwrap()
                            .set_y_offset_from_scanline(y_scroll);
                        self.v_ram_address_register.lock().unwrap().set_name_table(
                            name_table,
                        );
                        self.v_ram_address_register.lock().unwrap().set_y_idx(y);
                        self.v_ram_address_register.lock().unwrap().set_x_idx(x);
                        // println!(
                        //     "0x:2006_1_v_dump: {:0x}",
                        //     self.v_ram_address_register.lock().unwrap().dump()
                        // );
                        // self.first_or_second_write_toggle.lock().unwrap().set(false);
                    }
                }
                self.first_or_second_write_toggle.lock().unwrap().toggle();
            }
            0x2007 => {
                let address = self.v_ram_address_register.lock().unwrap().dump();
                println!("v_ram_address_register: {:0x}", address);
                self.v_ram.lock().unwrap().set8(address, data);
                self.v_ram_address_register.lock().unwrap().increment(
                    self.vram_offset_flag(),
                )
            }
            _ => {}
        }

        self.memory[address as usize] = data;
    }
    fn set16(&mut self, addressess: (u16, u16), data: u16) {
        self.set8(addressess.0, (data % 0x100) as u8);
        self.set8(addressess.1, (data >> 0b100) as u8);
        // self.memory[addressess.0 as usize] = (data % 0x100) as u8;
        // self.memory[addressess.1 as usize] = (data >> 0b100) as u8;
    }
    fn vram_offset_flag(&self) -> bool {
        ((self.memory[0x2000] >> 2) & 0b00000001) == 1

    }
}

#[derive(Debug)]
enum OpCode {
    ADC,
    SBC,
    AND,
    ORA,
    EOR,
    ASL,
    LSR,
    ROL,
    ROR,
    BCC,
    BCS,
    BEQ,
    BNE,
    BVC,
    BVS,
    BPL,
    BMI,
    BIT,
    JMP,
    JSR,
    RTS,
    BRK,
    RTI,
    CMP,
    CPX,
    CPY,
    INC,
    DEC,
    INX,
    DEX,
    INY,
    DEY,
    CLC,
    SEC,
    CLI,
    SEI,
    CLD,
    SED,
    CLV,
    LDA,
    LDX,
    LDY,
    STA,
    STX,
    STY,
    TAX,
    TXA,
    TAY,
    TYA,
    TSX,
    TXS,
    PHA,
    PLA,
    PHP,
    PLP,
    NOP,
}

#[derive(Debug)]
enum AddressingMode {
    Accumulator,
    Immediate,
    Absolute,
    ZeroPage,
    IndexedZeroPage,
    IndexedAbsolute,
    Implied,
    Relative,
    IndexedIndirect,
    IndirectIndexed,
    AbsoluteIndirect,
}

#[derive(Debug)]
enum IndexRegister {
    X,
    Y,
}
#[derive(Debug)]
enum Operand {
    Value(u8),
    Index(u16),
}
