use inst::*;

use crate::bits::{as_address, Bitwise};
use crate::fc::mem::*;

pub mod inst;

// TODO: change visibility of various enums (from pub to private)??

#[derive(Clone, Copy)]
pub struct ProcFlags {
    c: bool,
    z: bool,
    i: bool,
    d: bool,
    b: bool,
    v: bool,
    n: bool,
}

impl From<u8> for ProcFlags {
    fn from(val: u8) -> Self {
        ProcFlags {
            c: val.test_bit(0),
            z: val.test_bit(1),
            i: val.test_bit(2),
            d: val.test_bit(3),
            b: val.test_bit(4),
            v: val.test_bit(6),
            n: val.test_bit(7),
        }
    }
}

impl Into<u8> for ProcFlags {
    fn into(self) -> u8 {
        self.c as u8 |
        (self.z as u8) << 1 |
        (self.i as u8) << 2 |
        (self.d as u8) << 3 |
        (self.b as u8) << 4 |
        (self.v as u8) << 6 |
        (self.n as u8) << 7
    }
}

pub struct Registers {
    pub a: u8,    // Accumulator
    pub x: u8,    // X index register
    pub y: u8,    // Y index register
    pub p: ProcFlags,    // Processor status
    pub sp: u8,   // Stack pointer
    pub pc: u16,  // Program counter
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            p: 0x00.into(),
            sp: 0x00,
            pc: 0x8000,
        }
    }
}

pub struct CPU {
    reg: Registers,
    pub mem: FCMem, // TODO?: move this somewhere else and have a reference?
    cycles: u64,
}

impl CPU {
    pub fn new() -> CPU {
        CPU {
            reg: Registers::new(),
            mem: FCMem::empty(),
            cycles: 0,
        }
    }

    pub fn print_state(&self) -> () {
        println!("CPU STATE:");
        println!("  a: {:02x}, x: {:02x}, y: {:02x}", self.reg.a, self.reg.x, self.reg.y);
        let p: u8 = self.reg.p.into();
        println!("  p: {:08b}", p);
        println!("  sp:{:02x} pc:{:04x}", self.reg.sp, self.reg.pc);
        println!("  cycles: {}", self.cycles);
    }

    /// "Cycle" the cpu.
    /// 
    /// Cycles: `1`
    pub fn cycle(&mut self) -> () {
        self.cycles += 1;
    }

    /// Read the value at the address `addr` without any cycles
    /// 
    /// Cycles: `0`
    pub fn read_addr_nocycle(&mut self, addr: u16) -> u8 {
        self.mem.read(addr)
    }

    /// Read the value at the address `addr`
    /// 
    /// Cycles: `1`
    pub fn read_addr_cycle(&mut self, addr: u16) -> u8 {
        self.cycle();
        self.mem.read(addr)
    }

    /// Write the value `val` to the address `addr`
    /// 
    /// Cycles: `1`
    pub fn write_addr_cycle(&mut self, addr: u16, val: u8) -> () {
        self.cycle();
        self.mem.write(addr, val);
    }

    /// Read the value pointe to by the pc without any cycles.
    /// 
    /// Cycles: `0`
    pub fn pc_read_nocycle(&mut self) -> u8 {
        self.mem.read(self.reg.pc)
    }
    
    /// Read the value pointe to by the pc.
    /// 
    /// Cycles: `1`
    pub fn pc_read(&mut self) -> u8 {
        self.cycle();
        self.mem.read(self.reg.pc)
    }

    /// Increment the pc.
    /// 
    /// Cycles: `0`
    pub fn pc_inc(&mut self) -> () {
        self.reg.pc += 1;   // https://www.nesdev.org/wiki/Cycle_counting#Instruction_timings
    }

    /// Read the value pointed to by the pc, then increment the pc.
    /// 
    /// Cycles: `1`
    pub fn pc_read_inc(&mut self) -> u8 {
        let val = self.pc_read();   // +1 cycle
        self.pc_inc();
        val
    }



