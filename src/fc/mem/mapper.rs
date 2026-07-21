pub mod nrom;
pub mod mmc1;
pub mod mmc3;

use crate::fc::{mem::cart::NESFile, ppu};

use super::Memory;

#[derive(PartialEq, Debug)]
pub enum MapperType {
    NROM,
    MMC1,
    MMC2,
    MMC3,
    MMC4,
    MMC5,
    MMC6,
    UNKNOWN(u16),
}

pub trait Mapper : Memory {
    fn read_chr(&self, addr: u16) -> u8;
    fn write_chr(&mut self, addr: u16, val: u8) -> ();
    fn nametable_read(&self, addr: u16, vram: [u8; ppu::VRAM_SIZE]) -> u8;
    fn nametable_write(&mut self, addr: u16, val: u8, vram: &mut [u8; ppu::VRAM_SIZE]) -> ();
    fn read_no_sideeffect(&self, addr: u16) -> u8;
}

// We can only create a mapper from a nes file if the mapper is actually "real".
pub trait RealMapper : Mapper {
    fn from_nesfile(nesfile: &NESFile) -> Self;
}
