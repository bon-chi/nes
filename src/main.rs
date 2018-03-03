extern crate nes;

use nes::nes::Nes;

fn main() {
    let nes = Nes::new("sample1.nes");
    nes.run()
}