    /// Read the value at address $00(pc+x/y).
    /// 
    /// Cycles: `2` (if page crossed)
    pub fn zp_read_cycle(&mut self, ir: IndexRegister) -> u8 {
        let operand = self.pc_read();   // +1 cycle
        let delta = match ir {
            IndexRegister::N => 0,
            IndexRegister::X => self.reg.x, //TODO: have +1 cycle?
            IndexRegister::Y => self.reg.y, //TODO: have +1 cycle?
        };
        // if operand as u16 + delta as u16 > 0xff {
        //     self.cycle();
        // }
        self.read_addr_cycle(as_address(operand + delta, 0x00)) // +1 cycle
    }

    /// Read the value at address $00(pc+x/y), and increment the pc.
    /// 
    /// Cycles: `2` (`+1` if page crossed)
    pub fn zp_read_inc(&mut self, ir: IndexRegister) -> u8 {
        let operand = self.pc_read_inc();   // +1 cycle
        let delta = match ir {
            IndexRegister::N => 0,
            IndexRegister::X => self.reg.x,    // +1 cycle
            IndexRegister::Y => self.reg.y,    // +1 cycle
        };
        if operand as u16 + delta as u16 > 0xff {
            self.cycle();
        }
        self.read_addr_cycle(as_address(operand + delta, 0x00)) // +1 cycle
    }

    /// Write the value `val` to the address $00(pc+x/y), and increment the pc.
    /// 
    /// Cycles: `2` (`+1` if page crossed)
    pub fn zp_write_inc(&mut self, val: u8, ir: IndexRegister) -> () {
        let operand = self.pc_read_inc();  // +1 cycle
        let delta = match ir {
            IndexRegister::N => 0,
            IndexRegister::X => { self.cycle(); self.reg.x},// always +1 cycle
            IndexRegister::Y => { self.cycle(); self.reg.y},// always +1 cycle
        };
        let addr = as_address(operand + delta, 0x00);
        self.write_addr_cycle(addr, val);   // +1 cycle
    }


    /// Read the value at address $(pc)<<8|(pc+1) +x/y
    /// 
    /// Cycles: `2`
    pub fn abs_read_cycle(&mut self, ir: IndexRegister) -> u8 {
        let l = self.pc_read();     // +1 cycle
        // "m: u8 = self.pc_read(pc+1);"
        let m = {self.cycle(); self.mem.read(self.reg.pc+1)};   // +1 cycle

        let addr = as_address(l, m);
        let delta = match ir {
            IndexRegister::N => 0,
            IndexRegister::X => self.reg.x, //TODO: have +1 cycle?
            IndexRegister::Y => self.reg.y, //TODO: have +1 cycle?
        } as u16;
        // if (addr & 0xff) + delta > 0xff {
        //     self.cycle();
        // }
        // TODO? change this?
        self.read_addr_nocycle(addr + delta)  // +0 cycle
    }

    /// Read the value at address $(pc)<<8|(pc+1) +x/y, and increment the pc.
    /// 
    /// Cycles: `3` (`+1` if page crossed)
    pub fn abs_read_inc(&mut self, ir: IndexRegister) -> u8 {
        let l = self.pc_read_inc();     // +1 cycle
        let m = self.pc_read_inc();     // +1 cycle
        let addr = as_address(l, m);
        let delta = match ir {
            IndexRegister::N => 0,
            IndexRegister::X => self.reg.x, // +1 cycle
            IndexRegister::Y => self.reg.y, // +1 cycle
        } as u16;
        if (addr & 0xff) + delta > 0xff {
            self.cycle();
        }
        self.read_addr_cycle(addr + delta)  // +1 cycle
    }

    /// Write the value `val` to the address $(pc)<<8|(pc+1) +x/y, and increment the pc.
    /// 
    /// Cycles: `3` (`+1` if page crossed)
    pub fn abs_write_inc(&mut self, val: u8, ir: IndexRegister) -> () {
        let l = self.pc_read_inc(); // +1 cycle
        let m = self.pc_read_inc(); // +1 cycle
        let addr = as_address(l, m);
        let delta = match ir {
            IndexRegister::N => 0,
            IndexRegister::X => { self.cycle(); self.reg.x},
            IndexRegister::Y => { self.cycle(); self.reg.y},
        } as u16;
        // ?from wiki: "assumes the worst case of page crossing and always spends 1 extra read cycle"
        self.write_addr_cycle(addr + delta, val);   // +1 cycle
    }



