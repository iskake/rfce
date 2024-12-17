#[derive(PartialEq)]
pub enum IndexRegister {
    None,
    X,
    Y,
}

pub enum AddrMode {
    Imp,   // Implicit
    Acc,   // Accumulator
    Imm,   // Immediate
    ZP(IndexRegister),    // Zero Page ($0000-$00ff)
    // ZPX,   // Zero Page,X
    // ZPY,   // Zero Page,Y
    Rel,   // Relative
    Abs(IndexRegister),   // Absolute
    // AbsX,  // Absolute,X
    // AbsY,  // Absolute,Y
    Ind(IndexRegister),   // Indirect
    // IndX,  // Indexed Indirect (X)
    // IndY,  // Indexed Indirect (Y)
}

pub enum Inst { 
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC,
    CLD, CLI, CLV, CMP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP,
    JSR, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, ROL, ROR, RTI,
    RTS, SBC, SEC, SED, SEI, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA,
    ILL,STP,
}
