use inst::*;

use crate::bits::{as_address, Bitwise};
use crate::fc::mem::*;

pub mod inst;

// TODO: change visibility of various enums (from pub to private)??

pub struct Registers {
    pub a: u8,    // Accumulator
    pub x: u8,    // X index register
    pub y: u8,    // Y index register
    pub p: u8,    // Processor status
    pub sp: u8,   // Stack pointer
    pub pc: u16,  // Program counter
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            p: 0x00,
            sp: 0x00,
            pc: 0x8000,
        }
    }
}

pub struct CPU {
    pub reg: Registers,
    pub mem: Memory_, // TODO?: move this somewhere else?
    pub cycles: u64,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            reg: Registers::new(),
            mem: [0; 0x10000],
            cycles: 0,
        }
    }

    pub fn print_state(&self) -> () {
        println!("CPU STATE:");
        println!("  a: {:02x}, x: {:02x}, y: {:02x}", self.reg.a, self.reg.x, self.reg.y);
        println!("  p: {:08b}", self.reg.p);
        println!("  sp:{:02x} pc:{:04x}", self.reg.sp, self.reg.pc);
    }

    pub fn cycle(&mut self) -> () {
        self.cycles += 1;
    }

    pub fn pc_read(&mut self) -> u8 {
        self.mem.read(self.reg.pc)
    }

    pub fn pc_inc(&mut self) -> () {
        self.reg.pc += 1;
        self.cycle();   // TODO CHECK!!!!!!
                        // https://www.nesdev.org/wiki/Cycle_counting#Instruction_timings
    }

    pub fn pc_read_inc(&mut self) -> u8 {
        let val = self.pc_read();
        self.pc_inc();  // +1 cycle
        val
    }

    pub fn zp_read(&mut self, ir: IndexRegister) -> u8 {
        let pcval = self.pc_read();
        let delta = match ir {
            IndexRegister::None => 0,
            IndexRegister::X => self.reg.x,
            IndexRegister::Y => self.reg.y,
        };
        self.mem.read(as_address(pcval + delta, 0x00))
    }

    pub fn zp_read_inc(&mut self, ir: IndexRegister) -> u8 {
        let val = self.zp_read(ir);
        self.pc_inc();  // +1 cycle
        val
    }

    pub fn abs_read(&mut self, ir: IndexRegister) -> u8 {
        let l = self.pc_read();
        let m = self.pc_read();
        let delta = match ir {
            IndexRegister::None => 0,
            IndexRegister::X => self.reg.x,
            IndexRegister::Y => self.reg.y,
        } as u16; // !!!TODO!!! check the page crossing thing pls
        self.mem.read(as_address(l,m) + delta)
    }

    // TODO? might be a good idea to move this somewhere else?
    pub fn lda(&mut self, am: AddrMode) -> () {
        // todo: everything lol
        let val = match am {
            AddrMode::Imm => self.pc_read_inc(),
            AddrMode::ZP(ir) => self.zp_read_inc(ir),
            _ => unreachable!(),
        };
        let z = val == 0;
        let n = val.test_bit(7);
    
        self.reg.a = val;
        self.reg.p |= ((n as u8) << 7) | (z as u8) << 1;
        self.cycles += 2;
    }
}