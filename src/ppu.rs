use crate::memory::Memory;
use crate::nes_rom::NametableMirroring;
use crate::ppu::ppu_memory::PpuMemory;

pub mod ppu_memory;

struct PpuAddr {
    hi: u8,
    lo: u8,
    is_hi: bool,
}

impl PpuAddr {
    fn new() -> Self {
        Self {
            hi: 0,
            lo: 0,
            is_hi: true,
        }
    }

    fn set(&mut self, value: u8) {
        if self.is_hi {
            self.hi = value;
        } else {
            self.lo = value;
        }

        self.is_hi = !self.is_hi;
    }

    fn get_addr(&self) -> u16 {
        const PPU_ADDR_MASK: u16 = 0x3FFF;
        ((self.hi as u16) << 8 | (self.lo as u16)) & PPU_ADDR_MASK
    }

    fn increment_addr(&mut self, amount: u8) {
        let new_addr = self.get_addr().wrapping_add(amount as u16);
        self.hi = (new_addr >> 8) as u8;
        self.lo = (new_addr & 0xFF) as u8;
    }

    pub fn reset_latch(&mut self) {
        self.is_hi = true;
    }
}

struct PpuScroll {
    x: u8,
    y: u8,
    is_x: bool,
}

impl PpuScroll {
    pub fn new() -> Self {
        PpuScroll {
            x: 0,
            y: 0,
            // TODO technically this should use the same latch as PPU_ADDR
            is_x: true,
        }
    }

    fn set(&mut self, value: u8) {
        if self.is_x {
            self.x = value;
        } else {
            self.y = value;
        }

        self.is_x = !self.is_x;
    }

    fn get_x(&self) -> u8 {
        self.x
    }

    fn get_y(&self) -> u8 {
        self.y
    }

    pub fn reset_latch(&mut self) {
        self.is_x = true;
    }
}

const PPU_CTRL_NAMETABLE_MASK: u8 = 0x3;
const PPU_CTRL_VRAM_ADD_INCREMENT_BIT: u8 = 2;
const PPU_CTRL_SPRITE_PATTERN_ADDR_BIT: u8 = 3;
const PPU_CTRL_BACKRGROUND_ADDR_BIT: u8 = 4;
const PPU_CTRL_SPRITE_SIZE_BIT: u8 = 5;
const PPU_CTRL_MASTER_SLAVE_SELECT_BIT: u8 = 6;
const PPU_CTRL_VBLANK_NMI_BIT: u8 = 7;

const PPU_STATUS_SPRITE_OVERFLOW_BIT: u8 = 5;
const PPU_STATUS_SPRITE_HIT_BIT: u8 = 6;
const PPU_STATUS_VBLANK_BIT: u8 = 7;

const PPU_MASK_GREYSCALE_BIT: u8 = 0;

const PPU_MASK_SHOW_LEFTMOST_BACKGROUND_BIT: u8 = 1;
const PPU_MASK_SHOW_LEFTMOST_SPRITES_BIT: u8 = 2;
const PPU_MASK_BACKGROUND_RENDERING_BIT: u8 = 3;
const PPU_MASK_SPRITE_RENDERING_BIT: u8 = 4;
const PPU_MASK_RED_BIT: u8 = 5;
const PPU_MASK_GREEN_BIT: u8 = 6;
const PPU_MASK_BLUE_BIT: u8 = 7;

const SCANLINES: u32 = 262;
const VISIBLE_SCANLIENS: u32 = 240;
const SCANLINE_CYCLES: u32 = 341;

pub struct Ppu<M: Memory> {
    ctrl: u8,
    mask: u8,
    status: u8,
    oam_addr: u8,
    oam_data: u8,
    scroll: PpuScroll,
    addr: PpuAddr,
    data_buffer: u8,
    oam_dma: u8,

    scanline: u32,
    cycle: u32,
    nmi: bool,
    new_frame: bool,

    pub memory: M,
}

impl Ppu<PpuMemory> {
    pub fn new(chr_rom: Vec<u8>, mirroring: NametableMirroring) -> Self {
        Self {
            ctrl: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            oam_data: 0,
            scroll: PpuScroll::new(),
            addr: PpuAddr::new(),
            data_buffer: 0,
            oam_dma: 0,
            scanline: 1,
            cycle: 0,
            nmi: false,
            new_frame: false,
            memory: PpuMemory::new(chr_rom, mirroring),
        }
    }
}

impl<M: Memory> Ppu<M> {
    fn new_with_memory(memory: M) -> Self {
        Self {
            ctrl: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            oam_data: 0,
            scroll: PpuScroll::new(),
            addr: PpuAddr::new(),
            data_buffer: 0,
            oam_dma: 0,
            scanline: 1,
            cycle: 0,
            nmi: false,
            new_frame: false,
            memory,
        }
    }

    pub fn tick(&mut self, cycle: u32) {
        self.cycle = cycle;
        if self.cycle > SCANLINE_CYCLES {
            self.cycle -= SCANLINE_CYCLES;
            self.scanline += 1;

            if self.scanline == VISIBLE_SCANLIENS + 1 {
                self.set_status_bit(PPU_STATUS_VBLANK_BIT, true);
                if self.get_ctrl_bit(PPU_CTRL_VBLANK_NMI_BIT) {
                    self.nmi = true;
                }
            }

            if self.scanline > SCANLINES {
                self.scanline = 0;
                self.nmi = false;
                self.set_status_bit(PPU_STATUS_SPRITE_HIT_BIT, false);
                self.set_status_bit(PPU_STATUS_VBLANK_BIT, false);
                self.new_frame = true;
            }
        }
    }

    pub fn poll_nmi(&mut self) -> bool {
        let value = self.nmi;
        self.nmi = false;
        value
    }

