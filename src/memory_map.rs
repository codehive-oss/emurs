use crate::nes_rom::NesRom;
use crate::ram::RAM;

pub struct MemoryMap {
    sram: RAM,
    rom: NesRom,
    prg_ram: RAM
}

impl MemoryMap {
    pub fn new(rom: NesRom) -> Self {
        Self {
            sram: RAM::new(0x8000),
            rom,
            prg_ram: RAM::new(0x2000)
        }
    }

    pub fn read(&self, a: u16) -> u8 {
        if a < 0x2000 {
            self.sram.read(a & 0x07FF)
        } else if a >= 0x6000 && a < 0x8000 {
            self.prg_ram.read(a - 0x6000)
        } else if a >= 0x8000 {
            self.rom.prg_rom[(a as usize - 0x8000) % self.rom.prg_rom.len()]
        } else {
            0
            // panic!("Tried to read unmapped address: {:#X}", a)
        }
    }

    pub fn write(&mut self, a: u16, v: u8) {
        if a < 0x2000 {
            self.sram.write(a & 0x07FF, v);
        } else if a >= 0x6000 && a < 0x8000 {
            self.prg_ram.write(a - 0x6000, v);
        }  else {
            // panic!("Tried to write to unmapped address: {:#X}", a)
        }
    }

    pub fn reset_vector(&self) -> u16 {
        ((self.read(0xFFFD) as u16) << 8) | (self.read(0xFFFD) as u16)
    }
}
