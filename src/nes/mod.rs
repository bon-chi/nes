pub mod cpu;
pub mod ppu;
use nes::cpu::Cpu;
use nes::cpu::PrgRam;
use std::path::Path;
pub struct Nes {
    // cpu: Cpu,
    prg_ram: PrgRam,
}

impl Nes {
    pub fn new(casette_name: &str) -> Nes {
        // let path_string = format!("{}", casette_name);
        let path_string = format!("cassette/{}", String::from(casette_name));
        let path = Path::new(&path_string);
        let prg_ram = PrgRam::load(&path);
        // println!("{:?}", prg_ram);
        Nes { prg_ram }
    }
}
