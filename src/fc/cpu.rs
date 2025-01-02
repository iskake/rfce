use inst::*;

use crate::bits::{as_address, Addr, Bitwise};
use crate::fc::mem::*;

use super::PPU;

pub mod inst;

pub const CPU_FREQ: u64 = 1_789_773;

pub const MASTER_FREQ: u64 = 21_477_272;
pub const MASTER_FREQ_60HZ: u64 = 21_441_960;

pub const NMI_VECTOR: u16 = 0xfffa;
pub const RESET_VECTOR: u16 = 0xfffc;
pub const IRQ_VECTOR: u16 = 0xfffe;

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
        self.c as u8
            | (self.z as u8) << 1
            | (self.i as u8) << 2
            | (self.d as u8) << 3
            | (self.b as u8) << 4
            // | 0b0010_0000
            | (self.v as u8) << 6
            | (self.n as u8) << 7
    }
}

pub struct Registers {
    a: u8,        // Accumulator
    x: u8,        // X index register
    y: u8,        // Y index register
    p: ProcFlags, // Processor status
    sp: u8,       // Stack pointer
    pc: u16,      // Program counter
    irq: bool,    // Interrupt request flag
    nmi: bool,    // Non-maskable interrupt flag
    dma: bool,    // OAM DMA 'flag'
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            a: 0x00,
            x: 0x00,
            y: 0x00,
            p: 0b0000_0100.into(),
            sp: 0xfd,
            pc: RESET_VECTOR,
            irq: false,
            nmi: false,
            dma: false,
        }
    }
}

pub struct CPU {
    reg: Registers,
    pub mem: MemMap, // TODO?: move this somewhere else and have a reference?
    pub ppu: PPU,
    cycles: u64,
}

impl CPU {
    pub fn new(mem: MemMap, ppu: PPU) -> CPU {
        CPU {
            reg: Registers::new(),
            mem,
            ppu,
            cycles: 0,
        }
    }

    pub fn print_state(&self) -> () {
        println!("CPU STATE:");
        println!(
            "  a: {:02x}, x: {:02x}, y: {:02x}",
            self.reg.a, self.reg.x, self.reg.y
        );
        let p: u8 = self.reg.p.into();
        println!("  p: {:02x} ({:08b})", p, p);
        println!("  sp:{:02x} pc:{:04x}", self.reg.sp, self.reg.pc);
        println!(
            "  cycles: {}, nmi: {}, irq: {}",
            self.cycles, self.reg.nmi, self.reg.irq
        );
        self.ppu.print_state();

        let inst = self.fetch_next_inst_nocycle();
        let operand_u8 = self.mem_read_no_sideeffect(self.reg.pc + 1);
        let operand_u16 = as_address(operand_u8, self.mem_read_no_sideeffect(self.reg.pc + 2));
        let operand_rel = (self.reg.pc + 2).wrapping_add_signed((operand_u8 as i8).into());

        println!(
            "-> {}",
            inst.to_string()
                .replace("_b", &format!("{operand_u8:02x}"))
                .replace("_w", &format!("{operand_u16:04x}"))
                .replace("_r", &format!("{operand_rel:04x}"))
        );
    }

    pub fn init(&mut self) -> () {
        self.ppu.init();
        let l = self.read_addr_nocycle(RESET_VECTOR);
        let m = self.read_addr_nocycle(RESET_VECTOR + 1);
        let addr = as_address(l, m);
        self.reg.pc = addr;
        // This is purely based on the value of the Mesen debugger after RESET
        self.cycles = 7;
    }

    pub fn reset(&mut self) -> () {
        self.ppu.reset();
        self.reg.pc = RESET_VECTOR;
        self.reg.sp -= 3;
        self.reg.p.i = true;

        self.init();
    }

    pub fn handle_nmi(&mut self) -> () {
        println!("NMI");

        self.push(self.reg.pc.msb()); // +2 cycles
        self.push(self.reg.pc.lsb()); // +2 cycles

        self.push(self.reg.p.into()); // +2 cycles

        let l = self.read_addr_nocycle(NMI_VECTOR);
        let m = self.read_addr_nocycle(NMI_VECTOR + 1);
        let addr = as_address(l, m);
        self.cycle(); // +1 cycle

        self.reg.pc = addr;
        self.reg.nmi = true;
    }

