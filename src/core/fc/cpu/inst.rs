use crate::bits::{Addr, Bitwise, as_address};

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

impl AddrMode {
    pub fn arg_count(&self) -> u8 {
        match self {
            Imp => 0,
            Acc => 0,
            Imm => 1,
            ZP(_) => 1,
            Abs(_) => 2,
            Ind(ir) => match ir {
                N => 2,
                X => 1,
                Y => 1,
            },
            Rel => 1,
        }
    }
}

impl std::fmt::Display for AddrMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Imp => write!(f, ""),
            Acc => write!(f, " a"),
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
                N => write!(f, " ($_w)"),
                X => write!(f, " ($_b,x)"),
                Y => write!(f, " ($_b),y"),
            },
            Rel => write!(f, " $_r"),
        }
    }
}

#[derive(Clone,Copy,Debug)]
#[rustfmt::skip]
pub enum Inst {
    // Official
    ADC(AddrMode), AND(AddrMode), ASL(AddrMode), BCC(AddrMode), BCS(AddrMode), BEQ(AddrMode), BIT(AddrMode),
    BMI(AddrMode), BNE(AddrMode), BPL(AddrMode), BRK(AddrMode), BVC(AddrMode), BVS(AddrMode), CLC(AddrMode),
    CLD(AddrMode), CLI(AddrMode), CLV(AddrMode), CMP(AddrMode), CPX(AddrMode), CPY(AddrMode), DEC(AddrMode),
    DEX(AddrMode), DEY(AddrMode), EOR(AddrMode), INC(AddrMode), INX(AddrMode), INY(AddrMode), JMP(AddrMode),
    JSR(AddrMode), LDA(AddrMode), LDX(AddrMode), LDY(AddrMode), LSR(AddrMode), NOP(AddrMode), ORA(AddrMode),
    PHA(AddrMode), PHP(AddrMode), PLA(AddrMode), PLP(AddrMode), ROL(AddrMode), ROR(AddrMode), RTI(AddrMode),
    RTS(AddrMode), SBC(AddrMode), SEC(AddrMode), SED(AddrMode), SEI(AddrMode), STA(AddrMode), STX(AddrMode),
    STY(AddrMode), TAX(AddrMode), TAY(AddrMode), TSX(AddrMode), TXA(AddrMode), TXS(AddrMode), TYA(AddrMode),
    // Unofficial
    ALR(AddrMode), ANC(AddrMode), ARR(AddrMode), AHX(AddrMode), AXS(AddrMode), DCP(AddrMode), RLA(AddrMode),
    RRA(AddrMode), LAS(AddrMode), LAX(AddrMode), SAX(AddrMode), SLO(AddrMode), SRE(AddrMode), SHX(AddrMode),
    SHY(AddrMode), ISC(AddrMode), TAS(AddrMode), XAA(AddrMode),
    STP(u8),
}

