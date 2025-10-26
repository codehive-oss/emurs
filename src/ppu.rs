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

    fn increment_addr(&mut self, amount: u16) {
        //TODO
    }
}

const PPU_CTRL_NAMETABLE_MASK: u8 = 0x3;
const PPU_VRAM_ADD_INCREMENT_BIT: u8 = 2;
const PPU_SPRITE_PATTERN_ADDR_BIT: u8 = 3;
const PPU_BACKRGROUND_ADDR_BIT: u8 = 4;
const PPU_SPRITE_SIZE_BIT: u8 = 5;
const PPU_MASTER_SLAVE_SELECT_BIT: u8 = 6;
const PPU_VBLANK_NMI_BIT: u8 = 7;

struct PpuRegisters {
    ctrl: u8,
    mask: u8,
    status: u8,
    oam_addr: u8,
    oam_data: u8,
    scroll: u8,
    addr: PpuAddr,
    data_buffer: u8,
    oam_dma: u8,
}

impl PpuRegisters {
    pub fn new() -> Self {
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
        }
    }
}

pub struct Ppu {
    registers: PpuRegisters,
    memory: PpuMemory,
}

impl Ppu {
    pub fn new(chr_rom: Vec<u8>, mirroring: NametableMirroring) -> Self {
        Self {
            registers: PpuRegisters::new(),
            memory: PpuMemory::new(chr_rom, mirroring),
        }
    }
}
