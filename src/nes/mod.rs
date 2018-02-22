pub mod cpu;
pub mod ppu;

extern crate piston_window;
use nes::cpu::Cpu;
use nes::cpu::PrgRam;
use nes::ppu::Ppu2;
use nes::ppu::{VRam, VRamAddressRegister, FirstOrSecondWriteToggle};
use std::path::Path;
use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use self::piston_window::Key;
use nes::piston_window::Context;
use nes::piston_window::Graphics;

pub struct Nes {
    cpu: Cpu,
    ppu: Ppu2,
}

impl Nes {
    pub fn new(casette_name: &str) -> Nes {
        // let path_string = format!("{}", casette_name);
        let path_string = format!("cassette/{}", String::from(casette_name));
        let path = Path::new(&path_string);
        let v_ram_address_register = Arc::new(Mutex::new(VRamAddressRegister::new()));
        let temporary_v_ram_address = Arc::new(Mutex::new(VRamAddressRegister::new()));
        let fine_x_scroll = Arc::new(Mutex::new(0));
        let first_or_second_write_toggle = Arc::new(Mutex::new(FirstOrSecondWriteToggle::new()));
        // let v_ram = Arc::new(Mutex::new(VRam::new()));
        let v_ram = Arc::new(Mutex::new(VRam::load(&path)));
        let prg_ram = Arc::new(Mutex::new(
            (PrgRam::load(
                &path,
                v_ram_address_register.clone(),
                temporary_v_ram_address.clone(),
                fine_x_scroll.clone(),
                first_or_second_write_toggle.clone(),
                v_ram.clone(),
            )),
        ));
        let cpu = Cpu::new(prg_ram.clone());
        let ppu = Ppu2::new(
            prg_ram.clone(),
            v_ram.clone(),
            v_ram_address_register.clone(),
            temporary_v_ram_address.clone(),
            fine_x_scroll.clone(),
            first_or_second_write_toggle.clone(),
        );
        Nes { cpu, ppu }
    }
    pub fn run(&mut self, tx: Sender<u8>, rxk: Receiver<Option<Key>>) {
        self.cpu.run(tx, rxk);
    }

    pub fn run3(mut self) {
        let (tx, rx) = mpsc::channel::<u8>();
        let (txk, rxk) = mpsc::channel::<Option<Key>>();
        let mut cpu = self.cpu;
        let t = thread::spawn(move || { cpu.run(tx, rxk); });
        thread::sleep_ms(5000);
        self.ppu.dump();
        self.ppu.run3(txk, rx);
    }

    pub fn run2<G: Graphics>(mut self, tx: Sender<u8>, rxk: Receiver<Option<Key>>, c: &Context, g: &mut G) {
        let mut cpu = self.cpu;
        // let mut ppu = self.ppu;
        let t = thread::spawn(move || { cpu.run(tx, rxk); });
        self.ppu.run(c, g);
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
