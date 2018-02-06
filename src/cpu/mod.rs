use std::ops::Range;

/// [CPU](http://wiki.nesdev.com/w/index.php/CPU_registers)
struct Cpu {
    /// Accumulator
    a: Register8,
    /// Indexe Register
    x: Register8,
    y: Register8,
    /// Program Counter
    pc: Register16,
    sp: Register8,
    p: Register8,
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
    fn new() -> Cpu {
        Cpu {
            a: b'0',
            x: b'0',
            y: b'0',
            pc: 0x0,
            sp: b'0',
            p: b'0',
        }
    }

    fn run(&self) {
        self.fetch();
        self.exec();
    }
    fn fetch(&self) {}
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


    fn reset(&mut self, memory_map: &mut CpuMemoryMap) {
        // memory_map.set8(CpuMemoryMap::STACk_ADDRESSES.index(0), self.sp);
        // 3
        memory_map.set8(CpuMemoryMap::STACk_ADDRESS + (self.sp as u16) - 2, self.p);
        memory_map.set16(
            (
                CpuMemoryMap::STACk_ADDRESS + (self.sp as u16) - 1,
                CpuMemoryMap::STACk_ADDRESS + (self.sp as u16),
            ),
            self.pc,
        );
        // 4
        self.set_flag(StatusFlag::InterruptDisable);

        // 5
        self.set_pc(memory_map.fetch16(CpuMemoryMap::RESET_ADDRESSES));
    }
    fn nmi(&self) {}
    fn irq(&self) {}
}

struct CpuMemoryMap {
    map: [u8; 2 ^ 16],
}

impl CpuMemoryMap {
    const STACk_ADDRESSES: Range<u16> = 0x0100..0x0200;
    const STACk_ADDRESS: u16 = 0x0100;
    const NMI_ADDRESSES: (u16, u16) = (0xFFFA, 0xFFFB);
    const RESET_ADDRESSES: (u16, u16) = (0xFFFC, 0xFFFD);
    const IRQ_ADDRESSES: (u16, u16) = (0xFFFE, 0xFFFF);

    fn fetch8(&self, address: u16) -> Register8 {
        self.map[address as usize]
    }

    fn fetch16(&self, addresses: (u16, u16)) -> Register16 {
        (((self.map[addresses.1 as usize] as u16) << 0b1000) +
             (self.map[addresses.0 as usize]) as u16)
    }

    fn set8(&mut self, address: u16, data: u8) {
        self.map[address as usize] = data;
    }
    fn set16(&mut self, addressess: (u16, u16), data: u16) {
        self.map[addressess.0 as usize] = (data % 0x100) as u8;
        self.map[addressess.1 as usize] = (data >> 0b100) as u8;
    }
}
// trait ControlBus {
//     fn fetch8(&self) -> u8;
// }
// trait AddressBus {
//     fn fetch16(&self) -> u16;
// }

type Register8 = u8;
type Register16 = u16;
