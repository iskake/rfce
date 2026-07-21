use log::info;

use crate::fc::mem::{Memory, cart::NESFile, mapper::{Mapper, MapperType::{self}, RealMapper}};

const PRG_BANK_SIZE_16K: usize = 0x4000;
const PRG_BANK_SIZE_32K: usize = 0x8000;
const CHR_BANK_SIZE: usize = 0x2000;

#[derive(Debug)]
enum NametableArrangement {
    HorizontalMirroring,
    VerticalMirroring,
    SingleLowerBank,
    SingleUpperBank,
}

enum PRGBankMode {
    FixAll,
    FixFirst,
    FixLast,
}

enum CHRBankMode {
    Switch8K,
    Switch2x4K,
}

struct Registers {
    shift: u8,
    control: u8,
    prg_bank0: usize,
    prg_bank1: usize,
    chr_bank0: usize,
    chr_bank1: usize,
}

pub struct MMC1Mapper {
    prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    chr_rxm: Vec<u8>,
    prg_bank_mode: PRGBankMode,
    chr_bank_mode: CHRBankMode,
    nametable_arrange: NametableArrangement,
    reg: Registers,
}

impl MMC1Mapper {
    fn write_control(&mut self, val: u8) -> () {
        use NametableArrangement::*;
        use PRGBankMode::*;
        use CHRBankMode::*;

        self.reg.control = val & 0b11111;

        self.nametable_arrange = match self.reg.control & 0b11 {
            0b00 => SingleLowerBank,
            0b01 => SingleUpperBank,
            0b10 => VerticalMirroring,
            0b11 => HorizontalMirroring,
            _ => unreachable!(),
        };

        self.prg_bank_mode = match (self.reg.control & 0b1100) >> 2 {
            0b00 | 0b01 => FixAll,
            0b10 => FixFirst,
            0b11 => FixLast,
            _ => unreachable!(),
        };

        self.chr_bank_mode = if self.reg.control & 0b10000 == 0 {
            Switch8K
        } else {
            Switch2x4K
        };
    }

    fn write_chr_bank0(&mut self, val: u8) -> () {
        // TODO: (SEROM, SHROM, SH1ROM), (SNROM), (SOROM, SUROM, SXROM), (SZROM)
        self.reg.chr_bank0 = val as usize & 0b11111;
    }

    fn write_chr_bank1(&mut self, val: u8) -> () {
        // TODO: (SEROM, SHROM, SH1ROM), (SNROM), (SOROM, SUROM, SXROM), (SZROM)
        self.reg.chr_bank1 = val as usize & 0b11111;
    }

    fn write_prg_bank(&mut self, val: u8) -> () {
        // TODO: MMC1A and MMC1B use highest bit (0b10000) to mean different things
        match self.prg_bank_mode {
            PRGBankMode::FixAll   => self.reg.prg_bank0 = val as usize & 0b01110,
            PRGBankMode::FixFirst => self.reg.prg_bank0 = val as usize & 0b01111,
            PRGBankMode::FixLast  => self.reg.prg_bank1 = val as usize & 0b01111,
        }
    }

    fn write_internal(&mut self, addr: u16, val: u8) -> () {
        match addr {
            0x8000..=0x9fff => {
                // Control
                self.write_control(val);
            }
            0xa000..=0xbfff => {
                // CHR bank 0
                self.write_chr_bank0(val);
            }
            0xc000..=0xdfff => {
                // CHR bank 1
                self.write_chr_bank1(val);
            }
            0xe000..=0xffff => {
                // PRG bank
                self.write_prg_bank(val);
            }
            _ => todo!("write {val:02x} to address {addr:04x}"),
        }
    }
}