impl std::fmt::Display for Inst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ADC(am) => write!(f, "adc{am}"),
            AND(am) => write!(f, "and{am}"),
            ASL(am) => write!(f, "asl{am}"),
            BCC(am) => write!(f, "bcc{am}"),
            BCS(am) => write!(f, "bcs{am}"),
            BEQ(am) => write!(f, "beq{am}"),
            BIT(am) => write!(f, "bit{am}"),
            BMI(am) => write!(f, "bmi{am}"),
            BNE(am) => write!(f, "bne{am}"),
            BPL(am) => write!(f, "bpl{am}"),
            BRK(am) => write!(f, "brk{am}"),
            BVC(am) => write!(f, "bvc{am}"),
            BVS(am) => write!(f, "bvs{am}"),
            CLC(am) => write!(f, "clc{am}"),
            CLD(am) => write!(f, "cld{am}"),
            CLI(am) => write!(f, "cli{am}"),
            CLV(am) => write!(f, "clv{am}"),
            CMP(am) => write!(f, "cmp{am}"),
            CPX(am) => write!(f, "cpx{am}"),
            CPY(am) => write!(f, "cpy{am}"),
            DEC(am) => write!(f, "dec{am}"),
            DEX(am) => write!(f, "dex{am}"),
            DEY(am) => write!(f, "dey{am}"),
            EOR(am) => write!(f, "eor{am}"),
            INC(am) => write!(f, "inc{am}"),
            INX(am) => write!(f, "inx{am}"),
            INY(am) => write!(f, "iny{am}"),
            JMP(am) => write!(f, "jmp{am}"),
            JSR(am) => write!(f, "jsr{am}"),
            LDA(am) => write!(f, "lda{am}"),
            LDX(am) => write!(f, "ldx{am}"),
            LDY(am) => write!(f, "ldy{am}"),
            LSR(am) => write!(f, "lsr{am}"),
            NOP(am) => write!(f, "nop{am}"),
            ORA(am) => write!(f, "ora{am}"),
            PHA(am) => write!(f, "pha{am}"),
            PHP(am) => write!(f, "php{am}"),
            PLA(am) => write!(f, "pla{am}"),
            PLP(am) => write!(f, "plp{am}"),
            ROL(am) => write!(f, "rol{am}"),
            ROR(am) => write!(f, "ror{am}"),
            RTI(am) => write!(f, "rti{am}"),
            RTS(am) => write!(f, "rts{am}"),
            SBC(am) => write!(f, "sbc{am}"),
            SEC(am) => write!(f, "sec{am}"),
            SED(am) => write!(f, "sed{am}"),
            SEI(am) => write!(f, "sei{am}"),
            STA(am) => write!(f, "sta{am}"),
            STX(am) => write!(f, "stx{am}"),
            STY(am) => write!(f, "sty{am}"),
            TAX(am) => write!(f, "tax{am}"),
            TAY(am) => write!(f, "tay{am}"),
            TSX(am) => write!(f, "tsx{am}"),
            TXA(am) => write!(f, "txa{am}"),
            TXS(am) => write!(f, "txs{am}"),
            TYA(am) => write!(f, "tya{am}"),
            // Unofficial
            ALR(am) => write!(f, "alr{am}"),
            ANC(am) => write!(f, "anc{am}"),
            ARR(am) => write!(f, "arr{am}"),
            AHX(am) => write!(f, "ahx{am}"),
            AXS(am) => write!(f, "axs{am}"),
            DCP(am) => write!(f, "dcp{am}"),
            RLA(am) => write!(f, "rla{am}"),
            RRA(am) => write!(f, "rra{am}"),
            LAS(am) => write!(f, "las{am}"),
            LAX(am) => write!(f, "lax{am}"),
            SAX(am) => write!(f, "sax{am}"),
            SLO(am) => write!(f, "slo{am}"),
            SRE(am) => write!(f, "sre{am}"),
            SHX(am) => write!(f, "shx{am}"),
            SHY(am) => write!(f, "shy{am}"),
            ISC(am) => write!(f, "isc{am}"),
            TAS(am) => write!(f, "tas{am}"),
            XAA(am) => write!(f, "xaa{am}"),
            STP(x)        => write!(f, "stp (${x})"),
        }
    }
}

use AddrMode::*;
use IndexRegister::*;
use Inst::*;