    fn handle_oam_dma(&mut self) {
        println!("OAM DMA");

        let src_msb = self.ppu.oamdma();
        for dst in 0..=255u8 {
            // ?Internally (or, at least in mesen,) OAMADDR is incremented for each read
            let src_addr = as_address(dst, src_msb);

            let val = self.read_addr_cycle(src_addr); // +1 cycle

            self.ppu.write_oam(dst, val);
            self.cycle(); // +1 cycle
        }
        // "The copy takes 513 or 514 cycles"
        self.cycle(); // +1 cycle

        self.reg.dma = false;
    }

    /// "Cycle" the cpu.
    ///
    /// Cycles: `1`
    pub fn cycle(&mut self) -> () {
        self.cycles += 1;
        self.ppu.cycle(&mut self.mem);
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        match addr {
            0x2000..=0x3fff => self.ppu.read_mmio((addr % 8) + 0x2000, &mut self.mem),
            _ => self.mem.read(addr),
        }
    }

    // TODO: better function name
    fn mem_read_no_sideeffect(&self, addr: u16) -> u8 {
        match addr {
            0x2000..=0x3fff => self.ppu.read_mmio_no_sideeffect((addr % 8) + 0x2000),
            _ => self.mem.read(addr),
        }
    }

    fn mem_write(&mut self, addr: u16, val: u8) -> () {
        match addr {
            0x2000..=0x3fff => self.ppu.write_mmio((addr % 8) + 0x2000, val, &mut self.mem),
            0x4014 => {
                self.ppu.write_oamdma(val);
                self.reg.dma = true
            }
            _ => self.mem.write(addr, val),
        };
    }

    /// Read the value at the address `addr` without any cycles (including in the PPU).
    ///
    /// Cycles: `0`
    pub fn read_addr_nocycle(&self, addr: u16) -> u8 {
        self.mem_read_no_sideeffect(addr)
    }

    pub fn read_addr_ppu(&self, addr: u16) -> u8 {
        // TODO: make sure no side effects happen because of this
        self.ppu.read_addr(addr, &self.mem)
    }

    /// Read the value at the address `addr`
    ///
    /// Cycles: `1`
    pub fn read_addr_cycle(&mut self, addr: u16) -> u8 {
        self.cycle();
        self.mem_read(addr)
    }

    /// Write the value `val` to the address `addr`
    ///
    /// Cycles: `1`
    pub fn write_addr_cycle(&mut self, addr: u16, val: u8) -> () {
        self.cycle();
        self.mem_write(addr, val);
    }

    /// Read the value pointe to by the pc without any cycles (including in the PPU).
    ///
    /// Cycles: `0`
    pub fn pc_read_nocycle(&self) -> u8 {
        self.mem_read_no_sideeffect(self.reg.pc)
    }

    /// Read the value pointe to by the pc.
    ///
    /// Cycles: `1`
    pub fn pc_read(&mut self) -> u8 {
        self.cycle();
        self.mem_read(self.reg.pc)
    }

    /// Increment the pc.
    ///
    /// Cycles: `0`
    pub fn pc_inc(&mut self) -> () {
        self.reg.pc += 1; // https://www.nesdev.org/wiki/Cycle_counting#Instruction_timings
    }

    /// Read the value pointed to by the pc, then increment the pc.
    ///
    /// Cycles: `1`
    pub fn pc_read_inc(&mut self) -> u8 {
        let val = self.pc_read(); // +1 cycle
        self.pc_inc();
        val
    }

    /// Add an offset to the value of the pc
    ///
    /// Cycles: `1` (`+1` if to a new page)
    fn pc_offset_cycle(&mut self, offset: i8) -> () {
        let old_pc = self.reg.pc;
        let new_pc = old_pc.wrapping_add_signed(offset.into());
        if new_pc.msb() != old_pc.msb() {
            self.cycle(); // +1 cycle
        }
        self.reg.pc = new_pc;
        self.cycle(); // +1 cycle
    }

