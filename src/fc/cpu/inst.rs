use crate::bits::Bitwise;

use super::CPU;

#[derive(Clone, Copy, Debug)]
pub enum IndexRegister {
    N, // None
    X,
    Y,
}

#[derive(Clone, Copy, Debug)]
pub enum AddrMode {
    Imp,                // Implicit
    Acc,                // Accumulator
    Imm,                // Immediate
    ZP(IndexRegister),  // Zero Page ($0000-$00ff)
    Rel,                // Relative
    Abs(IndexRegister), // Absolute (N/X/Y)
    Ind(IndexRegister), // Indirect
                        // (Indirect,X)
                        // (Indirect),Y
}

#[derive(Clone,Copy,Debug)]
#[rustfmt::skip]
pub enum Inst { 
    ADC(AddrMode), AND(AddrMode), ASL(AddrMode), BCC(AddrMode), BCS(AddrMode), BEQ(AddrMode), BIT(AddrMode),
    BMI(AddrMode), BNE(AddrMode), BPL(AddrMode), BRK(AddrMode), BVC(AddrMode), BVS(AddrMode), CLC(AddrMode),
    CLD(AddrMode), CLI(AddrMode), CLV(AddrMode), CMP(AddrMode), CPX(AddrMode), CPY(AddrMode), DEC(AddrMode),
    DEX(AddrMode), DEY(AddrMode), EOR(AddrMode), INC(AddrMode), INX(AddrMode), INY(AddrMode), JMP(AddrMode),
    JSR(AddrMode), LDA(AddrMode), LDX(AddrMode), LDY(AddrMode), LSR(AddrMode), NOP(AddrMode), ORA(AddrMode),
    PHA(AddrMode), PHP(AddrMode), PLA(AddrMode), PLP(AddrMode), ROL(AddrMode), ROR(AddrMode), RTI(AddrMode),
    RTS(AddrMode), SBC(AddrMode), SEC(AddrMode), SED(AddrMode), SEI(AddrMode), STA(AddrMode), STX(AddrMode),
    STY(AddrMode), TAX(AddrMode), TAY(AddrMode), TSX(AddrMode), TXA(AddrMode), TXS(AddrMode), TYA(AddrMode),
    ILL(u8), STP(AddrMode),
}

use AddrMode::*;
use IndexRegister::*;
use Inst::*;

