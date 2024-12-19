// For the actual specifications see the wiki:
// https://www.nesdev.org/wiki/NES_2.0

use std::{
    fs::File,
    io::{self, Read},
};

use crate::{bits::Bitwise, fc::mem::mapper::MapperType};

const NES_FILE_IDENTIFIER: [u8; 4] = [b'N', b'E', b'S', 0x1a];

pub struct NESFile {
    pub header: NESFileHeader, // ? change this to private?
    pub data: Vec<u8>,
}

impl NESFile {
    pub fn from_file(filename: &str) -> Result<NESFile, io::Error> {
        let mut f = File::open(filename)?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        Ok(NESFile::from_vec(buf))
    }

    pub fn from_vec(mut bytes: Vec<u8>) -> NESFile {
        if bytes.as_slice()[0..4] == NES_FILE_IDENTIFIER {
            println!("  Correct identifier :D");

            let data = bytes.split_off(16);
            // println!("{}", bytes.len());
            let header = NESFileHeader::from_slice(&bytes.as_slice()[4..16]);

            println!("NES2.0 header? {}", header.is_nes20_format());
            NESFile {
                header: header,
                data: data,
            }
        } else {
            // TODO: actually handle this
            panic!("File does not have the correct format (identifier corrupted/missing)")
        }
    }

    pub fn mapper_type(&self) -> MapperType {
        match self.header.mapper_number() {
            0 => MapperType::NROM,
            i => MapperType::UNKNOWN(i),
        }
    }
}


pub struct NESFileHeader {
    prg_rom_size_lsb: u8,
    chr_rom_size_lsb: u8,
    flags6: u8,
    flags7: u8,
    flags8: u8,
    flags9: u8,
    flags10: u8,
    // NES 2.0 only
    flags11: u8,
    flags12: u8,
    flags13: u8,
    flags14: u8,
    flags15: u8,
}

impl NESFileHeader {
    /// Construct a `NESFileHeader` from a `&[u8]`
    fn from_slice(bytes: &[u8]) -> NESFileHeader {
        NESFileHeader {
            prg_rom_size_lsb: bytes[0],
            chr_rom_size_lsb: bytes[1],
            flags6: bytes[2],
            flags7: bytes[3],
            flags8: bytes[4],
            flags9: bytes[5],
            flags10: bytes[6],
            flags11: bytes[7],
            flags12: bytes[8],
            flags13: bytes[9],
            flags14: bytes[10],
            flags15: bytes[11],
        }
    }

    /// Whether the header has the NES2.0 format or not
    pub fn is_nes20_format(&self) -> bool {
        self.flags7 & 0x0c == 0x08
    }

    // TODO? move these functions to NESFile instead?

    /// Nametable layout according to byte 6 of the header
    ///
    /// - `0`: vertical arrangement (horizontal mirror) / mapper controlled
    /// - `1`: horizontal arrangement (vertical mirror)
    pub fn nametable_layout(&self) -> bool {
        self.flags6.test_bit(0)
    }

    /// Whether a "battery"/non-volatile memory is present according to byte 6 of the header
    ///
    /// - `0`: not present
    /// - `1`: present
    pub fn battery(&self) -> bool {
        self.flags6.test_bit(1)
    }

    /// Whether a 512-bit trainer is present according to byte 6 of the header
    ///
    /// - `0`: not present
    /// - `1`: "present between header and PRG-ROM data"
    pub fn trainer(&self) -> bool {
        self.flags6.test_bit(2)
    }

    /// Whether a 512-bit trainer is present according to byte 6 of the header
    ///
    /// - `0`: not present
    /// - `1`: "present between header and PRG-ROM data"
    pub fn alt_nametable_layout(&self) -> bool {
        self.flags6.test_bit(3)
    }

    /// Get the mapper number stored in byte 6of the header (and bytes 7,8 when using the NES2.0 format)
    pub fn mapper_number(&self) -> u16 {
        let m0 = (self.flags6 & 0xf0) >> 4;
        if !self.is_nes20_format() {
            m0 as u16
        } else {
            let m1 = (self.flags7 & 0xf0) >> 4;
            let m2 = self.flags8 & 0x0f;
            ((m2 as u16) << 8) | ((m1 as u16) << 4) | m0 as u16
        }
    }

    /// Get the submapper number stored in byte 7 of the header (NES2.0 only)
    pub fn submapper_number(&self) -> u8 {
        if self.is_nes20_format() {
            (self.flags7 & 0xf0) >> 4
        } else {
            0
        }
    }

