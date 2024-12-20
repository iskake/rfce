use crate::fc::NESFile;

use super::Memory;

const NROM256_PRG_ROM_SIZE: usize = 32_768;

#[derive(PartialEq, Debug)]
pub enum MapperType {
    NROM,
    UNKNOWN(u16),
}

pub trait Mapper : Memory {
    fn read_chr(&self, addr: u16) -> u8;
    fn write_chr(&mut self, addr: u16, val: u8) -> ();
}

pub struct NROMMapper {
    prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    chr_rom: Vec<u8>,
    nametable_v_mirror: bool,
    // TODO?
    // ..."PRG RAM: 2 or 4 KiB"
    // ..."CHR capacity: 8KiB ROM"
    // ...bus conflicts?
}

impl NROMMapper {
    pub fn from_nesfile(nesfile: &NESFile) -> NROMMapper {
        assert!(nesfile.mapper_type() == MapperType::NROM);
        let prg_rom_size = nesfile.header.prg_rom_size();
        let chr_rom_size = nesfile.header.chr_rom_size();
        let prg_ram_size = nesfile.header.prg_ram_size();
        // TODO: chr ram?
        let nametable_v_mirror = nesfile.header.nametable_layout();

        if nesfile.header.trainer() {
            unimplemented!("NROM trainer handling");
        }

        println!("NROM with:");
        println!("  PRG-ROM SIZE: {}", prg_rom_size);
        println!("  PRG-RAM SIZE: {}", prg_ram_size);
        println!("  CHR-ROM SIZE: {}", chr_rom_size);
        println!("  Nametable mirroring: {} ({} arrangement)",
            if nametable_v_mirror { "vertical" } else { "horizontal" },
            if nametable_v_mirror { "horizontal" } else { "vertical" }
        );
        println!();

        let prg_rom = nesfile.data[0..prg_rom_size].to_vec();
        let prg_ram: Vec<u8> = vec![0; prg_ram_size];  // TODO
        let chr_rom = nesfile.data[prg_rom_size..(prg_rom_size + chr_rom_size)].to_vec();

        NROMMapper { prg_rom, prg_ram, chr_rom, nametable_v_mirror}
    }
}

impl Memory for NROMMapper {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            // TODO: CHR-ROM?
            0x6000..=0x7fff => {
                // TODO: handle mirroring/no ram
                self.prg_ram[(addr - 0x6000) as usize]
            },
            0x8000..=0xbfff => {
                self.prg_rom[(addr - 0x8000) as usize]
            },
            0xc000..=0xffff => {
                if self.prg_rom.len() == NROM256_PRG_ROM_SIZE {
                    self.prg_rom[(addr - 0x8000) as usize]
                } else {
                    self.prg_rom[(addr - 0xc000) as usize]
                }
            }
            _ => unreachable!(),
        }
    }

    fn write(&mut self, addr: u16, val: u8) -> () {
        match addr {
            0x6000..=0x7fff => {
                // TODO: handle mirroring/no ram
                self.prg_ram[(addr - 0x6000) as usize] = val;
            },
            _ => (),
        }
    }
}

impl Mapper for NROMMapper {
    fn read_chr(&self, addr: u16) -> u8 {
        self.chr_rom[addr as usize]
    }

    fn write_chr(&mut self, _addr: u16, _val: u8) -> () {
        ()  // TODO?
    }
}