#[rustfmt::skip]
pub const INST_TABLE: [Inst; 256] = [
    BRK(Imp   ), ORA(Ind(X)), STP(0x02  ), SLO(Ind(X)), NOP(ZP(N) ), ORA(ZP(N) ), ASL(ZP(N) ), SLO(ZP(N) ), 
    PHP(Imp   ), ORA(Imm   ), ASL(Acc   ), ANC(Imm   ), NOP(Abs(N)), ORA(Abs(N)), ASL(Abs(N)), SLO(Abs(N)), 
    BPL(Rel   ), ORA(Ind(Y)), STP(0x12  ), SLO(Ind(Y)), NOP(ZP(X) ), ORA(ZP(X) ), ASL(ZP(X) ), SLO(ZP(X) ), 
    CLC(Imp   ), ORA(Abs(Y)), NOP(Imp   ), SLO(Abs(Y)), NOP(Abs(X)), ORA(Abs(X)), ASL(Abs(X)), SLO(Abs(X)), 
    JSR(Abs(N)), AND(Ind(X)), STP(0x22  ), RLA(Ind(X)), BIT(ZP(N) ), AND(ZP(N) ), ROL(ZP(N) ), RLA(ZP(N) ), 
    PLP(Imp   ), AND(Imm   ), ROL(Acc   ), ANC(Imm   ), BIT(Abs(N)), AND(Abs(N)), ROL(Abs(N)), RLA(Abs(N)), 
    BMI(Rel   ), AND(Ind(Y)), STP(0x32  ), RLA(Ind(Y)), NOP(ZP(X) ), AND(ZP(X) ), ROL(ZP(X) ), RLA(ZP(X) ), 
    SEC(Imp   ), AND(Abs(Y)), NOP(Imp   ), RLA(Abs(Y)), NOP(Abs(X)), AND(Abs(X)), ROL(Abs(X)), RLA(Abs(X)), 
    RTI(Imp   ), EOR(Ind(X)), STP(0x42  ), SRE(Ind(X)), NOP(ZP(N) ), EOR(ZP(N) ), LSR(ZP(N) ), SRE(ZP(N) ), 
    PHA(Imp   ), EOR(Imm   ), LSR(Acc   ), ALR(Imm   ), JMP(Abs(N)), EOR(Abs(N)), LSR(Abs(N)), SRE(Abs(N)), 
    BVC(Rel   ), EOR(Ind(Y)), STP(0x52  ), SRE(Ind(Y)), NOP(ZP(X) ), EOR(ZP(X) ), LSR(ZP(X) ), SRE(ZP(X) ), 
    CLI(Imp   ), EOR(Abs(Y)), NOP(Imp   ), SRE(Abs(Y)), NOP(Abs(X)), EOR(Abs(X)), LSR(Abs(X)), SRE(Abs(X)), 
    RTS(Imp   ), ADC(Ind(X)), STP(0x62  ), RRA(Ind(X)), NOP(ZP(N) ), ADC(ZP(N) ), ROR(ZP(N) ), RRA(ZP(N) ), 
    PLA(Imp   ), ADC(Imm   ), ROR(Acc   ), ARR(Imm   ), JMP(Ind(N)), ADC(Abs(N)), ROR(Abs(N)), RRA(Abs(N)), 
    BVS(Rel   ), ADC(Ind(Y)), STP(0x72  ), RRA(Ind(Y)), NOP(ZP(X) ), ADC(ZP(X) ), ROR(ZP(X) ), RRA(ZP(X) ), 
    SEI(Imp   ), ADC(Abs(Y)), NOP(Imp   ), RRA(Abs(Y)), NOP(Abs(X)), ADC(Abs(X)), ROR(Abs(X)), RRA(Abs(X)), 
    NOP(Imm   ), STA(Ind(X)), NOP(Imm   ), SAX(Ind(X)), STY(ZP(N) ), STA(ZP(N) ), STX(ZP(N) ), SAX(ZP(N) ), 
    DEY(Imp   ), NOP(Imm   ), TXA(Imp   ), XAA(Imm   ), STY(Abs(N)), STA(Abs(N)), STX(Abs(N)), SAX(Abs(N)), 
    BCC(Rel   ), STA(Ind(Y)), STP(0x92  ), AHX(Ind(Y)), STY(ZP(X) ), STA(ZP(X) ), STX(ZP(Y) ), SAX(ZP(Y) ), 
    TYA(Imp   ), STA(Abs(Y)), TXS(Imp   ), TAS(Abs(Y)), SHY(Abs(X)), STA(Abs(X)), SHX(Abs(Y)), AHX(Abs(Y)), 
    LDY(Imm   ), LDA(Ind(X)), LDX(Imm   ), LAX(Ind(X)), LDY(ZP(N) ), LDA(ZP(N) ), LDX(ZP(N) ), LAX(ZP(N) ), 
    TAY(Imp   ), LDA(Imm   ), TAX(Imp   ), LAX(Imm   ), LDY(Abs(N)), LDA(Abs(N)), LDX(Abs(N)), LAX(Abs(N)), 
    BCS(Rel   ), LDA(Ind(Y)), STP(0xb2  ), LAX(Ind(Y)), LDY(ZP(X) ), LDA(ZP(X) ), LDX(ZP(Y) ), LAX(ZP(Y) ), 
    CLV(Imp   ), LDA(Abs(Y)), TSX(Imp   ), LAS(Abs(Y)), LDY(Abs(X)), LDA(Abs(X)), LDX(Abs(Y)), LAX(Abs(Y)), 
    CPY(Imm   ), CMP(Ind(X)), NOP(Imm   ), DCP(Ind(X)), CPY(ZP(N) ), CMP(ZP(N) ), DEC(ZP(N) ), DCP(ZP(N) ), 
    INY(Imp   ), CMP(Imm   ), DEX(Imp   ), AXS(Imm   ), CPY(Abs(N)), CMP(Abs(N)), DEC(Abs(N)), DCP(Abs(N)), 
    BNE(Rel   ), CMP(Ind(Y)), STP(0xd2  ), DCP(Ind(Y)), NOP(ZP(X) ), CMP(ZP(X) ), DEC(ZP(X) ), DCP(ZP(X) ), 
    CLD(Imp   ), CMP(Abs(Y)), NOP(Imp   ), DCP(Abs(Y)), NOP(Abs(X)), CMP(Abs(X)), DEC(Abs(X)), DCP(Abs(X)), 
    CPX(Imm   ), SBC(Ind(X)), NOP(Imm   ), ISC(Ind(X)), CPX(ZP(N) ), SBC(ZP(N) ), INC(ZP(N) ), ISC(ZP(N) ), 
    INX(Imp   ), SBC(Imm   ), NOP(Imp   ), SBC(Imm   ), CPX(Abs(N)), SBC(Abs(N)), INC(Abs(N)), ISC(Abs(N)), 
    BEQ(Rel   ), SBC(Ind(Y)), STP(0xf2  ), ISC(Ind(Y)), NOP(ZP(X) ), SBC(ZP(X) ), INC(ZP(X) ), ISC(ZP(X) ), 
    SED(Imp   ), SBC(Abs(Y)), NOP(Imp   ), ISC(Abs(Y)), NOP(Abs(X)), SBC(Abs(X)), INC(Abs(X)), ISC(Abs(X)), 
];

