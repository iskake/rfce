use mem::MemMap;

use crate::fc::cpu::*;
use crate::fc::mem::cart::*;

pub mod cpu;
pub mod dbg;
pub mod mem;

pub struct FC {
    cpu: CPU,
}

impl FC {
    pub fn new() -> FC {
        FC { cpu: CPU::new() }
    }

    pub fn from_file(filename: &str) -> FC {
        let mut cpu = CPU::new();
        let nesfile = NESFile::from_file(filename).expect(&format!("File not found: {filename}"));
        cpu.mem = MemMap::from_nesfile(nesfile);
        FC { cpu }
    }

    fn load_rom(&mut self, filename: &str) -> () {
        // TODO: change this?
        let mut cpu = CPU::new();
        let nesfile = NESFile::from_file(filename).expect(&format!("File not found: {filename}"));
        cpu.mem = MemMap::from_nesfile(nesfile);
        self.cpu = cpu;
    }

    fn init(&mut self) -> () {
        self.cpu.init();
    }

    pub fn step(&mut self) -> () {
        self.cpu.fetch_and_run();
    }

    pub fn step_dbg(&mut self) -> () {
        self.cpu.fetch_and_run_dbg();
    }
}
