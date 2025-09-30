use std::fmt;
use std::thread::sleep;
use std::time::Duration;
use std::{fs::File, io::Read};

struct Registers {
    pc: u16, // Program Counter
    a: u8,   // Accumulator
    x: u8,   // Index Register X
    y: u8,   // Index Register Y
    p: u8,   // Status Register
    s: u8,   // Stack Pointer
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PC:{:04X}  A:{:02X}  X:{:02X}  Y:{:02X}  P:{:02X}  S:{:02X}\nN|V|-|B|D|I|Z|C\n{}|{}|{}|{}|{}|{}|{}|{}\n",
            self.pc, self.a, self.x, self.y, self.p, self.s,
            if (self.p & (1 << 7)) != 0 { 1 } else {0},
            if (self.p & (1 << 6)) != 0 { 1 } else {0},
            if (self.p & (1 << 5)) != 0 { 1 } else {0},
            if (self.p & (1 << 4)) != 0 { 1 } else {0},
            if (self.p & (1 << 3)) != 0 { 1 } else {0},
            if (self.p & (1 << 2)) != 0 { 1 } else {0},
            if (self.p & (1 << 1)) != 0 { 1 } else {0},
            if (self.p & (1 << 0)) != 0 { 1 } else {0},
        )
    }
}

impl Registers {
    fn new() -> Self {
        Self {
            a: 0,
            x: 0,
            y: 0,
            s: 0,
            pc: 0x0400,
            p: 0,
        }
    }

    fn update_zn_flags(&mut self, value: u8) {
        self.update_zero_bit(value == 0x00);
        self.update_negative_bit((value & (1 << 7)) != 0x00);
    }
    fn update_a(&mut self, value: u8) {
        self.a = value;
        self.update_zn_flags(value);
    }
    fn update_x(&mut self, value: u8) {
        self.x = value;
        self.update_zn_flags(value);
    }
    fn update_y(&mut self, value: u8) {
        self.y = value;
        self.update_zn_flags(value);
    }

    fn update_status_bit(&mut self, bit: u32, value: bool) {
        if value {
            self.p |= 1 << bit;
        } else {
            self.p &= !(1 << bit);
        }
    }
    fn get_status_bit(&self, bit: u32) -> bool {
        self.p & (1 << bit) != 0
    }

    fn update_carry_bit(&mut self, value: bool) {
        self.update_status_bit(0, value)
    }
    fn get_carry_bit(&self) -> bool {
        self.get_status_bit(0)
    }

    fn update_zero_bit(&mut self, value: bool) {
        self.update_status_bit(1, value)
    }
    fn get_zero_bit(&self) -> bool {
        self.get_status_bit(1)
    }

    fn update_interupt_bit(&mut self, value: bool) {
        self.update_status_bit(2, value)
    }
    fn get_interupt_bit(&self) -> bool {
        self.get_status_bit(2)
    }

    fn update_decimal_bit(&mut self, value: bool) {
        self.update_status_bit(3, value)
    }
    fn get_decimal_bit(&self) -> bool {
        self.get_status_bit(3)
    }

    fn update_break_bit(&mut self, value: bool) {
        self.update_status_bit(4, value)
    }
    fn get_break_bit(&self) -> bool {
        self.get_status_bit(4)
    }

    fn update_overflow_bit(&mut self, value: bool) {
        self.update_status_bit(6, value)
    }
    fn get_overflow_bit(&self) -> bool {
        self.get_status_bit(6)
    }

    fn update_negative_bit(&mut self, value: bool) {
        self.update_status_bit(7, value)
    }
    fn get_negative_bit(&self) -> bool {
        self.get_status_bit(7)
    }
}

struct Memory(Box<[u8; 0x10000]>);

impl Memory {
    fn new() -> Self {
        Self(Box::new([0; 0x10000]))
    }

    #[inline]
    fn read(&self, a: u16) -> u8 {
        self.0[a as usize]
    }

    #[inline]
    fn write(&mut self, a: u16, v: u8) {
        self.0[a as usize] = v;
    }
}

struct Emulator {
    registers: Registers,
    memory: Memory,
    clock_speed: u32,
}

impl Emulator {
    fn clock_cycle(&self) {
        let sec: f64 = 1.0 / f64::from(self.clock_speed);
        sleep(Duration::from_secs_f64(sec));
    }

    fn read_memory(&self, a: u16) -> u8 {
        self.clock_cycle();
        self.memory.read(a)
    }

    fn write_memory(&mut self, a: u16, v: u8) {
        self.clock_cycle();
        self.memory.write(a, v)
    }

    fn next(&mut self) -> u8 {
        let current = self.read_memory(self.registers.pc);
        self.registers.pc = self.registers.pc.wrapping_add(1);

        current
    }