impl Inst {
    pub fn get(opcode: u8) -> Inst {
        INST_TABLE[opcode as usize]
    }

    pub fn run(self, cpu: &mut CPU) -> () {
        match self {
            NOP(am) => nop(cpu, am),
            ADC(am) => adc(cpu, am, false),
            AND(am) => and(cpu, am),
            EOR(am) => eor(cpu, am),
            ORA(am) => ora(cpu, am),
            ASL(am) => { rot(cpu, am, false, true); },
            BIT(am) => bit(cpu, am),
            BCC(_a) => branch(cpu, !cpu.reg.p.c),
            BCS(_a) => branch(cpu, cpu.reg.p.c),
            BEQ(_a) => branch(cpu, cpu.reg.p.z),
            BMI(_a) => branch(cpu, cpu.reg.p.n),
            BNE(_a) => branch(cpu, !cpu.reg.p.z),
            BPL(_a) => branch(cpu, !cpu.reg.p.n),
            BVC(_a) => branch(cpu, !cpu.reg.p.v),
            BVS(_a) => branch(cpu, cpu.reg.p.v),
            JMP(am) => jmp(cpu, am, false),
            JSR(am) => jmp(cpu, am, true),
            RTS(_a) => rts(cpu, false),
            RTI(_a) => rts(cpu, true),
            BRK(_a) => brk(cpu),
            CLC(_a) => cpu.reg.p.c = false,
            CLD(_a) => cpu.reg.p.d = false,
            CLI(_a) => cpu.reg.p.i = false,
            CLV(_a) => cpu.reg.p.v = false,
            SEC(_a) => cpu.reg.p.c = true,
            SED(_a) => cpu.reg.p.d = true,
            SEI(_a) => cpu.reg.p.i = true,
            CMP(am) => cmp(cpu, am, InstrReg::A),
            CPX(am) => cmp(cpu, am, InstrReg::X),
            CPY(am) => cmp(cpu, am, InstrReg::Y),
            DEC(am) => inc(cpu, am, true),
            DEX(_a) => set_x(cpu, cpu.reg.x - 1),
            DEY(_a) => set_y(cpu, cpu.reg.y - 1),
            INC(am) => inc(cpu, am, false),
            INX(_a) => set_x(cpu, cpu.reg.x + 1),
            INY(_a) => set_y(cpu, cpu.reg.y + 1),
            LDA(am) => ld(cpu, am, InstrReg::A),
            LDX(am) => ld(cpu, am, InstrReg::X),
            LDY(am) => ld(cpu, am, InstrReg::Y),
            LSR(am) => { rot(cpu, am, false, false); },
            PHA(_a) => cpu.push(cpu.reg.a),
            PHP(_a) => cpu.push(Into::<u8>::into(cpu.reg.p) | 0b0011_0000),
            PLA(_a) => pla(cpu),
            PLP(_a) => cpu.reg.p = (cpu.pull() & 0b1100_1111).into(),
            ROL(am) => { rot(cpu, am, true, true); },
            ROR(am) => { rot(cpu, am, true, false); },
            SBC(am) => adc(cpu, am, true),
            STA(am) => st(cpu, am, InstrReg::A),
            STX(am) => st(cpu, am, InstrReg::X),
            STY(am) => st(cpu, am, InstrReg::Y),
            TAX(_a) => set_x(cpu, cpu.reg.a),
            TAY(_a) => set_y(cpu, cpu.reg.a),
            TXA(_a) => set_a(cpu, cpu.reg.x),
            TYA(_a) => set_a(cpu, cpu.reg.y),
            TSX(_a) => set_x(cpu, cpu.reg.sp),
            TXS(_a) => cpu.reg.sp = cpu.reg.x,
            // Unofficial
            ALR(am) => alr(cpu, am),
            ANC(am) => anc(cpu, am),
            ARR(am) => arr(cpu, am),
            AHX(am) => shn(cpu, am, InstrReg::A),
            AXS(am) => axs(cpu, am),
            DCP(am) => dcp(cpu, am, false),
            RLA(am) => rla(cpu, am, true, true),
            RRA(am) => rra(cpu, am),
            LAX(am) => lax(cpu, am),
            SAX(am) => sax(cpu, am),
            SLO(am) => slo(cpu, am, false, true),
            SRE(am) => sre(cpu, am, false, false),
            SHX(am) => shn(cpu, am, InstrReg::X),
            SHY(am) => shn(cpu, am, InstrReg::Y),
            ISC(am) => dcp(cpu, am, true),
            TAS(am) => tas(cpu, am),
            LAS(am) => las(cpu, am),
            XAA(am) => xaa(cpu, am),
            STP(op) => stp(cpu, op),
        }
    }
}


