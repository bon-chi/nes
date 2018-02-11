use std::fmt;
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

    pub fn run(&mut self) {
        loop {
            let (op_code, addressing_mode, register) = self.fetch();
            let operand = self.get_operand(addressing_mode, register);
            self.exec(op_code, operand);
        }
    }
    fn fetch(&mut self) -> (OpCode, AddressingMode, Option<IndexRegister>) {
        let instruction = self.ram.0[self.pc as usize];
        let (op_code, addressing_mode, register): (OpCode, AddressingMode, Option<IndexRegister>) = match instruction {

            0x21 => (OpCode::AND, AddressingMode::IndexedIndirect, None),
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
        #[cfg(feature="debug_log")]
        println!(
            "{:0x} {:?} {:?} {:?} ",
            self.pc,
            op_code,
            addressing_mode,
            register,
        );
        #[cfg(not(test))]
        println!("debughoge");

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
                        let address = PrgRam::concat_addresses(address1, address0) + self.x as u16;
                        Some(Operand::Index(address))
                    }
                    None => {
                        let address0 = self.ram_value();
                        self.increment_pc();
                        let address1 = self.ram_value();
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
                    i @ -128...0 => self.pc + 1 - (-i.abs() as u16),
                    i @ 0...127 => self.pc + 1 + (i as u16),
                    _ => panic!("invalid"),
                };
                Some(Operand::Index(index))
                // if self.ram_value() as i16 < 0 {
                //
                // } else {
                // Some(Operand::Index(
                //     ((self.ram_value() ) + self.pc + 1) as u16,
                // ))
                // }
                // Some(Operand::Index(
                //     ((self.ram_value() as i16) + self.pc + 1) as u16,
                // ))
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
        self.ram.0[self.pc as usize]
    }
    fn get_ram_value(&self, idx: u16) -> u8 {
        self.ram.0[idx as usize]
    }
    fn set_ram_value(&mut self, value: u8, idx: u16) {
        self.ram.set8(idx, value);
    }

    // fn fetch(&mut self) -> (OpCode, AddressingMode, Option<IndexRegister>, Option<u8>, Option<u8>) {
    //     let instruction = self.ram.0[self.pc as usize];
    //     // println!("{:0x}, {:0x}", self.pc, instruction);
    //     let (op_code, addressing_mode, register, operand0, operand1): (OpCode,
    //                                                                    AddressingMode,
    //                                                                    Option<IndexRegister>,
    //                                                                    Option<u8>,
    //                                                                    Option<u8>) = match instruction {
    //         0x21 => {
    //             self.increment_pc();
    //             let address = PrgRam::concat_addresses(0x00 as u8, self.pc) + (self.x as u16);
    //             let xx: u8 = self.ram.0[address as usize];
    //             let yy: u8 = self.ram.0[(address + 1) as usize];
    //             // let operand0 = self.ram.0[PrgRam::concat_addresses(yy, xx)];
    //             (
    //                 OpCode::AND,
    //                 AddressingMode::IndexedIndirect,
    //                 Some(IndexRegister::X),
    //                 Some(yy),
    //                 Some(xx),
    //             )
    //
    //         }
    //         0x78 => (OpCode::SEI, AddressingMode::Implied, None, None, None),
    //         0x88 => (OpCode::DEY, AddressingMode::Implied, None, None, None),
    //         0x8d => {
    //             self.increment_pc();
    //             let operand0 = self.ram.0[self.pc as usize];
    //             self.increment_pc();
    //             let operand1 = self.ram.0[self.pc as usize];
    //             (
    //                 OpCode::STA,
    //                 AddressingMode::Absolute,
    //                 None,
    //                 Some(operand0),
    //                 Some(operand1),
    //             )
    //         }
    //         0x9a => (OpCode::TXS, AddressingMode::Implied, None, None, None),
    //         0xa0 => {
    //             self.increment_pc();
    //             (
    //                 OpCode::LDY,
    //                 AddressingMode::Immediate,
    //                 None,
    //                 Some(self.ram.0[self.pc as usize]),
    //                 None,
    //             )
    //         }
    //         0xa2 => {
    //             self.increment_pc();
    //             (
    //                 OpCode::LDX,
    //                 AddressingMode::Immediate,
    //                 None,
    //                 Some(self.ram.0[self.pc as usize]),
    //                 None,
    //             )
    //         }
    //         0xa9 => {
    //             self.increment_pc();
    //             (
    //                 OpCode::LDA,
    //                 AddressingMode::Immediate,
    //                 None,
    //                 Some(self.ram.0[self.pc as usize]),
    //                 None,
    //             )
    //         }
    //         0xbd => {
    //             self.increment_pc();
    //             let operand0 = self.ram.0[self.pc as usize];
    //             self.increment_pc();
    //             let operand1 = self.ram.0[self.pc as usize];
    //             (
    //                 OpCode::LDA,
    //                 AddressingMode::Absolute,
    //                 Some(IndexRegister::X),
    //                 Some(operand0),
    //                 Some(operand1),
    //             )
    //
    //         }
    //         0xd0 => (OpCode::BNE, AddressingMode::Relative, None, None, None),
    //         0xe8 => (OpCode::INX, AddressingMode::Implied, None, None, None),
    //         _ => {
    //             panic!(
    //                 "unknown instruction: {:0x} at: {:0x}, cpu_dump: {:?}",
    //                 instruction,
    //                 self.pc,
    //                 self
    //             );
    //         }
    //     };
    //     #[cfg(feature="debug_log")]
    //     println!(
    //         "{:0x} {:?} {:?} {:?} {:?}{:?} ",
    //         self.pc,
    //         op_code,
    //         addressing_mode,
    //         register,
    //         operand0,
    //         operand1
    //     );
    //     #[cfg(not(test))]
    //     println!("debughoge");
    //
    //     self.increment_pc();
    //     (op_code, addressing_mode, register, operand0, operand1)
    // }
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
            OpCode::INX => self.x += 1,
            OpCode::DEY => self.y -= 1,
            OpCode::BNE => {
                if self.get_zero_flag() {
                    self.increment_pc();
                    let dest = self.ram.0[(self.pc + 1) as usize] as u16;
                    match operand {
                        Some(Operand::Index(idx)) => self.pc = idx,
                        _ => panic!("invalid operand: {:?}", operand),
                    }
                } else {
                    self.pc = self.pc + 1;
                }
            }
            OpCode::AND => {
                match operand {
                    Some(Operand::Index(idx)) => self.a = self.ram_value() & self.a,
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
    // fn exec(
    //     &mut self,
    //     op_code: OpCode,
    //     addressing_mode: AddressingMode,
    //     register: Option<IndexRegister>,
    //     operand0: Option<u8>,
    //     operand1: Option<u8>,
    // ) {
    //     match op_code {
    //         OpCode::SEI => self.sei(),
    //         OpCode::LDA => {
    //             match addressing_mode {
    //                 AddressingMode::Immediate => self.a = operand0.unwrap(),
    //                 AddressingMode::Absolute => {
    //                     match register {
    //                         Some(IndexRegister::X) => {
    //                             let address = PrgRam::concat_addresses(operand1.unwrap(), operand0.unwrap()) + self.x as u16;
    //                             self.a = self.ram.fetch8(address);
    //
    //                         }
    //                         _ => {
    //                             panic!(
    //                                 "invalid register: {:?}, op_code is {:?} and addressing_mode is {:?}",
    //                                 register,
    //                                 op_code,
    //                                 addressing_mode
    //                             );
    //                         }
    //                     }
    //                 }
    //                 _ => {
    //                     panic!(
    //                         "invalid addressing_mode: {:?}, op_code is {:?}",
    //                         op_code,
    //                         addressing_mode
    //                     )
    //                 }
    //             }
    //         }
    //         OpCode::LDX => {
    //             match addressing_mode {
    //                 AddressingMode::Immediate => self.x = operand0.unwrap(),
    //                 _ => {
    //                     panic!(
    //                         "invalid addressing_mode: {:?}, op_code is {:?}",
    //                         op_code,
    //                         addressing_mode
    //                     )
    //                 }
    //             }
    //         }
    //         OpCode::LDY => {
    //             match addressing_mode {
    //                 AddressingMode::Immediate => self.y = operand0.unwrap(),
    //                 _ => {
    //                     panic!(
    //                         "invalid addressing_mode: {:?}, op_code is {:?}",
    //                         op_code,
    //                         addressing_mode
    //                     )
    //                 }
    //             }
    //         }
    //         OpCode::TXS => self.x = self.sp,
    //         OpCode::STA => {
    //             match addressing_mode {
    //                 AddressingMode::Absolute => {
    //                     match register {
    //                         Some(IndexRegister::X) => panic!("hoge"),
    //                         None => {
    //                             self.ram.set8(
    //                                 PrgRam::concat_addresses(operand1.unwrap(), operand0.unwrap()),
    //                                 self.a,
    //                             );
    //
    //                         }
    //                         _ => {
    //                             panic!(
    //                                 "invalid register: {:?}, op_code is {:?} and addressing_mode is {:?}",
    //                                 register,
    //                                 op_code,
    //                                 addressing_mode
    //                             );
    //
    //                         }
    //                     }
    //                 }
    //                 _ => {
    //                     panic!(
    //                         "invalid addressing_mode: {:?}, op_code is {:?}",
    //                         op_code,
    //                         addressing_mode
    //                     )
    //                 }
    //
    //             }
    //         }
    //         OpCode::INX => {
    //             match addressing_mode {
    //                 AddressingMode::Implied => {
    //                     self.x += 1;
    //                 }
    //                 _ => {
    //                     panic!(
    //                         "invalid addressing_mode: {:?}, op_code is {:?}",
    //                         op_code,
    //                         addressing_mode
    //                     )
    //                 }
    //             }
    //         }
    //         OpCode::DEY => {
    //             match addressing_mode {
    //                 AddressingMode::Implied => {
    //                     self.y -= 1;
    //                 }
    //                 _ => {
    //                     panic!(
    //                         "invalid addressing_mode: {:?}, op_code is {:?}",
    //                         op_code,
    //                         addressing_mode
    //                     )
    //                 }
    //             }
    //         }
    //         OpCode::BNE => {
    //             match addressing_mode {
    //                 AddressingMode::Relative => {
    //                     if self.get_zero_flag() {
    //                         self.increment_pc();
    //                         let dest = self.ram.0[(self.pc + 1) as usize] as u16;
    //                         self.pc = self.pc + dest;
    //                     }
    //                     self.increment_pc();
    //                     self.increment_pc();
    //                 }
    //                 _ => {
    //                     panic!(
    //                         "invalid addressing_mode: {:?}, op_code is {:?}",
    //                         op_code,
    //                         addressing_mode
    //                     )
    //                 }
    //             }
    //         }
    //         OpCode::AND => {
    //             match addressing_mode {
    //                 AddressingMode::IndexedIndirect => {
    //                     match register {
    //                         Some(IndexRegister::X) => {
    //                             let address = PrgRam::concat_addresses(operand0.unwrap(), operand1.unwrap());
    //                             self.ram.0[address as usize] = self.ram.0[address as usize] & self.a;
    //                         }
    //                         _ => {
    //                             panic!(
    //                                 "invalid register: {:?}, op_code is {:?} and addressing_mode is {:?}",
    //                                 register,
    //                                 op_code,
    //                                 addressing_mode
    //                             );
    //
    //                         }
    //                     }
    //                 }
    //                 _ => {
    //                     panic!(
    //                         "invalid addressing_mode: {:?}, op_code is {:?}",
    //                         op_code,
    //                         addressing_mode
    //                     )
    //                 }
    //
    //             }
    //         }
    //         _ => panic!("invalid exec op_code: {:?}", op_code),
    //     }
    // }
    // exec instruction
    fn do_exec(&self) {}
    fn store_resutl(&self) {}
    // jump instruction
    fn load_address_to_pc(&self) {}

    fn set_pc(&mut self, value: Register16) {
        self.pc = value;
    }

    fn get_zero_flag(&self) -> bool {
        (self.p & 0b00000010) == 0b00000010
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

        let mut memory: [u8; 0xFFFF] = [0; 0xFFFF];

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
        Box::new(PrgRam(memory))
        // prg
    }
    fn fetch8(&self, address: u16) -> Register8 {
        self.0[address as usize]
    }

    fn fetch16(&self, addresses: (u16, u16)) -> Register16 {
        (((self.0[addresses.1 as usize] as u16) << 0b1000) + (self.0[addresses.0 as usize]) as u16)
    }

    pub fn concat_addresses(address0: u8, address1: u8) -> u16 {
        ((address0 as u16) << 0b1000) + address1 as u16
    }
    fn set8(&mut self, address: u16, data: u8) {
        self.0[address as usize] = data;
    }
    fn set16(&mut self, addressess: (u16, u16), data: u16) {
        self.0[addressess.0 as usize] = (data % 0x100) as u8;
        self.0[addressess.1 as usize] = (data >> 0b100) as u8;
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
