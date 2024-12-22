use crate::bits::{as_address, Addr, Bitwise};

use super::{CPU, IRQ_VECTOR};

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

impl std::fmt::Display for AddrMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Imp => write!(f, ""),
            Acc => write!(f, ""),
            Imm => write!(f, " #$_b"),
            ZP(ir)  => write!(f, " $_b{}", match ir {
                N => "",
                X => ",x",
                Y => ",y",
            }),
            Abs(ir) => write!(f, " $_w{}", match ir {
                N => "",
                X => ",x",
                Y => ",y",
            }),
            Ind(ir) => match ir {
                N => write!(f, " $_w"),
                X => write!(f, " ($_w,x)"),
                Y => write!(f, " ($_w),y"),
            },
            Rel => write!(f, " $_r"),
        }
    }
}

#[derive(Clone,Copy,Debug)]
#[rustfmt::skip]
pub enum Inst { 
    // TODO: undefined opcodes
    ADC(AddrMode), AND(AddrMode), ASL(AddrMode), BCC(AddrMode), BCS(AddrMode), BEQ(AddrMode), BIT(AddrMode),
    BMI(AddrMode), BNE(AddrMode), BPL(AddrMode), BRK(AddrMode), BVC(AddrMode), BVS(AddrMode), CLC(AddrMode),
    CLD(AddrMode), CLI(AddrMode), CLV(AddrMode), CMP(AddrMode), CPX(AddrMode), CPY(AddrMode), DEC(AddrMode),
    DEX(AddrMode), DEY(AddrMode), EOR(AddrMode), INC(AddrMode), INX(AddrMode), INY(AddrMode), JMP(AddrMode),
    JSR(AddrMode), LDA(AddrMode), LDX(AddrMode), LDY(AddrMode), LSR(AddrMode), NOP(AddrMode), ORA(AddrMode),
    PHA(AddrMode), PHP(AddrMode), PLA(AddrMode), PLP(AddrMode), ROL(AddrMode), ROR(AddrMode), RTI(AddrMode),
    RTS(AddrMode), SBC(AddrMode), SEC(AddrMode), SED(AddrMode), SEI(AddrMode), STA(AddrMode), STX(AddrMode),
    STY(AddrMode), TAX(AddrMode), TAY(AddrMode), TSX(AddrMode), TXA(AddrMode), TXS(AddrMode), TYA(AddrMode),
    ILL(u8),
}

