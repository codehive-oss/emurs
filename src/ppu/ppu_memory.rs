use crate::memory::{Memory, Ram};
use crate::nes_rom::NametableMirroring;

pub struct PpuMemory {
    chr_rom: Vec<u8>,
    vram: Ram,
    mirroring: NametableMirroring,
    palette_table: Ram,
}

impl PpuMemory {
    pub fn new(chr_rom: Vec<u8>, mirroring: NametableMirroring) -> Self {
        Self {
            chr_rom,
            vram: Ram::new(0x800),
            mirroring,
            palette_table: Ram::new(0x20),
        }
    }

    fn mirror_vram_addr(&self, vram_addr: u16) -> u16 {
        match self.mirroring {
            NametableMirroring::Vertical => match vram_addr {
                0x000..0x400 => vram_addr,
                0x400..0x800 => vram_addr,
                0x800..0xC00 => vram_addr - 0x800,
                0xC00..0x1000 => vram_addr - 0x800,
                _ => unreachable!(),
            },
            NametableMirroring::Horizontal => match vram_addr {
                0x000..0x400 => vram_addr,
                0x400..0x800 => vram_addr - 0x400,
                0x800..0xC00 => vram_addr - 0x400,
                0xC00..0x1000 => vram_addr - 0x800,
                _ => unreachable!(),
            },
        }
    }
}

impl Memory for PpuMemory {
    fn read(&self, addr: u16) -> u8 {
        if addr < 0x2000 {
            self.chr_rom[addr as usize]
        } else if (0x2000..0x3000).contains(&addr) {
            let vram_addr = addr - 0x2000;
            self.vram.read(self.mirror_vram_addr(vram_addr))
        } else if (0x3000..0x3F00).contains(&addr) {
            panic!("Unexpected PPU memory read  in range 0x3000-0x3EFF")
        } else if (0x3F00..0x4000).contains(&addr) {
            self.palette_table
                .read((addr - 0x3F00) % (self.palette_table.size() as u16))
        } else {
            panic!("Invalid PPU memory read: {:#X}", addr);
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        if addr < 0x2000 {
            panic!("Attempted write to CHR ROM at {:#X}", addr);
        } else if (0x2000..0x3000).contains(&addr) {
            let vram_addr = addr - 0x2000;
            self.vram.write(self.mirror_vram_addr(vram_addr), data);
        } else if (0x3000..0x3F00).contains(&addr) {
            panic!("Unexpected PPU memory write in range 0x3000-0x3EFF")
        } else if (0x3F00..0x4000).contains(&addr) {
            self.palette_table
                .write((addr - 0x3F00) % (self.palette_table.size() as u16), data);
        } else {
            panic!("Invalid PPU memory write: {:#X}", addr);
        }
    }
}