    fn read_zeropage(&mut self) -> u8 {
        let addr = self.next();
        self.read_memory(u16::from(addr))
    }

    fn read_zeropage_x(&mut self) -> u8 {
        let addr = self.next();
        let addr = addr.wrapping_add(self.registers.x);
        self.clock_cycle();
        self.read_memory(u16::from(addr))
    }

    fn read_zeropage_y(&mut self) -> u8 {
        let addr = self.next();
        let addr = addr.wrapping_add(self.registers.y);
        self.clock_cycle();
        self.read_memory(u16::from(addr))
    }

    fn read_absolute(&mut self) -> u8 {
        let lo = self.next();
        let hi = self.next();
        let addr = u16::from(lo) | (u16::from(hi) << 8);
        self.read_memory(addr)
    }

    fn read_absolute_x(&mut self) -> u8 {
        let lo = self.next();
        let mut hi = self.next();
        let (lo, has_overflow) = lo.overflowing_add(self.registers.x);
        self.clock_cycle();

        if has_overflow {
            hi += 1;
            self.clock_cycle();
        }

        let addr = u16::from(lo) | (u16::from(hi) << 8);
        self.read_memory(addr)
    }

    fn read_absolute_y(&mut self) -> u8 {
        let lo = self.next();
        let mut hi = self.next();
        let (lo, has_overflow) = lo.overflowing_add(self.registers.y);
        self.clock_cycle();

        if has_overflow {
            hi += 1;
            self.clock_cycle();
        }

        let addr = u16::from(lo) | (u16::from(hi) << 8);
        self.read_memory(addr)
    }

    fn adc(&mut self, data: u8) {
        let carry = if self.registers.get_carry_bit() { 1 } else { 0 };
        let (result, carry_1) = self.registers.a.overflowing_add(data);
        let (result, carry_2) = result.overflowing_add(carry);
        self.registers.update_carry_bit(carry_1 || carry_2);
        self.registers
            .update_overflow_bit(((self.registers.a ^ result) & (data ^ result) & 0x80) != 0);
        self.registers.update_a(result);
    }

    fn sbc(&mut self, data: u8) {
        self.adc(!data);
    }
}

impl Emulator {
    fn nop(&mut self) {}

    fn adc_immediate(&mut self) {
        let data = self.next();
        self.adc(data);
    }
    fn adc_zeropage(&mut self) {
        let data = self.read_zeropage();
        self.adc(data);
    }
    fn adc_zeropage_x(&mut self) {
        let data = self.read_zeropage_x();
        self.adc(data);
    }
    fn adc_absolute(&mut self) {
        let data = self.read_absolute();
        self.adc(data);
    }
    fn adc_absolute_x(&mut self) {
        let data = self.read_absolute_x();
        self.adc(data);
    }
    fn adc_absolute_y(&mut self) {
        let data = self.read_absolute_y();
        self.adc(data);
    }
    // fn adc_preindexed_indirect_x(&mut self) {}
    // fn adc_postindexed_indirect_y(&mut self) {}

    // fn and_immediate(&mut self) {}
    // fn and_zeropage(&mut self) {}
    // fn and_zeropage_x(&mut self) {}
    // fn and_absolute(&mut self) {}
    // fn and_absolute_x(&mut self) {}
    // fn and_absolute_y(&mut self) {}
    // fn and_preindexed_indirect_x(&mut self) {}
    // fn and_postindexed_indirect_y(&mut self) {}
    //
    fn sbc_immediate(&mut self) {
        let data = self.next();
        self.sbc(data);
    }
    fn sbc_zeropage(&mut self) {
        let data = self.read_zeropage();
        self.sbc(data);
    }
    fn sbc_zeropage_x(&mut self) {
        let data = self.read_zeropage_x();
        self.sbc(data);
    }
    fn sbc_absolute(&mut self) {
        let data = self.read_absolute();
        self.sbc(data);
    }
    fn sbc_absolute_x(&mut self) {
        let data = self.read_absolute_x();
        self.sbc(data);
    }
    fn sbc_absolute_y(&mut self) {
        let data = self.read_absolute_y();
        self.sbc(data);
    }

    fn lda_immediate(&mut self) {
        let data = self.next();
        self.registers.update_a(data);
    }
    fn lda_zeropage(&mut self) {
        let data = self.read_zeropage();
        self.registers.update_a(data);
    }
    fn lda_zeropage_x(&mut self) {
        let data = self.read_zeropage_x();
        self.registers.update_a(data);
    }
    fn lda_absolute(&mut self) {
        let data = self.read_absolute();
        self.registers.update_a(data);
    }
    fn lda_absolute_x(&mut self) {
        let data = self.read_absolute_x();
        self.registers.update_a(data);
    }
    fn lda_absolute_y(&mut self) {
        let data = self.read_absolute_y();
        self.registers.update_a(data);
    }
    // fn lda_preindexed_indirect_x(&mut self) {}
    // fn lda_postindexed_indirect_y(&mut self) {}

