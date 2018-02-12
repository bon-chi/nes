pub mod cpu;
pub mod ppu;

extern crate piston_window;
use nes::cpu::Cpu;
use nes::cpu::PrgRam;
use std::path::Path;
use std::sync::mpsc::{Sender, Receiver};
use self::piston_window::Key;

pub struct Nes {
    cpu: Cpu,
}

impl Nes {
    pub fn new(casette_name: &str) -> Nes {
        // let path_string = format!("{}", casette_name);
        let path_string = format!("cassette/{}", String::from(casette_name));
        let path = Path::new(&path_string);
        let cpu = Cpu::new(&path);
        Nes { cpu }
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