#[rustfmt::skip]
pub const INST_TABLE: [Inst; 256] = [
    BRK(Imp   ), ORA(Ind(X)), ILL(0x02  ), ILL(0x03  ), ILL(0x04  ), ORA(ZP(N) ), ASL(ZP(N) ), ILL(0x07  ), 
    PHP(Imp   ), ORA(Imm   ), ASL(Acc   ), ILL(0x0b  ), ILL(0x0c  ), ORA(Abs(N)), ASL(Abs(N)), ILL(0x0f  ), 
    BPL(Rel   ), ORA(Ind(Y)), ILL(0x12  ), ILL(0x13  ), ILL(0x14  ), ORA(ZP(X) ), ASL(ZP(X) ), ILL(0x17  ), 
    CLC(Imp   ), ORA(Abs(Y)), ILL(0x1a  ), ILL(0x1b  ), ILL(0x1c  ), ORA(Abs(X)), ASL(Abs(X)), ILL(0x1f  ), 
    JSR(Abs(N)), AND(Ind(X)), ILL(0x22  ), ILL(0x23  ), BIT(ZP(N) ), AND(ZP(N) ), ROL(ZP(N) ), ILL(0x27  ), 
    PLP(Imp   ), AND(Imm   ), ROL(Acc   ), ILL(0x2b  ), BIT(Abs(N)), AND(Abs(N)), ROL(Abs(N)), ILL(0x2f  ), 
    BMI(Rel   ), AND(Ind(Y)), ILL(0x32  ), ILL(0x33  ), ILL(0x34  ), AND(ZP(X) ), ROL(ZP(X) ), ILL(0x37  ), 
    SEC(Imp   ), AND(Abs(Y)), ILL(0x3a  ), ILL(0x3b  ), ILL(0x3c  ), AND(Abs(X)), ROL(Abs(X)), ILL(0x3f  ), 
    RTI(Imp   ), EOR(Ind(X)), ILL(0x42  ), ILL(0x43  ), ILL(0x44  ), EOR(ZP(N) ), LSR(ZP(N) ), ILL(0x47  ), 
    PHA(Imp   ), EOR(Imm   ), LSR(Acc   ), ILL(0x4b  ), JMP(Abs(N)), EOR(Abs(N)), LSR(Abs(N)), ILL(0x4f  ), 
    BVC(Rel   ), EOR(Ind(Y)), ILL(0x52  ), ILL(0x53  ), ILL(0x54  ), EOR(ZP(X) ), LSR(ZP(X) ), ILL(0x57  ), 
    CLI(Imp   ), EOR(Abs(Y)), ILL(0x5a  ), ILL(0x5b  ), ILL(0x5c  ), EOR(Abs(X)), LSR(Abs(X)), ILL(0x5f  ), 
    RTS(Imp   ), ADC(Ind(X)), ILL(0x62  ), ILL(0x63  ), ILL(0x64  ), ADC(ZP(N) ), ROR(ZP(N) ), ILL(0x67  ), 
    PLA(Imp   ), ADC(Imm   ), ROR(Acc   ), ILL(0x6b  ), JMP(Imp   ), ADC(Abs(N)), ROR(Abs(N)), ILL(0x6f  ), 
    BVS(Rel   ), ADC(Ind(Y)), ILL(0x72  ), ILL(0x73  ), ILL(0x74  ), ADC(ZP(X) ), ROR(ZP(X) ), ILL(0x77  ), 
    SEI(Imp   ), ADC(Abs(Y)), ILL(0x7a  ), ILL(0x7b  ), ILL(0x7c  ), ADC(Abs(X)), ROR(Abs(X)), ILL(0x7f  ), 
    ILL(0x80  ), STA(Ind(X)), ILL(0x82  ), ILL(0x83  ), STY(ZP(N) ), STA(ZP(N) ), STX(ZP(N) ), ILL(0x87  ), 
    DEY(Imp   ), ILL(0x89  ), TXA(Imp   ), ILL(0x8b  ), STY(Abs(N)), STA(Abs(N)), STX(Abs(N)), ILL(0x8f  ), 
    BCC(Rel   ), STA(Ind(Y)), ILL(0x92  ), ILL(0x93  ), STY(ZP(X) ), STA(ZP(X) ), STX(ZP(Y) ), ILL(0x97  ), 
    TYA(Imp   ), STA(Abs(Y)), TXS(Imp   ), ILL(0x9b  ), ILL(0x9c  ), STA(Abs(X)), ILL(0x9e  ), ILL(0x9f  ), 
    LDY(Imm   ), LDA(Ind(X)), LDX(Imm   ), ILL(0xa3  ), LDY(ZP(N) ), LDA(ZP(N) ), LDX(ZP(N) ), ILL(0xa7  ), 
    TAY(Imp   ), LDA(Imm   ), TAX(Imp   ), ILL(0xab  ), LDY(Abs(N)), LDA(Abs(N)), LDX(Abs(N)), ILL(0xaf  ), 
    BCS(Rel   ), LDA(Ind(Y)), ILL(0xb2  ), ILL(0xb3  ), LDY(ZP(X) ), LDA(ZP(X) ), LDX(ZP(Y) ), ILL(0xb7  ), 
    CLV(Imp   ), LDA(Abs(Y)), TSX(Imp   ), ILL(0xbb  ), LDY(Abs(X)), LDA(Abs(X)), LDX(Abs(Y)), ILL(0xbf  ), 
    CPY(Imm   ), CMP(Ind(X)), ILL(0xc2  ), ILL(0xc3  ), CPY(ZP(N) ), CMP(ZP(N) ), DEC(ZP(N) ), ILL(0xc7  ), 
    INY(Imp   ), CMP(Imm   ), DEX(Imp   ), ILL(0xcb  ), CPY(Abs(N)), CMP(Abs(N)), DEC(Abs(N)), ILL(0xcf  ), 
    BNE(Rel   ), CMP(Ind(Y)), ILL(0xd2  ), ILL(0xd3  ), ILL(0xd4  ), CMP(ZP(X) ), DEC(ZP(X) ), ILL(0xd7  ), 
    CLD(Imp   ), CMP(Abs(Y)), ILL(0xda  ), ILL(0xdb  ), ILL(0xdc  ), CMP(Abs(X)), DEC(Abs(X)), ILL(0xdf  ), 
    CPX(Imm   ), SBC(Ind(X)), ILL(0xe2  ), ILL(0xe3  ), CPX(ZP(N) ), SBC(ZP(N) ), INC(ZP(N) ), ILL(0xe7  ), 
    INX(Imp   ), SBC(Imm   ), NOP(Imp   ), ILL(0xeb  ), CPX(Abs(N)), SBC(Abs(N)), INC(Abs(N)), ILL(0xef  ), 
    BEQ(Rel   ), SBC(Ind(Y)), ILL(0xf2  ), ILL(0xf3  ), ILL(0xf4  ), SBC(ZP(X) ), INC(ZP(X) ), ILL(0xf7  ), 
    SED(Imp   ), SBC(Abs(Y)), ILL(0xfa  ), ILL(0xfb  ), ILL(0xfc  ), SBC(Abs(X)), INC(Abs(X)), ILL(0xff  ), 
];

