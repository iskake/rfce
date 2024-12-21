use crate::bits::Bitwise;

use super::mem::MemMap;

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

const SCANLINE_DURATION: u32 = 341;
const FRAME_SCANLINES:   u32 = 262;
// const FRAME_DURATION_EVEN: u32 = SCANLINE_DURATION * FRAME_SCANLINES;
// const FRAME_DURATION_ODD:  u32 = FRAME_DURATION_EVEN - 1;

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
    reg: Registers,
    // pub chr: Box<dyn Mapper>,
    pub pal: [u8; PALETTE_RAM_SIZE],
    pub oam: [u8; OAM_SIZE * SPRITE_SIZE],
    pub vram: [u8; VRAM_SIZE],
    cycle: u32,
    scanline: u32,
    frame: u64,
    odd_frame: bool,
    frame_buf: Vec<u8>,
}

impl PPU {
    pub fn new() -> PPU {
        PPU {
            reg: Registers::new(),
            // chr: Box::new([0; PATTERN_TABLE_SIZE * 2].to_vec()),
            pal: [0; PALETTE_RAM_SIZE],
            oam: [0; OAM_SIZE * SPRITE_SIZE],
            vram: [0; VRAM_SIZE],
            cycle: 0,
            scanline: 0,
            frame: 0,
            odd_frame: false, //?
            frame_buf: Vec::with_capacity(PICTURE_HEIGHT * PICTURE_WIDTH * PIXEL_SIZE),
        }
    }

    pub(super) fn cycles(&self) -> u32 {
        self.cycle
    }

    pub(super) fn scanlines(&self) -> u32 {
        self.scanline
    }

    pub fn init(&mut self) -> () {
        // Once again, this is purely based on the value of the Mesen debugger after RESET
        self.cycle = 25;
        self.frame = 1;
        self.odd_frame = true; //? is this needed anymore?
    }

