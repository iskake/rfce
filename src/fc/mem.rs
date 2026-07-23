use cart::NESFile;
use log::warn;
use mapper::{Mapper, RealMapper};
use mapper::nrom::NROMMapper;

use crate::fc::input::Controller;
use crate::fc::mem::mapper::mmc1::MMC1Mapper;

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

pub enum MapperImpl {
    DUMMY(DummyMapper),
    NROM(NROMMapper),
    MMC1(MMC1Mapper),
}

impl Mapper for MapperImpl {
    fn read_no_sideeffect(&self, addr: u16) -> u8 {
        match self {
            MapperImpl::DUMMY(m) => m.read_no_sideeffect(addr),
            MapperImpl::NROM(m)   => m.read_no_sideeffect(addr),
            MapperImpl::MMC1(m)   => m.read_no_sideeffect(addr),
        }
    }

    fn read_chr(&self, addr: u16) -> u8 {
        match self {
            MapperImpl::DUMMY(m) => m.read_chr(addr),
            MapperImpl::NROM(m)   => m.read_chr(addr),
            MapperImpl::MMC1(m)   => m.read_chr(addr),
        }
    }

    fn write_chr(&mut self, addr: u16, val: u8) -> () {
        match self {
            MapperImpl::DUMMY(m) => m.write_chr(addr, val),
            MapperImpl::NROM(m)   => m.write_chr(addr, val),
            MapperImpl::MMC1(m)   => m.write_chr(addr, val),
        }
    }

    fn nametable_read(&self, addr: u16, vram: [u8; super::ppu::VRAM_SIZE]) -> u8 {
        match self {
            MapperImpl::DUMMY(m) => m.nametable_read(addr, vram),
            MapperImpl::NROM(m) => m.nametable_read(addr, vram),
            MapperImpl::MMC1(m) => m.nametable_read(addr, vram),
        }
    }

    fn nametable_write(&mut self, addr: u16, val: u8, vram: &mut [u8; super::ppu::VRAM_SIZE]) -> () {
        match self {
            MapperImpl::DUMMY(m) => m.nametable_write(addr, val, vram),
            MapperImpl::NROM(m) => m.nametable_write(addr, val, vram),
            MapperImpl::MMC1(m) => m.nametable_write(addr, val, vram),
        }
    }
}

impl Memory for MapperImpl {
    fn read(&mut self, addr: u16) -> u8 {
        match self {
            MapperImpl::DUMMY(m) => m.read(addr),
            MapperImpl::NROM(m)   => m.read(addr),
            MapperImpl::MMC1(m)   => m.read(addr),
        }
    }

    fn write(&mut self, addr: u16, val: u8) -> () {
        match self {
            MapperImpl::DUMMY(m) => m.write(addr, val),
            MapperImpl::NROM(m)   => m.write(addr, val),
            MapperImpl::MMC1(m)   => m.write(addr, val),
        }
    }
}

pub struct MemMap {
    ram: [u8; 0x800],
    // ...  // TODO: apu, ...
    pub input: Controller,
    pub mapper: Box<MapperImpl>,
}

impl Memory for MemMap {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x07ff => self.ram[addr as usize],
            0x0800..=0x1fff => self.ram[(addr & 0x7ff) as usize],
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
            0x0800..=0x1fff => self.ram[(addr & 0x7ff) as usize] = val,
            0x4000..=0x4015 => (), // TODO: apu registers
            0x4016          => self.input.write(val), // Joystick strobe
            0x4017          => (), // TODO: apu frame counter control
            0x4018..=0x401f => (), // APU test mode & unused IRQ timer
            0x4020..=0xffff => self.mapper.write(addr, val),
            _ => unreachable!("Attempted to read PPU MMIO (address ${:04x})", addr),
        };
    }
}

macro_rules! create_mapper {
    ($mapper_type:ident, $mapper_type_mapper:ident, $nesfile:expr) => {
        {
            let mapper = Box::new(MapperImpl::$mapper_type($mapper_type_mapper::from_nesfile($nesfile)));
            let mem_map = MemMap {
                ram: [0; 0x800],
                input: Controller::new(),
                mapper,
            };
            Ok(mem_map)
        }
    };
}

macro_rules! unsupported_mapper {
    ($mapper_type:expr) => {
        {
            warn!("WARNING: mapper not implemented ({})", $mapper_type);
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Unsupported mapper ({})", $mapper_type)
            ))
        }
    };
}

impl MemMap {
    pub fn empty() -> MemMap {
        MemMap {
            ram: [0; 0x800],
            input: Controller::new(),
            mapper: Box::new(MapperImpl::DUMMY([0; MAPPER_SPACE])),
        }
    }

    pub fn from_mapper(mapper: Box<MapperImpl>) -> MemMap {
        MemMap {
            ram: [0; 0x800],
            input: Controller::new(),
            mapper
        }
    }

    pub fn from_nesfile(nesfile: &NESFile) -> Result<MemMap, std::io::Error> {
        match nesfile.mapper_type() {
            mapper::MapperType::NROM => create_mapper!(NROM, NROMMapper, nesfile),
            mapper::MapperType::MMC1 => create_mapper!(MMC1, MMC1Mapper, nesfile),
            mapper::MapperType::MMC2 => unsupported_mapper!("MMC6"),
            mapper::MapperType::MMC3 => unsupported_mapper!("MMC6"),
            mapper::MapperType::MMC4 => unsupported_mapper!("MMC6"),
            mapper::MapperType::MMC5 => unsupported_mapper!("MMC6"),
            mapper::MapperType::MMC6 => unsupported_mapper!("MMC6"),
            mapper::MapperType::UNKNOWN(i) => unsupported_mapper!(format!("{i:03}"))
        }
    }

    pub(crate) fn read_no_sideeffect(&self, addr: u16, ) -> u8 {
        match addr {
            0x0000..=0x07ff => self.ram[addr as usize],
            0x0800..=0x1fff => self.ram[(addr & 0x7ff) as usize],
            0x4000..=0x4015 => 0xff, // TODO: apu registers
            0x4016          => self.input.read_joy1_no_sideeffect(),
            0x4017          => self.input.read_joy2_no_sideeffect(),
            0x4018..=0x401f => 0xff, // APU test mode & unused IRQ timer
            0x4020..=0xffff => self.mapper.read_no_sideeffect(addr),
            _ => unreachable!("Attempted to read PPU MMIO (address ${:04x})", addr),
        }
    }
}
