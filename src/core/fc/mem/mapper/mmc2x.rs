use log::info;

use crate::{
    bits::Bitwise,
    core::fc::{
        mem::{
            Memory,
            NametableArrangement::{self, HorizontalMirroring, VerticalMirroring},
            cart::NESFile,
            mapper::{Mapper, MapperType, RealMapper},
        },
        ppu,
    },
};

const PRG_BANK_SIZE_MMC2: usize = 0x2000;
const PRG_BANK_SIZE_MMC4: usize = 0x4000;
const CHR_BANK_SIZE: usize = 0x1000;

#[derive(Debug)]
enum MMCVariety {
    MMC2,
    MMC4,
}

pub struct MMC2Mapper {
    board_type: MMCVariety,
    prg_bank: usize,

    chr_latch0_is_fe: bool,
    chr_latch1_is_fe: bool,
    chr_bank_0000_fd: usize,
    chr_bank_0000_fe: usize,
    chr_bank_1000_fd: usize,
    chr_bank_1000_fe: usize,

    prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    chr_rom: Vec<u8>,
    nametable_arrange: NametableArrangement,
    pub(crate) open_bus: u8,
}

impl MMC2Mapper {
    fn update_latches(&mut self, addr: u16) {
        match self.board_type {
            MMCVariety::MMC2 => match addr {
                0x0fd8 => self.chr_latch0_is_fe = false,
                0x0fe8 => self.chr_latch0_is_fe = true,
                0x1fd8..=0x1fdf => self.chr_latch1_is_fe = false,
                0x1fe8..=0x1fef => self.chr_latch1_is_fe = true,
                _ => {}
            },
            MMCVariety::MMC4 => match addr {
                0x0fd8..=0x0fdf => self.chr_latch0_is_fe = false,
                0x0fe8..=0x0fef => self.chr_latch0_is_fe = true,
                0x1fd8..=0x1fdf => self.chr_latch1_is_fe = false,
                0x1fe8..=0x1fef => self.chr_latch1_is_fe = true,
                _ => {}
            },
        }
    }
}

impl RealMapper for MMC2Mapper {
    fn from_nesfile(nesfile: &NESFile) -> MMC2Mapper {
        assert!(nesfile.mapper_type() == MapperType::MMC2 || nesfile.mapper_type() == MapperType::MMC4);

        let board_type = match nesfile.mapper_type() {
            MapperType::MMC2 => MMCVariety::MMC2,
            MapperType::MMC4 => MMCVariety::MMC4,
            _ => unreachable!(),
        };

        let prg_rom_size = nesfile.prg_rom_size();
        let prg_ram_size = if nesfile.prg_ram_size() == 0 {
            if nesfile.mapper_type() == MapperType::MMC4 {
                0x2000 // MMC4 always has 8KiB RAM
            } else {
                0
            }
        } else {
            nesfile.prg_ram_size()
        };
        let chr_rom_size = if nesfile.chr_rom_size() == 0 {
            0x2000
        } else {
            nesfile.chr_rom_size()
        };
        let nametable_arrange = nesfile.nametable_layout();

        info!("{board_type:?} with:");
        info!(
            "  PRG-ROM SIZE: {} (0x{:x}) {} banks",
            prg_rom_size,
            prg_rom_size,
            prg_rom_size
                / if nesfile.mapper_type() == MapperType::MMC2 {
                    PRG_BANK_SIZE_MMC2
                } else {
                    PRG_BANK_SIZE_MMC4
                }
        );
        info!("  PRG-RAM SIZE: {} (0x{:x})", prg_ram_size, prg_ram_size);
        info!("  CHR-ROM SIZE: {} (0x{:x})", chr_rom_size, chr_rom_size);
        info!("  Nametable mirroring: {:?}", nametable_arrange);

        let prg_rom = nesfile.data[0..prg_rom_size].to_vec();
        let prg_ram = vec![0; prg_ram_size];
        let chr_rom = nesfile.data[prg_rom_size..(prg_rom_size + chr_rom_size)].to_vec();

        MMC2Mapper {
            board_type,
            prg_rom,
            prg_ram,
            chr_rom,
            nametable_arrange,

            prg_bank: 0, // ?

            chr_latch0_is_fe: false, // ?
            chr_latch1_is_fe: false, // ?

            chr_bank_0000_fd: 0, // ?
            chr_bank_0000_fe: 0, // ?
            chr_bank_1000_fd: 0, // ?
            chr_bank_1000_fe: 0, // ?

            open_bus: 0x00,
        }
    }
}