impl RealMapper for MMC1Mapper {
    fn from_nesfile(nesfile: &NESFile) -> MMC1Mapper {
        assert!(nesfile.mapper_type() == MapperType::MMC1);
        let prg_rom_size = nesfile.prg_rom_size();
        let prg_ram_size = nesfile.prg_ram_size();

        let chr_rom_size = nesfile.chr_rom_size();
        let chr_ram_size = nesfile.chr_ram_size();

        let battery = nesfile.battery();

        let nametable_arrange = if nesfile.nametable_layout() {
            NametableArrangement::HorizontalMirroring
        } else {
            NametableArrangement::VerticalMirroring
        };

        info!("MMC1 with:");
        info!("  PRG-ROM SIZE: {} (0x{:x}); {} banks", prg_rom_size, prg_rom_size, prg_rom_size / PRG_BANK_SIZE_16K);
        info!("  PRG-RAM SIZE: {} (0x{:x})", prg_ram_size, prg_ram_size);
        info!("  CHR-ROM SIZE: {} (0x{:x})", chr_rom_size, chr_rom_size);
        info!("  CHR-RAM SIZE: {} (0x{:x})", chr_ram_size, chr_ram_size);
        info!("  BATTERY: {}", battery);
        info!("  Nametable mirroring: {:?}", nametable_arrange);

        let prg_banks_num = prg_rom_size / PRG_BANK_SIZE_16K;

        let prg_rom = nesfile.data[0..prg_rom_size].to_vec();
        let prg_ram = vec![0; prg_ram_size];

        // TODO: possible to have both?
        let chr_rxm = if chr_rom_size != 0 {
            nesfile.data[prg_rom_size..(prg_rom_size + chr_rom_size)].to_vec()
        } else if chr_ram_size != 0 {
            vec![0; chr_ram_size]
        } else {
            vec![]
        };

        let prg_bank_mode = PRGBankMode::FixLast;   // ?
        let chr_bank_mode = CHRBankMode::Switch8K;  // ?

        MMC1Mapper {
            prg_rom,
            prg_ram,
            chr_rxm,
            nametable_arrange,
            reg: Registers {
                shift: 0x00,
                control: 0x00,
                prg_bank0: 0x00,    // ?
                prg_bank1: prg_banks_num - 1,    // ? 
                chr_bank0: 0x00,    // ?
                chr_bank1: 0x00,    // ?
            },
            prg_bank_mode,
            chr_bank_mode,
        }
    }
}

impl Memory for MMC1Mapper {
    fn read(&mut self, addr: u16) -> u8 {
        self.read_no_sideeffect(addr)
    }

    fn write(&mut self, addr: u16, val: u8) -> () {
        // Shift register shenanigans
        if val & 0b1000_0000 != 0 {
            // Reset shift register
            self.reg.shift = 0b1000_0000;
            self.write_internal(0x8000, self.reg.control | 0x0c);
        } else {
            // Shift the register
            let mut internal_write = false;

            if self.reg.shift & 1 == 1 {
                internal_write = true;
            }

            self.reg.shift >>= 1;
            self.reg.shift |= (val & 1) << 7;

            if internal_write {
                // Shift register is full, so now we can write its value to the internal registers.
                self.write_internal(addr, self.reg.shift);
                self.reg.shift = 0b1000_0000;
            }
        }
    }
}

impl Mapper for MMC1Mapper {
    fn read_chr(&self, addr: u16) -> u8 {
        todo!()
    }

    fn write_chr(&mut self, addr: u16, val: u8) -> () {
        todo!()
    }

    fn nametable_read(&self, addr: u16, vram: [u8; crate::fc::ppu::VRAM_SIZE]) -> u8 {
        todo!()
    }

    fn nametable_write(&mut self, addr: u16, val: u8, vram: &mut [u8; crate::fc::ppu::VRAM_SIZE]) -> () {
        todo!()
    }

    fn read_no_sideeffect(&self, addr: u16) -> u8 {
        match addr {
            0x6000..=0x7fff => {
                // "8KB PRG-RAM bank (optional)"
                self.prg_ram[(addr - 0x6000) as usize]
            },
            0x8000..=0xbfff => {
                // "16KB PRG-ROM bank, either switchable or fixed to the first bank"
                if let PRGBankMode::FixAll = self.prg_bank_mode {
                    self.prg_rom[(addr - 0x8000) as usize + self.reg.prg_bank0 * PRG_BANK_SIZE_32K]
                } else {
                    self.prg_rom[(addr - 0x8000) as usize + self.reg.prg_bank0 * PRG_BANK_SIZE_16K]
                }
            }
            0xc000..=0xffff => {
                // "16KB PRG-ROM bank, either fixed to the last bank or switchable"
                if let PRGBankMode::FixAll = self.prg_bank_mode {
                    self.prg_rom[(addr - 0x8000) as usize + self.reg.prg_bank0 * PRG_BANK_SIZE_32K]
                } else {
                    info!("reading address: {addr:04x}, curr bank: {}", self.reg.prg_bank1);
                    self.prg_rom[(addr - 0x8000) as usize + self.reg.prg_bank1 * PRG_BANK_SIZE_16K]
                }
            }
            _ => unreachable!()
        }

    }
}
