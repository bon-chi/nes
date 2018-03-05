use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};
use std::fmt;
use errors::*;
use nes::ppu::{VRam, VRamAddressRegister, FineXScroll, FirstOrSecondWriteToggle};

/// [CPU](http://wiki.nesdev.com/w/index.php/CPU)
pub struct Cpu {
    /// [Registers](http://wiki.nesdev.com/w/index.php/CPU_registers)
    // Accumulator
    a: u8,
    // Indexe Register
    x: u8,
    y: u8,
    // Program Counter
    pc: u16,
    // Stgack Pointer
    sp: u8,
    // statusRegister
    p: u8,

    prg_ram: PrgRam,
}

impl Cpu {
    pub fn new(prg_ram: PrgRam) -> Cpu {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: 0x8000,
            sp: 0,
            p: 0,
            prg_ram,
        }
    }

    pub fn run(mut self) -> Result<()> {
        loop {
            let instruction = self.fetch_instruction();
            let (op_code, addressing_mode, register) = Self::parse_instruction(instruction).map_err(|e| {
                format!("{}, at: {:0x}, cpu_dump: {:?}", e.description(), self.pc, self)
            })?;
            #[cfg(feature="print_cpu_instruction")]
            println!("{:0x} {:?} {:?} {:?} ", self.pc, op_code, addressing_mode, register);
            self.increment_pc();
            let operand = self.get_operand(addressing_mode, register).map_err(|e| {
                format!("{}, at: {:0x}, cpu_dump: {:?}", e.description(), self.pc, self)
            })?;
            let _result = self.exec(op_code, operand).map_err(|e| {
                format!("{}, at: {:0x}, cpu_dump: {:?}", e.description(), self.pc, self)
            })?;
        }
        #[allow(unreachable_code)] Ok(())
    }

    fn fetch_instruction(&self) -> u8 {
        self.prg_ram_value()
    }

    fn parse_instruction(instruction: u8) -> Result<(OpCode, AddressingMode, Option<IndexRegister>)> {
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
            0xbd => (OpCode::LDA, AddressingMode::Absolute, Some(IndexRegister::X)),
            0xd0 => (OpCode::BNE, AddressingMode::Relative, None),
            0xe8 => (OpCode::INX, AddressingMode::Implied, None),
            _ => Err(format!("unknown instruction: {:0x}", instruction,))?,
        };
        Ok((op_code, addressing_mode, register))
    }

    fn prg_ram_value(&self) -> u8 {
        self.prg_ram.memory[self.pc as usize]
    }

    fn get_prg_ram_value(&self, idx: u16) -> u8 {
        self.prg_ram.memory[idx as usize]
    }

    fn set_prg_ram_value(&mut self, value: u8, idx: u16) {
        self.prg_ram.set8(idx, value);
    }

    fn increment_pc(&mut self) {
        self.pc = self.pc + 1;
    }

    fn get_operand(&mut self, addressing_mode: AddressingMode, register: Option<IndexRegister>) -> Result<Option<Operand>> {
        let operand: Option<Operand> = match addressing_mode {
            AddressingMode::Implied => None,
            AddressingMode::Immediate => {
                let operand = self.prg_ram_value();
                self.increment_pc();
                Some(Operand::Value(operand))
            }
            AddressingMode::Absolute => {
                match register {
                    Some(IndexRegister::X) => {
                        let address0 = self.prg_ram_value();
                        self.increment_pc();
                        let address1 = self.prg_ram_value();
                        self.increment_pc();
                        let address = PrgRam::concat_addresses(address1, address0) + self.x as u16;
                        Some(Operand::Index(address))
                    }
                    None => {
                        let address0 = self.prg_ram_value();
                        self.increment_pc();
                        let address1 = self.prg_ram_value();
                        self.increment_pc();
                        let address = PrgRam::concat_addresses(address1, address0) as u16;
                        Some(Operand::Index(address))
                    }
                    _ => {
                        panic!("not implemented register: {:?}, addressing_mode is {:?}", register, addressing_mode);
                    }
                }
            }
            AddressingMode::Relative => {
                let index = match self.prg_ram_value() as i8 {
                    i @ -128...0 => self.pc + 1 - ((-i).abs() as u16),
                    i @ 0...127 => self.pc + 1 + (i as u16),
                    _ => panic!("invalid prg ram value"),
                };
                Some(Operand::Index(index))
            }
            AddressingMode::IndexedIndirect => {
                match register {
                    Some(IndexRegister::X) => {
                        let index = PrgRam::concat_addresses(0x00 as u8, self.prg_ram_value()) + (self.x as u16);
                        let low: u8 = self.get_prg_ram_value(index);
                        let high: u8 = self.get_prg_ram_value(index + 1);
                        self.increment_pc();
                        Some(Operand::Value(self.get_prg_ram_value(PrgRam::concat_addresses(high, low))))
                    }
                    _ => {
                        panic!("not implemented register: {:?}, addressing_mode is {:?}", register, addressing_mode);
                    }
                }
            }
            _ => Err(format!("not implemented AddressingMode: {:?}", addressing_mode))?,
        };
        Ok(operand)
    }

    fn exec(&mut self, op_code: OpCode, operand: Option<Operand>) -> Result<()> {
        match op_code {
            OpCode::SEI => Ok(self.sei()),
            OpCode::LDA => {
                match operand {
                    Some(oper) => Ok(self.lda(oper)),
                    None => Err(format!("LDA invalid operand: {:?}", operand))?,
                }
            }
            OpCode::LDX => {
                match operand {
                    Some(oper) => Ok(self.ldx(oper)),
                    None => Err(format!("LDX invalid operand: {:?}", operand))?,
                }
            }
            OpCode::LDY => {
                match operand {
                    Some(oper) => Ok(self.ldy(oper)),
                    None => Err(format!("LDY invalid operand: {:?}", operand))?,
                }
            }
            OpCode::TXS => Ok(self.txs()),
            OpCode::STA => {
                match operand {
                    Some(oper) => Ok(self.sta(oper)),
                    None => Err(format!("STA invalid operand: {:?}", operand))?,
                }
            }
            OpCode::INX => Ok(self.inx()),
            OpCode::DEY => Ok(self.dey()),
            OpCode::BNE => {
                match operand {
                    Some(oper) => Ok(self.bne(oper)),
                    None => Err(format!("BNE invalid operand: {:?}", operand))?,
                }
            }
            OpCode::JMP => {
                match operand {
                    Some(oper) => Ok(self.jmp(oper)),
                    None => Err(format!("JMP invalid operand: {:?}", operand))?,
                }
            }
            OpCode::JSR => {
                match operand {
                    Some(oper) => {
                        self.jsr(oper);
                        Ok(())
                    }
                    None => Err(format!("JSR invalid operand: {:?}", operand))?,
                }
            }
            OpCode::AND => {
                match operand {
                    Some(oper) => Ok(self.and(oper)),
                    None => Err(format!("AND invalid operand: {:?}", operand))?,
                }
            }
            _ => panic!("invalid exec op_code: {:?}, operand is {:?}", op_code, operand),
        }
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

    fn push_stack(&mut self, value: u8) {
        let address = PrgRam::concat_addresses(0x01 as u8, self.sp);
        self.set_prg_ram_value(value, address);
        self.sp -= 1;
    }

    fn get_zero_flag(&self) -> bool {
        (self.p & 0b00000010) == 0b00000010
    }

    #[allow(dead_code)]
    pub fn dump(&self) {
        self.prg_ram.dump();
    }
}

