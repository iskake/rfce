use super::mem::{MemMap, Memory};

const OAM_SIZE: usize = 64;
const SPRITE_SIZE: usize = 4;
const PATTERN_TABLE_SIZE: usize = 0x1000;

pub struct PPU {
    pub mmio: MMIORegisters,
    // pub chr: Box<dyn Mapper>,
    pub oam: [u8; OAM_SIZE * SPRITE_SIZE],
    cycles: u64,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            mmio: MMIORegisters::new(),
            // chr: Box::new([0; PATTERN_TABLE_SIZE * 2].to_vec()),
            oam: [0; OAM_SIZE * SPRITE_SIZE],
            cycles: 0,
        }
    }

    pub(super) fn cycles(&self) -> u64 {
        self.cycles
    }

    pub fn init(&mut self) -> () {
        // Once again, this is purely based on the value of the Mesen debugger after RESET
        self.cycles = 25;
    }

    pub fn cycle(&mut self, mut mem: &MemMap) -> () {
        let val = mem.mapper.read_chr(0x0000);
        println!("CHR[0]: {}", val);
        // TODO: should this call some other fn 3 times instead?
        self.cycles += 3;
    }
}

pub struct MMIORegisters {
    pputcl: u8,
    ppumask: u8,
    ppustatus: u8,
    oamaddr: u8,
    oamdata: u8,
    ppuscroll: u16,
    ppuscroll_read_once: bool,
    ppuaddr: u16,
    ppuaddr_read_once: bool,
    ppudata: u8,
    oamdma: u8,
}

impl Memory for MMIORegisters {
    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x2000 => 0xff,
            0x2001 => 0xff,
            0x2002 => self.read_ppustatus(),
            0x2003 => 0xff,
            0x2004 => self.read_oamdata(),
            0x2005 => 0xff,
            0x2006 => 0xff,
            0x2007 => self.read_ppudata(),
            0x4014 => 0xff,
            _ => todo!(),
        }
    }

    fn write(&mut self, addr: u16, val: u8) -> () {
        match addr {
            0x2000 => self.write_ppuctrl(val),
            0x2001 => self.write_ppumask(val),
            0x2002 => (),
            0x2003 => self.write_oamaddr(val),
            0x2004 => self.write_oamdata(val),
            0x2005 => self.write_ppuscroll(val),
            0x2006 => self.write_ppuaddr(val),
            0x2007 => self.write_ppudata(val),
            0x4014 => self.write_oamdma(val),
            _ => todo!(),
        }
    }
}

impl MMIORegisters {
    pub fn new() -> MMIORegisters {
        MMIORegisters {
            pputcl: 0b0000_0000,
            ppumask: 0b0000_0000,
            ppustatus: 0b1010_0000, //?
            oamaddr: 0x00,
            ppuscroll: 0x0000,
            ppuaddr: 0x0000,
            ppudata: 0x00,
            oamdata: 0x00, //?
            oamdma: 0x00,  //?
            ppuscroll_read_once: false,
            ppuaddr_read_once: false,
        }
    }

    pub fn write_ppuctrl(&mut self, val: u8) -> () {
        todo!()
    }

    pub fn write_ppumask(&mut self, val: u8) {
        todo!()
    }

    pub fn read_ppustatus(&self) -> u8 {
        todo!()
    }

    pub fn write_oamaddr(&self, val: u8) {
        todo!()
    }

    pub fn read_oamdata(&self) -> u8 {
        todo!()
    }

    pub fn write_oamdata(&mut self, val: u8) {
        todo!()
    }

    pub fn write_ppuscroll(&mut self, val: u8) {
        todo!()
    }

    pub fn write_ppuaddr(&mut self, val: u8) {
        todo!()
    }

    pub fn read_ppudata(&self) -> u8 {
        todo!()
    }

    pub fn write_ppudata(&mut self, val: u8) {
        todo!()
    }

    pub fn write_oamdma(&mut self, val: u8) {
        todo!()
    }
}
