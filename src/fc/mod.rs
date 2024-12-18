use mem::FCMem;

use crate::fc::cpu::*;

pub mod cpu;
pub mod mem;
pub mod dbg;

pub struct FC {
    cpu: CPU
}

impl FC {
    pub fn new() -> FC {
        FC { cpu: CPU::new() }
    }

    pub fn from_file(filename: &str) -> FC {
        let mut cpu = CPU::new();
        cpu.mem = FCMem::from_file(filename)
            .expect(&format!("File not found: {filename}"));
        FC { cpu }
    }

    pub fn step(&mut self) -> () {
        self.cpu.fetch_and_run();
    }

    pub fn step_dbg(&mut self) -> () {
        self.cpu.fetch_and_run_dbg();
    }
}

pub fn tester() {
    let mut fc = FC::from_file("test.bin");

    /*
    // fc.cpu.mem.write(0x8000, 0xa9);    // lda #$99
    // fc.cpu.mem.write(0x8001, 0x99);
    fc.step_dbg();
    fc.cpu.print_state();
    // fc.cpu.mem.write(0x8002, 0xea);    // nop
    fc.step_dbg();
    fc.cpu.print_state();
    // fc.cpu.mem.write(0x8003, 0x85);    // sta $23
    // fc.cpu.mem.write(0x8004, 0x23);
    fc.step_dbg();
    fc.cpu.print_state();
    println!("$0023: {:02x}", fc.cpu.mem.read(0x0023));
    // fc.cpu.mem.write(0x8005, 0x95);    // sta $24,x
    // fc.cpu.mem.write(0x8006, 0x24);
    fc.step_dbg();
    fc.cpu.print_state();
    println!("$0024: {:02x}", fc.cpu.mem.read(0x0024));
    fc.step_dbg();
    */
    for i in 1..=51 {
        print!("!!!iter: {}", i);
        fc.cpu.print_state();
        fc.step_dbg();
    }
}