fn nop(cpu: &mut CPU, am: AddrMode) {
    if let AddrMode::Imp = am {
        // Note: Imp does nothing, however since min cycles per instruction is 2,
        // an extra cycle is performed either way (outside of this function.)
        return;
    }

    // TODO: check properly every addressing mode
    cpu.operand_read_inc(am);   // +n cycles
}

fn pla(cpu: &mut CPU) {
    cpu.reg.a = cpu.pull();
    cpu.reg.p.z = cpu.reg.a == 0;
    cpu.reg.p.n = cpu.reg.a.test_bit(7)
}

macro_rules! a_op_val {
    ($fn_name: ident, $op: tt) => {
        fn $fn_name(cpu: &mut CPU, val: u8) {
            cpu.reg.a = cpu.reg.a $op val;
            cpu.reg.p.z = cpu.reg.a == 0;
            cpu.reg.p.n = cpu.reg.a.test_bit(7)
        }
    };
}


a_op_val!(and_val, &);
a_op_val!(ora_val, |);
a_op_val!(eor_val, ^);

macro_rules! a_op_fn {
    ($fn_name: ident, $op_fn: ident) => {
        fn $fn_name(cpu: &mut CPU, am: AddrMode) {
            let val = cpu.operand_read_inc(am);
            $op_fn(cpu, val);
        }
    };
}

a_op_fn!(and, and_val);
a_op_fn!(ora, ora_val);
a_op_fn!(eor, eor_val);

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

fn increment(cpu: &mut CPU, am: AddrMode, decrement: bool) -> u8 {
    let m = cpu.operand_read_cycle(am);
    let result = if decrement { m - 1 } else { m + 1 };
    cpu.reg.p.z = result == 0;
    cpu.reg.p.n = result.test_bit(7);
    cpu.operand_write_inc(am, result);
    result
}

