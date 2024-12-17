pub struct CPU {
    pub reg: Registers,
    pub mem: Memory, // TODO?: move this somewhere else?
}

impl CPU {
    pub fn lda(&mut self, am: AddrMode) -> () {
        // todo: everything lol
        let val = match am {
            AddrMode::Imm => self.mem[self.reg.pc as usize],
            _ => panic!("unimplemented!!! :O"),
        };
        let z = val == 0;
        let n =  test_bit(val, 7);
    
        self.reg.a = val;
        self.reg.p |= ((n as u8) << 7) | (z as u8) << 1;
    }
}

// TODO: create a proper data type to handle generic memory for cartridges and all that good stuff
type Memory = [u8; 0x10000];

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

// TODO: change visibility of various enums (from pub to private)
pub enum Inst { 
    ADC, AND, ASL, BCC, BCS, BEQ, BIT, BMI, BNE, BPL, BRK, BVC, BVS, CLC,
    CLD, CLI, CLV, CMP, CPX, CPY, DEC, DEX, DEY, EOR, INC, INX, INY, JMP,
    JSR, LDA, LDX, LDY, LSR, NOP, ORA, PHA, PHP, PLA, PLP, ROL, ROR, RTI,
    RTS, SBC, SEC, SED, SEI, STA, STX, STY, TAX, TAY, TSX, TXA, TXS, TYA,
    ILL,STP,
}

pub enum IndexRegsiter {
    None,
    X,
    Y,
}

pub enum AddrMode {
    Imp,   // Implicit
    Acc,   // Accumulator
    Imm,   // Immediate
    ZP(IndexRegsiter),    // Zero Page ($0000-$00ff)
    // ZPX,   // Zero Page,X
    // ZPY,   // Zero Page,Y
    Rel,   // Relative
    Abs(IndexRegsiter),   // Absolute
    // AbsX,  // Absolute,X
    // AbsY,  // Absolute,Y
    Ind(IndexRegsiter),   // Indirect
    // IndX,  // Indexed Indirect (X)
    // IndY,  // Indexed Indirect (Y)
}

fn test_bit(x: u8, i: u8) -> bool {
    ((x & (1 << i)) >> i) == 1
}