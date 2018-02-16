pub mod cpu;
pub mod ppu;

extern crate piston_window;
use nes::cpu::Cpu;
use nes::cpu::PrgRam;
use nes::ppu::Ppu2;
use std::path::Path;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc, Mutex};
use self::piston_window::Key;

pub struct Nes {
    cpu: Cpu,
    ppu: Ppu2,
}

impl Nes {
    pub fn new(casette_name: &str) -> Nes {
        // let path_string = format!("{}", casette_name);
        let path_string = format!("cassette/{}", String::from(casette_name));
        let path = Path::new(&path_string);
        let prg_ram = Arc::new(Mutex::new(PrgRam::load(&path)));
        let cpu = Cpu::new(prg_ram.clone());
        let ppu = Ppu2::new(prg_ram.clone());
        Nes { cpu, ppu }
    }
    pub fn run(&mut self, tx: Sender<u8>, rxk: Receiver<Option<Key>>) {
        self.cpu.run(tx, rxk);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn run_test() {
        // let mut nes = Nes::new("sample1.nes");
        // nes.run();
    }
}
