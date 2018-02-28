use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};
use std::fmt;
use errors::*;
use nes::ppu::VRam;

/// [CPU](http://wiki.nesdev.com/w/index.php/CPU)
pub struct Cpu {
    /// [Registers](http://wiki.nesdev.com/w/index.php/CPU_registers)
    // Program Counter
    pc: u16,

    prg_ram: PrgRam,
}

impl Cpu {
    pub fn new(prg_ram: PrgRam) -> Cpu {
        Cpu {
            pc: 0x8000,
            prg_ram,
        }
    }

    pub fn run(mut self) -> Result<()> {
        loop {
            let instruction = self.fetch_instruction();
            let (op_code, addressing_mode, register) = Self::parse_instruction(instruction)
                .map_err(|e| {
                    format!(
                        "{}, at: {:0x}, cpu_dump: {:?}",
                        e.description(),
                        self.pc,
                        self
                    )
                })?;
            self.increment_pc();
            let operand = self.get_operand(addressing_mode, register).map_err(|e| {
                format!(
                    "{}, at: {:0x}, cpu_dump: {:?}",
                    e.description(),
                    self.pc,
                    self
                )
            })?;
        }
        Ok(())
    }

    fn fetch_instruction(&self) -> u8 {
        self.prg_ram_value()
    }

    fn parse_instruction(
        instruction: u8,
    ) -> Result<(OpCode, AddressingMode, Option<IndexRegister>)> {
        let (op_code, addressing_mode, register): (OpCode,
                                                   AddressingMode,
                                                   Option<IndexRegister>) = match instruction {
            0x78 => (OpCode::SEI, AddressingMode::Implied, None),
            _ => Err(format!("unknown instruction: {:0x}", instruction,))?,
        };
        Ok((op_code, addressing_mode, register))
    }

    fn prg_ram_value(&self) -> u8 {
        self.prg_ram.memory[self.pc as usize]
    }

    fn increment_pc(&mut self) {
        self.pc = self.pc + 1;
    }

    fn get_operand(
        &mut self,
        addressing_mode: AddressingMode,
        register: Option<IndexRegister>,
    ) -> Result<Option<Operand>> {
        let operand: Option<Operand> = match addressing_mode {
            AddressingMode::Implied => None,
            _ => Err(format!("invalid AddressingMode: {:?}", addressing_mode))?,
        };
        Ok(operand)
    }

    pub fn get_pc(&self) -> u16 {
        self.pc
    }

    #[allow(dead_code)]
    pub fn dump(&self) {
        self.prg_ram.dump();
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
    v_ram: Arc<Mutex<VRam>>,
}

impl PrgRam {
    pub fn new(memory: Box<[u8; 0x10000]>, v_ram: Arc<Mutex<VRam>>) -> PrgRam {
        PrgRam { memory, v_ram }
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
enum IndexRegister {
    X,
    Y,
}

#[derive(Debug)]
enum Operand {
    Value(u8),
    Index(u16),
}