    /// Get the indirect address ... yeah
    /// 
    /// Cycles: `2`
    pub fn get_indirect(&mut self, addr: u16) -> u16 {
        let l = self.read_addr_cycle(addr);     // +1 cycle
        let m = self.read_addr_cycle(addr+1);   // +1 cycle
        as_address(l, m)
    }
    
    /// Read the indirect address in relation to x or y
    /// 
    /// Cycles: `5` for x, `4` (`+1` if page crossed) for y
    pub fn ind_read_inc(&mut self, ir: IndexRegister) -> u8 {
        let pcval = self.pc_read_inc(); // +1 cycle
        match ir {
            IndexRegister::X => {
                let ptr = as_address(pcval + self.reg.x, 0x00);
                let addr = self.get_indirect(ptr as u16);     // +2 cycles
                self.cycle();               //?? +1 cycle extra??
                self.read_addr_cycle(addr)  // +1 cycle
            },
            IndexRegister::Y => {
                let ptr = as_address(pcval, 0x00);
                let delta = self.reg.y as u16;
                let addr = self.get_indirect(ptr + delta);     // +2 cycles
                if (ptr & 0xff) + self.reg.y as u16 > 0xff {
                    self.cycle();   // +1 cycle
                }
                self.read_addr_cycle(addr)  // +1 cycle
            },
            IndexRegister::N => unreachable!()
        }
    }

    /// Write `val` to the indirect address in relation to x or y
    /// 
    /// Cycles: `5` for x and y
    pub fn ind_write_inc(&mut self, val: u8, ir: IndexRegister) -> () {
        // TODO: this one seems a bit sketchy... idk...
        let pcval = self.pc_read_inc(); // +1 cycle
        match ir {
            IndexRegister::X => {
                let ptr = as_address(pcval + self.reg.x, 0x00);
                let addr = self.get_indirect(ptr as u16);     // +2 cycles
                self.cycle();                       // +1 cycle extra
                self.write_addr_cycle(addr, val)    // +1 cycle
            },
            IndexRegister::Y => {
                let ptr = as_address(pcval, 0x00);
                let delta = self.reg.y as u16;
                let addr = self.get_indirect(ptr + delta);     // +2 cycles
                self.cycle();                       // +1 cycle extra
                self.write_addr_cycle(addr, val)    // +1 cycle
            },
            IndexRegister::N => unreachable!()
        }
    }

    

    fn fetch_next_op(&mut self) -> u8 {
        self.pc_read_inc()  // 1 cycle
    }

    fn fetch_next_inst(&mut self) -> Inst {
        INST_TABLE[self.fetch_next_op() as usize] // 1 cycle
    }

    fn fetch_next_op_inst(&mut self) -> (u8, Inst) {
        let op = self.fetch_next_op();  // 1 cycle
        (op, INST_TABLE[op as usize])
    }

    fn run_inst(&mut self, inst: Inst) {
        inst.run(self);
    }

    pub fn fetch_and_run(&mut self) -> () {
        let cycles_before = self.cycles;
        let inst = self.fetch_next_inst();  // 1 cycle
        self.run_inst(inst);                      // n cycles
        let cycles_after = self.cycles;
        if cycles_after - cycles_before == 1 {
            self.cycle();
        }
    }

    pub fn fetch_and_run_dbg(&mut self) -> () {
        let cycles_before = self.cycles;
        let (op, inst) = self.fetch_next_op_inst();  // 1 cycle
        self.run_inst(inst);                                   // n cycles
        let cycles_after = self.cycles;
        print!(" inst {inst:?} (op: ${op:02x}) ");
        if cycles_after - cycles_before == 1 {
            println!("took 2 cycles (added one extra)");
            self.cycle();
        } else {
            println!("took {} cycles", cycles_after - cycles_before);
        }
    }
}