impl Cpu {
    fn sei(&mut self) {
        self.set_flag(StatusFlag::InterruptDisable);
    }

    fn lda(&mut self, operand: Operand) {
        match operand {
            Operand::Index(idx) => self.a = self.get_prg_ram_value(idx),
            Operand::Value(value) => self.a = value,
        }
    }
    fn ldx(&mut self, operand: Operand) {
        match operand {
            Operand::Index(idx) => self.x = self.get_prg_ram_value(idx),
            Operand::Value(value) => self.x = value,
        }
    }

    fn ldy(&mut self, operand: Operand) {
        match operand {
            Operand::Index(idx) => self.y = self.get_prg_ram_value(idx),
            Operand::Value(value) => self.y = value,
        }
    }
    fn txs(&mut self) {
        self.x = self.sp
    }

    fn sta(&mut self, operand: Operand) {
        match operand {
            Operand::Index(idx) => {
                let a = self.a;
                self.set_prg_ram_value(a, idx)
            }
            _ => panic!("STA not implemented operand: {:?}", operand),
        }
    }

    fn inx(&mut self) {
        self.x += 1;
        if self.x == 0 {
            self.set_flag(StatusFlag::ZeroFlag);
        } else {
            self.reset_flag(StatusFlag::ZeroFlag);
        }

    }