    fn ldx_immediate(&mut self) {
        let data = self.next();
        self.registers.update_x(data);
    }
    fn ldx_zeropage(&mut self) {
        let data = self.read_zeropage();
        self.registers.update_x(data);
    }
    fn ldx_zeropage_y(&mut self) {
        let data = self.read_zeropage_y();
        self.registers.update_x(data);
    }
    fn ldx_absolute(&mut self) {
        let data = self.read_absolute();
        self.registers.update_x(data);
    }
    fn ldx_absolute_y(&mut self) {
        let data = self.read_absolute_y();
        self.registers.update_x(data);
    }

    fn ldy_immediate(&mut self) {
        let data = self.next();
        self.registers.update_y(data);
    }
    fn ldy_zeropage(&mut self) {
        let data = self.read_zeropage();
        self.registers.update_y(data);
    }
    fn ldy_zeropage_x(&mut self) {
        let data = self.read_zeropage_x();
        self.registers.update_y(data);
    }
    fn ldy_absolute(&mut self) {
        let data = self.read_absolute();
        self.registers.update_y(data);
    }
    fn ldy_absolute_x(&mut self) {
        let data = self.read_absolute_x();
        self.registers.update_y(data);
    }

    fn sta_zeropage(&mut self) {
        let addr = self.next();
        self.write_memory(u16::from(addr), self.registers.a);
    }
    fn sta_zeropage_x(&mut self) {
        let addr = self.next();

        let addr = addr.wrapping_add(self.registers.x);
        self.clock_cycle();

        self.write_memory(u16::from(addr), self.registers.a);
    }
    fn sta_absolute(&mut self) {
        let lo = self.next();
        let hi = self.next();
        let addr = u16::from(lo) | (u16::from(hi) << 8);

        self.write_memory(addr, self.registers.a);
    }

    fn jmp_absolute(&mut self) {
        let lo = self.next();
        let hi = self.next();
        let data = u16::from(lo) | (u16::from(hi) << 8);
        self.registers.pc = data;
    }
    fn jmp_indirect(&mut self) {
        let lo = self.next();
        let hi = self.next();
        let addr = u16::from(lo) | (u16::from(hi) << 8);
        let data_lo = self.read_memory(addr);
        let data_hi = self.read_memory(addr.wrapping_add(1));
        let data = u16::from(data_lo) | (u16::from(data_hi) << 8);
        self.registers.pc = data;
    }

    fn tax(&mut self) {
        self.registers.update_x(self.registers.a);
        self.clock_cycle();
    }
    fn tay(&mut self) {
        self.registers.update_y(self.registers.a);
        self.clock_cycle();
    }
    fn tsx(&mut self) {
        self.registers.update_x(self.registers.s);
        self.clock_cycle();
    }
    fn txa(&mut self) {
        self.registers.update_a(self.registers.x);
        self.clock_cycle();
    }
    fn txs(&mut self) {
        self.registers.s = self.registers.x;
        self.clock_cycle();
    }
    fn tya(&mut self) {
        self.registers.update_a(self.registers.y);
        self.clock_cycle();
    }

    fn branch_on_condition(&mut self, cond: bool) {
        let addr = self.next() as i8;
        if cond {
            self.clock_cycle();

            if addr < 0 {
                let addr = -addr as u16;
                let (data, has_underflow) = self.registers.pc.overflowing_sub(addr);

                if has_underflow {
                    self.clock_cycle();
                }

                self.registers.pc = data;
            } else {
                let (data, has_overflow) = self.registers.pc.overflowing_add(addr as u16);
                if has_overflow {
                    self.clock_cycle();
                }

                self.registers.pc = data;
            }
        }
    }
    fn bcc(&mut self) {
        self.branch_on_condition(!self.registers.get_carry_bit())
    }
    fn bcs(&mut self) {
        self.branch_on_condition(self.registers.get_carry_bit())
    }
    fn beq(&mut self) {
        self.branch_on_condition(self.registers.get_zero_bit())
    }
    fn bmi(&mut self) {
        self.branch_on_condition(self.registers.get_negative_bit())
    }
    fn bne(&mut self) {
        self.branch_on_condition(!self.registers.get_zero_bit())
    }
    fn bpl(&mut self) {
        self.branch_on_condition(!self.registers.get_negative_bit())
    }
    fn bvc(&mut self) {
        self.branch_on_condition(!self.registers.get_overflow_bit())
    }
    fn bvs(&mut self) {
        self.branch_on_condition(self.registers.get_overflow_bit())
    }

