use std::{
    fs::File,
    io::{self, Read},
};

use crate::fc::mem::mapper::MapperType;

const NES_FILE_IDENTIFIER: [u8; 4] = [b'N', b'E', b'S', 0x1a];

pub struct NESFile {
    header: NESFileHeader,
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
        match self.header.mapper_type() {
            0 => MapperType::NROM,
            i => MapperType::UNKNOWN(i),
        }
    }
}

// TODO: make an enum for iNES / NES 2.0 formats? or ignore because NES2.0 is backwards compatible?
struct NESFileHeader {
    pub prg_rom_size: u8,
    pub chr_rom_size: u8,
    pub flags6: u8,
    pub flags7: u8,
    pub flags8: u8,
    pub flags9: u8,
    pub flags10: u8,
    // NES 2.0:
    pub flags_chr_ram_size: u8,
    pub flags_cpu_ppu_timing: u8,
    pub flags_vs_system: u8,
    pub flags_misc_roms: u8,
    pub flags_default_expansion_device: u8,
}

impl NESFileHeader {
    fn from_slice(bytes: &[u8]) -> NESFileHeader {
        NESFileHeader {
            prg_rom_size: bytes[0],
            chr_rom_size: bytes[1],
            flags6: bytes[2],
            flags7: bytes[3],
            flags8: bytes[4],
            flags9: bytes[5],
            flags10: bytes[6],
            flags_chr_ram_size: bytes[7],
            flags_cpu_ppu_timing: bytes[8],
            flags_vs_system: bytes[9],
            flags_misc_roms: bytes[10],
            flags_default_expansion_device: bytes[11],
        }
    }

    fn is_nes20_format(&self) -> bool {
        self.flags7 & 0x0c == 0x08
    }

    pub fn mapper_type(&self) -> u16 {
        // TODO: nes2.0 headers...
        let m0 = (self.flags6 & 0xf0) >> 4;
        if !self.is_nes20_format() {
            m0 as u16
        } else {
            let m1 = (self.flags7 & 0xf0) >> 4;
            let m2 = (self.flags8 & 0x0f);
            ((m2 as u16) << 8) | ((m1 as u16) << 4) | m0 as u16
        }
    }
}