    pub fn poll_new_frame(&mut self) -> bool {
        let value = self.new_frame;
        self.new_frame = false;
        value
    }

    pub fn is_vblank(&self) -> bool {
        self.get_status_bit(PPU_STATUS_VBLANK_BIT)
    }

    pub fn write_ppu_ctrl(&mut self, value: u8) {
        let old_nmi = self.get_ctrl_bit(PPU_CTRL_VBLANK_NMI_BIT);
        self.ctrl = value;
        if !old_nmi && self.get_ctrl_bit(PPU_CTRL_VBLANK_NMI_BIT) && self.is_vblank() {
            self.nmi = true;
        }
    }

    pub fn get_ctrl_bit(&self, bit: u8) -> bool {
        self.ctrl >> bit & 1 == 1
    }

    pub fn set_ctrl_bit(&mut self, bit: u8, value: bool) {
        if value {
            self.ctrl |= 1 << bit;
        } else {
            self.ctrl &= !(1 << bit);
        }
    }

    pub fn write_ppu_mask(&mut self, value: u8) {
        self.mask = value;
    }

    pub fn get_mask_bit(&self, bit: u8) -> bool {
        self.mask >> bit & 1 == 1
    }

    pub fn set_mask_bit(&mut self, bit: u8, value: bool) {
        if value {
            self.mask |= 1 << bit;
        } else {
            self.mask &= !(1 << bit);
        }
    }

    pub fn read_ppu_status(&mut self) -> u8 {
        let value = self.status;
        self.set_status_bit(PPU_STATUS_VBLANK_BIT, false);
        self.addr.reset_latch();
        self.scroll.reset_latch();
        value
    }

    pub fn get_status_bit(&self, bit: u8) -> bool {
        self.status >> bit & 1 == 1
    }

    pub fn set_status_bit(&mut self, bit: u8, value: bool) {
        if value {
            self.status |= 1 << bit;
        } else {
            self.status &= !(1 << bit);
        }
    }

    pub fn write_ppu_scroll(&mut self, value: u8) {
        self.scroll.set(value);
    }

    pub fn write_ppu_addr(&mut self, value: u8) {
        self.addr.set(value)
    }

    pub fn read_ppu_data(&mut self) -> u8 {
        let addr = self.addr.get_addr();
        let result = if (0x3F00..=0x3FFF).contains(&addr) {
            self.memory.read(addr)
        } else {
            let value = self.data_buffer;
            self.data_buffer = self.memory.read(addr);
            value
        };

        self.addr.increment_addr(self.addr_increment_amount());
        result
    }

    pub fn write_ppu_data(&mut self, value: u8) {
        self.memory.write(self.addr.get_addr(), value);

        self.addr.increment_addr(self.addr_increment_amount());
    }

    fn addr_increment_amount(&self) -> u8 {
        if !self.get_ctrl_bit(PPU_CTRL_VRAM_ADD_INCREMENT_BIT) {
            1 // across
        } else {
            32 // down
        }
    }

    pub fn background_pattern_addr(&self) -> u16 {
        if self.get_ctrl_bit(PPU_CTRL_BACKRGROUND_ADDR_BIT) {
            0x1000
        } else {
            0x0
        }
    }

    pub fn base_nametable_index(&self) -> u8 {
        self.ctrl & PPU_CTRL_NAMETABLE_MASK
    }
}

#[cfg(test)]
mod test {
    use crate::memory::DummyMemory;
    use crate::ppu::{Ppu, PPU_CTRL_VRAM_ADD_INCREMENT_BIT};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    pub fn test_ppu_addr() {
        let memory = Rc::new(RefCell::new(DummyMemory::new()));
        let mut ppu = Ppu::new_with_memory(memory.clone());

        ppu.set_ctrl_bit(PPU_CTRL_VRAM_ADD_INCREMENT_BIT, false);

        ppu.write_ppu_addr(0x34);
        ppu.write_ppu_addr(0x56);
        assert_eq!(ppu.addr.get_addr(), 0x3456);

        ppu.read_ppu_data();
        assert_eq!(memory.borrow().last_read_addr(), 0x3456);
        assert_eq!(ppu.addr.get_addr(), 0x3457);

        assert_eq!(ppu.read_ppu_data(), 0x56); // read on dummy memory returns low byte of address
        assert_eq!(memory.borrow().last_read_addr(), 0x3457);
        assert_eq!(ppu.addr.get_addr(), 0x3458);

        ppu.write_ppu_data(0xCA);
        assert_eq!(memory.borrow().last_read_addr(), 0x3457);
        assert_eq!(memory.borrow().last_write_addr(), 0x3458);
        assert_eq!(memory.borrow().last_write_value(), 0xCA);
        assert_eq!(ppu.addr.get_addr(), 0x3459);

        ppu.set_ctrl_bit(PPU_CTRL_VRAM_ADD_INCREMENT_BIT, true);
        ppu.read_ppu_data();
        assert_eq!(ppu.addr.get_addr(), 0x3479); // now it increments by 0x20 because of the changed ctrl bit
        ppu.set_ctrl_bit(PPU_CTRL_VRAM_ADD_INCREMENT_BIT, false);

        ppu.addr.set(0x3f);
        ppu.addr.set(0xBD);
        assert_eq!(ppu.addr.get_addr(), 0x3fBD);
        assert_eq!(ppu.read_ppu_data(), 0xBD); // read in palette range returns value immediately
        assert_eq!(ppu.addr.get_addr(), 0x3fBE);

        ppu.addr.set(0x3f);
        ppu.addr.set(0xff);
        ppu.read_ppu_data();
        assert_eq!(ppu.addr.get_addr(), 0x0000); // wraparound after 0x3fff
    }
}
