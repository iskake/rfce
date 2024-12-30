use crate::bits::Bitwise;

use super::mem::MemMap;

pub const VRAM_SIZE: usize = NAMETABLE_SIZE * 2;
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
    // ??vvv
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
            frame_buf: vec![0; PICTURE_HEIGHT * PICTURE_WIDTH * PIXEL_SIZE],
        }
    }

    pub(super) fn print_state(&self) -> () {
        println!("PPU STATE:");
        println!(
            "  v: {:04x}, t: {:04x}, x: {:02x}",
            self.reg.v, self.reg.t, self.reg.scroll_x
        );
        let ppuctrl: u8 = self.reg.control.into();
        println!("  ppuctrl: {:02x}", ppuctrl);
        println!("  cycles: {}, scanlines: {}, frame: {}", self.cycle, self.scanline, self.frame);
    }

    pub fn init(&mut self) -> () {
        // Once again, this is purely based on the value of the Mesen debugger after RESET
        self.cycle = 25;
        self.scanline = 0;
        self.frame = 1;
    }

    pub fn reset(&mut self) -> () {
        // TODO
        self.reg.control = 0x00.into();
        self.reg.mask = 0x00.into();
        self.reg.write_toggle = false;
        self.reg.x_y_scroll = 0x0000;

        // self.reg.vram_data = 0x00;
        // ???vvv (as replacement of above line)
        self.reg.v = 0x00;
        self.reg.t = 0x00;
        self.reg.t = 0x00;
        // ???^^^
        self.cycle = 0;
        self.scanline = 0;
        self.frame = 0;
    }

    pub(crate) fn is_vblank(&self) -> bool {
        self.reg.status.vblank
    }

    pub(crate) fn should_do_nmi(&self) -> bool {
        self.reg.status.vblank && self.reg.control.nmi_enable
    }

    pub(crate) fn nmi_enable(&self) -> bool {
        self.reg.control.nmi_enable
    }

    pub(crate) fn oamdma(&self) -> u8 {
        self.reg.oam_dma
    }

    pub fn cycle(&mut self, mem: &mut MemMap) -> () {
        // TODO: should this call some other fn 3 times instead?
        // TODO: also, should this really just be a for loop...?
        for _ in 0..3 {
            assert!(self.cycle < SCANLINE_DURATION);
            assert!(self.scanline < FRAME_SCANLINES);

            // TODO
            match self.scanline {
                0..=239 => {    // rendering (visible scanlines)
                    // TODO
                    match self.cycle {
                        0 => {},         // idle cycle
                        1..=256 => {     // vram fetch/update
                            // Fetch data. Each byte takes 2 cycles
                            // let c = (self.cycle - 1) % 8;
                            // dbg!(c);
                            // match c {
                            //     1 => {},    // nametable byte
                            //     3 => {},    // attribute table byte
                            //     5 => {},    // pattern table tile low
                            //     7 => {},    // pattern table tile high (+8 bytes from pattern table tile low)
                            //     _ => {},
                            // }
                            if self.cycle == 256 {
                                // TODO: actual fetch+write bytes instead of full scanline
                                self.add_scanline_temp(mem);
                            }
                        },
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
                261 => {     // dummy scanline (pre-render scanline)
                    if self.cycle == 1 {
                        println!("disabled vblank");
                        self.reg.status.vblank = false;
                    }
                },
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
                    self.frame += 1;
                }
            }
        }
    }

    pub fn read_addr(&self, addr: u16, mem: &MemMap) -> u8 {
        match addr {
            0x0000..=0x1fff => mem.mapper.read_chr(addr),
            0x2000..=0x2fff => mem.mapper.nametable_read(addr, self.vram),
            0x3f00..=0x3fff => self.pal[((addr - 0x3f00) % 20) as usize],
            _ => unreachable!("addr: {addr:04x}"),
        }
    }

    pub fn write_addr(&mut self, addr: u16, val: u8, mem: &mut MemMap) -> () {
        match addr {
            0x0000..=0x1fff => mem.mapper.write_chr(addr, val),
            0x2000..=0x2fff => mem.mapper.nametable_write(addr, val, self.vram),
            0x3f00..=0x3fff => self.pal[((addr - 0x3f00) % 20) as usize] = val,
            _ => unreachable!("addr: {addr:04x}"),
        }
    }

    pub fn read_mmio(&mut self, addr: u16, mem: &mut MemMap) -> u8 {
        self.read_reg(addr, mem)
    }

    // TODO: better function name
    pub fn read_mmio_no_sideeffect(&self, addr: u16) -> u8 {
        match addr {
            ADDRESS_PPUCTRL   => self.reg.control.into(),
            ADDRESS_PPUMASK   => self.reg.mask.into(),
            ADDRESS_PPUSTATUS => self.reg.status.into(),
            ADDRESS_OAMADDR   => 0x00, //self.reg.oamaddr,
            ADDRESS_OAMDATA   => 0x00, //self.reg.oam_data,
            ADDRESS_PPUSCROLL => 0x00, //self.reg.io_bus,
            ADDRESS_PPUADDR   => 0x00, //self.reg.io_bus,
            ADDRESS_PPUDATA   => 0x00, //self.read_ppudata(),
            ADDRESS_OAMDMA    => 0x00, //self.reg.io_bus,
            _ => todo!(),
        }
    }

    pub fn write_mmio(&mut self, addr: u16, val: u8, mem: &mut MemMap) -> () {
        self.write_reg(addr, val, mem);
    }

    fn read_reg(&mut self, addr: u16, mem: &mut MemMap) -> u8 {
        match addr {
            ADDRESS_PPUCTRL   => self.reg.io_bus,
            ADDRESS_PPUMASK   => self.reg.io_bus,
            ADDRESS_PPUSTATUS => self.read_ppustatus(),
            ADDRESS_OAMADDR   => self.reg.io_bus,
            ADDRESS_OAMDATA   => self.read_oamdata(),
            ADDRESS_PPUSCROLL => self.reg.io_bus,
            ADDRESS_PPUADDR   => self.reg.io_bus,
            ADDRESS_PPUDATA   => self.read_ppudata(mem),
            ADDRESS_OAMDMA    => self.reg.io_bus,
            _ => todo!(),
        }
    }

    fn write_reg(&mut self, addr: u16, val: u8, mem: &mut MemMap) -> () {
        match addr {
            ADDRESS_PPUCTRL   => { self.reg.io_bus = val; self.write_ppuctrl(val)},
            ADDRESS_PPUMASK   => { self.reg.io_bus = val; self.write_ppumask(val)},
            ADDRESS_PPUSTATUS => { self.reg.io_bus = val; },
            ADDRESS_OAMADDR   => { self.reg.io_bus = val; self.write_oamaddr(val)},
            ADDRESS_OAMDATA   => { self.reg.io_bus = val; self.write_oamdata(val)},
            ADDRESS_PPUSCROLL => { self.reg.io_bus = val; self.write_ppuscroll(val)},
            ADDRESS_PPUADDR   => { self.reg.io_bus = val; self.write_ppuaddr(val)},
            ADDRESS_PPUDATA   => { self.reg.io_bus = val; self.write_ppudata(val, mem)},
            ADDRESS_OAMDMA    => { self.reg.io_bus = val; self.write_oamdma(val)},
            _ => todo!(),
        }
    }

    pub fn write_oam(&mut self, dst: u8, val: u8) -> () {
        self.oam[dst as usize] = val;
    }

    fn add_scanline_temp(&self, mem: &mut MemMap) -> () {
        for i in 0..=255 {
            // todo!();
            // self.read_addr(, mem)
        }
    }

    fn write_ppuctrl(&mut self, val: u8) -> () {
        // TODO: check for cycles < 29658 before allowing writes?
        self.reg.control = val.into();
        println!("Wrote ${val:02x} to PPUCTRL (${ADDRESS_PPUCTRL:04x})");
    }

    fn write_ppumask(&mut self, val: u8) {
        // TODO: "writes ignored until first pre-render scanline"
        self.reg.mask = val.into();
        println!("Wrote ${val:02x} to PPUMASK (${ADDRESS_PPUMASK:04x})");
    }

    fn read_ppustatus(&mut self) -> u8 {
        let so = (self.reg.status.sprite_overflow as u8) << 5;
        let s0 = (self.reg.status.sprite_0_hit as u8) << 6;
        let vb = (self.reg.status.vblank as u8) << 7;

        let val = vb | s0 | so | (self.reg.io_bus & 0b0001_1111);
        self.reg.status.vblank = false;
        self.reg.write_toggle = false;

        println!("Read ${val:02x} from PPUSTATUS (${ADDRESS_PPUSTATUS:04x})");

        val
    }

    fn write_oamaddr(&mut self, val: u8) {
        // TODO: "OAMADDR precautions"
        self.reg.oam_addr = val;
        println!("Wrote ${val:02x} to OAMADDR (${ADDRESS_OAMADDR:04x})");
    }

    fn read_oamdata(&self) -> u8 {
        todo!("read_oamdata")
    }

    fn write_oamdata(&mut self, val: u8) {
        todo!("write_oamdata")
    }

    fn write_ppuscroll(&mut self, val: u8) {
        if !self.reg.write_toggle {
            self.reg.t = self.reg.t & 0x7f00 | (val as u16);
        } else {
            self.reg.t = (((val & 0x7f) as u16) << 8) | self.reg.t & 0xff;
        }

        println!("Wrote ${val:02x} to PPUSCROLL (${ADDRESS_PPUSCROLL:04x}) (lo: {})", self.reg.write_toggle);

        self.reg.write_toggle = !self.reg.write_toggle;

        // TODO!!!! this should seemingly be delayed by some cycles
        self.reg.addr_bus = self.reg.t;
        self.reg.v = self.reg.addr_bus;
    }

    fn write_ppuaddr(&mut self, val: u8) {
        // TODO? "palette corruption" & "bus conflict"?
        if !self.reg.write_toggle {
            self.reg.t = ((((val & 0x7f) as u16) << 8) | self.reg.t & 0xff) & 0x3fff;
        } else {
            self.reg.t = (self.reg.t & 0x7f00 | (val as u16)) & 0x3fff; // ?
        }

        println!("Wrote ${val:02x} to PPUADDR (${ADDRESS_PPUADDR:04x}) (lo: {})", self.reg.write_toggle);

        self.reg.write_toggle = !self.reg.write_toggle;

        // TODO!!!! this should seemingly be delayed by some cycles
        self.reg.addr_bus = self.reg.t;
        self.reg.v = self.reg.addr_bus;
    }

    fn read_ppudata(&mut self, mem: &MemMap) -> u8 {
        // TODO? "reading palette ram" & "read conflict with dpcm samples"?
        let old_val = self.reg.read_buf;
        let addr = self.reg.v;

        self.reg.read_buf = self.read_addr(addr, mem);
        println!("Read ${old_val:02x} from PPUDATA (${ADDRESS_PPUDATA:04x}) new addr: ${addr:04x}",);

        let delta = self.reg.control.vram_addr_inc;
        println!("delta: {}", delta);
        self.reg.v = (self.reg.v + delta as u16) & 0x7fff;
        self.reg.addr_bus = self.reg.v;

        old_val
    }

    fn write_ppudata(&mut self, val: u8, mem: &mut MemMap) {
        let addr = self.reg.v;
        self.write_addr(addr, val, mem);
        println!("Wrote ${val:02x} to PPUDATA (${ADDRESS_PPUDATA:04x}) addr: ${addr:04x}",);

        let delta = self.reg.control.vram_addr_inc;
        self.reg.v = (self.reg.v + delta as u16) & 0x7fff;
        self.reg.addr_bus = self.reg.v;
    }

    // TODO: chandle in cpu?
    pub fn write_oamdma(&mut self, val: u8) {
        self.reg.oam_dma = val;
        println!("Wrote ${val:02x} to OAMDMA (${ADDRESS_OAMDMA:04x})",);
    }
}

