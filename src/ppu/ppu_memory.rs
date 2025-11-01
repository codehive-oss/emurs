use crate::memory::Memory;
use crate::nes_rom::NametableMirroring;

pub struct PpuMemory {
    chr_rom: Vec<u8>,
    mirroring: NametableMirroring,
}

impl PpuMemory {
    pub fn new(chr_rom: Vec<u8>, mirroring: NametableMirroring) -> Self {
        Self { chr_rom, mirroring }
    }
}

impl Memory for PpuMemory {
    fn read(&self, addr: u16) -> u8 {
        if addr < 0x2000 {
            self.chr_rom[addr as usize]
        } else {
            unimplemented!()
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        unimplemented!();
    }
}
