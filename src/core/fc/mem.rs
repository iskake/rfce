use std::fs::File;
use std::io::{Read, Write};

use cart::NESFile;
use log::{info, warn};
use mapper::nrom::NROMMapper;
use mapper::uxrom::UxROMMapper;
use mapper::cnrom::CNROMMapper;
use mapper::mmc1::MMC1Mapper;
use mapper::mmc2x::MMC2Mapper;
use mapper::mmc3::MMC3Mapper;
use mapper::{Mapper, RealMapper};

use crate::core::fc::input::Controller;

pub mod cart;
pub mod mapper;

const MAPPER_START_ADDRESS: usize = 0x4020;
const MAPPER_SPACE: usize = 0x10000 - MAPPER_START_ADDRESS;

#[derive(Debug, Clone, Copy)]
pub(crate) enum NametableArrangement {
    HorizontalMirroring,
    VerticalMirroring,
    SingleScreenA,
    SingleScreenB,
    FourScreen,
}

impl NametableArrangement {
    fn nametable_addr_fix(self: NametableArrangement, addr: u16) -> u16 {
        let a = addr - 0x2000;
        match self {
            NametableArrangement::HorizontalMirroring => ((a & 0x800) >> 1) | (a & 0x3ff),
            NametableArrangement::VerticalMirroring => a & 0x7ff,
            NametableArrangement::SingleScreenA => a & 0x3ff,
            NametableArrangement::SingleScreenB => (a & 0x3ff) + 0x400,
            NametableArrangement::FourScreen => a & 0x2fff,
        }
    }
}

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
    fn read_chr(&mut self, _addr: u16) -> u8 {
        0xff
    }

    fn read_chr_no_sideeffect(& self,_addr: u16) -> u8 {
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

    fn battery(&self) -> bool {
        false
    }
}

pub enum MapperImpl {
    DUMMY(DummyMapper),
    NROM(NROMMapper),
    UxROM(UxROMMapper),
    CNROM(CNROMMapper),
    MMC1(MMC1Mapper),
    MMC2(MMC2Mapper),
    MMC3(MMC3Mapper),
}

impl Mapper for MapperImpl {
    fn read_no_sideeffect(&self, addr: u16) -> u8 {
        match self {
            MapperImpl::DUMMY(m) => m.read_no_sideeffect(addr),
            MapperImpl::NROM(m)   => m.read_no_sideeffect(addr),
            MapperImpl::UxROM(m) => m.read_no_sideeffect(addr),
            MapperImpl::CNROM(m) => m.read_no_sideeffect(addr),
            MapperImpl::MMC1(m)   => m.read_no_sideeffect(addr),
            MapperImpl::MMC2(m)   => m.read_no_sideeffect(addr),
            MapperImpl::MMC3(m)   => m.read_no_sideeffect(addr),
        }
    }

    fn read_chr(&mut self, addr: u16) -> u8 {
        match self {
            MapperImpl::DUMMY(m) => m.read_chr(addr),
            MapperImpl::NROM(m)   => m.read_chr(addr),
            MapperImpl::UxROM(m) => m.read_chr(addr),
            MapperImpl::CNROM(m) => m.read_chr(addr),
            MapperImpl::MMC1(m)   => m.read_chr(addr),
            MapperImpl::MMC2(m)   => m.read_chr(addr),
            MapperImpl::MMC3(m)   => m.read_chr(addr),
        }
    }

    fn read_chr_no_sideeffect(&self, addr: u16) -> u8 {
        match self {
            MapperImpl::DUMMY(m) => m.read_chr_no_sideeffect(addr),
            MapperImpl::NROM(m)   => m.read_chr_no_sideeffect(addr),
            MapperImpl::UxROM(m) => m.read_chr_no_sideeffect(addr),
            MapperImpl::CNROM(m) => m.read_chr_no_sideeffect(addr),
            MapperImpl::MMC1(m)   => m.read_chr_no_sideeffect(addr),
            MapperImpl::MMC2(m)   => m.read_chr_no_sideeffect(addr),
            MapperImpl::MMC3(m)   => m.read_chr_no_sideeffect(addr),
        }
    }

    fn write_chr(&mut self, addr: u16, val: u8) -> () {
        match self {
            MapperImpl::DUMMY(m) => m.write_chr(addr, val),
            MapperImpl::NROM(m)   => m.write_chr(addr, val),
            MapperImpl::UxROM(m) => m.write_chr(addr, val),
            MapperImpl::CNROM(m) => m.write_chr(addr, val),
            MapperImpl::MMC1(m)   => m.write_chr(addr, val),
            MapperImpl::MMC2(m)   => m.write_chr(addr, val),
            MapperImpl::MMC3(m)   => m.write_chr(addr, val),
        }
    }

