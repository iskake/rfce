use std::io::Error;

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
    cart: Option<NESFile>,
}

impl FC {
    pub fn new() -> FC {
        FC {
            cpu: CPU::new(MemMap::empty(), PPU::new()),
            cart: None
        }
    }

    pub fn from_file(filename: &str) -> Result<FC, Error> {
        let nesfile = NESFile::from_file(filename)?;
        let mem = MemMap::from_nesfile(&nesfile);

        let ppu = PPU::new();
        let cpu = CPU::new(mem, ppu);
        Ok(FC { cpu, cart: Some(nesfile) })
    }

    /// Reads and loads the specified ROM, including initialization.
    fn load_rom(&mut self, filename: &str) -> Result<(), Error> {
        self.cart = Some(NESFile::from_file(filename)?);

        self.reset_hard()
    }

    /// "Hard reset" / power cycle the emulator.
    /// This is equivalent to loading the already loaded ROM from a file again.
    fn reset_hard(&mut self) -> Result<(), Error> {
        match &self.cart {
            None => Err(Error::new(std::io::ErrorKind::NotFound, "no ROM loaded")),
            Some(nesfile) => {
                let mem = MemMap::from_nesfile(&nesfile);

                let ppu = PPU::new();
                let cpu = CPU::new(mem, ppu);
                // self.ppu = ppu;
                self.cpu = cpu;
                self.init();
                Ok(())
            }
        }
    }

    /// "Soft reset" the emulator.
    fn reset(&mut self) -> () {
        self.cpu.reset();
    }

    fn init(&mut self) -> () {
        // TODO: all the other initialization things.
        self.cpu.init();
    }

    pub fn run_to_vblank(&mut self) -> () {
        self.cpu.run_to_vblank();
    }

    pub fn step(&mut self) -> () {
        self.cpu.fetch_and_run();
    }

    pub fn step_dbg(&mut self) -> () {
        self.cpu.fetch_and_run_dbg();
    }
}
