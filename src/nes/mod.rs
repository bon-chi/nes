mod cpu;
mod ppu;

use std::fs::File;
use std::path::Path;
use std::io::{BufReader, Read};
use std::sync::{Arc, Mutex};
use nes::cpu::{Cpu, PrgRam};
use nes::ppu::{Ppu, VRam};

pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
}
impl Nes {
    const I_NES_HEADER_SIZE: u16 = 0x0010;
    const COUNT_OF_PRG_ROM_UNITS_INDEX: u16 = 4;
    const COUNT_OF_CHR_ROM_UNITS_INDEX: u16 = 5;
    const SIZE_OF_PRG_ROM_UNIT: u16 = 0x4000;
    const SIZE_OF_CHR_ROM_UNIT: u16 = 0x2000;

    const RAM_SIZE: usize = 0x10000;
    const PRG_ROM_LOWER_IDX: usize = 0x8000;
    const PRG_ROM_UPPER_IDX: usize = 0xC000;

    pub fn new(casette_name: &str) -> Nes {
        let path_string = format!("cassette/{}", String::from(casette_name));
        let path = Path::new(&path_string);

        let (prg_ram, v_ram) = Self::load_ram(path);
        let cpu = Cpu::new(prg_ram);
        let ppu = Ppu::new(v_ram.clone());

        #[cfg(feature = "dump")] cpu.dump();
        ppu.dump();

        Nes { cpu, ppu }
    }

    /// load [iNES](http://wiki.nesdev.com/w/index.php/INES)
    fn load_ram(path: &Path) -> (PrgRam, Arc<Mutex<VRam>>) {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(why) => panic!("{}: path is {:?}", why, path),
        };
        let mut buffer = BufReader::new(file);
        let mut i_nes_data: Vec<u8> = Vec::new();
        let _ = buffer.read_to_end(&mut i_nes_data).unwrap();

        let prg_rom_start = Self::I_NES_HEADER_SIZE;
        let prg_rom_end = prg_rom_start + i_nes_data[Self::COUNT_OF_PRG_ROM_UNITS_INDEX as usize] as u16 * Self::SIZE_OF_PRG_ROM_UNIT - 1;

        let chr_rom_banks_num = i_nes_data[Self::COUNT_OF_CHR_ROM_UNITS_INDEX as usize];
        let chr_rom_start = prg_rom_end + 1;
        let chr_rom_end = chr_rom_start + chr_rom_banks_num as u16 * Self::SIZE_OF_CHR_ROM_UNIT - 1;

        // set prg_ram_memory
        let mut prg_rom: Vec<u8> = Vec::new();
        prg_rom.extend_from_slice(&i_nes_data[(prg_rom_start as usize)..(prg_rom_end as usize + 1)]);

        let mut prg_ram_memory: Box<[u8; Self::RAM_SIZE]> = Box::new([0; Self::RAM_SIZE]);
        prg_ram_memory[Self::PRG_ROM_LOWER_IDX..Self::PRG_ROM_UPPER_IDX].clone_from_slice(&prg_rom[..(Self::SIZE_OF_PRG_ROM_UNIT as usize)]);

        match i_nes_data[Self::COUNT_OF_PRG_ROM_UNITS_INDEX as usize] {
            1 => {
                prg_ram_memory[Self::PRG_ROM_UPPER_IDX..].clone_from_slice(&prg_rom[..(Self::SIZE_OF_PRG_ROM_UNIT as usize)]);

            }
            2...255 => {
                prg_ram_memory[Self::PRG_ROM_UPPER_IDX..].clone_from_slice(
                    &prg_rom
                        [(Self::SIZE_OF_PRG_ROM_UNIT as usize)..(Self::SIZE_OF_PRG_ROM_UNIT as usize * 2)],
                );
            }
            _ => {}
        }

        // set v_ram_memory
        let mut v_ram_memory: Box<[u8; Self::RAM_SIZE]> = Box::new([0; Self::RAM_SIZE]);
        v_ram_memory[..(chr_rom_end as usize + 1 - chr_rom_start as usize)]
            .clone_from_slice(&i_nes_data[(chr_rom_start as usize)..(chr_rom_end as usize + 1)]);

        let v_ram = Arc::new(Mutex::new(VRam::new(v_ram_memory)));
        let prg_ram = PrgRam::new(prg_ram_memory, v_ram.clone());

        (prg_ram, v_ram)
    }

    pub fn run(mut self) {
        if let Err(message) = self.cpu.run() {
            panic!("{}", message);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn run_test() {
        let mut nes = Nes::new("sample1.nes");
        nes.run();
    }
}