    fn dey(&mut self) {
        self.y -= 1;
        if self.y == 0 {
            self.set_flag(StatusFlag::ZeroFlag);
        } else {
            self.reset_flag(StatusFlag::ZeroFlag);
        }
    }

    fn bne(&mut self, operand: Operand) {
        match !self.get_zero_flag() {
            true => {
                self.increment_pc();
                match operand {
                    Operand::Index(idx) => self.pc = idx,
                    _ => panic!("BNE not implemented operand: {:?}", operand),
                }
            }
            false => {
                self.increment_pc();
            }
        }
    }

    fn jmp(&mut self, operand: Operand) {
        match operand {
            Operand::Index(index) => {
                self.pc = index;
            }
            _ => panic!("JMP not implemented operand: {:?}", operand),
        }
    }

    fn jsr(&mut self, operand: Operand) {
        match operand {
            Operand::Index(index) => {
                self.pc -= 1;
                let (high, low) = PrgRam::split_address(self.pc);
                self.push_stack(high);
                self.push_stack(low);
                self.pc = index;
            }
            _ => panic!("JSR not implemented operand: {:?}", operand),
        }
    }
    fn and(&mut self, operand: Operand) {
        match operand {
            Operand::Value(value) => {
                self.a = value & self.a;
                if self.a == 0 {
                    self.set_flag(StatusFlag::ZeroFlag);
                } else {
                    self.reset_flag(StatusFlag::ZeroFlag);
                }
            }
            _ => panic!("AND not implemented operand: {:?}", operand),
        }
    }
}

impl fmt::Debug for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "PC:{:0x}",
            self.pc,
        )
    }
}

/// [CPU memory](http://wiki.nesdev.com/w/index.php/CPU_memory_map)
/// RAM is 2KB.
pub struct PrgRam {
    memory: Box<[u8; 0x10000]>,
    #[allow(dead_code)]
    v_ram: Arc<Mutex<VRam>>,

    // PPU internal registers
    v_ram_address_register: Arc<Mutex<VRamAddressRegister>>, // yyy, NN, YYYYY, XXXXX
    temporary_v_ram_address_register: Arc<Mutex<VRamAddressRegister>>,
    fine_x_scroll: Arc<Mutex<FineXScroll>>,
    first_or_second_write_toggle: Arc<Mutex<FirstOrSecondWriteToggle>>,
}

impl PrgRam {
    const PPU_CONTROL_REGISTER1: u16 = 0x2000;
    const PPU_STATUS_REGISTER: u16 = 0x2002;
    const V_RAM_ADDRESS_REGISTER1: u16 = 0x2005;
    const V_RAM_ADDRESS_REGISTER2: u16 = 0x2006;
    const V_RAM_IO_REGISTER: u16 = 0x2007;
    pub fn new(
        memory: Box<[u8; 0x10000]>,
        v_ram: Arc<Mutex<VRam>>,
        v_ram_address_register: Arc<Mutex<VRamAddressRegister>>,
        temporary_v_ram_address_register: Arc<Mutex<VRamAddressRegister>>,
        fine_x_scroll: Arc<Mutex<FineXScroll>>,
        first_or_second_write_toggle: Arc<Mutex<FirstOrSecondWriteToggle>>,
    ) -> PrgRam {
        PrgRam {
            memory,
            v_ram,
            v_ram_address_register,
            temporary_v_ram_address_register,
            fine_x_scroll,
            first_or_second_write_toggle,
        }
    }

