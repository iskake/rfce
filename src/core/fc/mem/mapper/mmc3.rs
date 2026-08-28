use log::{debug, info};

use crate::core::fc::{
    mem::{
        Memory,
        NametableArrangement::{self, FourScreen},
        mapper::{
            Mapper, MapperType, RealMapper,
            mmc3::{
                CHRBankMode::{Swap2KiBAt0000, Swap2KiBAt1000},
                NametableArrangement::{HorizontalMirroring, VerticalMirroring},
                PRGBankMode::{Swap8000, SwapC000},
            },
        },
    },
    ppu,
};

const PRG_BANK_SIZE: usize = 0x2000;
const CHR_BANK_SIZE: usize = 0x400;

#[derive(Debug)]
enum PRGBankMode {
    Swap8000,
    SwapC000,
}

#[derive(Debug)]
enum CHRBankMode {
    Swap2KiBAt0000,
    Swap2KiBAt1000,
}

struct Registers {
    bank_select: u8,

    chr_bank0: usize,
    chr_bank1: usize,
    chr_bank2: usize,
    chr_bank3: usize,
    chr_bank4: usize,
    chr_bank5: usize,

    prg_bank0: usize,
    prg_bank1: usize,
    prg_ram_protected: bool,
    prg_ram_enabled: bool,
    irq_latch_val: u8,
    irq_counter: u8,
    irq_reload: bool,
}

pub struct MMC3Mapper {
    battery: bool,
    prg_rom: Vec<u8>,
    prg_ram: Vec<u8>,
    chr_rom: Vec<u8>,
    _chr_ram: Vec<u8>,  // ?
    nametable_arrange: NametableArrangement,
    irq_enabled: bool,
    prg_bank_mode: PRGBankMode,
    chr_bank_mode: CHRBankMode,
    reg: Registers,
    irq_triggered: bool,
    pub(crate) open_bus: u8,
}

impl RealMapper for MMC3Mapper {
    fn from_nesfile(nesfile: &crate::core::fc::mem::cart::NESFile) -> Self {
        assert!(nesfile.mapper_type() == MapperType::MMC3);
        let prg_rom_size = nesfile.prg_rom_size();
        let battery = nesfile.battery();
        let prg_ram_size = if nesfile.is_nes20_format() {
            if battery {
                nesfile.prg_nvram_eeprom_size()
            } else {
                nesfile.prg_ram_size()
            }
        } else {
            0x2000 // ?
        };

        let chr_rom_size = nesfile.chr_rom_size();
        let chr_ram_size = nesfile.chr_ram_size();

        let prg_banks_num = prg_rom_size / PRG_BANK_SIZE;
        let chr_banks_num = chr_rom_size / CHR_BANK_SIZE;

        let nametable_arrange = if nesfile.alt_nametable_layout() {
            NametableArrangement::FourScreen
        } else if nesfile.nametable_layout() {
            NametableArrangement::HorizontalMirroring
        } else {
            NametableArrangement::VerticalMirroring
        };

        info!("MMC3 with:");
        info!(
            "  PRG-ROM SIZE: {} (0x{:x}); {} 8KiB banks",
            prg_rom_size, prg_rom_size, prg_banks_num
        );
        info!("  PRG-RAM SIZE: {} (0x{:x})", prg_ram_size, prg_ram_size);
        info!(
            "  CHR-ROM SIZE: {} (0x{:x}); {} 1KiB banks",
            chr_rom_size, chr_rom_size, chr_banks_num
        );
        info!(
            "  CHR-RAM SIZE: {} (0x{:x}){}",
            chr_ram_size,
            chr_ram_size,
            if chr_ram_size > 0 { " (likely false)" } else { "" }
        );
        info!("  BATTERY: {}", battery);
        info!("  Nametable mirroring: {:?}", nametable_arrange);

        let prg_rom = nesfile.data[0..prg_rom_size].to_vec();
        let prg_ram = vec![0; prg_ram_size];
        let chr_rom = nesfile.data[prg_rom_size..(prg_rom_size + chr_rom_size)].to_vec();
        let chr_ram = vec![0; chr_ram_size];

        MMC3Mapper {
            battery,
            prg_rom,
            prg_ram,
            chr_rom,
            _chr_ram: chr_ram,
            nametable_arrange,
            irq_enabled: false,
            prg_bank_mode: Swap8000,
            chr_bank_mode: Swap2KiBAt0000,
            irq_triggered: false,
            reg: Registers {
                bank_select: 0x00,
                chr_bank0: 0,
                chr_bank1: 2,
                chr_bank2: 4,
                chr_bank3: 5,
                chr_bank4: 6,
                chr_bank5: 7,
                prg_bank0: 0,
                prg_bank1: 1,
                prg_ram_protected: false,
                prg_ram_enabled: false,
                irq_latch_val: 0x00,
                irq_counter: 0x00,
                irq_reload: false,
            },

            open_bus: 0x00,
        }
    }
}