impl Inst {
    pub fn get(opcode: u8) -> Inst {
        INST_TABLE[opcode as usize]
    }

    pub fn run(self, cpu: &mut CPU) -> () {
        match self {
            Inst::NOP(_) => (),
            Inst::ADC(am) => adc(cpu, am, false),
            Inst::AND(am) => a_fn(cpu, am, |a, m| a & m),
            Inst::EOR(am) => a_fn(cpu, am, |a, m| a ^ m),
            Inst::ORA(am) => a_fn(cpu, am, |a, m| a | m),
            Inst::ASL(am) => rot(cpu, am, false, true),
            Inst::BCC(_) => branch(cpu, !cpu.reg.p.c),
            Inst::BCS(_) => branch(cpu, cpu.reg.p.c),
            Inst::BEQ(_) => branch(cpu, cpu.reg.p.z),
            Inst::BIT(am) => bit(cpu, am),
            Inst::BMI(_) => branch(cpu, cpu.reg.p.n),
            Inst::BNE(_) => branch(cpu, !cpu.reg.p.z),
            Inst::BPL(_) => branch(cpu, !cpu.reg.p.n),
            Inst::BRK(_am) => todo!("instruction BRK"),
            Inst::BVC(_am) => todo!("instruction BVC"),
            Inst::BVS(_am) => todo!("instruction BVS"),
            Inst::CLC(_) => cpu.reg.p.c = false,
            Inst::CLD(_) => cpu.reg.p.d = false,
            Inst::CLI(_) => cpu.reg.p.i = false,
            Inst::CLV(_) => cpu.reg.p.v = false,
            Inst::CMP(am) => cmp(cpu, am, InstrReg::A),
            Inst::CPX(am) => cmp(cpu, am, InstrReg::X),
            Inst::CPY(am) => cmp(cpu, am, InstrReg::Y),
            Inst::DEC(_am) => todo!("instruction DEC"),
            Inst::DEX(_am) => todo!("instruction DEX"),
            Inst::DEY(_am) => todo!("instruction DEY"),
            Inst::INC(_am) => todo!("instruction INC"),
            Inst::INX(_am) => todo!("instruction INX"),
            Inst::INY(_am) => todo!("instruction INY"),
            Inst::JMP(_am) => todo!("instruction JMP"),
            Inst::JSR(_am) => todo!("instruction JSR"),
            Inst::LDA(am) => ld(cpu, am, InstrReg::A),
            Inst::LDX(am) => ld(cpu, am, InstrReg::X),
            Inst::LDY(am) => ld(cpu, am, InstrReg::Y),
            Inst::LSR(am) => rot(cpu, am, false, false),
            Inst::PHA(_am) => todo!("instruction PHA"),
            Inst::PHP(_am) => todo!("instruction PHP"),
            Inst::PLA(_am) => todo!("instruction PLA"),
            Inst::PLP(_am) => todo!("instruction PLP"),
            Inst::ROL(am) => rot(cpu, am, true, true),
            Inst::ROR(am) => rot(cpu, am, true, false),
            Inst::RTI(_am) => todo!("instruction RTI"),
            Inst::RTS(_am) => todo!("instruction RTS"),
            Inst::SBC(am) => adc(cpu, am, true),
            Inst::SEC(_) => cpu.reg.p.c = true,
            Inst::SED(_) => cpu.reg.p.d = true,
            Inst::SEI(_) => cpu.reg.p.i = true,
            Inst::STA(am) => st(cpu, am, InstrReg::A),
            Inst::STX(am) => st(cpu, am, InstrReg::X),
            Inst::STY(am) => st(cpu, am, InstrReg::Y),
            Inst::TAX(_am) => todo!("instruction TAX"),
            Inst::TAY(_am) => todo!("instruction TAY"),
            Inst::TSX(_am) => todo!("instruction TSX"),
            Inst::TXA(_am) => todo!("instruction TXA"),
            Inst::TXS(_am) => todo!("instruction TXS"),
            Inst::TYA(_am) => todo!("instruction TYA"),
            // Extra:
            Inst::STP(_am) => todo!("instruction STP"),
            Inst::ILL(op) => ill(cpu, op),
        }
    }
}