impl std::fmt::Display for Inst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ADC(am) => write!(f, "ADC{am}"),
            AND(am) => write!(f, "AND{am}"),
            ASL(am) => write!(f, "ASL{am}"),
            BCC(am) => write!(f, "BCC{am}"),
            BCS(am) => write!(f, "BCS{am}"),
            BEQ(am) => write!(f, "BEQ{am}"),
            BIT(am) => write!(f, "BIT{am}"),
            BMI(am) => write!(f, "BMI{am}"),
            BNE(am) => write!(f, "BNE{am}"),
            BPL(am) => write!(f, "BPL{am}"),
            BRK(am) => write!(f, "BRK{am}"),
            BVC(am) => write!(f, "BVC{am}"),
            BVS(am) => write!(f, "BVS{am}"),
            CLC(am) => write!(f, "CLC{am}"),
            CLD(am) => write!(f, "CLD{am}"),
            CLI(am) => write!(f, "CLI{am}"),
            CLV(am) => write!(f, "CLV{am}"),
            CMP(am) => write!(f, "CMP{am}"),
            CPX(am) => write!(f, "CPX{am}"),
            CPY(am) => write!(f, "CPY{am}"),
            DEC(am) => write!(f, "DEC{am}"),
            DEX(am) => write!(f, "DEX{am}"),
            DEY(am) => write!(f, "DEY{am}"),
            EOR(am) => write!(f, "EOR{am}"),
            INC(am) => write!(f, "INC{am}"),
            INX(am) => write!(f, "INX{am}"),
            INY(am) => write!(f, "INY{am}"),
            JMP(am) => write!(f, "JMP{am}"),
            JSR(am) => write!(f, "JSR{am}"),
            LDA(am) => write!(f, "LDA{am}"),
            LDX(am) => write!(f, "LDX{am}"),
            LDY(am) => write!(f, "LDY{am}"),
            LSR(am) => write!(f, "LSR{am}"),
            NOP(am) => write!(f, "NOP{am}"),
            ORA(am) => write!(f, "ORA{am}"),
            PHA(am) => write!(f, "PHA{am}"),
            PHP(am) => write!(f, "PHP{am}"),
            PLA(am) => write!(f, "PLA{am}"),
            PLP(am) => write!(f, "PLP{am}"),
            ROL(am) => write!(f, "ROL{am}"),
            ROR(am) => write!(f, "ROR{am}"),
            RTI(am) => write!(f, "RTI{am}"),
            RTS(am) => write!(f, "RTS{am}"),
            SBC(am) => write!(f, "SBC{am}"),
            SEC(am) => write!(f, "SEC{am}"),
            SED(am) => write!(f, "SED{am}"),
            SEI(am) => write!(f, "SEI{am}"),
            STA(am) => write!(f, "STA{am}"),
            STX(am) => write!(f, "STX{am}"),
            STY(am) => write!(f, "STY{am}"),
            TAX(am) => write!(f, "TAX{am}"),
            TAY(am) => write!(f, "TAY{am}"),
            TSX(am) => write!(f, "TSX{am}"),
            TXA(am) => write!(f, "TXA{am}"),
            TXS(am) => write!(f, "TXS{am}"),
            TYA(am) => write!(f, "TYA{am}"),
            ILL(x) => write!(f, "ILL ${x}"),
        }
    }
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
    PLA(Imp   ), ADC(Imm   ), ROR(Acc   ), ILL(0x6b  ), JMP(Ind(N)), ADC(Abs(N)), ROR(Abs(N)), ILL(0x6f  ), 
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
            Inst::NOP(_a) => (),
            Inst::ADC(am) => adc(cpu, am, false),
            Inst::AND(am) => a_op_fn(cpu, am, |a, m| a & m),
            Inst::EOR(am) => a_op_fn(cpu, am, |a, m| a ^ m),
            Inst::ORA(am) => a_op_fn(cpu, am, |a, m| a | m),
            Inst::ASL(am) => rot(cpu, am, false, true),
            Inst::BIT(am) => bit(cpu, am),
            Inst::BCC(_a) => branch(cpu, !cpu.reg.p.c),
            Inst::BCS(_a) => branch(cpu, cpu.reg.p.c),
            Inst::BEQ(_a) => branch(cpu, cpu.reg.p.z),
            Inst::BMI(_a) => branch(cpu, cpu.reg.p.n),
            Inst::BNE(_a) => branch(cpu, !cpu.reg.p.z),
            Inst::BPL(_a) => branch(cpu, !cpu.reg.p.n),
            Inst::BVC(_a) => branch(cpu, !cpu.reg.p.v),
            Inst::BVS(_a) => branch(cpu, cpu.reg.p.v),
            Inst::JMP(am) => jmp(cpu, am, false),
            Inst::JSR(am) => jmp(cpu, am, true),
            Inst::RTS(_a) => rts(cpu, false),
            Inst::RTI(_a) => rts(cpu, true),
            Inst::BRK(_a) => brk(cpu),
            Inst::CLC(_a) => cpu.reg.p.c = false,
            Inst::CLD(_a) => cpu.reg.p.d = false,
            Inst::CLI(_a) => cpu.reg.p.i = false,
            Inst::CLV(_a) => cpu.reg.p.v = false,
            Inst::SEC(_a) => cpu.reg.p.c = true,
            Inst::SED(_a) => cpu.reg.p.d = true,
            Inst::SEI(_a) => cpu.reg.p.i = true,
            Inst::CMP(am) => cmp(cpu, am, InstrReg::A),
            Inst::CPX(am) => cmp(cpu, am, InstrReg::X),
            Inst::CPY(am) => cmp(cpu, am, InstrReg::Y),
            Inst::DEC(am) => inc(cpu, am, true),
            Inst::DEX(_a) => set_x(cpu, cpu.reg.x - 1),
            Inst::DEY(_a) => set_y(cpu, cpu.reg.y - 1),
            Inst::INC(am) => inc(cpu, am, false),
            Inst::INX(_a) => set_x(cpu, cpu.reg.x + 1),
            Inst::INY(_a) => set_y(cpu, cpu.reg.y + 1),
            Inst::LDA(am) => ld(cpu, am, InstrReg::A),
            Inst::LDX(am) => ld(cpu, am, InstrReg::X),
            Inst::LDY(am) => ld(cpu, am, InstrReg::Y),
            Inst::LSR(am) => rot(cpu, am, false, false),
            Inst::PHA(_a) => cpu.push(cpu.reg.a),
            Inst::PHP(_a) => cpu.push(cpu.reg.p.into()),
            Inst::PLA(_a) => cpu.reg.a = cpu.pull(),
            Inst::PLP(_a) => cpu.reg.p = cpu.pull().into(),
            Inst::ROL(am) => rot(cpu, am, true, true),
            Inst::ROR(am) => rot(cpu, am, true, false),
            Inst::SBC(am) => adc(cpu, am, true),
            Inst::STA(am) => st(cpu, am, InstrReg::A),
            Inst::STX(am) => st(cpu, am, InstrReg::X),
            Inst::STY(am) => st(cpu, am, InstrReg::Y),
            Inst::TAX(_a) => set_x(cpu, cpu.reg.a),
            Inst::TAY(_a) => set_y(cpu, cpu.reg.a),
            Inst::TXA(_a) => set_a(cpu, cpu.reg.x),
            Inst::TYA(_a) => set_a(cpu, cpu.reg.y),
            Inst::TSX(_a) => cpu.reg.x = cpu.reg.sp,
            Inst::TXS(_a) => cpu.reg.sp = cpu.reg.x,
            Inst::ILL(op) => ill(cpu, op),
        }
    }
}

