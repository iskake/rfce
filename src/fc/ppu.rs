mod tile;

use log::debug;
use rgb::*;
use tile::Tile;

use crate::bits::Bitwise;

use super::{mem::MemMap, CPU_FREQ};

pub const VRAM_SIZE: usize = NAMETABLE_SIZE * 2;

const NAMETABLE_SIZE: usize = 0x0400;
const ATTRIBUTE_TABLE_SIZE: usize = 64;
const PATTERN_TABLE_SIZE: u16 = 0x1000;

const OAM_SIZE: usize = 64;
const SPRITE_SIZE: usize = 4;
const TILE_SIZE: u16 = 16;
const PALETTE_RAM_SIZE: usize = 0x20;

pub const PICTURE_WIDTH:  usize = 256;
pub const PICTURE_HEIGHT: usize = 240;

pub const SCANLINE_DURATION: u32 = 341;
pub const FRAME_SCANLINES:   u32 = 262;
pub const PPU_FREQ: f64 = CPU_FREQ * 3.0;

pub const FRAMERATE: f64 = PPU_FREQ / (341.0 * 261.0 + 340.5);

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
    // rendering
    curr_tile_idx: u8,
    prev_attribute_byte: u8,
    curr_attribute_byte: u8,
    temp_attribute_byte: u8,
    curr_pattern_lo: u8,
    curr_pattern_hi: u8,
    shift_reg_lo: u16,
    shift_reg_hi: u16,
    // ??vvv
    frame_buf: Vec<u8>,
    nametable_buf: Vec<u8>,
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
            frame_buf: vec![0; PICTURE_HEIGHT * PICTURE_WIDTH * 3],
            nametable_buf: vec![0; PICTURE_HEIGHT * PICTURE_WIDTH * 4 * 3],

            curr_tile_idx: 0,
            prev_attribute_byte: 0,
            curr_attribute_byte: 0,
            temp_attribute_byte: 0,
            curr_pattern_lo: 0,
            curr_pattern_hi: 0,
            shift_reg_lo: 0,
            shift_reg_hi: 0,
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

    pub(crate) fn cycles(&self) -> u32 {
        self.cycle
    }

    pub(crate) fn scanlines(&self) -> u32 {
        self.scanline
    }

    pub(crate) fn _is_vblank(&self) -> bool {
        self.reg.status.vblank
    }

    pub(crate) fn just_finished_rendering(&self) -> bool {
        self.scanline == 240
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

            self.render(mem);

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
            0x3f00..=0x3fff => self.pal[((addr - 0x3f00) % 0x20) as usize],
            _ => unreachable!("addr: {addr:04x}"),
        }
    }

    pub fn write_addr(&mut self, addr: u16, val: u8, mem: &mut MemMap) -> () {
        match addr {
            0x0000..=0x1fff => mem.mapper.write_chr(addr, val),
            0x2000..=0x2fff => mem.mapper.nametable_write(addr, val, &mut self.vram),
            0x3f00..=0x3fff => self.write_pal(addr, val),
            _ => unreachable!("addr: {addr:04x}"),
        }
    }

    fn write_pal(&mut self, addr: u16, val: u8) {
        if addr & 0b11 == 0 {
            self.pal[((addr - 0x3f00) % 0x20) as usize] = val;
            self.pal[((addr - 0x3f10) % 0x20) as usize] = val;
        } else {
            self.pal[((addr - 0x3f00) % 0x20) as usize] = val;
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
            _ => unreachable!(),
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
            _ => unreachable!(),
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
            _ => unreachable!(),
        }
    }

    pub fn write_oam(&mut self, dst: u8, val: u8) -> () {
        self.oam[dst as usize] = val;
    }

    fn write_ppuctrl(&mut self, val: u8) -> () {
        // TODO: check for cycles < 29658 before allowing writes?
        self.reg.control = val.into();
        debug!("Wrote ${val:02x} to PPUCTRL (${ADDRESS_PPUCTRL:04x})");
    }

    fn write_ppumask(&mut self, val: u8) {
        // TODO: "writes ignored until first pre-render scanline"
        self.reg.mask = val.into();
        debug!("Wrote ${val:02x} to PPUMASK (${ADDRESS_PPUMASK:04x})");
    }

    fn read_ppustatus(&mut self) -> u8 {
        let so = (self.reg.status.sprite_overflow as u8) << 5;
        let s0 = (self.reg.status.sprite_0_hit as u8) << 6;
        let vb = (self.reg.status.vblank as u8) << 7;

        let val = vb | s0 | so | (self.reg.io_bus & 0b0001_1111);
        self.reg.status.vblank = false;
        self.reg.write_toggle = false;

        debug!("Read ${val:02x} from PPUSTATUS (${ADDRESS_PPUSTATUS:04x})");

        val
    }

    fn write_oamaddr(&mut self, val: u8) {
        // TODO: "OAMADDR precautions"
        self.reg.oam_addr = val;
        debug!("Wrote ${val:02x} to OAMADDR (${ADDRESS_OAMADDR:04x})");
    }

    fn read_oamdata(&self) -> u8 {
        let _val = self.reg.oam_data;
        todo!("read_oamdata")
    }

    fn write_oamdata(&mut self, _val: u8) {
        todo!("write_oamdata")
    }

    fn write_ppuscroll(&mut self, val: u8) {
        if !self.reg.write_toggle {
            self.reg.t = self.reg.t & 0x7f00 | (val as u16);
        } else {
            self.reg.t = (((val & 0x7f) as u16) << 8) | self.reg.t & 0xff;
            self.reg.scroll_x = val & 0x07;
        }

        debug!("Wrote ${val:02x} to PPUSCROLL (${ADDRESS_PPUSCROLL:04x}) (lo: {})", self.reg.write_toggle);

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

        debug!("Wrote ${val:02x} to PPUADDR (${ADDRESS_PPUADDR:04x}) (lo: {})", self.reg.write_toggle);

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
        debug!("Read ${old_val:02x} from PPUDATA (${ADDRESS_PPUDATA:04x}) new addr: ${addr:04x}",);

        let delta = self.reg.control.vram_addr_inc;
        debug!("delta: {}", delta);
        self.reg.v = (self.reg.v + delta as u16) & 0x7fff;
        self.reg.addr_bus = self.reg.v;

        old_val
    }

    fn write_ppudata(&mut self, val: u8, mem: &mut MemMap) {
        let addr = self.reg.v;
        self.write_addr(addr, val, mem);
        debug!("Wrote ${val:02x} to PPUDATA (${ADDRESS_PPUDATA:04x}) addr: ${addr:04x}",);

        let delta = self.reg.control.vram_addr_inc;
        self.reg.v = (self.reg.v + delta as u16) & 0x7fff;
        self.reg.addr_bus = self.reg.v;
    }

    pub fn write_oamdma(&mut self, val: u8) {
        self.reg.oam_dma = val;
        debug!("Wrote ${val:02x} to OAMDMA (${ADDRESS_OAMDMA:04x})",);
    }

    pub(crate) fn get_frame_buf(&self) -> &[u8] {
        &self.frame_buf
    }

    fn rendering_enabled(&mut self) -> bool {
        self.reg.mask.bg_enable || self.reg.mask.sprites_enable
    }

    #[inline]
    fn render(&mut self, mem: &mut MemMap) {
        // TODO
        match self.scanline {
            0..=239 => {    // rendering (visible scanlines)
                self.render_dot(mem);
            },
            240 => {},      // idle (post-render scanline)
            241..=260 => {  // vblank
                if self.cycle == 1 && self.scanline == 241 {
                    self.reg.status.vblank = true;
                    debug!("enabled vblank")
                }
            },
            261 => {     // dummy scanline (pre-render scanline)
                self.rendering_fetch_data(mem); // TODO? should be blank scanline?

                if self.cycle == 1 {
                    debug!("disabled vblank");
                    self.reg.status.vblank = false;
                }

                if self.cycle >= 280 && self.cycle <= 304 {
                    // "Copy vertical scrolling value from t"
                    self.reg.v = (self.reg.v & !0x7be0) | (self.reg.t & 0x7be0);
                }
            },
            _ => unreachable!(),
        }
    }

    #[inline]
    fn render_dot(&mut self, mem: &mut MemMap) {
        match self.cycle {
            0 => {},         // idle cycle
            1..=256 => {     // vram fetch/update
                self.rendering_fetch_data(mem);

                let x = (self.cycle - 1) as usize;
                let y = self.scanline as usize;
                let idx = y * PICTURE_WIDTH + x;
                let px = self.get_next_pixel();

                self.shl_shift_registers(1);

                self.frame_buf.as_rgb_mut()[idx] = px;
            },
            257..=320 => {   // fetch tile data for sprites on the next scanline
                if self.cycle == 257 {
                    if self.rendering_enabled() {
                        // "Copy horizontal scrolling value from t"
                        self.reg.v = (self.reg.v & !0x041f) | (self.reg.t & 0x041f);
                    }
                }

                // TODO: garbage reads?
                // TODO2: "The shifters are reloaded during ticks 9, 17, 25, ..., 257."
                // if self.rendering_enabled() {
                //     self.rendering_fetch_data(mem);
                // }
            },
            321..=336 => {  // fetch tile data for first two tiles for the next scanline
                self.rendering_fetch_data(mem);

                if self.rendering_enabled() {
                    // Load the data into the registers
                    if self.cycle == 328 || self.cycle == 336 {
                        self.shl_shift_registers(8);
                    }
                }
            },
            337..=340 => {
                if self.cycle == 339 && self.scanline % 2 == 1 {
                    self.cycle = 340;
                }
            }, // fetch two bytes
            _ => unreachable!(),
        }
        // }
    }

    fn shl_shift_registers(&mut self, amount: u16) {
        // Update the shift register(s)
        self.shift_reg_lo <<= amount;
        self.shift_reg_hi <<= amount;
    }

    fn fill_shift_registers(&mut self) {
        self.shift_reg_lo |= self.curr_pattern_lo as u16;
        self.shift_reg_hi |= self.curr_pattern_hi as u16;
    }

    #[inline]
    fn rendering_fetch_data(&mut self, mem: &mut MemMap) {
        // Fetch data. Each byte takes 2 cycles
        if !self.rendering_enabled() {
            return;
        }

        let c = (self.cycle) % 8;
        match c {
            1 => {  // nametable byte
                // "The shifters are reloaded during ticks 9, 17, 25, ..., 257."
                self.fill_shift_registers();

                // TODO? replace this with something that isn't 3 variables...
                self.prev_attribute_byte = self.curr_attribute_byte;
                self.curr_attribute_byte = self.temp_attribute_byte;

                let addr = 0x2000 | (self.reg.v & 0x0fff);
                self.curr_tile_idx = self.read_addr(addr, mem);
            },
            3 => {  // attribute table byte
                let v = self.reg.v;
                let addr = 0x23c0 | (v & 0x0c00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07);
                let val = self.read_addr(addr, mem);

                // Since it changes mid tile rendering, we need the pevious one too.
                self.temp_attribute_byte = val;
            },
            5 | 7 => {  // pattern table tile low
                let bg_addr = self.reg.control.bg_pattern_addr;
                let y_scroll = self.reg.v >> 12;
                let addr: u16 = ((self.curr_tile_idx as u16) << 4) | y_scroll | bg_addr;

                if c == 5 {
                    self.curr_pattern_lo = self.read_addr(addr, mem)
                } else {
                    self.curr_pattern_hi = self.read_addr(addr + 8, mem)
                }
            },
            0 => {
                self.inc_hori();

                if self.cycle == 256 {
                    self.inc_vert();
                }
            }
            _ => {},
        }

        // if self.cycle == 160 && self.scanline == 48 {
        //     let addr: u16 = ((self.curr_tile_idx as u16) << 4) | (self.reg.v >> 12) | self.reg.control.bg_pattern_addr;
        //     println!("tile_idx: {:02x} attribute byte: {:02x}, chr_h,l: {:02x},{:02x} (addr:{:04x})\n shift: {:04x},{:04x} v: {:04x} at ({},{})",
        //         self.curr_tile_idx,
        //         self.curr_attribute_byte,
        //         self.curr_pattern_hi,
        //         self.curr_pattern_lo,
        //         addr,
        //         self.shift_reg_lo,
        //         self.shift_reg_hi,
        //         self.reg.v,
        //         self.cycle,
        //         self.scanline
        //     );
        // }

        // if self.cycle == 72+8 && self.scanline == 144 {
        //     let addr: u16 = ((self.curr_tile_idx as u16) << 4) | (self.reg.v >> 12) | self.reg.control.bg_pattern_addr;
        //     println!("tile_idx: {:02x} attribute byte: {:02x}, chr_h,l: {:02x},{:02x} (addr:{:04x})\n shift: {:04x},{:04x} v: {:04x} at ({},{})",
        //         self.curr_tile_idx,
        //         self.curr_attribute_byte,
        //         self.curr_pattern_hi,
        //         self.curr_pattern_lo,
        //         addr,
        //         self.shift_reg_lo,
        //         self.shift_reg_hi,
        //         self.reg.v,
        //         self.cycle,
        //         self.scanline
        //     );
        // }
    }

    #[inline]
    fn get_next_pixel(&mut self) -> RGB<u8> {
        let scroll = self.reg.scroll_x;
        let mut color_idx = 0;

        let bg_color = ((((self.shift_reg_lo as u16) << scroll) & 0x8000) >> 15)
                         | ((((self.shift_reg_hi as u16) << scroll) & 0x8000) >> 14);

        if self.reg.mask.bg_enable {
            color_idx = bg_color as u8;
        }
        let pix_val = color_idx;

        // TODO: actually handle scrolling etc.
        let tile_x = (self.cycle / 8) as usize;
        let tile_y = (self.scanline / 8) as usize;

        let tile_attr = if (scroll as u32 + (self.cycle) % 9) < 9 {
            self.prev_attribute_byte
        } else {
            self.curr_attribute_byte
        };

        let pal_idx = self.pal_idx_from_attr(tile_attr, tile_x, tile_y);
        let px = self.pixel_color(pal_idx, pix_val, false);

        as_rgb(px)
    }

    /// Get the color of a specific pixel / palette pair.
    fn pixel_color(&self, pal_idx: u8, pix_val: u8, sprite: bool) -> u8 {
        assert!(pal_idx == (pal_idx & 0b11));
        assert!(pix_val == (pix_val & 0b11));

        let mut idx = ((sprite as u8) << 4) | (pal_idx << 2) | pix_val;

        if pix_val == 0 {
            idx &= 0b1_0000;
        }

        let mut color = self.pal[idx as usize];

        if self.reg.mask.grayscale {
            color &= 0x30;
        }
        color
    }

    pub(crate) fn generate_nametables_image_temp(&mut self, mem: &MemMap) -> &[u8] {
        let img_w = 2 * PICTURE_WIDTH;

        for nametable_num in 0..4 {
            for nametable_byte in 0..(NAMETABLE_SIZE - ATTRIBUTE_TABLE_SIZE /* * 4 */) {
                let nametable_x = nametable_byte % 32;
                let nametable_y = nametable_byte >> 5;

                let i = nametable_byte as u16;

                let nametable_addr = self.reg.control.nametable_addr;
                let tile_addr = i + nametable_addr + (nametable_num * NAMETABLE_SIZE) as u16;

                let tile_idx = self.read_addr(tile_addr, mem);
                let tile_ptr = tile_idx as u16 * TILE_SIZE;

                let bg_addr = self.reg.control.bg_pattern_addr;
                let tile_bytes: Vec<u8> = (tile_ptr..tile_ptr+TILE_SIZE)
                    .map(|x| mem.mapper.read_chr(x + bg_addr))
                    .collect();

                assert_eq!(tile_bytes.len(), TILE_SIZE as usize);
                let tile = Tile::from_slice(tile_bytes.as_slice()).unwrap();

                for j in 0..8 {
                    for k in 0..8 {
                        let pixel_color = tile.color_at(k, j);

                        let pixel_x = (nametable_x * 8) + k;
                        let pixel_y = (nametable_y * 8) + j;
                        let ntbl_num_x_coord = PICTURE_WIDTH * (nametable_num & 0b01);
                        let ntbl_num_y_coord = PICTURE_HEIGHT * ((nametable_num & 0b10) >> 1);
                        let idx = img_w * (pixel_y + ntbl_num_y_coord) + (pixel_x + ntbl_num_x_coord);

                        let pal_idx = self.pal_idx_from_attr_table(nametable_num, nametable_x, nametable_y, mem);

                        let color = self.pixel_color(pal_idx, pixel_color, false);
                        let px = as_rgb(color);

                        self.nametable_buf.as_rgb_mut()[idx] = px;
                    }
                }
            }
        }
        &self.nametable_buf
    }

    /// Get the index into the attribute table corresponding to the specified x/y coordinate (0-31 / 0-29)
    fn pal_idx_from_attr_table(&self, nametable_num: usize, tile_x: usize, tile_y: usize, mem: &MemMap) -> u8 {
        assert!(nametable_num == (nametable_num & 0b11));

        let base_addr = 0x23c0 | (nametable_num << 10);
        let x = tile_x / 4;
        let y = tile_y / 4;
        let addr = base_addr + (y * 0x8) + x;

        let attr = self.read_addr(addr as u16, mem);

        self.pal_idx_from_attr(attr, tile_x, tile_y)
    }

    fn pal_idx_from_attr(&self, attr: u8, tile_x: usize, tile_y: usize) -> u8 {
        let dx = (tile_x / 2) & 0b1;
        let dy = (tile_y / 2) & 0b1;
        let delta = 2 * ((dy << 1) | dx);

        // let top_left = attr & 0b11;
        // let top_right = (attr & 0b1100)>> 2;
        // let bottom_left = (attr & 0b110000)>> 4;
        // let bottom_right = (attr & 0b11000000)>> 6;

        (attr & (0b11 << delta)) >> delta
    }

    fn inc_hori(&mut self) -> () {
        if (self.reg.v & 0x1f) == 31 {  // if coarse X == 31
            self.reg.v &= !0x001f;      // coarse X = 0
            self.reg.v ^= 0x0400;       // switch horizontal nametable
        } else {
            self.reg.v += 1             // increment coarse X
        }
    }

    fn inc_vert(&mut self) -> () {
        let mut v = self.reg.v;
        if (v & 0x7000) != 0x7000 {             // if fine Y < 7
            v += 0x1000                         // increment fine Y
        } else {
            v &= !0x7000;                       // fine Y = 0
            let mut y = (v & 0x03E0) >> 5; // let y = coarse Y
            if y == 29 {
                y = 0;                          // coarse Y = 0
                v ^= 0x0800;                    // switch vertical nametable
            } else if y == 31 {
                y = 0                           // coarse Y = 0, nametable not switched
            } else {
                y += 1                          // increment coarse Y
            }
            v = (v & !0x03E0) | (y << 5)        // put coarse Y back into v
        }
        self.reg.v = v;
    }
}

