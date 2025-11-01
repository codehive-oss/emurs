use crate::memory::{Memory, Ram};
use crate::nes_rom::NesRom;
use crate::ppu::ppu_memory::PpuMemory;
use crate::ppu::Ppu;

pub struct CpuMemory {
    sram: Ram,
    rom: NesRom,
    prg_ram: Ram,
    ppu: Ppu<PpuMemory>,
}

impl CpuMemory {
    pub fn new(rom: NesRom) -> Self {
        Self {
            sram: Ram::new(0x8000),
            rom: rom.clone(),
            prg_ram: Ram::new(0x2000),
            ppu: Ppu::new(rom.chr_rom, rom.nametable_mirroring),
        }
    }

    pub fn read(&mut self, a: u16) -> u8 {
        if a < 0x2000 {
            self.sram.read(a & 0x07FF)
        } else if (0x2000..0x4000).contains(&a) {
            let register = (a - 0x2000) % 8;
            match register {
                2 => self.ppu.read_ppu_status(),
                4 => unimplemented!(),
                7 => self.ppu.read_ppu_data(),
                _ => panic!(
                    "Unexpected PPU register read: {:#X} (Register {})",
                    a, register
                ),
            }
        } else if (0x6000..0x8000).contains(&a) {
            self.prg_ram.read(a - 0x6000)
        } else if a >= 0x8000 {
            self.rom.prg_rom[(a as usize - 0x8000) % self.rom.prg_rom.len()]
        } else {
            panic!("Tried to read unmapped address: {:#X}", a)
        }
    }

    pub fn write(&mut self, a: u16, v: u8) {
        if a < 0x2000 {
            self.sram.write(a & 0x07FF, v);
        } else if (0x2000..0x4000).contains(&a) {
            let register = (a - 0x2000) % 8;
            match register {
                0 => self.ppu.write_ppu_ctrl(v),
                1 => self.ppu.write_ppu_mask(v),
                3 => unimplemented!(),
                4 => unimplemented!(),
                5 => self.ppu.write_ppu_scroll(v),
                6 => self.ppu.write_ppu_addr(v),
                7 => self.ppu.write_ppu_data(v),
                _ => panic!(
                    "Unexpected PPU register read: {:#X} (Register {})",
                    a, register
                ),
            };
        } else if a == 0x4014 {
            unimplemented!()
        } else if (0x6000..0x8000).contains(&a) {
            self.prg_ram.write(a - 0x6000, v);
        } else {
            panic!("Tried to write to unmapped address: {:#X}", a)
        }
    }

    pub fn reset_vector(&self) -> u16 {
        let hi = self.rom.prg_rom[(0xFFFD - 0x8000) % self.rom.prg_rom.len()] as u16;
        let lo = self.rom.prg_rom[(0xFFFC - 0x8000) % self.rom.prg_rom.len()] as u16;
        (hi << 8) | lo
    }
}