fn inc(cpu: &mut CPU, am: AddrMode, decrement: bool) -> () {
    increment(cpu, am, decrement);
}

#[derive(Debug)]
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

fn add_with_carry(cpu: &mut CPU, a: u16, m: u16, c: u16) -> () {
    let result = a + m + c;

    cpu.reg.a = result as u8;
    cpu.reg.p.c = result > 0xff;
    cpu.reg.p.z = result as u8 == 0;
    cpu.reg.p.v = (!(a ^ m) & (a ^ result)).test_bit(7);
    cpu.reg.p.n = result.test_bit(7);
}

fn adc(cpu: &mut CPU, am: AddrMode, sbc: bool) -> () {
    let a = cpu.reg.a as u16;
    let m = (cpu.operand_read_inc(am) ^ (if sbc { 0xff } else { 0 })) as u16;
    let c = cpu.reg.p.c as u16;

    add_with_carry(cpu, a, m, c);
}

fn compare(cpu: &mut CPU, r: u8, m: u8) -> () {
    let result = r - m;

    cpu.reg.p.c = r >= m;
    cpu.reg.p.z = r == m;
    cpu.reg.p.n = result.test_bit(7);
}

fn cmp(cpu: &mut CPU, am: AddrMode, instr_reg: InstrReg) -> () {
    let r = instr_reg.get(cpu);
    let m = cpu.operand_read_inc(am);
    compare(cpu, r, m);
}

fn rot(cpu: &mut CPU, am: AddrMode, rotate: bool, left: bool) -> u8 {
    let shift_fn: fn(u8) -> u8 = if left { |x| x << 1 } else { |x| x >> 1 };
    let pad_amount = if left { 0 } else { 7 };
    let old_bit = if left { 7 } else { 0 };

    let val = cpu.operand_read_cycle(am);   // +n cycles (note: always +5 for ind y!)
    let new_val = shift_fn(val) | (if rotate { cpu.reg.p.c as u8 } else { 0 } << pad_amount);

    let val = match am {
        Acc => {
            cpu.reg.p.c = val.test_bit(old_bit);
            cpu.reg.a = new_val;
            new_val
        }
        ZP(ir) => {
            cpu.reg.p.c = val.test_bit(old_bit);
            cpu.zp_write_inc(new_val, ir); // +2(+1) cycles
            new_val
        }
        Abs(ir) => {
            cpu.reg.p.c = val.test_bit(old_bit);
            cpu.abs_write_inc(new_val, ir); // +3(+1) cycles
            new_val
            // = 5(+1)
        }
        // Used for unofficial instructions (rla, rra, slo, sre)
        Ind(ir) => {
            cpu.reg.p.c = val.test_bit(old_bit);
            cpu.ind_write_inc(new_val, ir); // +5 cycles
            new_val
        }
        _ => unreachable!()
    };
    cpu.reg.p.z = val == 0;
    cpu.reg.p.n = val.test_bit(7);
    val
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
                cpu.push(cpu.reg.pc.msb()); // +2
                cpu.push(cpu.reg.pc.lsb()); // +2

                // as a side effect of using nocycle, pc is already "pc-1"
                // (which is what _should_ be pushed)
                let val = cpu.pc_read_nocycle(); // !!TODO: check (jsr is 5 cycles (w/o opcode))
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
    let delta = if rti {
        cpu.reg.p = cpu.pull_noextra().into(); // +1
        0
    } else {
        cpu.cycle(); // +1
        1
    };
    let l = cpu.pull(); // +3
    let m = cpu.pull_noextra(); // +1
    let addr = as_address(l, m);
    cpu.reg.pc = addr + delta;
}

fn brk(cpu: &mut CPU) -> () {
    // TODO: interrupts
    cpu.push((cpu.reg.pc + 1).msb()); // +2
    cpu.push((cpu.reg.pc + 1).lsb()); // +2
    cpu.push(Into::<u8>::into(cpu.reg.p) | 0b0011_0000); // +2
    // cpu.reg.p.b = true; // ?

    let l = cpu.read_addr_nocycle(IRQ_VECTOR);
    let m = cpu.read_addr_nocycle(IRQ_VECTOR + 1);
    let addr = as_address(l, m);
    cpu.reg.pc = addr;
}

