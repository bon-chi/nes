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
}

pub struct PrgRam {
    memory: Box<[u8; 0xFFFF]>,
    v_ram: Arc<Mutex<VRam>>,
}

impl PrgRam {
    pub fn new(memory: Box<[u8; 0xFFFF]>, v_ram: Arc<Mutex<VRam>>) -> PrgRam {
        PrgRam { memory, v_ram }
    }
}