impl MMC3Mapper {
    pub(crate) fn print_state(&self) -> () {
        println!("MMC3 STATE:");
        println!(
            "  IRQ - latch: {:02x}, counter: {:02x}, reload flag: {}, enable: {}",
            self.reg.irq_latch_val, self.reg.irq_counter, self.reg.irq_reload, self.irq_enabled
        );
        println!(
            "  CHR banks: [0: {}], [1: {}], [2: {}], [3: {}], [4: {}], [5: {}] - PRG BANKS: [6: {}], [7: {}]",
            self.reg.chr_bank0,
            self.reg.chr_bank1,
            self.reg.chr_bank2,
            self.reg.chr_bank3,
            self.reg.chr_bank4,
            self.reg.chr_bank5,
            self.reg.prg_bank0,
            self.reg.prg_bank1
        )
    }

    fn write_bank_select(&mut self, val: u8) -> () {
        self.reg.bank_select = val & 0b111;

        self.prg_bank_mode = if (val & 0b0100_0000) == 0 { Swap8000 } else { SwapC000 };

        self.chr_bank_mode = if (val & 0b1000_0000) == 0 {
            Swap2KiBAt0000
        } else {
            Swap2KiBAt1000
        };

        debug!("Wrote 0x{0:02x} (0b{0:08b}) to bank select", self.reg.bank_select);
        debug!("  Bank register to update: 0b{:03b}", val & 0b111);
        debug!("  PRG bank mode: {:?}", self.prg_bank_mode);
        debug!("  CHR bank mode: {:?}", self.chr_bank_mode);
    }

    fn write_bank_data(&mut self, val: u8) -> () {
        match self.reg.bank_select & 0b111 {
            0b000 => self.reg.chr_bank0 = (val & 0b1111_1110) as usize,
            0b001 => self.reg.chr_bank1 = (val & 0b1111_1110) as usize,
            0b010 => self.reg.chr_bank2 = val as usize,
            0b011 => self.reg.chr_bank3 = val as usize,
            0b100 => self.reg.chr_bank4 = val as usize,
            0b101 => self.reg.chr_bank5 = val as usize,
            0b110 => self.reg.prg_bank0 = (val & 0b0011_1111) as usize,
            0b111 => self.reg.prg_bank1 = (val & 0b0011_1111) as usize,
            _ => unreachable!(),
        }

        debug!("Wrote 0x{0:02x} (0b{0:08b}) to bank data", val);
        match self.reg.bank_select & 0b111 {
            0b000 => debug!("  Set CHR bank 0 (R0) to {0} (0x{0:02x})", val & 0b1111_1110),
            0b001 => debug!("  Set CHR bank 1 (R1) to {0} (0x{0:02x})", val & 0b1111_1110),
            0b010 => debug!("  Set CHR bank 2 (R2) to {0} (0x{0:02x})", val),
            0b011 => debug!("  Set CHR bank 3 (R3) to {0} (0x{0:02x})", val),
            0b100 => debug!("  Set CHR bank 4 (R4) to {0} (0x{0:02x})", val),
            0b101 => debug!("  Set CHR bank 5 (R5) to {0} (0x{0:02x})", val),
            0b110 => debug!("  Set PRG bank 0 (R6) to {0} (0x{0:02x})", val & 0b0011_1111),
            0b111 => debug!("  Set PRG bank 1 (R7) to {0} (0x{0:02x})", val & 0b0011_1111),
            _ => unreachable!(),
        }
    }

    fn write_nametable_arrange(&mut self, val: u8) -> () {
        match self.nametable_arrange {
            FourScreen => {}
            _ => {
                self.nametable_arrange = if val & 1 == 0 {
                    VerticalMirroring
                } else {
                    HorizontalMirroring
                }
            }
        }
        debug!("Wrote {} to nametable arrange", val & 1)
    }

    fn write_prg_ram_protect(&mut self, val: u8) -> () {
        // TODO?: MMC6
        self.reg.prg_ram_protected = val & 0b01000000 != 0;
        self.reg.prg_ram_enabled = val & 0b10000000 != 0;
        debug!(
            "Wrote 0x{0:02x} (0b:{0:08b}) to PRG RAM protect (protected: {1}, enabled: {2})",
            val & 0b11000000,
            self.reg.prg_ram_protected,
            self.reg.prg_ram_enabled
        );
    }