// Unofficial
macro_rules! rot_inst_unofficial {
    ($fn_name: ident, $fn: ident, adc) => {
        fn $fn_name(cpu: &mut CPU, am: AddrMode) {
            log::info!("Unofficial {} (rotate right) with addr mode: {am:?}", stringify!($fn_name));
            let val = $fn(cpu, am, true, false);

            let a = cpu.reg.a as u16;
            let m = val as u16;
            let c = cpu.reg.p.c as u16;

            add_with_carry(cpu, a, m, c);
        }
    };

    ($fn_name: ident, rot, $op: ident) => {
        fn $fn_name(cpu: &mut CPU, am: AddrMode, rotate: bool, left: bool) {
            log::info!("Unofficial {} ({} {}) with addr mode: {am:?}", stringify!($fn_name), if rotate {"rotate"} else {"shift"}, if left {"left"} else {"right"});
            let val = rot(cpu, am, rotate, left);

            $op(cpu, val)
        }
    };
}

rot_inst_unofficial!(rla, rot, and_val);
rot_inst_unofficial!(rra, rot, adc);
rot_inst_unofficial!(slo, rot, ora_val);
rot_inst_unofficial!(sre, rot, eor_val);

fn dcp(cpu: &mut CPU, am: AddrMode, isc: bool) {
    log::info!("Unofficial {} with addr mode: {am:?}", stringify!($fn_name));
    let val = increment(cpu, am, !isc);

    if isc {
        // SBC
        let a = cpu.reg.a as u16;
        let m = (val ^ 0xff) as u16;
        let c = cpu.reg.p.c as u16;

        add_with_carry(cpu, a, m, c);
    } else {
        // CMP
        compare(cpu, cpu.reg.a, val);
    }
}

fn alr(cpu: &mut CPU, am: AddrMode) {
    log::info!("Unofficial alr with addr mode: {am:?}");
    let val = cpu.operand_read_inc(am);
    and_val(cpu, val);

    rot(cpu, AddrMode::Acc, false, false);
}

fn anc(cpu: &mut CPU, am: AddrMode) {
    log::info!("Unofficial anc with addr mode: {am:?}");
    let val = cpu.operand_read_inc(am);
    and_val(cpu, val);

    cpu.reg.p.c = cpu.reg.p.n;
}

fn arr(cpu: &mut CPU, am: AddrMode) {
    log::info!("Unofficial arr with addr mode: {am:?}");
    let val = cpu.operand_read_inc(am);
    and_val(cpu, val);

    rot(cpu, AddrMode::Acc, true, false);

    let a = cpu.reg.a;
    cpu.reg.p.c = a.test_bit(6);
    cpu.reg.p.v = a.test_bit(6) ^ a.test_bit(5);
}

fn axs(cpu: &mut CPU, am: AddrMode) {
    log::info!("Unofficial axs with addr mode: {am:?}");
    let val = cpu.operand_read_inc(am);

    let a = cpu.reg.a as u16;
    let x = cpu.reg.x as u16;
    let m = val as u16;

    let result = (a & x) - m;

    cpu.reg.x = result as u8;

    cpu.reg.p.z = result as u8 == 0;
    cpu.reg.p.c = result < 0x100;
    cpu.reg.p.n = result.test_bit(7);
}

fn lax(cpu: &mut CPU, am: AddrMode) {
    log::info!("Unofficial lax with addr mode: {am:?}");
    ld(cpu, am, InstrReg::A);
    cpu.reg.x = cpu.reg.a;
}

fn sax(cpu: &mut CPU, am: AddrMode) {
    log::info!("Unofficial sax with addr mode: {am:?}");
    let val = cpu.reg.a & cpu.reg.x;
    cpu.operand_write_inc(am, val);
}