    pub fn concat_addresses(address0: u8, address1: u8) -> u16 {
        ((address0 as u16) << 0b1000) + address1 as u16
    }

    fn set8(&mut self, address: u16, data: u8) {
        match address {
            Self::PPU_CONTROL_REGISTER1 => {
                self.temporary_v_ram_address_register
                    .lock()
                    .unwrap()
                    .set_name_table(data & 0b00000011)
                    .unwrap();
            }
            Self::PPU_STATUS_REGISTER => {
                self.first_or_second_write_toggle.lock().unwrap().set(false);
            }
            Self::V_RAM_ADDRESS_REGISTER1 => {
                match self.first_or_second_write_toggle.lock().unwrap().is_true() {
                    false => {
                        self.fine_x_scroll
                            .lock()
                            .unwrap()
                            .set_value(data & 0b00000111)
                            .unwrap();
                        self.temporary_v_ram_address_register
                            .lock()
                            .unwrap()
                            .set_coarse_x_scroll((data >> 3) & 0b00011111)
                            .unwrap();
                    }
                    true => {
                        self.temporary_v_ram_address_register
                            .lock()
                            .unwrap()
                            .set_fine_y_scroll(data & 0b00000111)
                            .unwrap();
                        self.temporary_v_ram_address_register
                            .lock()
                            .unwrap()
                            .set_coarse_y_scroll(data >> 3)
                            .unwrap();
                    }

                }
                self.first_or_second_write_toggle.lock().unwrap().toggle();
            }
            Self::V_RAM_ADDRESS_REGISTER2 => {
                match self.first_or_second_write_toggle.lock().unwrap().is_true() {
                    false => {
                        self.temporary_v_ram_address_register
                            .lock()
                            .unwrap()
                            .set_upper_6bits(data & 0b00111111)
                            .unwrap();
                    }
                    true => {
                        self.temporary_v_ram_address_register
                            .lock()
                            .unwrap()
                            .set_lower_8bits(data);
                        let v = self.temporary_v_ram_address_register
                            .lock()
                            .unwrap()
                            .value();
                        self.v_ram_address_register
                            .lock()
                            .unwrap()
                            .set_full_15bits(v)
                            .unwrap();
                    }
                }
                self.first_or_second_write_toggle.lock().unwrap().toggle();
            }
            Self::V_RAM_IO_REGISTER => {
                let address = self.v_ram_address_register.lock().unwrap().value();
                self.v_ram.lock().unwrap().set(address, data);
                match ((self.memory[0x2000] >> 2) & 0b00000001) == 1 {
                    true => self.v_ram_address_register.lock().unwrap().increment_y(),
                    false => self.v_ram_address_register.lock().unwrap().increment_x(),
                }
            }
            _ => {}
        }
        self.memory[address as usize] = data;
    }

    fn split_address(address: u16) -> (u8, u8) {
        ((address >> 0b1000) as u8, (address % 0x100) as u8)
    }

    pub fn dump(&self) {
        let dump_file = "prg_ram.dump";
        let mut f = BufWriter::new(File::create(dump_file).unwrap());
        for v in self.memory.iter() {
            f.write(&[*v]).unwrap();
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
/// [OperationCode](http://wiki.nesdev.com/w/index.php/6502_instructions)
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
#[allow(dead_code)]
/// [AddressingMode](http://wiki.nesdev.com/w/index.php/CPU_addressing_modes<Paste>)
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
#[allow(dead_code)]
enum IndexRegister {
    X,
    Y,
}

#[derive(Debug)]
enum Operand {
    Value(u8),
    Index(u16),
}


#[derive(Debug)]
#[allow(dead_code)]
/// [StatusFlag](http://wiki.nesdev.com/w/index.php/CPU_status_flag_behavior)
enum StatusFlag {
    CarryFlag,
    ZeroFlag,
    InterruptDisable,
    DecimalMode,
    BreakCommand,
    OverflowFlag,
    NegativeFlag,
}