    fn write_irq_latch(&mut self, val: u8) -> () {
        // "Writing to $C000 does not immediately affect the value within the counter - this value
        // is only used when the counter is reloaded, whether from reaching 0 or from writing to $C001"
        self.reg.irq_latch_val = val;
        debug!("Wrote 0x{0:02x} (0b{0:08b}) to IRQ latch", val);
    }

    fn write_irq_reload(&mut self, _: u8) -> () {
        // "Writing to $C001 will cause the counter to be cleared, and set _reload flag_ to true.
        // It will be reloaded on the NEXT rising edge of filtered A12."
        self.reg.irq_counter = 0x00;
        self.reg.irq_reload = true;
        debug!("Wrote to IRQ reload");
    }

    fn write_irq_disable(&mut self, _: u8) -> () {
        // "Writing to $E000 will only prevent the MMC3 from generating IRQs - the counter will continue to run."
        self.irq_enabled = false;
        // "acknowledge any pending interrupts"
        debug!("Disabled IRQ")
    }

    fn write_irq_enable(&mut self, _: u8) -> () {
        // "Writing to $E001 will simply allow the MMC3 to generate IRQs - the counter remains unaffected."
        self.irq_enabled = true;
        debug!("enabled IRQ")
    }

    pub(crate) fn dec_irq_counter(&mut self) {
        if self.reg.irq_counter == 0 || self.reg.irq_reload {
            self.reg.irq_counter = self.reg.irq_latch_val;
            debug!("Reload IRQ counter to {}", self.reg.irq_counter);
        } else {
            self.reg.irq_counter -= 1;
            debug!("Decrement IRQ counter to {}", self.reg.irq_counter);
        }

        if self.reg.irq_counter == 0 && self.irq_enabled {
            self.irq_triggered = true;
            debug!("IRQ trigger");
        }

        self.reg.irq_reload = false;
    }

    pub(crate) fn irq_triggered(&self) -> bool {
        self.irq_triggered
    }

    pub(crate) fn irq_un_trigger(&mut self) {
        self.irq_triggered = false;
    }

    pub(crate) fn replace_sram(&mut self, sram: Vec<u8>) -> Result<(), std::io::Error> {
        if self.prg_ram.len() != sram.len() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Size of save RAM is incorrect, expected {} got {}",
                    self.prg_ram.len(),
                    sram.len()
                )
            ));
        }

        self.prg_ram = sram;
        Ok(())
    }

    pub(crate) fn sram(&self) -> &Vec<u8> {
        &self.prg_ram
    }

    pub(crate) fn sram_mut(&mut self) -> &mut Vec<u8> {
        &mut self.prg_ram
    }
}

impl Memory for MMC3Mapper {
    fn read(&mut self, addr: u16) -> u8 {
        self.read_no_sideeffect(addr)
    }

    fn write(&mut self, addr: u16, val: u8) -> () {
        debug!("Writing {val:02x} to address: {addr:04x}");
        match addr {
            0x6000..=0x7fff => {
                if !self.reg.prg_ram_protected {
                    self.prg_ram[(addr - 0x6000) as usize] = val;
                }
            }
            0x8000..=0x9fff => {
                if addr & 1 == 0 {
                    // Even - bank select
                    self.write_bank_select(val);
                } else {
                    // Odd - bank data
                    self.write_bank_data(val);
                }
            }
            0xa000..=0xbfff => {
                if addr & 1 == 0 {
                    // Even - nametable arrangement
                    self.write_nametable_arrange(val);
                } else {
                    // Odd - PRG RAM protect
                    self.write_prg_ram_protect(val);
                }
            }
            0xc000..=0xdfff => {
                if addr & 1 == 0 {
                    // Even - IRQ latch
                    self.write_irq_latch(val);
                } else {
                    // Odd - IRQ reload
                    self.write_irq_reload(val);
                }
            }
            0xe000..=0xffff => {
                if addr & 1 == 0 {
                    // Even - IRQ disable
                    self.write_irq_disable(val);
                } else {
                    // Odd - IRQ enable
                    self.write_irq_enable(val);
                }
            }
            _ => unreachable!(),
        }
    }
}

macro_rules! bank_addr {
    (CHR; $addr:expr, $addr_delta:expr, $bank_n:expr) => {
        ($addr - $addr_delta) as usize + $bank_n * CHR_BANK_SIZE
    };

    (PRG; $addr:expr, $addr_delta:expr, $bank_n:expr) => {
        ($addr - $addr_delta) as usize + $bank_n * PRG_BANK_SIZE
    };
}