    /// Read the value at address $00(pc+x/y).
    ///
    /// Cycles: `2` (if page crossed)
    pub fn zp_read_cycle(&mut self, ir: IndexRegister) -> u8 {
        let operand = self.pc_read(); // +1 cycle
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
    /// Cycles: `2` (`+1` if x/y)
    pub fn zp_read_inc(&mut self, ir: IndexRegister) -> u8 {
        let operand = self.pc_read_inc(); // +1 cycle
        let delta = match ir {
            IndexRegister::N => 0,
            IndexRegister::X => {
                self.cycle();
                self.reg.x
            } // +1 cycle
            IndexRegister::Y => {
                self.cycle();
                self.reg.y
            } // +1 cycle
        };
        // if operand as u16 + delta as u16 > 0xff {
        // self.cycle();
        // }
        self.read_addr_cycle(as_address(operand + delta, 0x00)) // +1 cycle
    }

    /// Write the value `val` to the address $00(pc+x/y), and increment the pc.
    ///
    /// Cycles: `2` (`+1` if x/y)
    pub fn zp_write_inc(&mut self, val: u8, ir: IndexRegister) -> () {
        let operand = self.pc_read_inc(); // +1 cycle
        let delta = match ir {
            IndexRegister::N => 0,
            IndexRegister::X => {
                self.cycle();
                self.reg.x
            } // always +1 cycle
            IndexRegister::Y => {
                self.cycle();
                self.reg.y
            } // always +1 cycle
        };
        let addr = as_address(operand + delta, 0x00);
        self.write_addr_cycle(addr, val); // +1 cycle
    }

    /// Read the value at address $(pc)<<8|(pc+1) +x/y
    ///
    /// Cycles: `2`
    pub fn abs_read_cycle(&mut self, ir: IndexRegister) -> u8 {
        let l = self.pc_read(); // +1 cycle

        // "m: u8 = self.pc_read(pc+1);"
        let m = {
            self.cycle();
            self.mem_read(self.reg.pc + 1)
        }; // +1 cycle

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
        self.read_addr_nocycle(addr + delta) // +0 cycle
    }

    /// Read the value at address $(pc)<<8|(pc+1) +x/y, and increment the pc.
    ///
    /// Cycles: `3` (`+1` if page crossed)
    pub fn abs_read_inc(&mut self, ir: IndexRegister) -> u8 {
        let l = self.pc_read_inc(); // +1 cycle
        let m = self.pc_read_inc(); // +1 cycle
        let addr = as_address(l, m);
        let delta = match ir {
            IndexRegister::N => 0,
            IndexRegister::X => self.reg.x, // +1 cycle
            IndexRegister::Y => self.reg.y, // +1 cycle
        } as u16;
        if (addr & 0xff00) != (addr + delta) & 0xff00 {
            self.cycle();
        }
        self.read_addr_cycle(addr + delta) // +1 cycle
    }

    /// Write the value `val` to the address $(pc)<<8|(pc+1) +x/y, and increment the pc.
    ///
    /// Cycles: `3`(`+1` for x/y)
    pub fn abs_write_inc(&mut self, val: u8, ir: IndexRegister) -> () {
        let l = self.pc_read_inc(); // +1 cycle
        let m = self.pc_read_inc(); // +1 cycle
        let addr = as_address(l, m);
        let delta = match ir {
            IndexRegister::N => 0,
            IndexRegister::X => {
                self.cycle();
                self.reg.x
            }
            IndexRegister::Y => {
                self.cycle();
                self.reg.y
            }
        } as u16;
        // ?from wiki: "assumes the worst case of page crossing and always spends 1 extra read cycle"
        self.write_addr_cycle(addr + delta, val); // +1 cycle
    }

    /// Read the operand in the way specified by the addressing mode, without increasing the pc
    ///
    /// Cycles:
    /// - Acc: `0`
    /// - Imm: `1`
    /// - Zp : `2`
    /// - Abs: `2`
    pub fn operand_read_cycle(&mut self, am: AddrMode) -> u8 {
        match am {
            // TODO: shouldn't it really be the write func that has the fewer cycle things?
            AddrMode::Acc => self.reg.a,
            AddrMode::Imm => self.pc_read(),              // +1
            AddrMode::ZP(ir) => self.zp_read_cycle(ir),   // +2
            AddrMode::Abs(ir) => self.abs_read_cycle(ir), // +2
            AddrMode::Ind(_) => panic!("Operand not needed for any such instructions."),
            AddrMode::Imp => panic!("Implied does not have an operand"),
            AddrMode::Rel => panic!("Relative operand is only used in branch instructions"),
        }
    }

    /// Read the operand in the way specified by the addressing mode
    ///
    /// Cycles:
    /// - Acc: `0`
    /// - Imm: `1`
    /// - Zp : `2`(`+1` for x/y page crossing)
    /// - Abs: `3`(`+1` for x/y page crossing)
    /// - Ind: `5`("`-1`"" y NOT page crossing)
    pub fn operand_read_inc(&mut self, am: AddrMode) -> u8 {
        match am {
            AddrMode::Acc => self.reg.a,
            AddrMode::Imm => self.pc_read_inc(),        // +1
            AddrMode::ZP(ir) => self.zp_read_inc(ir),   // +2(+1)
            AddrMode::Abs(ir) => self.abs_read_inc(ir), // +3(+1)
            AddrMode::Ind(ir) => self.ind_read_inc(ir), // +5(-1)
            AddrMode::Imp => panic!("Implied does not have an operand"),
            AddrMode::Rel => panic!("Relative operand is only used in branch instructions"),
        }
    }

    /// Write `val` to the operand in the way specified by the addressing mode
    ///
    /// Cycles:
    /// - Zp : `2`(`+1` for x/y)
    /// - Abs: `3`(`+1` for x/y)
    /// - Ind: `5` for both x/y
    pub fn operand_write_inc(&mut self, am: AddrMode, val: u8) {
        match am {
            AddrMode::ZP(ir) => self.zp_write_inc(val, ir), // +2(+1 x/y)
            AddrMode::Abs(ir) => self.abs_write_inc(val, ir), // +3
            AddrMode::Ind(ir) => self.ind_write_inc(val, ir), // +5
            AddrMode::Acc => unreachable!(),
            AddrMode::Imp => unreachable!(),
            AddrMode::Imm => unreachable!(),
            AddrMode::Rel => unreachable!(),
        }
    }

    /// Get the indirect address ... yeah
    ///
    /// Cycles: `2`
    pub fn get_indirect(&mut self, addr: u16) -> u16 {
        if addr & 0xff == 0x00ff {
            // Wraparound
            let l = self.read_addr_cycle((addr & 0xff00) | 0xff); // +1 cycle
            let m = self.read_addr_cycle((addr & 0xff00) | 0x00); // +1 cycle
            as_address(l, m)
        } else {
            let l = self.read_addr_cycle(addr); // +1 cycle
            let m = self.read_addr_cycle(addr + 1); // +1 cycle
            as_address(l, m)
        }
    }

    /// Read the indirect address in relation to x or y
    ///
    /// Cycles: `5` for x, `4` (`+1` if page crossed) for y
    pub fn ind_read_inc(&mut self, ir: IndexRegister) -> u8 {
        let m = self.pc_read_inc(); // +1 cycle
        match ir {
            IndexRegister::X => {
                let ptr = as_address(m + self.reg.x, 0x00);
                let addr = self.get_indirect(ptr); // +2 cycles
                self.cycle(); //?? +1 cycle extra??
                self.read_addr_cycle(addr) // +1 cycle
            }
            IndexRegister::Y => {
                let ptr = as_address(m, 0x00);
                let delta = self.reg.y as u16;
                let addr = self.get_indirect(ptr); // +2 cycles
                if (addr & 0xff00) != ((addr + delta) & 0xff00) {
                    self.cycle(); // +1 cycle
                }
                self.read_addr_cycle(addr + delta) // +1 cycle
            }
            IndexRegister::N => unreachable!(),
        }
    }

    /// Write `val` to the indirect address in relation to x or y
    ///
    /// Cycles: `5` for x and y
    pub fn ind_write_inc(&mut self, val: u8, ir: IndexRegister) -> () {
        let pcval = self.pc_read_inc(); // +1 cycle
        match ir {
            IndexRegister::X => {
                let ptr = as_address(pcval + self.reg.x, 0x00);
                let addr = self.get_indirect(ptr as u16); // +2 cycles
                self.cycle(); // +1 cycle extra
                self.write_addr_cycle(addr, val) // +1 cycle
            }
            IndexRegister::Y => {
                let ptr = as_address(pcval, 0x00);
                let delta = self.reg.y as u16;
                let addr = self.get_indirect(ptr) + delta; // +2 cycles
                self.cycle(); // +1 cycle extra
                self.write_addr_cycle(addr, val) // +1 cycle
            }
            IndexRegister::N => unreachable!(),
        }
    }

    /// Pull from the stack, increasing sp by 1
    ///
    /// Cycles: `3`
    fn pull(&mut self) -> u8 {
        self.cycle(); // +1
        self.reg.sp += 1;
        self.cycle(); // +1
        self.read_addr_cycle(as_address(self.reg.sp, 0x01)) // +1
    }

    /// Pull from the stack, increasing sp by 1. Does NOT 'waste' any extra cycles
    ///
    /// Cycles: `1`
    fn pull_noextra(&mut self) -> u8 {
        self.reg.sp += 1;
        self.read_addr_cycle(as_address(self.reg.sp, 0x01)) // +1
    }

    /// Push `val` onto the stack, decreasing sp by 1
    ///
    /// Cycles: `2`
    fn push(&mut self, val: u8) -> () {
        self.write_addr_cycle(as_address(self.reg.sp, 0x01), val); // +1
        self.reg.sp -= 1;
        self.cycle(); // +1
    }

    fn fetch_next_op(&mut self) -> u8 {
        self.pc_read_inc() // 1 cycle
    }

    fn fetch_next_inst(&mut self) -> Inst {
        INST_TABLE[self.fetch_next_op() as usize] // 1 cycle
    }

    fn fetch_next_inst_nocycle(&self) -> Inst {
        INST_TABLE[self.pc_read_nocycle() as usize]
    }

    fn run_inst(&mut self, inst: Inst) -> () {
        inst.run(self);
    }

    pub fn fetch_and_run(&mut self) -> () {
        let cycles_before = self.cycles;

        // OAM DMA handling
        if self.reg.dma {
            self.handle_oam_dma();
        }

        let inst = self.fetch_next_inst(); // 1 cycle
        self.run_inst(inst); // n cycles

        let cycles_after = self.cycles;

        if cycles_after - cycles_before == 1 {
            self.cycle();
        }

        // Interrupt handling
        if self.ppu.should_do_nmi() && !self.reg.nmi {
            self.handle_nmi();
        } else if self.reg.nmi && !self.ppu.nmi_enable() {
            self.reg.nmi = false;
        }
    }

    pub fn run_to_vblank(&mut self) -> () {
        let start = std::time::Instant::now();
        loop {
            let before = self.ppu.is_vblank();
            self.fetch_and_run();
            let after = self.ppu.is_vblank();

            if after && after != before {
                let end = start.elapsed();
                println!("Time: {:.2?}", end);
                self.ppu.generate_nametables_image_temp(&mut self.mem);
                break;
            }
        }
    }

    pub fn fetch_and_run_dbg(&mut self) -> () {
        let cycles_before = self.cycles;

        self.fetch_and_run();

        let cycles_after = self.cycles;

        println!("took {} cycles", cycles_after - cycles_before);
        // let (op, inst) = self.fetch_next_op_inst(); // 1 cycle
        // self.run_inst(inst); // n cycles
        // let cycles_after = self.cycles;
        // print!(" inst {inst:?} (op: ${op:02x}) ");
        // if cycles_after - cycles_before == 1 {
        //     println!("took 2 cycles (added one extra)");
        //     self.cycle();
        // } else {
        //     println!("took {} cycles", cycles_after - cycles_before);
        // }
    }

    pub fn pc(&self) -> u16 {
        self.reg.pc
    }
}
