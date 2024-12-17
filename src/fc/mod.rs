use inst::{AddrMode, Inst};

use crate::fc::cpu::*;

pub mod cpu;
pub mod mem;

pub fn tester() {
    let mut cpu = CPU::new();

    cpu.print_state();
    cpu.reg.a = 2;
    cpu.print_state();

    let inst: Inst = Inst::LDA;
    match inst {
        Inst::LDA => cpu.lda(AddrMode::Imm),
        _ => panic!("AAA!!!!"),
    }
    cpu.print_state();
}
