use super::mem::{MemMap, Memory};

const VRAM_SIZE: usize = NAMETABLE_SIZE * 4;
const PATTERN_TABLE_SIZE: usize = 0x1000;
const NAMETABLE_SIZE: usize = 0x0400;
const NAMETABLE_ATTRIBUTE_TABLE_SIZE: usize = 64;

const OAM_SIZE: usize = 64;
const SPRITE_SIZE: usize = 4;
const PALETTE_RAM_SIZE: usize = 0x20;

const PIXEL_SIZE: usize = 1;    // ??

const PICTURE_WIDTH:  usize = 256;
const PICTURE_HEIGHT: usize = 240;

const SCANLINE_DURATION: u64 = 341;
const FRAME_SCANLINES: u64 = 262;
const FRAME_DURATION_EVEN: u64 = SCANLINE_DURATION * FRAME_SCANLINES;
const FRAME_DURATION_ODD:  u64 = FRAME_DURATION_EVEN - 1;

const ADDRESS_PPUCTRL:   u16 = 0x2000;
const ADDRESS_PPUMASK:   u16 = 0x2001;
const ADDRESS_PPUSTATUS: u16 = 0x2002;
const ADDRESS_OAMADDR:   u16 = 0x2003;
const ADDRESS_OAMDATA:   u16 = 0x2004;
const ADDRESS_PPUSCROLL: u16 = 0x2005;
const ADDRESS_PPUADDR:   u16 = 0x2006;
const ADDRESS_PPUDATA:   u16 = 0x2007;
const ADDRESS_OAMDMA:    u16 = 0x4014;

pub struct PPU {
    pub mmio: MMIORegisters,
    // pub chr: Box<dyn Mapper>,
    pub pal: [u8; PALETTE_RAM_SIZE],
    pub oam: [u8; OAM_SIZE * SPRITE_SIZE],
    pub vram: [u8; VRAM_SIZE],
    cycles: u64,
    scanlines: u64,
    frame: u64,
    odd_frame: bool,
    frame_buf: Vec<u8>,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            mmio: MMIORegisters::new(),
            // chr: Box::new([0; PATTERN_TABLE_SIZE * 2].to_vec()),
            pal: [0; PALETTE_RAM_SIZE],
            oam: [0; OAM_SIZE * SPRITE_SIZE],
            vram: [0; VRAM_SIZE],
            cycles: 0,
            scanlines: 0,
            frame: 0,
            odd_frame: false, //?
            frame_buf: Vec::with_capacity(PICTURE_HEIGHT * PICTURE_WIDTH * PIXEL_SIZE),
        }
    }

    pub(super) fn cycles(&self) -> u64 {
        self.cycles
    }

    pub(super) fn scanlines(&self) -> u64 {
        self.scanlines
    }

    pub fn init(&mut self) -> () {
        // Once again, this is purely based on the value of the Mesen debugger after RESET
        self.cycles = 25;
        self.frame = 1;
        self.odd_frame = true; //? is this needed anymore?
    }

    pub fn cycle(&mut self, mem: &mut MemMap) -> () {
        let val = mem.mapper.read_chr(0x0000);
        // println!("CHR[0]: {}", val);
        // TODO: should this call some other fn 3 times instead?
        for _ in 0..3 {
            assert!(self.cycles < SCANLINE_DURATION);
            assert!(self.scanlines < FRAME_SCANLINES);

            // TODO
            match self.scanlines {
                0..=239 => {    // rendering (visible scanlines)
                    // TODO
                    match self.cycles {
                        0 => {},         // idle cycle
                        1..=256 => {},   // vram fetch/update
                        257..=320 => {}, // fetch tile data for sprites on the next scanline
                        321..=336 => {}, // fetch tile data for first two tiles for the next scanline
                        337..=340 => {}, // fetch two bytes
                        _ => unreachable!(),
                    }
                },
                240 => {},      // idle (post-render scanline)
                241..=260 => {  // vblank
                    if self.cycles == 1 && self.scanlines == 241 {
                        self.mmio.enable_vblank_flag();
                        println!("enabled vblank")
                    }
                },
                261 => {},      // dummy scanline (pre-render scanline)
                _ => unreachable!(),
            }

            self.cycles += 1;
            if self.cycles >= SCANLINE_DURATION {
                // TODO? do something more..?
                self.cycles = 0;
                self.scanlines +=1;

                if self.scanlines >= FRAME_SCANLINES {
                    // TODO? do something..?
                    self.scanlines = 0;
                }
            }
        }
    }

    fn read(&mut self, addr: u16, mem: &mut MemMap) -> u8 {
        match addr {
            0x0000..=0x1fff => mem.mapper.read_chr(addr),
            0x2000..=0x2fff => todo!(), //mem.mapper.read_nametable(addr),
            0x3f00..=0x3fff => self.pal[((addr - 0x3f00) % 20) as usize],
            _ => unreachable!(),
        }
    }

    fn write(&mut self, addr: u16, val: u8, mem: &mut MemMap) -> () {
        match addr {
            0x0000..=0x1fff => mem.mapper.write_chr(addr, val),
            0x2000..=0x2fff => todo!(), //mem.mapper.write_nametable(addr, val),
            0x3f00..=0x3fff => self.pal[((addr - 0x3f00) % 20) as usize] = val,
            _ => unreachable!(),
        }
    }
}