fn a_op_fn(cpu: &mut CPU, am: AddrMode, f: fn(u8, u8) -> u8) {
    cpu.reg.a = f(cpu.reg.a, cpu.operand_read_inc(am));
    cpu.reg.p.z = cpu.reg.a == 0;
    cpu.reg.p.n = cpu.reg.a.test_bit(7)
}

fn set_a(cpu: &mut CPU, f: u8) {
    cpu.reg.a = f;
    cpu.reg.p.z = cpu.reg.a == 0;
    cpu.reg.p.n = cpu.reg.a.test_bit(7)
}

fn set_x(cpu: &mut CPU, f: u8) {
    cpu.reg.x = f;
    cpu.reg.p.z = cpu.reg.x == 0;
    cpu.reg.p.n = cpu.reg.x.test_bit(7)
}

fn set_y(cpu: &mut CPU, f: u8) {
    cpu.reg.y = f;
    cpu.reg.p.z = cpu.reg.y == 0;
    cpu.reg.p.n = cpu.reg.y.test_bit(7)
}

fn bit(cpu: &mut CPU, am: AddrMode) -> () {
    let a = cpu.reg.a;
    let m = cpu.operand_read_inc(am);
    let result = a & m;

    cpu.reg.p.z = result == 0;
    cpu.reg.p.v = m.test_bit(6);
    cpu.reg.p.n = m.test_bit(7);
}

fn inc(cpu: &mut CPU, am: AddrMode, decrement: bool) -> () {
    let m = cpu.operand_read_cycle(am);
    let result = if decrement { m - 1 } else { m + 1 };
    cpu.reg.p.z = result == 0;
    cpu.reg.p.n = result.test_bit(7);
    cpu.operand_write_inc(am, result);
}

enum InstrReg {
    A,
    X,
    Y,
}

impl InstrReg {
    fn set(self, cpu: &mut CPU, val: u8) -> () {
        match self {
            InstrReg::A => cpu.reg.a = val,
            InstrReg::X => cpu.reg.x = val,
            InstrReg::Y => cpu.reg.y = val,
        };
    }