    fn nametable_read(&self, addr: u16, vram: [u8; super::ppu::VRAM_SIZE]) -> u8 {
        match self {
            MapperImpl::DUMMY(m) => m.nametable_read(addr, vram),
            MapperImpl::NROM(m)   => m.nametable_read(addr, vram),
            MapperImpl::UxROM(m) => m.nametable_read(addr, vram),
            MapperImpl::CNROM(m) => m.nametable_read(addr, vram),
            MapperImpl::MMC1(m)   => m.nametable_read(addr, vram),
            MapperImpl::MMC2(m)   => m.nametable_read(addr, vram),
            MapperImpl::MMC3(m)   => m.nametable_read(addr, vram),
        }
    }

    fn nametable_write(&mut self, addr: u16, val: u8, vram: &mut [u8; super::ppu::VRAM_SIZE]) -> () {
        match self {
            MapperImpl::DUMMY(m) => m.nametable_write(addr, val, vram),
            MapperImpl::NROM(m)   => m.nametable_write(addr, val, vram),
            MapperImpl::UxROM(m) => m.nametable_write(addr, val, vram),
            MapperImpl::CNROM(m) => m.nametable_write(addr, val, vram),
            MapperImpl::MMC1(m)   => m.nametable_write(addr, val, vram),
            MapperImpl::MMC2(m)   => m.nametable_write(addr, val, vram),
            MapperImpl::MMC3(m)   => m.nametable_write(addr, val, vram),
        }
    }

    fn battery(&self) -> bool {
        match self {
            MapperImpl::DUMMY(m) => m.battery(),
            MapperImpl::NROM(m)   => m.battery(),
            MapperImpl::UxROM(m) => m.battery(),
            MapperImpl::CNROM(m) => m.battery(),
            MapperImpl::MMC1(m)   => m.battery(),
            MapperImpl::MMC2(m)   => m.battery(),
            MapperImpl::MMC3(m)   => m.battery(),
        }
    }
}

impl Memory for MapperImpl {
    fn read(&mut self, addr: u16) -> u8 {
        match self {
            MapperImpl::DUMMY(m) => m.read(addr),
            MapperImpl::NROM(m)   => m.read(addr),
            MapperImpl::UxROM(m) => m.read(addr),
            MapperImpl::CNROM(m) => m.read(addr),
            MapperImpl::MMC1(m)   => m.read(addr),
            MapperImpl::MMC2(m)   => m.read(addr),
            MapperImpl::MMC3(m)   => m.read(addr),
        }
    }

    fn write(&mut self, addr: u16, val: u8) -> () {
        match self {
            MapperImpl::DUMMY(m) => m.write(addr, val),
            MapperImpl::NROM(m)   => m.write(addr, val),
            MapperImpl::UxROM(m) => m.write(addr, val),
            MapperImpl::CNROM(m) => m.write(addr, val),
            MapperImpl::MMC1(m)   => m.write(addr, val),
            MapperImpl::MMC2(m)   => m.write(addr, val),
            MapperImpl::MMC3(m)   => m.write(addr, val),
        }
    }
}

pub struct MemMap {
    ram: [u8; 0x800],
    pub input: Controller,
    pub mapper: Box<MapperImpl>,
}

impl Memory for MemMap {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x07ff => self.ram[addr as usize],
            0x0800..=0x1fff => self.ram[(addr & 0x7ff) as usize],
            0x4016          => self.input.read_joy1(), // Joystick 1 data
            0x4017          => self.input.read_joy2(), // Joystick 2 data
            0x4020..=0xffff => self.mapper.read(addr),
            _ => unreachable!("Attempted to read PPU MMIO (address ${:04x})", addr),
        }
    }

    fn write(&mut self, addr: u16, val: u8) -> () {
        match addr {
            0x0000..=0x07ff => self.ram[addr as usize] = val,
            0x0800..=0x1fff => self.ram[(addr & 0x7ff) as usize] = val,
            0x4016          => self.input.write(val), // Joystick strobe
            0x4020..=0xffff => self.mapper.write(addr, val),
            _ => unreachable!("Attempted to read PPU MMIO (address ${:04x})", addr),
        };
    }
}

macro_rules! create_mapper {
    ($mapper_type:ident, $mapper_type_mapper:ident, $nesfile:expr) => {{
        let mapper = Box::new(MapperImpl::$mapper_type($mapper_type_mapper::from_nesfile($nesfile)));
        let mem_map = MemMap {
            ram: [0; 0x800],
            input: Controller::new(),
            mapper,
        };
        Ok(mem_map)
    }};
}

