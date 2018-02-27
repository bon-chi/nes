use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};

/// [PPU](http://wiki.nesdev.com/w/index.php/PPU) is short for Picture Processing Unit
pub struct Ppu {
    v_ram: Arc<Mutex<VRam>>,
}

impl Ppu {
    pub fn new(v_ram: Arc<Mutex<VRam>>) -> Ppu {
        Ppu { v_ram }
    }

    #[allow(dead_code)]
    pub fn dump(&self) {
        self.v_ram.lock().unwrap().dump();
    }
}

/// [PPU memory](http://wiki.nesdev.com/w/index.php/PPU_memory_map)
/// RAM is 2KB.
pub struct VRam(Box<[u8; 0x10000]>);

impl VRam {
    pub fn new(memory: Box<[u8; 0x10000]>) -> VRam {
        VRam(memory)
    }

    fn dump(&self) {
        let dump_file = "v_ram.dump";
        let mut f = BufWriter::new(File::create(dump_file).unwrap());
        for v in self.0.iter() {
            f.write(&[*v]).unwrap();
        }
    }
}
