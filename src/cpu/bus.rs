use crate::cpu::controller::Controller;
use crate::cpu::{INTERRUPT_VECTOR_RES_HI, INTERRUPT_VECTOR_RES_LO};
use crate::memory::{Memory, Ram};
use crate::nes_rom::NesRom;
use crate::ppu::ppu_memory::PpuMemory;
use crate::ppu::Ppu;

pub struct Bus {
    sram: Ram,
    pub rom: NesRom,
    prg_ram: Ram,
    pub ppu: Ppu<PpuMemory>,
    pub controller: Controller,
    pub cycle: u32,
}

impl Bus {
    pub fn new(rom: NesRom) -> Self {
        Self {
            sram: Ram::new(0x8000),
            rom: rom.clone(),
            prg_ram: Ram::new(0x2000),
            ppu: Ppu::new(rom.chr_rom, rom.nametable_mirroring),
            controller: Controller::new(),
            cycle: 0,
        }
    }

    pub fn tick(&mut self, cycle: u32) {
        let delta = cycle - self.cycle;
        self.cycle = cycle;
        self.ppu.tick(delta * 3);
    }

    pub fn poll_nmi(&mut self) -> bool {
        self.ppu.poll_nmi()
    }

    pub fn poll_new_frame(&mut self) -> bool {
        self.ppu.poll_new_frame()
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
        } else if a == 0x4016  {
            self.controller.read()
        } else if a == 0x4017 {
            // TODO player 2 controller
            0
        }else if (0x6000..0x8000).contains(&a) {
            self.prg_ram.read(a - 0x6000)
        } else if a >= 0x8000 {
            self.rom.prg_rom[(a as usize - 0x8000) % self.rom.prg_rom.len()]
        } else {
            println!("Tried to read unmapped address: {:#X}", a);
            0
            // panic!("Tried to read unmapped address: {:#X}", a)
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
                3 => {
                    // unimplemented!()
                }
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
            // unimplemented!()
        } else if a == 0x4016 {
            self.controller.write(v);
        }else if (0x4000..=0x4017).contains(&a) {
            // TODO APU
        } else if (0x6000..0x8000).contains(&a) {
            self.prg_ram.write(a - 0x6000, v);
        } else {
            panic!("Tried to write to unmapped address: {:#X}", a)
        }
    }

    pub fn reset_vector(&self) -> u16 {
        let hi = self.rom.prg_rom[(INTERRUPT_VECTOR_RES_HI - 0x8000) as usize % self.rom.prg_rom.len()] as u16;
        let lo = self.rom.prg_rom[(INTERRUPT_VECTOR_RES_LO - 0x8000) as usize % self.rom.prg_rom.len()] as u16;
        (hi << 8) | lo
    }
}