    fn clc(&mut self) {
        self.registers.update_carry_bit(false);
        self.clock_cycle();
    }
    fn cld(&mut self) {
        self.registers.update_decimal_bit(false);
        self.clock_cycle();
    }
    fn cli(&mut self) {
        self.registers.update_interupt_bit(false);
        self.clock_cycle();
    }
    fn clv(&mut self) {
        self.registers.update_overflow_bit(false);
        self.clock_cycle();
    }

    fn dex(&mut self) {
        self.registers.update_x(self.registers.x.wrapping_sub(1));
        self.clock_cycle();
    }

    fn dey(&mut self) {
        self.registers.update_y(self.registers.y.wrapping_sub(1));
        self.clock_cycle();
    }
}

impl Emulator {
    fn new(path: &str) -> Result<Self, anyhow::Error> {
        let mut emulator = Self {
            registers: Registers::new(),
            memory: Memory::new(),
            clock_speed: 4,
        };

        let mut file = File::open(path)?;
        file.read(&mut *emulator.memory.0)?;

        Ok(emulator)
    }

    fn run(&mut self) {
        loop {
            println!("{}", self.registers);
            let instruction = self.next();
            println!("interpreting {instruction:X}");

            match instruction {
                0xEA => self.nop(),
                0x69 => self.adc_immediate(),
                0x65 => self.adc_zeropage(),
                0x75 => self.adc_zeropage_x(),
                0x6D => self.adc_absolute(),
                0x7D => self.adc_absolute_x(),
                0x79 => self.adc_absolute_y(),
                // 0x61 => self.adc_preindexed_indirect_x(),
                // 0x71 => self.adc_postindexed_indirect_y(),
                0xE9 => self.sbc_immediate(),
                0xE5 => self.sbc_zeropage(),
                0xF5 => self.sbc_zeropage_x(),
                0xED => self.sbc_absolute(),
                0xFD => self.sbc_absolute_x(),
                0xF9 => self.sbc_absolute_y(),
                // 0x29 => self.and_immediate(),
                // 0x25 => self.and_zeropage(),
                // 0x35 => self.and_zeropage_x(),
                // 0x2D => self.and_absolute(),
                // 0x3D => self.and_absolute_x(),
                // 0x39 => self.and_absolute_y(),
                // 0x21 => self.and_preindexed_indirect_x(),
                // 0x31 => self.and_postindexed_indirect_y(),
                0x4C => self.jmp_absolute(),
                0x6C => self.jmp_indirect(),

                0xA9 => self.lda_immediate(),
                0xA5 => self.lda_zeropage(),
                0xB5 => self.lda_zeropage_x(),
                0xAD => self.lda_absolute(),
                0xBD => self.lda_absolute_x(),
                0xB9 => self.lda_absolute_y(),

                0xA2 => self.ldx_immediate(),
                0xA6 => self.ldx_zeropage(),
                0xB6 => self.ldx_zeropage_y(),
                0xAE => self.ldx_absolute(),
                0xBE => self.ldx_absolute_y(),

                0xA0 => self.ldy_immediate(),
                0xA4 => self.ldy_zeropage(),
                0xB4 => self.ldy_zeropage_x(),
                0xAC => self.ldy_absolute(),
                0xBC => self.ldy_absolute_x(),

                0x85 => self.sta_zeropage(),
                0x95 => self.sta_zeropage_x(),
                0x8D => self.sta_absolute(),
                // 0x9D => self.sta_absolute_x(),
                // 0x99 => self.sta_absolute_y(),
                // 0x81 => self.sta_preindexed_indirect_x(),
                // 0x91 => self.sta_postindexed_indirect_y(),
                0xAA => self.tax(),
                0xA8 => self.tay(),
                0xBA => self.tsx(),
                0x8A => self.txa(),
                0x9A => self.txs(),
                0x98 => self.tya(),

                0x90 => self.bcc(),
                0xB0 => self.bcs(),
                0xF0 => self.beq(),
                0x30 => self.bmi(),
                0xD0 => self.bne(),
                0x10 => self.bpl(),
                0x50 => self.bvc(),
                0x70 => self.bvs(),

                0x18 => self.clc(),
                0xD8 => self.cld(),
                0x58 => self.cli(),
                0xB8 => self.clv(),

                0xCA => self.dex(),
                0x88 => self.dey(),

                x => {
                    unimplemented!("instruction not implemented: {:X}", x);
                }
            }
        }
    }
}

fn main() -> Result<(), anyhow::Error> {
    println!("Starting Emulator!");

    let mut emulator =
        Emulator::new("./6502_65C02_functional_tests/bin_files/6502_functional_test.bin")?;
    emulator.run();

    Ok(())
}
