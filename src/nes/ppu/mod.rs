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

    #[allow(dead_code)]
    pub fn dump(&self) {
        self.v_ram.lock().unwrap().dump();
    }
}

pub struct VRam(Box<[u8; 0xFFFF]>);
impl VRam {
    pub fn new(memory: Box<[u8; 0xFFFF]>) -> VRam {
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
