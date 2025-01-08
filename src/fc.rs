use std::io::Error;
use std::path::Path;

use mem::MemMap;

use crate::fc::cpu::*;
use crate::fc::mem::cart::*;
use crate::fc::ppu::*;

pub mod cpu;
pub mod ppu;
pub mod mem;
pub mod dbg;

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

    pub fn from_file(filename: &Path) -> Result<Box<FC>, Error> {
        let nesfile = NESFile::from_file(filename)?;
        let mem = MemMap::from_nesfile(&nesfile)?;

        let ppu = PPU::new();
        let cpu = CPU::new(mem, ppu);
        Ok(Box::new(FC { cpu, cart: Some(nesfile) }))
    }

    /// Reads and loads the specified ROM, including initialization.
    pub fn load_rom(&mut self, filename: &Path) -> Result<(), Error> {
        self.cart = Some(NESFile::from_file(filename)?);

        self.reset_hard()
    }

    /// "Hard reset" / power cycle the emulator.
    /// This is equivalent to loading the already loaded ROM from a file again.
    pub fn reset_hard(&mut self) -> Result<(), Error> {
        match &self.cart {
            None => Err(Error::new(std::io::ErrorKind::NotFound, "no ROM loaded")),
            Some(nesfile) => {
                let mem = MemMap::from_nesfile(&nesfile)?;

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
    pub fn reset(&mut self) -> () {
        self.cpu.reset();
    }

    pub fn init(&mut self) -> () {
        // TODO: all the other initialization things.
        self.cpu.init();
    }

    pub fn run_until_render_done(&mut self) -> () {
        self.cpu.run_to_rendering_finished();
    }

    pub fn step(&mut self) -> () {
        self.cpu.fetch_and_run();
    }

    pub fn step_dbg(&mut self) -> () {
        self.cpu.fetch_and_run_dbg();
    }

    pub fn get_frame(&self) -> &[u8] {
        self.cpu.ppu.get_frame_buf()
    }

    pub fn get_nametables_dbg(&mut self) -> &[u8] {
        self.cpu.ppu.generate_nametables_image_temp(&self.cpu.mem)
    }
}
