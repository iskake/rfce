use mem::MemMap;

use crate::fc::cpu::*;
use crate::fc::mem::cart::*;
use crate::fc::ppu::*;

pub mod cpu;
pub mod dbg;
pub mod mem;
pub mod ppu;

pub enum ConsoleType {
    Famicom,
    VSSystem,
    PlayChoice,
    Other,
}

pub struct FC {
    cpu: CPU,
    // ppu: PPU,    // ? move PPU here instead of storing in CPU??
}

impl FC {
    pub fn new() -> FC {
        FC { cpu: CPU::new(MemMap::empty(), PPU::new()) }
    }

    pub fn from_file(filename: &str) -> FC {
        let nesfile = NESFile::from_file(filename).expect(&format!("File not found: {filename}"));
        let mem = MemMap::from_nesfile(&nesfile);

        let ppu = PPU::new();
        let cpu = CPU::new(mem, ppu);
        FC { cpu }
    }

    fn load_rom(&mut self, filename: &str) -> () {
        let nesfile = NESFile::from_file(filename).expect(&format!("File not found: {filename}"));
        let mem = MemMap::from_nesfile(&nesfile);

        let ppu = PPU::new();
        let cpu = CPU::new(mem, ppu);
        // self.ppu = ppu;
        self.cpu = cpu;
    }

    fn init(&mut self) -> () {
        // TODO: all the other initialization things.
        self.cpu.init();
    }

    pub fn step(&mut self) -> () {
        self.cpu.fetch_and_run();
    }

    pub fn step_dbg(&mut self) -> () {
        self.cpu.fetch_and_run_dbg();
    }
}
