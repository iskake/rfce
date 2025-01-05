use log::info;

use crate::fc::{ppu, NESFile};

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
    fn nametable_read(&self, addr: u16, vram: [u8; ppu::VRAM_SIZE]) -> u8;
    fn nametable_write(&mut self, addr: u16, val: u8, vram: &mut [u8; ppu::VRAM_SIZE]) -> ();
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
        let prg_rom_size = nesfile.prg_rom_size();
        let chr_rom_size = nesfile.chr_rom_size();
        let prg_ram_size = nesfile.prg_ram_size();
        // TODO: chr ram?
        let nametable_v_mirror = nesfile.nametable_layout();

        if nesfile.trainer() {
            unimplemented!("NROM trainer handling");
        }

        info!("NROM with:");
        info!("  PRG-ROM SIZE: {} (0x{:x})", prg_rom_size, prg_rom_size);
        info!("  PRG-RAM SIZE: {} (0x{:x})", prg_ram_size, prg_ram_size);
        info!("  CHR-ROM SIZE: {} (0x{:x})", chr_rom_size, chr_rom_size);
        info!("  Nametable mirroring: {} ({} arrangement)\n",
            if nametable_v_mirror { "vertical" } else { "horizontal" },
            if nametable_v_mirror { "horizontal" } else { "vertical" }
        );

        let prg_rom = nesfile.data[0..prg_rom_size].to_vec();
        let prg_ram = vec![0; prg_ram_size];
        let chr_rom = nesfile.data[prg_rom_size..(prg_rom_size + chr_rom_size)].to_vec();

        NROMMapper { prg_rom, prg_ram, chr_rom, nametable_v_mirror}
    }

    fn nametable_addr_fix(&self, addr: u16) -> u16 {
        let a = addr - 0x2000;
        if self.nametable_v_mirror {
            a & 0x7ff
        } else {
            (a & 0x800 >> 1) | a & 0x3ff
        }
    }
}

impl Memory for NROMMapper {
    fn read(&self, addr: u16) -> u8 {
        match addr {
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
        // TODO?
        self.chr_rom[(addr as usize) % self.chr_rom.len()]
    }

    fn write_chr(&mut self, _addr: u16, _val: u8) -> () {
        ()  // TODO?
    }

    fn nametable_read(&self, addr: u16, vram: [u8; ppu::VRAM_SIZE]) -> u8 {
        let addr = self.nametable_addr_fix(addr);
        vram[addr as usize]
    }

    fn nametable_write(&mut self, addr: u16, val: u8, vram: &mut [u8; ppu::VRAM_SIZE]) -> () {
        let addr = self.nametable_addr_fix(addr);
        vram[addr as usize] = val;
    }
}
