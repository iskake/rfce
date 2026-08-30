use log::{debug, info};

use crate::core::fc::{
    mem::{
        Memory, NametableArrangement, cart::NESFile, mapper::{Mapper, MapperType, RealMapper},
    }, ppu,
};

const BANK_SIZE: usize = 0x2000;

#[derive(Debug)]
enum CNROMVariety {
    // iNES mapper 003
    INES003UnkBusConflict,
    INES003NonBusConflict,
    INES003AndBusConflict,

    // iNES mapper 185
    INES185Sub0,
    INES185Sub4,
    INES185Sub5,
    INES185Sub6,
    INES185Sub7,
}

impl CNROMVariety {
    fn has_cs_val(&self, cs_val: u8, last_ppu_write: u8) -> bool {
        match self {
            // Hack(?) using heuristic described in MesenCE `CnromProtect.h`:
            //   "if C AND $0F is nonzero, and if C does not equal $13: CHR is enabled"
            CNROMVariety::INES185Sub0 => cs_val & 0b11 != 0 && last_ppu_write != 0x13,
            CNROMVariety::INES185Sub4 => cs_val & 0b11 == 0b00,
            CNROMVariety::INES185Sub5 => cs_val & 0b11 == 0b01,
            CNROMVariety::INES185Sub6 => cs_val & 0b11 == 0b10,
            CNROMVariety::INES185Sub7 => cs_val & 0b11 == 0b11,
            _ => false,
        }
    }
}

pub struct CNROMMapper {
    board_type: CNROMVariety,
    bank_reg: u8,

    prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    chr_rom: Vec<u8>,

    nametable_arrange: NametableArrangement,
    pub(crate) open_bus: u8,

    last_ppu_write: u8,
}

impl RealMapper for CNROMMapper {
    fn from_nesfile(nesfile: &NESFile) -> CNROMMapper {
        assert!(nesfile.mapper_type() == MapperType::CNROM);

        let board_type = match nesfile.mapper_number() {
            // TODO: submappers
            3 => {
                match nesfile.submapper_number() {
                    0 => CNROMVariety::INES003UnkBusConflict,
                    1 => CNROMVariety::INES003NonBusConflict,
                    2 => CNROMVariety::INES003AndBusConflict,
                    i => unreachable!("mapper 003 submapper {i}"),
                }
            },
            185 => {
                match nesfile.submapper_number() {
                    0 => CNROMVariety::INES185Sub0,
                    4 => CNROMVariety::INES185Sub4,
                    5 => CNROMVariety::INES185Sub5,
                    6 => CNROMVariety::INES185Sub6,
                    7 => CNROMVariety::INES185Sub7,
                    i => unreachable!("mapper 185 submapper {i}"),
                }
            },
            _ => unreachable!(),
        };

        let prg_rom_size = nesfile.prg_rom_size_or_default(0x8000);
        let chr_rom_size = nesfile.chr_rom_size_or_default(0x8000);

        let prg_ram_size = nesfile.prg_ram_size();  // only used in "Hayauchi Super Igo"

        let nametable_arrange = nesfile.nametable_layout();

        info!("CNROM with:");
        info!("  PRG-ROM SIZE: {} (0x{:x})", prg_rom_size, prg_rom_size);
        info!("  CHR-ROM SIZE: {} (0x{:x})", chr_rom_size, chr_rom_size);
        info!("  Nametable mirroring: {:?}", nametable_arrange);
        info!("  Board variety: {:?}", board_type);

        let prg_rom = nesfile.data[0..prg_rom_size].to_vec();
        let prg_ram = vec![0; prg_ram_size];
        let chr_rom = if nesfile.data.len() < prg_rom_size + chr_rom_size {
            let actual_size = nesfile.data.len();
            let claimed_size = prg_rom_size + chr_rom_size;
            info!("  INCORRECT DATA IN NES FILE!");
            info!("    claimed ROM size: {0} (${0:x})", claimed_size);
            info!("    actual ROM size:  {0} (${0:x})", actual_size);
            info!("    difference:       {0} (${0:x})", claimed_size - actual_size);
            nesfile.data[prg_rom_size..].to_vec()
        } else {
            nesfile.data[prg_rom_size..(prg_rom_size + chr_rom_size)].to_vec()
        };

        CNROMMapper {
            board_type,
            prg_rom,
            prg_ram,
            chr_rom,
            bank_reg: 0x00, // ?
            nametable_arrange,
            open_bus: 0x00,

            last_ppu_write: 0
        }
    }
}

