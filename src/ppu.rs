use crate::memory::Memory;
use crate::nes_rom::NametableMirroring;
use crate::ppu::ppu_memory::PpuMemory;

mod ppu_memory;

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
}

const PPU_CTRL_NAMETABLE_MASK: u8 = 0x3;
const PPU_VRAM_ADD_INCREMENT_BIT: u8 = 2;
const PPU_SPRITE_PATTERN_ADDR_BIT: u8 = 3;
const PPU_BACKRGROUND_ADDR_BIT: u8 = 4;
const PPU_SPRITE_SIZE_BIT: u8 = 5;
const PPU_MASTER_SLAVE_SELECT_BIT: u8 = 6;
const PPU_VBLANK_NMI_BIT: u8 = 7;

pub struct Ppu<M: Memory> {
    ctrl: u8,
    mask: u8,
    status: u8,
    oam_addr: u8,
    oam_data: u8,
    scroll: u8,
    addr: PpuAddr,
    data_buffer: u8,
    oam_dma: u8,

    memory: M,
}

impl Ppu<PpuMemory> {
    pub fn new(chr_rom: Vec<u8>, mirroring: NametableMirroring) -> Self {
        Self {
            ctrl: 0,
            mask: 0,
            status: 0,
            oam_addr: 0,
            oam_data: 0,
            scroll: 0,
            addr: PpuAddr::new(),
            data_buffer: 0,
            oam_dma: 0,
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
            scroll: 0,
            addr: PpuAddr::new(),
            data_buffer: 0,
            oam_dma: 0,
            memory,
        }
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

    fn addr_increment_amount(&self) -> u8 {
        if !self.get_ctrl_bit(PPU_VRAM_ADD_INCREMENT_BIT) {
            1 // across
        } else {
            32 // down
        }
    }
}

#[cfg(test)]
mod test {
    use crate::memory::DummyMemory;
    use crate::ppu::{Ppu, PPU_VRAM_ADD_INCREMENT_BIT};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    pub fn test_ppu_addr() {
        let memory = Rc::new(RefCell::new(DummyMemory::new()));
        let mut ppu = Ppu::new_with_memory(memory.clone());

        ppu.set_ctrl_bit(PPU_VRAM_ADD_INCREMENT_BIT, false);

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

        ppu.set_ctrl_bit(PPU_VRAM_ADD_INCREMENT_BIT, true);
        ppu.read_ppu_data();
        assert_eq!(ppu.addr.get_addr(), 0x3479); // now it increments by 0x20 because of the changed ctrl bit
        ppu.set_ctrl_bit(PPU_VRAM_ADD_INCREMENT_BIT, false);

        ppu.addr.set(0x3f);
        ppu.addr.set(0xBD);
        assert_eq!(ppu.addr.get_addr(), 0x3fBD);
        assert_eq!(ppu.read_ppu_data(), 0xBD); // read in palette range returns value immediately

        ppu.addr.set(0x3f);
        ppu.addr.set(0xff);
        ppu.read_ppu_data();
        assert_eq!(ppu.addr.get_addr(), 0x0000); // wraparound after 0x3fff
    }
}
