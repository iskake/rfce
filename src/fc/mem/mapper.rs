pub mod nrom;

use crate::fc::ppu;

use super::Memory;

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
    fn read_no_sideeffect(&self, addr: u16) -> u8;
}