// Unstable
fn shn(cpu: &mut CPU, am: AddrMode, instr_reg: InstrReg) {
    // TODO: make the instruction perform as expected.
    log::info!("Unofficial sh{instr_reg:?} with addr mode: {am:?}");

    // {addr} = A & N & {H+1}

    // "An incorrectly-implemented version of STX a,Y. [<-- in the case of SHX]
    // Unless interrupted by DMC DMA on the 4th clock
    // (i.e. RDY goes low between fetching the high byte of the address and the dummy read),
    // data written is ANDed with (high byte of literal address +1).
    // In case there's a page crossing, the high byte of the computed address is ANDed with X
    // regardless of what RDY does."

    let addr = match am {
        AddrMode::Ind(_) => {
            // Note: ir will always be Y
            let zp = cpu.pc_read_inc();  // +1 cycle
            let l = cpu.read_addr_cycle(as_address(zp, 0x00));      // +1 cycle
            let m = cpu.read_addr_cycle(as_address(zp + 1, 0x00));  // +1 cycle
            as_address(l, m)
        }
        _ => {
            let l = cpu.pc_read_inc();  // +1 cycle
            let m = cpu.pc_read_inc();  // +1 cycle
            as_address(l, m)
        }
    };

    let to_write_val = match instr_reg {
        InstrReg::A => cpu.reg.a & cpu.reg.x,
        InstrReg::X => cpu.reg.y,
        InstrReg::Y => cpu.reg.x,
    };

    let ind_reg_val = match am {
        AddrMode::Abs(X) => cpu.reg.x,
        AddrMode::Abs(Y) | AddrMode::Ind(Y) => cpu.reg.y,
        _ => unreachable!(),
    };


    // "Unless interrupted by DMC DMA on the 4th clock
    // (i.e. RDY goes low between fetching the high byte of the address and the dummy read),
    // data written is ANDed with (high byte of literal address +1)."
    let cycles_before = cpu.cycles;

    let pre_addr = addr + ind_reg_val as u16;
    let page_crossing = (addr & 0xff00) != (pre_addr as u16) & 0xff00;
    cpu.read_addr_cycle(pre_addr);  // Dummy read. +1 cycle i.e. the 4th / 5th cycle

    let dma_interrupted = cpu.cycles - cycles_before > 1;

    // "In case there's a page crossing, the high byte of the computed address is ANDed with X
    // regardless of what RDY does."
    let write_addr = if page_crossing {
        // "If the target address crosses a page boundary because of indexing, the instruction may not
        // store at the intended address. Instead the high byte of the target address will get
        // incremented as expected, and then ANDed with the value stored"
        as_address(pre_addr.lsb(), (pre_addr.msb() + 1) & to_write_val)
    } else {
        pre_addr
    };

    log::info!("addrs: {addr:04x}, {pre_addr:04x}, {write_addr:04x}");

    if dma_interrupted {
        log::info!("DMA interrupted!!!");
        // DMA occurred
        // "Unless interrupted by DMC DMA on the 4th clock [...]"
        cpu.write_addr_cycle(write_addr, to_write_val);
    } else {
        cpu.write_addr_cycle(write_addr, to_write_val & (pre_addr.msb() + 1));
    }
}

fn tas(cpu: &mut CPU, am: AddrMode) {
    log::info!("Unofficial tas with addr mode: {am:?}");
    log::info!("(uses sha, as seen in the next line)");
    shn(cpu, am, InstrReg::A);
    cpu.reg.sp = cpu.reg.a & cpu.reg.x;
}

fn las(cpu: &mut CPU, am: AddrMode) {
    log::info!("Unofficial las with addr mode: {am:?}");
    let val = cpu.operand_read_inc(am);
    let sp = cpu.reg.sp;

    let result = val & sp;

    cpu.reg.a = result;
    cpu.reg.x = result;
    cpu.reg.sp = result;

    cpu.reg.p.z = result == 0;
    cpu.reg.p.n = result.test_bit(7);
}

// Unpredictable
fn xaa(cpu: &mut CPU, am: AddrMode) {
    log::info!("Unofficial xaa with addr mode: {am:?}");

    // This instruction depends on analog behavior... so instead we simply use 0xff as magic.
    let magic = 0xff;
    let a = cpu.reg.a;
    let x = cpu.reg.x;
    let val = cpu.operand_read_inc(am);

    let result = (a | magic) & x & val;

    cpu.reg.a = result;
    cpu.reg.p.z = result == 0;
    cpu.reg.p.n = result.test_bit(7);
}

fn stp(cpu: &mut CPU, opcode: u8) -> () {
    log::warn!("Hit stp instruction (opcode ${opcode:02x}), CPU will be halted.");
    cpu.halted = true;
}