    /// Get the 'type' of console according to byte 7 of the header (and byte 13 if console type is 3)
    pub fn console_type(&self) -> u8 {
        let val = self.flags7 & 0b11;
        if self.is_nes20_format() && val == 3 {
            // Extended console type
            ((self.flags13 & 0xf) << 2) | val
        } else {
            // Non-extended console type
            val
        }
    }

    /// Get the size of PRG-ROM in bytes
    pub fn prg_rom_size(&self) -> usize {
        if !self.is_nes20_format() {
            let val = self.prg_rom_size_lsb as usize;
            val * 0x4000
        } else {
            if (self.flags9 & 0xf) == 0xf {
                // Exponent-multiplier notation
                let multiplier = (self.chr_rom_size_lsb & 0b11) as usize;
                let exponent = ((self.chr_rom_size_lsb & 0b1111_1100) >> 2) as usize;
                // TODO: check for overflow?
                (1 << exponent) * (multiplier * 2 + 1)
            } else {
                let val = ((self.flags9 as usize & 0xf) << 4) | self.prg_rom_size_lsb as usize;
                val * 0x4000
            }
        }
    }

    /// Get the size of CHR-ROM in bytes
    pub fn chr_rom_size(&self) -> usize {
        if !self.is_nes20_format() {
            let val = self.chr_rom_size_lsb as usize;
            val
        } else {
            if (self.flags9 & 0xf0) == 0xf0 {
                // Exponent-multiplier notation
                let multiplier = (self.chr_rom_size_lsb & 0b11) as usize;
                let exponent = ((self.chr_rom_size_lsb & 0b1111_1100) >> 2) as usize;
                // TODO: check for overflow?
                (1 << exponent) * (multiplier * 2 + 1)
            } else {
                // Normal
                let val = ((self.flags9 as usize & 0xf0) << 4) | self.chr_rom_size_lsb as usize;
                val * 0x2000
            }
        }
    }

    /// Get the PRG-RAM (volatile) size in bytes (NES2.0 only)
    pub fn prg_ram_size(&self) -> usize {
        if self.is_nes20_format() {
            let shift_count = self.flags10 & 0x0f;
            if shift_count == 0 {
                0
            } else {
                64 << shift_count
            }
        } else {
            0
        }
    }

    /// Get the PRG-NVRAM/EEPROM (non-volatile) size in bytes (NES2.0 only)
    pub fn prg_nvram_eeprom_size(&self) -> usize {
        if self.is_nes20_format() {
            let shift_count = (self.flags10 & 0xf0) >> 4;
            if shift_count == 0 {
                0
            } else {
                64 << shift_count
            }
        } else {
            0
        }
    }

    /// Get the CHR-RAM (volatile) size in bytes (NES2.0 only)
    pub fn chr_ram_size(&self) -> usize {
        if self.is_nes20_format() {
            let shift_count = self.flags11 & 0x0f;
            if shift_count == 0 {
                0
            } else {
                64 << shift_count
            }
        } else {
            0
        }
    }

    /// Get the CHR-NVRAM (non-volatile) size in bytes (NES2.0 only)
    pub fn chr_nvram_size(&self) -> usize {
        if self.is_nes20_format() {
            let shift_count = (self.flags11 & 0xf0) >> 4;
            if shift_count == 0 {
                0
            } else {
                64 << shift_count
            }
        } else {
            0
        }
    }

    /// Get the CPU/PPU timing mode (NES2.0 only)
    ///
    /// - 0: RP2C02 ("NTSC NES")
    /// - 1: RP2C07 ("Licensed PAL NES")
    /// - 2: Multiple-region
    /// - 3: UA6538 ("Dendy")
    pub fn cpu_ppu_timing_mode(&self) -> u8 {
        if self.is_nes20_format() {
            self.flags12 & 0b11
        } else {
            0
        }
    }

    // TODO: vs system type...

    /// Get the number of miscellaneous ROMs present (NES2.0 only)
    pub fn misc_roms_count(&self) -> u8 {
        if self.is_nes20_format() {
            self.flags14 & 0b11
        } else {
            0
        }
    }

    /// Get the default expansion device (NES2.0 only)
    pub fn default_expansion_device(&self) -> u8 {
        if self.is_nes20_format() {
            self.flags15 & 0x3f
        } else {
            0
        }
    }
}