pub struct MMIORegisters {
    io_bus: u8,
    ppuctrl: u8,
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

impl MMIORegisters {
    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            ADDRESS_PPUCTRL   => 0xff,
            ADDRESS_PPUMASK   => 0xff,
            ADDRESS_PPUSTATUS => self.read_ppustatus(),
            ADDRESS_OAMADDR   => 0xff,
            ADDRESS_OAMDATA   => self.read_oamdata(),
            ADDRESS_PPUSCROLL => 0xff,
            ADDRESS_PPUADDR   => 0xff,
            ADDRESS_PPUDATA   => self.read_ppudata(),
            ADDRESS_OAMDMA    => 0xff,
            _ => todo!(),
        }
    }

    pub fn write(&mut self, addr: u16, val: u8) -> () {
        match addr {
            ADDRESS_PPUCTRL   => { self.io_bus = val; self.write_ppuctrl(val)},
            ADDRESS_PPUMASK   => { self.io_bus = val; self.write_ppumask(val)},
            ADDRESS_PPUSTATUS => { self.io_bus = val; },
            ADDRESS_OAMADDR   => { self.io_bus = val; self.write_oamaddr(val)},
            ADDRESS_OAMDATA   => { self.io_bus = val; self.write_oamdata(val)},
            ADDRESS_PPUSCROLL => { self.io_bus = val; self.write_ppuscroll(val)},
            ADDRESS_PPUADDR   => { self.io_bus = val; self.write_ppuaddr(val)},
            ADDRESS_PPUDATA   => { self.io_bus = val; self.write_ppudata(val)},
            ADDRESS_OAMDMA    => { self.io_bus = val; self.write_oamdma(val)},
            _ => todo!(),
        }
    }

    pub fn new() -> MMIORegisters {
        MMIORegisters {
            io_bus: 0x00,
            ppuctrl: 0b0000_0000,
            ppumask: 0b0000_0000,
            ppustatus: 0b0000_0000, //?
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
        // TODO: check for cycles < 29658 before allowing writes
        self.ppuctrl = val;
        println!("Wrote ${val:02x} to PPUCTRL")
    }

    pub fn write_ppumask(&mut self, val: u8) {
        todo!("write_ppumask")
    }

    pub fn read_ppustatus(&mut self) -> u8 {
        let val = self.ppustatus | (self.io_bus & 0b1_1111);
        println!("Read ${val:02x} from PPUSTATUS");
        self.disable_vblank_flag();
        val
    }

    pub fn enable_vblank_flag(&mut self) -> () {
        self.ppustatus |= 0b1000_0000;
    }

    pub fn disable_vblank_flag(&mut self) -> () {
        self.ppustatus &= 0b0111_1111;
    }

    pub fn write_oamaddr(&mut self, val: u8) {
        todo!("write_oamaddr")
    }

    pub fn read_oamdata(&self) -> u8 {
        todo!("read_oamdata")
    }

    pub fn write_oamdata(&mut self, val: u8) {
        todo!("write_oamdata")
    }

    pub fn write_ppuscroll(&mut self, val: u8) {
        todo!("write_ppuscroll")
    }

    pub fn write_ppuaddr(&mut self, val: u8) {
        todo!("write_ppuaddr")
    }

    pub fn read_ppudata(&self) -> u8 {
        todo!("read_ppudata")
    }

    pub fn write_ppudata(&mut self, val: u8) {
        todo!("write_ppudata")
    }

    pub fn write_oamdma(&mut self, val: u8) {
        todo!("write_oamdma")
    }
}
