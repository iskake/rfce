use std::{
    fs::File,
    io::{self, Read},
};

const NES_FILE_IDENTIFIER: [u8; 4] = [b'N', b'E', b'S', 0x1a];

pub struct NESFile {
    pub header: NESFileHeader,
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
            panic!("oh no")
        }
    }
}

pub struct NESFileHeader {
    prg_rom_size: u8,
    chr_rom_size: u8,
    flags6: u8,
    flags7: u8,
    flags8: u8,
    flags9: u8,
    flags10: u8,
    // NES 2.0:
    flags_chr_ram_size: u8,
    flags_cpu_ppu_timing: u8,
    flags_vs_system: u8,
    flags_misc_roms: u8,
    flags_default_expansion_device: u8,
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
}