fn a_fn(cpu: &mut CPU, am: AddrMode, f: fn(u8, u8) -> u8) {
    cpu.reg.a = f(cpu.reg.a, cpu.read_operand_inc(am));
    cpu.reg.p.z = cpu.reg.a == 0;
    cpu.reg.p.n = cpu.reg.a.test_bit(7)
}

fn bit(cpu: &mut CPU, am: AddrMode) -> () {
    let a = cpu.reg.a;
    let m = cpu.read_operand_inc(am);
    let result = a & m;

    cpu.reg.p.z = result == 0;
    cpu.reg.p.v = m.test_bit(6);
    cpu.reg.p.n = m.test_bit(7);
}

enum InstrReg {
    A,
    X,
    Y,
}

fn ld(cpu: &mut CPU, am: AddrMode, inst_reg: InstrReg) -> () {
    let val = cpu.read_operand_inc(am);
    let z = val == 0;
    let n = val.test_bit(7);

    match inst_reg {
        InstrReg::A => cpu.reg.a = val,
        InstrReg::X => cpu.reg.x = val,
        InstrReg::Y => cpu.reg.y = val,
    };
    cpu.reg.p.z = z;
    cpu.reg.p.n = n;
}

fn st(cpu: &mut CPU, am: AddrMode, instr_reg: InstrReg) -> () {
    let val = match instr_reg {
        InstrReg::A => cpu.reg.a,
        InstrReg::X => cpu.reg.x,
        InstrReg::Y => cpu.reg.y,
    };

    cpu.write_operand_inc(am, val);
}

fn adc(cpu: &mut CPU, am: AddrMode, sbc: bool) -> () {
    let a = cpu.reg.a as u16;
    let m = (cpu.read_operand_inc(am) ^ (if sbc { 0xff } else { 0 })) as u16;
    let c = cpu.reg.p.c as u16;

    let result = a + m + c;

    cpu.reg.a = result as u8;
    cpu.reg.p.c = result > 0xff;
    cpu.reg.p.z = result as u8 == 0;
    cpu.reg.p.v = (!(a ^ m) & (a ^ result)).test_bit(7);
    cpu.reg.p.n = result.test_bit(7);
}

fn cmp(cpu: &mut CPU, am: AddrMode, instr_reg: InstrReg) -> () {
    let r = match instr_reg {
        InstrReg::A => cpu.reg.a,
        InstrReg::X => cpu.reg.x,
        InstrReg::Y => cpu.reg.y,
    };
    let m = cpu.read_operand_inc(am);
    let result = r - m;

    cpu.reg.p.c = r >= m;
    cpu.reg.p.c = r == m;
    cpu.reg.p.n = result.test_bit(7);
}

fn rot(cpu: &mut CPU, am: AddrMode, rotate: bool, left: bool) -> () {
    let shift_fn: fn(u8) -> u8 = if left { |x| x << 1 } else { |x| x >> 1 };
    let pad_amount = if left { 0 } else { 7 };

    let val = match am {
        Acc => {
            let a = cpu.reg.a;
            cpu.reg.p.c = a.test_bit(7); // TODO: is this _really_ just in the case of A? check.
            let new_val = (a << 1) | cpu.reg.p.c as u8;
            cpu.reg.a = new_val;
            new_val
        }
        ZP(ir) => {
            let val = cpu.zp_read_cycle(ir); // !! +2 cycles !!
            cpu.reg.p.c = val.test_bit(7);
            // cpu.cycle();    // +1 cycle for modify stage
            let new_val =
                shift_fn(val) | (if rotate { cpu.reg.p.c as u8 } else { 0 } << pad_amount);
            cpu.zp_write_inc(new_val, ir); // +2(+1) cycles
            new_val
        }
        Abs(ir) => {
            let val = cpu.abs_read_cycle(ir); // !! +2 cycles !!
            cpu.reg.p.c = val.test_bit(7);
            // cpu.cycle();    // +1 cycle for modify stage
            let new_val =
                shift_fn(val) | (if rotate { cpu.reg.p.c as u8 } else { 0 } << pad_amount);
            cpu.abs_write_inc(new_val, ir); // +3(+1) cycles
            new_val
            // = 5(+1)
        }
        _ => unreachable!(),
    };
    cpu.reg.p.n = val.test_bit(7);
}

fn branch(cpu: &mut CPU, condition: bool) -> () {
    let offset = cpu.pc_read_inc() as i8; // +1 cycle
    if condition {
        cpu.pc_offset_cycle(offset); // +1(+1) cycle(s)
    }
}

fn ill(_: &mut CPU, opcode: u8) -> () {
    panic!("ILLEGAL INSTRUCTION ${:02x}", opcode);
}