    pub fn cycle(&mut self, mem: &mut MemMap) -> () {
        // TODO: should this call some other fn 3 times instead?
        for _ in 0..3 {
            assert!(self.cycle < SCANLINE_DURATION);
            assert!(self.scanline < FRAME_SCANLINES);

            // TODO
            match self.scanline {
                0..=239 => {    // rendering (visible scanlines)
                    // TODO
                    match self.cycle {
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
                    if self.cycle == 1 && self.scanline == 241 {
                        self.reg.status.vblank = true;
                        println!("enabled vblank")
                    }
                },
                261 => {},      // dummy scanline (pre-render scanline)
                _ => unreachable!(),
            }

            self.cycle += 1;
            if self.cycle >= SCANLINE_DURATION {
                // TODO? do something more..?
                self.cycle = 0;
                self.scanline +=1;

                if self.scanline >= FRAME_SCANLINES {
                    // TODO? do something..?
                    self.scanline = 0;
                }
            }
        }
    }

    fn read_addr(&mut self, addr: u16, mem: &mut MemMap) -> u8 {
        match addr {
            0x0000..=0x1fff => mem.mapper.read_chr(addr),
            0x2000..=0x2fff => todo!(), //mem.mapper.read_nametable(addr),
            0x3f00..=0x3fff => self.pal[((addr - 0x3f00) % 20) as usize],
            _ => unreachable!(),
        }
    }

    fn write_addr(&mut self, addr: u16, val: u8, mem: &mut MemMap) -> () {
        match addr {
            0x0000..=0x1fff => mem.mapper.write_chr(addr, val),
            0x2000..=0x2fff => todo!(), //mem.mapper.write_nametable(addr, val),
            0x3f00..=0x3fff => self.pal[((addr - 0x3f00) % 20) as usize] = val,
            _ => unreachable!(),
        }
    }

    pub fn reset(&mut self) -> () {
        // TODO
        self.reg.control = 0x00.into();
        self.reg.mask = 0x00.into();
        self.reg.write_toggle = false;
        self.reg.x_y_scroll = 0x0000;
        self.reg.vram_data = 0x00;
        self.odd_frame = false; // ?
    }

    pub fn read_mmio(&mut self, addr: u16) -> u8 {
        self.reg.read(addr)
    }

    pub fn write_mmio(&mut self, addr: u16, val: u8) -> () {
        self.reg.write(addr, val);
    }
}

struct Registers {
    // $2000
    control: PPUControl,
    mask: PPUMask,
    status: PPUStatus,
    oamaddr: u8,
    oam_data: u8,
    x_y_scroll: u16,
    vram_addr: u16,
    vram_data: u8,
    // $4014
    oam_dma: u8,
    // Internal
    io_bus: u8,
    v: u16,
    t: u16,
    scroll_x: u8,
    write_toggle: bool,
}

struct PPUControl {
    nametable_addr: u8,
    vram_addr_inc: bool,
    spr_pattern_addr: u16,
    bg_pattern_addr: u16,
    sprites_large: bool,
    // ppu_master_slave: bool,
    vblank_nmi: bool,
}

impl From<u8> for PPUControl {
    fn from(val: u8) -> Self {
        PPUControl {
            nametable_addr:   val & 0b11,
            vram_addr_inc:    val.test_bit(2),
            spr_pattern_addr: if val.test_bit(3) { 0x1000 } else { 0x0000 },
            bg_pattern_addr:  if val.test_bit(4) { 0x1000 } else { 0x0000 },
            sprites_large:    val.test_bit(5),
            // ppu_master_slave: todo!(),
            vblank_nmi:       val.test_bit(7),
        }
    }
}

impl Into<u8> for PPUControl {
    fn into(self) -> u8 {
        (self.nametable_addr)
        | (self.vram_addr_inc as u8)           << 2
        | ((self.spr_pattern_addr >> 3) as u8) << 3
        | ((self.bg_pattern_addr >> 3) as u8)  << 4
        | (self.sprites_large as u8)           << 5
        | /* (self.emphasize_green as u8) */ 0 << 6
        | (self.vblank_nmi as u8)              << 7
   }
}

struct PPUMask {
    grayscale: bool,
    bg_mask: bool,
    sprites_mask: bool,
    bg_enable: bool,
    sprites_enable: bool,
    emphasize_red: bool,
    emphasize_green: bool,
    emphasize_blue: bool
}

impl From<u8> for PPUMask {
    fn from(val: u8) -> Self {
        PPUMask {
            grayscale:       val.test_bit(0),
            bg_mask:         val.test_bit(1),
            sprites_mask:    val.test_bit(2),
            bg_enable:       val.test_bit(3),
            sprites_enable:  val.test_bit(4),
            emphasize_red:   val.test_bit(5),
            emphasize_green: val.test_bit(6),
            emphasize_blue:  val.test_bit(7),
        }
    }
}

impl Into<u8> for PPUMask {
    fn into(self) -> u8 {
        (self.grayscale as u8)
        | (self.bg_mask as u8)         << 1
        | (self.sprites_mask as u8)    << 2
        | (self.bg_enable as u8)       << 3
        | (self.sprites_enable as u8)  << 4
        | (self.emphasize_red as u8)   << 5
        | (self.emphasize_green as u8) << 6
        | (self.emphasize_blue as u8)  << 7
   }
}

struct PPUStatus {
    sprite_overflow: bool,
    sprite_0_hit: bool,
    vblank: bool,
}

impl Registers {
    pub fn read(&mut self, addr: u16) -> u8 {
        match addr {
            ADDRESS_PPUCTRL   => self.io_bus,
            ADDRESS_PPUMASK   => self.io_bus,
            ADDRESS_PPUSTATUS => self.read_ppustatus(),
            ADDRESS_OAMADDR   => self.io_bus,
            ADDRESS_OAMDATA   => self.read_oamdata(),
            ADDRESS_PPUSCROLL => self.io_bus,
            ADDRESS_PPUADDR   => self.io_bus,
            ADDRESS_PPUDATA   => self.read_ppudata(),
            ADDRESS_OAMDMA    => self.io_bus,
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

    pub fn new() -> Registers {
        Registers {
            control: 0b0000_0000.into(),
            mask:    0b0000_0000.into(),
            status:  PPUStatus { sprite_overflow: false, sprite_0_hit: false, vblank: false },
            oamaddr: 0x00,
            x_y_scroll: 0x0000,
            vram_addr: 0x0000,
            vram_data: 0x00,
            oam_data: 0x00, //?
            oam_dma:  0x00, //?
            // Internal
            io_bus: 0x00,
            v: 0x00,        // ?
            t: 0x00,        // ?
            scroll_x: 0x00, // ?
            write_toggle: false,
        }
    }

    pub fn write_ppuctrl(&mut self, val: u8) -> () {
        // TODO: check for cycles < 29658 before allowing writes?
        self.control = val.into();
        println!("Wrote ${val:02x} to PPUCTRL (${ADDRESS_PPUCTRL:04x})")
    }

    pub fn write_ppumask(&mut self, val: u8) {
        // TODO: "writes ignored until first pre-render scanline"
        self.mask = val.into();
        println!("Wrote ${val:02x} to PPUMASK (${ADDRESS_PPUMASK:04x})")
    }

    pub fn read_ppustatus(&mut self) -> u8 {
        let so = (self.status.sprite_overflow as u8) << 5;
        let s0 = (self.status.sprite_0_hit as u8) << 6;
        let vb = (self.status.vblank as u8) << 7;

        let val = vb | s0 | so | (self.io_bus & 0b0001_1111);
        self.status.vblank = false;

        println!("Read ${val:02x} from PPUSTATUS (${ADDRESS_PPUSTATUS:04x})");

        val
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
