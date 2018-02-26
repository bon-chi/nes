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

    pub fn new(casette_name: &str) -> Nes {
        let path_string = format!("cassette/{}", String::from(casette_name));
        let path = Path::new(&path_string);

        let (prg_ram, v_ram) = Self::load_ram(path);
        let cpu = Cpu::new(prg_ram);
        let ppu = Ppu::new(v_ram.clone());

        Nes { cpu, ppu }
    }

    fn load_ram(path: &Path) -> (PrgRam, Arc<Mutex<VRam>>) {
        let file = match File::open(path) {
            Ok(file) => file,
            Err(why) => panic!("{}: path is {:?}", why, path),
        };
        let mut buffer = BufReader::new(file);
        let mut i_nes_data: Vec<u8> = Vec::new();
        let _ = buffer.read_to_end(&mut i_nes_data).unwrap();

        let prg_rom_start = Self::I_NES_HEADER_SIZE;
        let prg_rom_end = prg_rom_start +
            i_nes_data[Self::COUNT_OF_PRG_ROM_UNITS_INDEX as usize] as u16 *
                Self::SIZE_OF_PRG_ROM_UNIT - 1;

        let chr_rom_banks_num = i_nes_data[Self::COUNT_OF_CHR_ROM_UNITS_INDEX as usize];
        let chr_rom_start = prg_rom_end + 1;
        let chr_rom_end = chr_rom_start + chr_rom_banks_num as u16 * Self::SIZE_OF_CHR_ROM_UNIT - 1;

        let mut prg_rom: Vec<u8> = Vec::new();
        prg_rom.extend_from_slice(
            &i_nes_data[(prg_rom_start as usize)..(prg_rom_end as usize + 1)],
        );

        let mut chr_rom: Vec<u8> = Vec::new();
        chr_rom.extend_from_slice(
            &i_nes_data[(chr_rom_start as usize)..(chr_rom_end as usize + 1)],
        );

        let mut prg_ram_memory: Box<[u8; 0xFFFF]> = Box::new([0; 0xFFFF]);
        let memory_idx_low = 0x8000;
        let memory_idx_high = 0xC000;
        let mut prg_rom_idx = 0;
        for memory_idx in memory_idx_low..memory_idx_high {
            prg_ram_memory[memory_idx] = prg_rom[prg_rom_idx];
            prg_rom_idx += 1;
        }

        match chr_rom_banks_num {
            1 => {
                prg_rom_idx = 0;
                for memory_idx in memory_idx_low..memory_idx_high {
                    prg_ram_memory[memory_idx] = prg_rom[prg_rom_idx];
                    prg_rom_idx += 1;
                }
            }
            2...255 => {
                let memory_idx_end = 0x10000;
                for memory_idx in memory_idx_high..memory_idx_end {
                    prg_ram_memory[memory_idx] = prg_rom[prg_rom_idx];
                    prg_rom_idx += 1;
                }
            }
            _ => {}
        }

        let mut v_ram_memory: Box<[u8; 0xFFFF]> = Box::new([0; 0xFFFF]);
        for (i, chr_rom_data) in chr_rom.iter().enumerate() {
            v_ram_memory[i] = *chr_rom_data;
        }

        let v_ram = Arc::new(Mutex::new(VRam::new(v_ram_memory)));
        let prg_ram = PrgRam::new(prg_ram_memory, v_ram.clone());

        (prg_ram, v_ram)
    }
}
