use cart::NESFile;
use log::warn;
use mapper::{Mapper, NROMMapper};

use crate::fc::input::Controller;

pub mod cart;
pub mod mapper;

const MAPPER_START_ADDRESS: usize = 0x4020;
const MAPPER_SPACE: usize = 0x10000 - MAPPER_START_ADDRESS;

pub trait Memory {
    fn read(&mut self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8) -> ();
}

// "Dummy Mapper", used as last resort if mapper does not exist.
type DummyMapper = [u8; MAPPER_SPACE];

impl Memory for DummyMapper {
    fn read(&mut self, addr: u16) -> u8 {
        self.read_no_sideeffect(addr)
    }

    fn write(&mut self, _addr: u16, _val: u8) -> () {
        // self[(addr as usize) - 0x4020] = val;
    }
}

impl Mapper for DummyMapper {
    fn read_chr(&self, _addr: u16) -> u8 {
        0xff
    }

    fn write_chr(&mut self, _addr: u16, _val: u8) -> () {
        ()
    }

    fn nametable_read(&self, _addr: u16, _vram: [u8; super::ppu::VRAM_SIZE]) -> u8 {
        0xff
    }

    fn nametable_write(&mut self, _addr: u16, _val: u8, _vram: &mut [u8; super::ppu::VRAM_SIZE]) -> () {
        ()
    }

    fn read_no_sideeffect(&self, addr: u16) -> u8 {
        self[(addr as usize) - MAPPER_START_ADDRESS]
    }
}

pub struct MemMap {
    ram: [u8; 0x800],
    // ...  // TODO: apu, ...
    pub input: Controller,
    pub mapper: Box<dyn Mapper>,
}

impl Memory for MemMap {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x07ff => self.ram[addr as usize],
            0x0800..=0x1fff => self.ram[(addr % 0x800) as usize],
            0x4000..=0x4015 => 0xff, // TODO: apu registers
            0x4016          => self.input.read_joy1(), // Joystick 1 data
            0x4017          => self.input.read_joy2(), // Joystick 2 data
            0x4018..=0x401f => 0xff, // APU test mode & unused IRQ timer
            0x4020..=0xffff => self.mapper.read(addr),
            _ => unreachable!("Attempted to read PPU MMIO (address ${:04x})", addr),
        }
    }

    fn write(&mut self, addr: u16, val: u8) -> () {
        match addr {
            0x0000..=0x07ff => self.ram[addr as usize] = val,
            0x0800..=0x1fff => self.ram[(addr % 0x800) as usize] = val,
            0x4000..=0x4015 => (), // TODO: apu registers
            0x4016          => self.input.write(val), // Joystick strobe
            0x4017          => (), // TODO: apu frame counter control
            0x4018..=0x401f => (), // APU test mode & unused IRQ timer
            0x4020..=0xffff => self.mapper.write(addr, val),
            _ => unreachable!("Attempted to read PPU MMIO (address ${:04x})", addr),
        };
    }
}

impl MemMap {
    pub fn empty() -> MemMap {
        MemMap {
            ram: [0; 0x800],
            input: Controller::new(),
            mapper: Box::new([0; MAPPER_SPACE]),
        }
    }

    pub fn from_nesfile(nesfile: &NESFile) -> Result<MemMap, std::io::Error> {
        match nesfile.mapper_type() {
            mapper::MapperType::NROM => {
                let mapper = Box::new(NROMMapper::from_nesfile(nesfile));
                let mem_map = MemMap {
                    ram: [0; 0x800],
                    input: Controller::new(),
                    mapper,
                };
                Ok(mem_map)
            }
            mapper::MapperType::UNKNOWN(i) => {
                warn!("WARNING: UNKNOWN MAPPER ({i})");
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Unsupported mapper ({:03})", i)
                ))
            }
        }
    }

    pub(crate) fn read_no_sideeffect(&self, addr: u16, ) -> u8 {
        match addr {
            0x0000..=0x07ff => self.ram[addr as usize],
            0x0800..=0x1fff => self.ram[(addr % 0x800) as usize],
            0x4000..=0x4015 => 0xff, // TODO: apu registers
            0x4016          => self.input.read_joy1_no_sideeffect(),
            0x4017          => self.input.read_joy2_no_sideeffect(),
            0x4018..=0x401f => 0xff, // APU test mode & unused IRQ timer
            0x4020..=0xffff => self.mapper.read_no_sideeffect(addr),
            _ => unreachable!("Attempted to read PPU MMIO (address ${:04x})", addr),
        }
    }
}