struct Registers {
    // $2000
    control: PPUControl,
    mask: PPUMask,
    status: PPUStatus,
    oam_addr: u8,
    oam_data: u8,
    x_y_scroll: u16,
    // vram_addr: u16,
    // vram_data: u8,
    // $4014
    oam_dma: u8,
    // Internal
    io_bus: u8,
    v: u16,
    t: u16,
    scroll_x: u8,
    write_toggle: bool,
    addr_bus: u16,
    read_buf: u8,
}

#[derive(Clone, Copy)]
struct PPUControl {
    nametable_addr: u8,
    vram_addr_inc: u8,
    spr_pattern_addr: u16,
    bg_pattern_addr: u16,
    sprites_large: bool,
    // ppu_master_slave: bool,
    nmi_enable: bool,
}

#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
struct PPUStatus {
    sprite_overflow: bool,
    sprite_0_hit: bool,
    vblank: bool,
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            control: 0b0000_0000.into(),
            mask:    0b0000_0000.into(),
            status:  PPUStatus { sprite_overflow: false, sprite_0_hit: false, vblank: false },
            oam_addr: 0x00,
            x_y_scroll: 0x0000,
            // vram_addr: 0x0000,
            // vram_data: 0x00,
            oam_data: 0x00, //?
            oam_dma:  0x00, //?
            // Internal
            io_bus: 0x00,
            v: 0x00,        // ?
            t: 0x00,        // ?
            scroll_x: 0x00, // ?
            write_toggle: false,
            addr_bus: 0,
            read_buf: 0,
        }
    }
}

