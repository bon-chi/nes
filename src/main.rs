extern crate nes;
use nes::nes::Nes;
fn main() {
    // let cpu = Cpu::new();
    // let nes = Nes::new(String::from("sample.nes"));
    let mut nes = Nes::new("sample1.nes");
    nes.run();
}
