use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};
use nes::ppu::VRam;

pub struct Cpu {
    prg_ram: PrgRam,
}

impl Cpu {
    pub fn new(prg_ram: PrgRam) -> Cpu {
        Cpu { prg_ram }
    }

    #[allow(dead_code)]
    pub fn dump(&self) {
        self.prg_ram.dump();
    }
}

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