impl Mapper for MMC3Mapper {
    fn read_chr(&self, addr: u16) -> u8 {
        if let Swap2KiBAt0000 = self.chr_bank_mode {
            match addr {
                0x0000..=0x07ff => self.chr_rom[bank_addr!(CHR; addr, 0x0000, self.reg.chr_bank0)],
                0x0800..=0x0fff => self.chr_rom[bank_addr!(CHR; addr, 0x0800, self.reg.chr_bank1)],
                0x1000..=0x13ff => self.chr_rom[bank_addr!(CHR; addr, 0x1000, self.reg.chr_bank2)],
                0x1400..=0x17ff => self.chr_rom[bank_addr!(CHR; addr, 0x1400, self.reg.chr_bank3)],
                0x1800..=0x1bff => self.chr_rom[bank_addr!(CHR; addr, 0x1800, self.reg.chr_bank4)],
                0x1c00..=0x1fff => self.chr_rom[bank_addr!(CHR; addr, 0x1c00, self.reg.chr_bank5)],
                _ => unreachable!(),
            }
        } else {
            match addr {
                0x0000..=0x03ff => self.chr_rom[bank_addr!(CHR; addr, 0x0000, self.reg.chr_bank2)],
                0x0400..=0x07ff => self.chr_rom[bank_addr!(CHR; addr, 0x0400, self.reg.chr_bank3)],
                0x0800..=0x0bff => self.chr_rom[bank_addr!(CHR; addr, 0x0800, self.reg.chr_bank4)],
                0x0c00..=0x0fff => self.chr_rom[bank_addr!(CHR; addr, 0x0c00, self.reg.chr_bank5)],
                0x1000..=0x17ff => self.chr_rom[bank_addr!(CHR; addr, 0x1000, self.reg.chr_bank0)],
                0x1800..=0x1fff => self.chr_rom[bank_addr!(CHR; addr, 0x1800, self.reg.chr_bank1)],
                _ => unreachable!(),
            }
        }
    }

    fn write_chr(&mut self, addr: u16, val: u8) -> () {
        // TODO: CHR writes (only used for some games?)
        debug!("CHR write; value {val:02x} to address {addr:04x}");
    }

    fn nametable_read(&self, addr: u16, vram: [u8; ppu::VRAM_SIZE]) -> u8 {
        vram[self.nametable_arrange.nametable_addr_fix(addr) as usize]
    }

    fn nametable_write(&mut self, addr: u16, val: u8, vram: &mut [u8; ppu::VRAM_SIZE]) -> () {
        vram[self.nametable_arrange.nametable_addr_fix(addr) as usize] = val
    }

    fn read_no_sideeffect(&self, addr: u16) -> u8 {
        let banks = self.prg_rom.len() / PRG_BANK_SIZE;
        match addr {
            0x4020..=0x5fff => {
                info!("Open bus read at ${addr:04x}");
                self.open_bus
            }
            0x6000..=0x7fff => {
                // "8KB switchable RAM bank (optional)"
                if self.reg.prg_ram_enabled && self.prg_ram.len() > 0 {
                    self.prg_ram[(addr - 0x6000) as usize]
                } else {
                    info!("Open bus read at ${addr:04x}");
                    self.open_bus
                }
            }
            0x8000..=0x9fff => {
                // "8KB switchable/fixed PRG ROM bank"
                let bank = if let Swap8000 = self.prg_bank_mode {
                    // Switchable using prg_bank0
                    self.reg.prg_bank0 % banks
                } else {
                    // Fixed to second-to-last bank (-2)
                    banks - 2
                };

                self.prg_rom[bank_addr!(PRG; addr, 0x8000, bank)]
            }
            0xa000..=0xbfff => {
                // "8KB switchable PRG ROM bank"
                let bank = self.reg.prg_bank1 % banks;

                self.prg_rom[bank_addr!(PRG; addr, 0xa000, bank)]
            }
            0xc000..=0xdfff => {
                // "8KB switchable/fixed PRG ROM bank"
                let bank = if let Swap8000 = self.prg_bank_mode {
                    // Fixed to second-to-last bank (-2)
                    banks - 2
                } else {
                    // Switchable using prg_bank0
                    self.reg.prg_bank0 % banks
                };

                self.prg_rom[bank_addr!(PRG; addr, 0xc000, bank)]
            }
            0xe000..=0xffff => {
                // "8KB PRG ROM bank, fixed to the last bank"
                self.prg_rom[bank_addr!(PRG; addr, 0xe000, (banks - 1))]
            }
            _ => unreachable!(),
        }
    }

    fn battery(&self) -> bool {
        self.battery
    }
}