/// Get the rgb color corresponding to the ppu color.
fn as_rgb(color: u8) -> RGB8 {
    let r = PALETTE_COLORS[3 * color as usize];
    let g = PALETTE_COLORS[3 * color as usize + 1];
    let b = PALETTE_COLORS[3 * color as usize + 2];
    RGB8 { r, g, b }
}

// TODO? make it possible to change this?
const PALETTE_COLORS: [u8; 0x40 * 3] = [
    // NOTE: the colors are from the default mesen color palette.
    0x66, 0x66, 0x66, 0x00, 0x2a, 0x88, 0x14, 0x12, 0xa7, 0x3b, 0x00, 0xa4, 0x5c, 0x00, 0x7e, 0x6e,
    0x00, 0x40, 0x6c, 0x06, 0x00, 0x56, 0x1d, 0x00, 0x33, 0x35, 0x00, 0x0b, 0x48, 0x00, 0x00, 0x52,
    0x00, 0x00, 0x4f, 0x08, 0x00, 0x40, 0x4d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xad, 0xad, 0xad, 0x15, 0x5f, 0xd9, 0x42, 0x40, 0xff, 0x75, 0x27, 0xfe, 0xa0, 0x1a, 0xcc, 0xb7,
    0x1e, 0x7b, 0xb5, 0x31, 0x20, 0x99, 0x4e, 0x00, 0x6b, 0x6d, 0x00, 0x38, 0x87, 0x00, 0x0c, 0x93,
    0x00, 0x00, 0x8f, 0x32, 0x00, 0x7c, 0x8d, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xff, 0xfe, 0xff, 0x64, 0xb0, 0xff, 0x92, 0x90, 0xff, 0xc6, 0x76, 0xff, 0xf3, 0x6a, 0xff, 0xfe,
    0x6e, 0xcc, 0xfe, 0x81, 0x70, 0xea, 0x9e, 0x22, 0xbc, 0xbe, 0x00, 0x88, 0xd8, 0x00, 0x5c, 0xe4,
    0x30, 0x45, 0xe0, 0x82, 0x48, 0xcd, 0xde, 0x4f, 0x4f, 0x4f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0xff, 0xfe, 0xff, 0xc0, 0xdf, 0xff, 0xd3, 0xd2, 0xff, 0xe8, 0xc8, 0xff, 0xfb, 0xc2, 0xff, 0xfe,
    0xc4, 0xea, 0xfe, 0xcc, 0xc5, 0xf7, 0xd8, 0xa5, 0xe4, 0xe5, 0x94, 0xcf, 0xef, 0x96, 0xbd, 0xf4,
    0xab, 0xb3, 0xf3, 0xcc, 0xb5, 0xeb, 0xf2, 0xb8, 0xb8, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];


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
    nametable_addr: u16,
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
            nametable_addr:   ((val & 0b11) as u16) << 10 | 0x2000,
            vram_addr_inc:    if val.test_bit(2) {32} else {1},
            spr_pattern_addr: if val.test_bit(3) { PATTERN_TABLE_SIZE } else { 0x0000 },
            bg_pattern_addr:  if val.test_bit(4) { PATTERN_TABLE_SIZE } else { 0x0000 },
            sprites_large:    val.test_bit(5),
            // ppu_master_slave: val.test_bit(6),
            nmi_enable:       val.test_bit(7),
        }
    }
}

impl Into<u8> for PPUControl {
    fn into(self) -> u8 {
        ((self.nametable_addr & 0xc00) >> 10) as u8
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