impl Memory for MMC2Mapper {
    fn read(&mut self, addr: u16) -> u8 {
        self.read_no_sideeffect(addr)
    }

    fn write(&mut self, addr: u16, val: u8) -> () {
        match addr {
            0xa000..=0xafff => {
                // "PRG ROM bank select"
                self.prg_bank = (val as usize) & 0b1111;
            }
            0xb000..=0xbfff => {
                // "CHR ROM $FD/0000 bank select"
                self.chr_bank_0000_fd = (val as usize) & 0b1_1111;
            }
            0xc000..=0xcfff => {
                // "CHR ROM $FE/0000 bank select"
                self.chr_bank_0000_fe = (val as usize) & 0b1_1111;
            }
            0xd000..=0xdfff => {
                // "CHR ROM $FD/1000 bank select"
                self.chr_bank_1000_fd = (val as usize) & 0b1_1111;
            }
            0xe000..=0xefff => {
                // "CHR ROM $FE/1000 bank select"
                self.chr_bank_1000_fe = (val as usize) & 0b1_1111;
            }
            0xf000..=0xffff => {
                // "Mirroring"
                self.nametable_arrange = if val.test_bit(0) {
                    HorizontalMirroring
                } else {
                    VerticalMirroring
                }
            }
            _ => {}
        }
    }
}

impl Mapper for MMC2Mapper {
    fn read_chr(&mut self, addr: u16) -> u8 {
        let val = self.read_chr_no_sideeffect(addr);
        // TODO: is it before or after?
        self.update_latches(addr);

        val
    }

    fn read_chr_no_sideeffect(&self, addr: u16) -> u8 {
        let bank = if addr < 0x1000 {
            if self.chr_latch0_is_fe {
                self.chr_bank_0000_fe
            } else {
                self.chr_bank_0000_fd
            }
        } else {
            if self.chr_latch1_is_fe {
                self.chr_bank_1000_fe
            } else {
                self.chr_bank_1000_fd
            }
        };

        let addr = bank * CHR_BANK_SIZE + ((addr as usize) & 0xfff);
        self.chr_rom[addr % self.chr_rom.len()]
    }

    fn write_chr(&mut self, _addr: u16, _val: u8) -> () {}

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
                // PRG RAM
                if self.prg_ram.len() > 0 {
                    self.prg_ram[addr as usize % self.prg_ram.len()]
                } else {
                    info!("Open bus read at ${addr:04x}");
                    self.open_bus
                }
            }
            0x8000..=0xffff => {
                // PRG ROM
                match self.board_type {
                    MMCVariety::MMC2 => {
                        let banks = self.prg_rom.len() / PRG_BANK_SIZE_MMC2;

                        if addr < 0xa000 {
                            // $6000-$7fff: 8K switchable
                            self.prg_rom[(self.prg_bank % banks) * PRG_BANK_SIZE_MMC2 + (addr - 0x8000) as usize]
                        } else {
                            // $a000-$ffff: three fixed 8k banks
                            self.prg_rom[(banks - 3) * PRG_BANK_SIZE_MMC2 + (addr - 0xa000) as usize]
                        }
                    }
                    MMCVariety::MMC4 => {
                        let banks = self.prg_rom.len() / PRG_BANK_SIZE_MMC4;

                        if addr < 0xc000 {
                            // $6000-$bfff: 16K switchable
                            self.prg_rom[(self.prg_bank % banks) * PRG_BANK_SIZE_MMC4 + (addr - 0x8000) as usize]
                        } else {
                            // $c000-$ffff: 16k fixed
                            self.prg_rom[(banks - 1) * PRG_BANK_SIZE_MMC4 + (addr - 0xc000) as usize]
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
    }

    fn battery(&self) -> bool {
        false
    }
}
