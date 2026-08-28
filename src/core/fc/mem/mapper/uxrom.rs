use log::info;

use crate::core::fc::{
    mem::{
        Memory,
        cart::NESFile,
        mapper::{Mapper, MapperType, RealMapper},
    },
    ppu,
};

const BANK_SIZE: usize = 0x4000;

enum UxROMVariety {
    UxROM,
    UN1ROM,
    INES180, // only used in a single game....
}

pub struct UxROMMapper {
    board_type: UxROMVariety,
    bank: usize,

    prg_rom: Vec<u8>,
    chr_ram: Vec<u8>,
    nametable_v_mirror: bool,
    pub(crate) open_bus: u8,
}

impl UxROMMapper {
    fn nametable_addr_fix(&self, addr: u16) -> u16 {
        let a = addr & 0xfff;
        if self.nametable_v_mirror {
            a & 0x7ff
        } else {
            ((a & 0x800) >> 1) | (a & 0x3ff)
        }
    }
}

impl RealMapper for UxROMMapper {
    fn from_nesfile(nesfile: &NESFile) -> UxROMMapper {
        assert!(nesfile.mapper_type() == MapperType::UxROM);

        let board_type = match nesfile.mapper_number() {
            2 => UxROMVariety::UxROM,
            94 => UxROMVariety::UN1ROM,
            180 => UxROMVariety::INES180,
            _ => unreachable!(),
        };

        let prg_rom_size = nesfile.prg_rom_size();
        let chr_rom_size = if nesfile.chr_rom_size() == 0 {
            0x2000
        } else {
            nesfile.chr_rom_size()
        };
        let nametable_v_mirror = nesfile.nametable_layout();

        if nesfile.trainer() {
            unimplemented!("UxROM trainer handling");
        }

        info!("UxROM with:");
        info!("  PRG-ROM SIZE: {} (0x{:x})", prg_rom_size, prg_rom_size);
        info!("  CHR-ROM SIZE: {} (0x{:x})", chr_rom_size, chr_rom_size);
        info!(
            "  Nametable mirroring: {} ({} arrangement)",
            if nametable_v_mirror { "vertical" } else { "horizontal" },
            if nametable_v_mirror { "horizontal" } else { "vertical" }
        );

        let prg_rom = nesfile.data[0..prg_rom_size].to_vec();
        let chr_ram = vec![0; prg_rom_size];

        UxROMMapper {
            board_type,
            bank: 0x00, // ?
            prg_rom,
            chr_ram,
            nametable_v_mirror,
            open_bus: 0x00,
        }
    }
}

impl Memory for UxROMMapper {
    fn read(&mut self, addr: u16) -> u8 {
        self.read_no_sideeffect(addr)
    }

    fn write(&mut self, addr: u16, val: u8) -> () {
        if addr >= 0x8000 {
            match self.board_type {
                UxROMVariety::UN1ROM => self.bank = (val as usize >> 2) & 0b111,
                // TODO: UNROM uses 3 bits, UOROM uses 4 bits
                _ => self.bank = (val as usize) & 0b1111,
            }
        }
    }
}

impl Mapper for UxROMMapper {
    fn read_chr(&self, addr: u16) -> u8 {
        self.chr_ram[(addr as usize) % self.chr_ram.len()]
    }

    fn write_chr(&mut self, addr: u16, val: u8) -> () {
        let len = self.chr_ram.len();
        self.chr_ram[(addr as usize) % len] = val;
    }

    fn nametable_read(&self, addr: u16, vram: [u8; ppu::VRAM_SIZE]) -> u8 {
        let addr = self.nametable_addr_fix(addr);
        vram[addr as usize]
    }

    fn nametable_write(&mut self, addr: u16, val: u8, vram: &mut [u8; ppu::VRAM_SIZE]) -> () {
        let addr = self.nametable_addr_fix(addr);
        vram[addr as usize] = val;
    }

    fn read_no_sideeffect(&self, addr: u16) -> u8 {
        match addr {
            0x4020..=0x7fff => {
                info!("Open bus read at ${addr:04x}");
                self.open_bus
            }
            0x8000..=0xbfff => {
                // PRG ROM
                match self.board_type {
                    UxROMVariety::INES180 => self.prg_rom[(addr - 0x8000) as usize], // first bank
                    _ => self.prg_rom[self.bank * BANK_SIZE + (addr - 0x8000) as usize],
                }
            }
            0xc000..=0xffff => {
                // PRG ROM
                let banks = self.prg_rom.len() / BANK_SIZE;

                match self.board_type {
                    UxROMVariety::INES180 => self.prg_rom[self.bank * BANK_SIZE + (addr - 0xc000) as usize],
                    _ => self.prg_rom[(banks - 1) * BANK_SIZE + (addr - 0xc000) as usize], // last bank
                }
            }
            _ => unreachable!(),
        }
    }

    fn battery(&self) -> bool {
        false
    }
}