    fn get(self, cpu: &CPU) -> u8 {
        match self {
            InstrReg::A => cpu.reg.a,
            InstrReg::X => cpu.reg.x,
            InstrReg::Y => cpu.reg.y,
        }
    }
}

fn ld(cpu: &mut CPU, am: AddrMode, inst_reg: InstrReg) -> () {
    let val = cpu.operand_read_inc(am);
    let z = val == 0;
    let n = val.test_bit(7);

    inst_reg.set(cpu, val);
    cpu.reg.p.z = z;
    cpu.reg.p.n = n;
}

fn st(cpu: &mut CPU, am: AddrMode, instr_reg: InstrReg) -> () {
    let val = instr_reg.get(cpu);

    cpu.operand_write_inc(am, val);
}

fn adc(cpu: &mut CPU, am: AddrMode, sbc: bool) -> () {
    let a = cpu.reg.a as u16;
    let m = (cpu.operand_read_inc(am) ^ (if sbc { 0xff } else { 0 })) as u16;
    let c = cpu.reg.p.c as u16;

    let result = a + m + c;

    cpu.reg.a = result as u8;
    cpu.reg.p.c = result > 0xff;
    cpu.reg.p.z = result as u8 == 0;
    cpu.reg.p.v = (!(a ^ m) & (a ^ result)).test_bit(7);
    cpu.reg.p.n = result.test_bit(7);
}

fn cmp(cpu: &mut CPU, am: AddrMode, instr_reg: InstrReg) -> () {
    let r = instr_reg.get(cpu);
    let m = cpu.operand_read_inc(am);
    let result = r - m;

    cpu.reg.p.c = r >= m;
    cpu.reg.p.z = r == m;
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

fn jmp(cpu: &mut CPU, am: AddrMode, jsr: bool) -> () {
    match am {
        AddrMode::Abs(N) => {
            let l = cpu.pc_read_inc(); // +1 cycle
            let m = if !jsr {
                cpu.pc_read_inc() // +1 cycle
            } else {
                let val = cpu.pc_read_nocycle(); // !!TODO: check (jsr is 5 cycles (w/o opcode))

                // as a side effect of using nocycle, pc is already "pc-1"
                // (which is what _should_ be pushed)
                cpu.push(cpu.reg.pc.msb()); // +2
                cpu.push(cpu.reg.pc.lsb()); // +2
                val
            };
            let addr = as_address(l, m);
            cpu.reg.pc = addr;
        }
        AddrMode::Ind(N) => {
            let l = cpu.pc_read_inc(); // +1 cycle
            let m = cpu.pc_read_inc(); // +1 cycle
            let addr = as_address(l, m);
            let addr_indirect = cpu.get_indirect(addr); // +2 cycles
            cpu.reg.pc = addr_indirect;
        }
        _ => unreachable!(),
    }
}

fn rts(cpu: &mut CPU, rti: bool) -> () {
    if rti {
        cpu.reg.p = cpu.pull_noextra().into(); // +1
    } else {
        cpu.cycle(); // +1
    }
    let l = cpu.pull(); // +3
    let m = cpu.pull_noextra(); // +1
    let addr = as_address(l, m);
    cpu.reg.pc = addr + 1;
}

fn brk(cpu: &mut CPU) -> () {
    // TODO: interrupts
    cpu.push((cpu.reg.pc + 1).msb()); // +2
    cpu.push((cpu.reg.pc + 1).lsb()); // +2
    cpu.push(cpu.reg.p.into()); // +2
    // cpu.reg.p.b = true; // ?

    let l = cpu.read_addr_nocycle(IRQ_VECTOR);
    let m = cpu.read_addr_nocycle(IRQ_VECTOR + 1);
    let addr = as_address(l, m);
    cpu.reg.pc = addr;
}

fn ill(_: &mut CPU, opcode: u8) -> () {
    panic!("ILLEGAL INSTRUCTION ${:02x}", opcode);
}