macro_rules! unsupported_mapper {
    ($mapper_type:expr) => {{
        warn!("WARNING: mapper not implemented ({})", $mapper_type);
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Unsupported mapper ({})", $mapper_type),
        ))
    }};
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
            mapper,
        }
    }

    pub fn from_nesfile(nesfile: &NESFile) -> Result<MemMap, std::io::Error> {
        match nesfile.mapper_type() {
            mapper::MapperType::NROM => create_mapper!(NROM, NROMMapper, nesfile),
            mapper::MapperType::UxROM=> create_mapper!(UxROM, UxROMMapper, nesfile),
            mapper::MapperType::CNROM=> create_mapper!(CNROM, CNROMMapper, nesfile),
            mapper::MapperType::MMC1 => create_mapper!(MMC1, MMC1Mapper, nesfile),
            mapper::MapperType::MMC2 => create_mapper!(MMC2, MMC2Mapper, nesfile),
            mapper::MapperType::MMC3 => create_mapper!(MMC3, MMC3Mapper, nesfile),
            mapper::MapperType::MMC4 => create_mapper!(MMC2, MMC2Mapper, nesfile),
            mapper::MapperType::MMC5 => unsupported_mapper!("MMC5"),
            mapper::MapperType::MMC6 => unsupported_mapper!("MMC6"),
            mapper::MapperType::FDS  => unsupported_mapper!("DISK SYSTEM"), // TODO: fds uses the .fds file format instead of .nes
            mapper::MapperType::UNKNOWN(i) => unsupported_mapper!(format!("{i:03}")),
        }
    }

    pub(super) fn print_state(&self) -> () {
        match self.mapper.as_ref() {
            MapperImpl::MMC3(m) => m.print_state(),
            _ => {}
        }
    }

    pub(crate) fn read_no_sideeffect(&self, addr: u16) -> u8 {
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

    pub(crate) fn dec_irq_counter(&mut self) {
        match self.mapper.as_mut() {
            MapperImpl::MMC3(m) => m.dec_irq_counter(),
            _ => {}
        }
    }

    pub(crate) fn irq_triggered(&mut self) -> bool {
        // Sources of IRQ:
        // - APU DMC finish
        // - APU frame counter
        // - MMC3
        // - MMC5
        // - FDS
        // - (other mappers)

        // TODO
        let apu_dmc = false;

        // TODO
        let apu_frame_counter = false;

        let mapper = match self.mapper.as_ref() {
            MapperImpl::MMC3(m) => m.irq_triggered(),
            // MMC5
            // MMC6
            // FDS
            _ => false,
        };

        apu_dmc || apu_frame_counter || mapper
    }

    pub(crate) fn irq_un_trigger(&mut self) -> () {
        match self.mapper.as_mut() {
            MapperImpl::MMC3(m) => m.irq_un_trigger(),
            // MMC5
            // MMC6
            // FDS
            _ => {}
        };
    }

    pub(crate) fn has_battery(&self) -> bool {
        match self.mapper.as_ref() {
            MapperImpl::NROM(m) => m.battery(),
            MapperImpl::MMC1(m) => m.battery(),
            MapperImpl::MMC2(m) => m.battery(),
            MapperImpl::MMC3(m) => m.battery(),
            _ => false,
        }
    }

    pub(crate) fn read_sram_from_file(&mut self, save_path: &std::path::Path) -> Result<(), std::io::Error> {
        if self.has_battery() {
            let mut buf = Vec::new();
            let mut file = File::open(save_path)?;
            file.read_to_end(&mut buf)?;

            let prg_ram = match self.mapper.as_mut() {
                MapperImpl::NROM(m) => m.sram_mut(),
                MapperImpl::MMC1(m) => m.sram_mut(),
                MapperImpl::MMC3(m) => m.sram_mut(),
                // MMC5
                // MMC6
                // ? FDS ?
                _ => unreachable!(),
            };

            if prg_ram.len() != buf.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "Size of save RAM is incorrect, expected {} got {}",
                        prg_ram.len(),
                        buf.len()
                    )
                ));
            }

            *prg_ram = buf;
            return Ok(());
        }

        Ok(())
    }

    pub(crate) fn write_sram_to_file(&self, save_path: &std::path::Path) -> Result<(), std::io::Error>{
        let sram = match self.mapper.as_ref() {
            MapperImpl::NROM(m) => Some(m.sram()),
            MapperImpl::MMC1(m) => Some(m.sram()),
            // MapperImpl::MMC2(m) => Some(m.sram()),
            MapperImpl::MMC3(m) => Some(m.sram()),
            // MMC5
            // MMC6
            // FDS
            _ => None,
        };

        if let Some(sram) = sram {
            let mut file = File::create(save_path)?;
            file.write_all(sram)?;
            info!("Wrote save RAM to file: {save_path:?}");
        }
        Ok(())
    }

    pub(crate) fn set_open_bus(&mut self, val: u8) {
        self.input.open_bus = val;
        match self.mapper.as_mut() {
            MapperImpl::NROM(m) => m.open_bus = val,
            MapperImpl::MMC1(m) => m.open_bus = val,
            MapperImpl::MMC2(m) => m.open_bus = val,
            MapperImpl::MMC3(m) => m.open_bus = val,
            _ => (),
        }
    }
}