impl Memory for CNROMMapper {
    fn read(&mut self, addr: u16) -> u8 {
        self.read_no_sideeffect(addr)
    }

    fn write(&mut self, addr: u16, val: u8) -> () {
        use CNROMVariety::*;

        // ? Writes to $6000-$7fff: Speech Start/Message Select, used in "Family Trainer: Aerobatics Studio"

        if addr >= 0x8000 {
            match self.board_type {
               INES003NonBusConflict => self.bank_reg = val & 0b11_0011,
                INES003AndBusConflict
                // Here we assume AND-type bus conflicts for submapper 0 ("unknown"),
                // since "the original board is always subject to [them]".
                // (https://www.nesdev.org/wiki/CNROM#Regular_mapper_3_with_up_to_32_KiB)
                | INES003UnkBusConflict 
                // Same for mapper 185: "[it] always has AND-type bus conflicts."
                // (https://www.nesdev.org/wiki/CNROM#Mapper_185)
                | INES185Sub0
                | INES185Sub4
                | INES185Sub5
                | INES185Sub6
                | INES185Sub7 => {
                    debug!("Bus conflict");

                    let mask = if self.chr_rom.len() > 0x8000 {
                        // Ovewrsize mapper (only licensed use is "Family Trainer: Jogging Race")
                        0b00_1111
                    } else {
                        0b11_0011
                    };

                    self.bank_reg = (val & self.prg_rom[(addr as usize - 0x8000) % self.prg_rom.len()]) & mask;

                    self.last_ppu_write = val;
                },
            }
        }
    }
}

impl Mapper for CNROMMapper {
    fn read_chr(&mut self, addr: u16) -> u8 {
        self.read_chr_no_sideeffect(addr)
    }

    fn read_chr_no_sideeffect(&self, addr: u16) -> u8 {
        use CNROMVariety::*;

        match self.board_type {
            INES003UnkBusConflict
            | INES003NonBusConflict
            | INES003AndBusConflict => self.chr_rom[((self.bank_reg as usize & 0b11) * BANK_SIZE + addr as usize) & (self.chr_rom.len() - 1)],
            INES185Sub0
            | INES185Sub4
            | INES185Sub5
            | INES185Sub6
            | INES185Sub7 => {
                if self.board_type.has_cs_val(self.bank_reg, self.last_ppu_write) {
                    self.chr_rom[addr as usize]
                } else {
                    // TODO: "real" PPU open bus reads
                    info!("PPU Open bus read at ${addr:04x}");
                    // Hack(?):
                    // "the earlier revision of Mighty Bomb Jack in fact relies on open bus at PPU address $0000
                    // being something other than $00 (by means of a 10k pull-up resistor on the CHR ROM's D0 pin)."
                    ((addr & 0xff00) >> 8) as u8 | 1
                }
            }
        }
    }

    fn write_chr(&mut self, _addr: u16, _val: u8) -> () {
    }

    fn nametable_read(&self, addr: u16, vram: [u8; ppu::VRAM_SIZE]) -> u8 {
        let addr = self.nametable_arrange.nametable_addr_fix(addr);
        vram[addr as usize]
    }

    fn nametable_write(&mut self, addr: u16, val: u8, vram: &mut [u8; ppu::VRAM_SIZE]) -> () {
        let addr = self.nametable_arrange.nametable_addr_fix(addr);
        vram[addr as usize] = val;
    }

    fn read_no_sideeffect(&self, addr: u16) -> u8 {
        match addr {
            0x4020..=0x5fff => {
                info!("Open bus read at ${addr:04x}");
                self.open_bus
            }
            0x6000..=0x7fff => {
                // PRG RAM ("Hayauchi Super Igo" only)
                if self.prg_ram.len() > 0 {
                    let addr = addr as usize - 0x6000;
                    // Only used in a single game, which has 2KiB banks
                    self.prg_ram[addr & 0x7ff]
                } else {
                    info!("Open bus read at ${addr:04x}");
                    self.open_bus
                }
            }
            0x8000..=0xffff => {
                // PRG ROM
                self.prg_rom[(addr as usize - 0x8000) % self.prg_rom.len()]
            }
            _ => unreachable!(),
        }
    }

    fn battery(&self) -> bool {
        false
    }
}
