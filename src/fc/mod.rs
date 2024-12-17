use crate::fc::cpu::*;

pub mod cpu;

pub fn tester() {
    let mut cpu = CPU {
        reg: Registers::new(),
        mem: [0; 0x10000],
    };

    println!("{}", cpu.reg.a);
    cpu.reg.a = 2;
    println!("{}", cpu.reg.a);

    let inst: Inst = Inst::LDA;
    match inst {
        Inst::LDA => cpu.lda(AddrMode::Imm),
        _ => panic!("AAA!!!!"),
    }
    println!("{}", cpu.reg.a);
}
