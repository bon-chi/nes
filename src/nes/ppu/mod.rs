use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};

pub struct Ppu {
    v_ram: Arc<Mutex<VRam>>,
}

impl Ppu {
    pub fn new(v_ram: Arc<Mutex<VRam>>) -> Ppu {
        Ppu { v_ram }
    }
}

pub struct VRam(Box<[u8; 0xFFFF]>);
impl VRam {
    pub fn new(memory: Box<[u8; 0xFFFF]>) -> VRam {
        VRam(memory)
    }
}