impl From<u8> for PPUControl {
    fn from(val: u8) -> Self {
        PPUControl {
            nametable_addr:   val & 0b11,
            vram_addr_inc:    if val.test_bit(2) {32} else {1},
            spr_pattern_addr: if val.test_bit(3) { 0x1000 } else { 0x0000 },
            bg_pattern_addr:  if val.test_bit(4) { 0x1000 } else { 0x0000 },
            sprites_large:    val.test_bit(5),
            // ppu_master_slave: todo!(),
            nmi_enable:       val.test_bit(7),
        }
    }
}

impl Into<u8> for PPUControl {
    fn into(self) -> u8 {
        (self.nametable_addr)
        | ((self.vram_addr_inc >> 5) as u8)     << 2
        | ((self.spr_pattern_addr >> 12) as u8) << 3
        | ((self.bg_pattern_addr >> 12) as u8)  << 4
        | (self.sprites_large as u8)            << 5
        | /* (self.ppu_master_slave as u8) */0  << 6
        | (self.nmi_enable as u8)               << 7
   }
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

impl Into<u8> for PPUStatus {
    fn into(self) -> u8 {
        (self.sprite_overflow as u8) << 5
        | (self.sprite_0_hit as u8)  << 6
        | (self.vblank as u8)        << 7
   }
}