use crate::ram::RAM;
use std::fmt;
use std::thread::sleep;
use std::time::Duration;

const STATUS_NEGATIVE_BIT: u32 = 7;
const STATUS_OVERFLOW_BIT: u32 = 6;
const STATUS_IGNORED_BIT: u32 = 5;
const STATUS_BREAK_BIT: u32 = 4;
const STATUS_DECIMAL_BIT: u32 = 3;
const STATUS_INTERRUPT_BIT: u32 = 2;
const STATUS_ZERO_BIT: u32 = 1;
const STATUS_CARRY_BIT: u32 = 0;

const INTERRUPT_VECTOR_NMI_LO: u16 = 0xFFFA;
const INTERRUPT_VECTOR_NMI_HI: u16 = 0xFFFB;
const INTERRUPT_VECTOR_RES_LO: u16 = 0xFFFC;
const INTERRUPT_VECTOR_RES_HI: u16 = 0xFFFD;
const INTERRUPT_VECTOR_IRQ_LO: u16 = 0xFFFE;
const INTERRUPT_VECTOR_IRQ_HI: u16 = 0xFFFF;

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
            "PC:{:04X}  A:{:02X}  X:{:02X}  Y:{:02X}  P:{:02X}  S:{:02X}\nN|V|-|B|D|I|Z|C\n{}|{}|{}|{}|{}|{}|{}|{}",
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
        self.update_status_bit(STATUS_CARRY_BIT, value)
    }
    fn get_carry_bit(&self) -> bool {
        self.get_status_bit(STATUS_CARRY_BIT)
    }

    fn update_zero_bit(&mut self, value: bool) {
        self.update_status_bit(STATUS_ZERO_BIT, value)
    }
    fn get_zero_bit(&self) -> bool {
        self.get_status_bit(STATUS_ZERO_BIT)
    }

    fn update_interupt_bit(&mut self, value: bool) {
        self.update_status_bit(STATUS_INTERRUPT_BIT, value)
    }
    fn get_interupt_bit(&self) -> bool {
        self.get_status_bit(STATUS_INTERRUPT_BIT)
    }

    fn update_decimal_bit(&mut self, value: bool) {
        self.update_status_bit(STATUS_DECIMAL_BIT, value)
    }
    fn get_decimal_bit(&self) -> bool {
        self.get_status_bit(STATUS_DECIMAL_BIT)
    }

    fn update_overflow_bit(&mut self, value: bool) {
        self.update_status_bit(STATUS_OVERFLOW_BIT, value)
    }
    fn get_overflow_bit(&self) -> bool {
        self.get_status_bit(STATUS_OVERFLOW_BIT)
    }

    fn update_negative_bit(&mut self, value: bool) {
        self.update_status_bit(STATUS_NEGATIVE_BIT, value)
    }
    fn get_negative_bit(&self) -> bool {
        self.get_status_bit(STATUS_NEGATIVE_BIT)
    }
}

enum Access {
    Read,
    Write,
    ReadModify,
}

pub struct CPU {
    registers: Registers,
    memory: RAM,
    clock_speed: u32,
}

impl CPU {
    fn clock_cycle(&self) {
        if self.clock_speed == 0 {
            return;
        }
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

    fn addr_zeropage(&mut self) -> u16 {
        let addr = self.next();
        u16::from(addr)
    }

    fn addr_zeropage_x(&mut self) -> u16 {
        let addr = self.next();
        let addr = addr.wrapping_add(self.registers.x);
        self.clock_cycle();
        u16::from(addr)
    }

    fn addr_zeropage_y(&mut self) -> u16 {
        let addr = self.next();
        let addr = addr.wrapping_add(self.registers.y);
        self.clock_cycle();
        u16::from(addr)
    }

    fn addr_absolute(&mut self) -> u16 {
        let lo = self.next();
        let hi = self.next();
        let addr = u16::from(lo) | (u16::from(hi) << 8);
        addr
    }

    fn addr_absolute_x(&mut self, access: Access) -> u16 {
        let lo = self.next();
        let hi = self.next();
        let base = u16::from(lo) | (u16::from(hi) << 8);

        let x = self.registers.x as u16;
        let addr = base.wrapping_add(x);

        let page_cross = (base & 0xFF00) != (addr & 0xFF00);
        match access {
            Access::Read => {
                if page_cross {
                    self.clock_cycle();
                }
            }
            Access::Write | Access::ReadModify => {
                self.clock_cycle();
            }
        }

        addr
    }

    fn addr_absolute_y(&mut self, access: Access) -> u16 {
        let lo = self.next();
        let hi = self.next();
        let base = u16::from(lo) | (u16::from(hi) << 8);

        let y = self.registers.y as u16;
        let addr = base.wrapping_add(y);

        let page_cross = (base & 0xFF00) != (addr & 0xFF00);
        match access {
            Access::Read => {
                if page_cross {
                    self.clock_cycle();
                }
            }
            Access::Write | Access::ReadModify => {
                self.clock_cycle();
            }
        }

        addr
    }

    fn addr_preindexed_indirect_zeropage_x(&mut self) -> u16 {
        let first_addr = self.next();
        let first_addr = first_addr.wrapping_add(self.registers.x);
        self.clock_cycle();

        let lo = self.read_memory(u16::from(first_addr));
        let hi = self.read_memory(u16::from(first_addr.wrapping_add(1)));
        let addr = u16::from(lo) | (u16::from(hi) << 8);
        addr
    }

    fn addr_postindexed_indirect_zeropage_y(&mut self, access: Access) -> u16 {
        let addr = self.next();
        let lo = self.read_memory(u16::from(addr));
        let hi = self.read_memory(u16::from(addr.wrapping_add(1)));

        let base = u16::from(lo) | (u16::from(hi) << 8);

        let y = self.registers.y as u16;
        let addr = base.wrapping_add(y);

        let page_cross = (base & 0xFF00) != (addr & 0xFF00);
        match access {
            Access::Read => {
                if page_cross {
                    self.clock_cycle();
                }
            }
            Access::Write | Access::ReadModify => {
                self.clock_cycle();
            }
        }

        addr
    }

    fn read_zeropage(&mut self) -> u8 {
        let addr = self.addr_zeropage();
        self.read_memory(addr)
    }

    fn read_zeropage_x(&mut self) -> u8 {
        let addr = self.addr_zeropage_x();
        self.read_memory(addr)
    }

    fn read_zeropage_y(&mut self) -> u8 {
        let addr = self.addr_zeropage_y();
        self.read_memory(u16::from(addr))
    }

    fn read_absolute(&mut self) -> u8 {
        let addr = self.addr_absolute();
        self.read_memory(addr)
    }

    fn read_absolute_x(&mut self) -> u8 {
        let addr = self.addr_absolute_x(Access::Read);
        self.read_memory(addr)
    }

    fn read_absolute_y(&mut self) -> u8 {
        let addr = self.addr_absolute_y(Access::Read);
        self.read_memory(addr)
    }

    fn read_preindexed_indirect_zeropage_x(&mut self) -> u8 {
        let addr = self.addr_preindexed_indirect_zeropage_x();
        self.read_memory(addr)
    }

    fn read_postindexed_indirect_zeropage_y(&mut self) -> u8 {
        let addr = self.addr_postindexed_indirect_zeropage_y(Access::Read);
        self.read_memory(addr)
    }

    fn write_zeropage(&mut self, data: u8) {
        let addr = self.addr_zeropage();
        self.write_memory(addr, data);
    }
    fn write_zeropage_x(&mut self, data: u8) {
        let addr = self.addr_zeropage_x();
        self.write_memory(addr, data);
    }
    fn write_zeropage_y(&mut self, data: u8) {
        let addr = self.addr_zeropage_y();
        self.write_memory(addr, data);
    }
    fn write_absolute(&mut self, data: u8) {
        let addr = self.addr_absolute();
        self.write_memory(addr, data);
    }

    fn adc(&mut self, m: u8) {
        let a = self.registers.a;
        let c = if self.registers.get_carry_bit() { 1 } else { 0 };

        let sum = a as u16 + m as u16 + c;
        let result = sum as u8;

        let v = (!(a ^ m) & (a ^ result) & 0x80) != 0;

        if self.registers.get_decimal_bit() {
            let mut adj = 0u16;
            if ((a & 0x0F) as u16 + (m & 0x0F) as u16 + c) > 9 {
                adj += 0x06;
            }
            if sum > 0x99 {
                adj += 0x60;
            }
            let bcd = result.wrapping_add(adj as u8);

            let carry = sum > 0x99;

            self.registers.update_carry_bit(carry);
            self.registers.update_overflow_bit(v);
            self.registers.update_a(bcd);
        } else {
            self.registers.update_carry_bit(sum > 0xFF);
            self.registers.update_overflow_bit(v);
            self.registers.update_a(result);
        }
    }

    fn sbc(&mut self, m: u8) {
        let a = self.registers.a;
        let c = if self.registers.get_carry_bit() { 1 } else { 0 };

        let diff = a as i16 - m as i16 - (1 - c);
        let result = diff as u8;

        let v = ((a ^ m) & (a ^ result) & 0x80) != 0;

        if self.registers.get_decimal_bit() {
            let mut adj = 0i16;

            if ((a & 0x0F) as i16) - ((m & 0x0F) as i16) - (1 - c) < 0 {
                adj -= 0x06;
            }

            if diff < 0 {
                adj -= 0x60;
            }

            let bcd = (result as i16).wrapping_add(adj) as u8;

            let carry = diff >= 0;

            self.registers.update_carry_bit(carry);
            self.registers.update_overflow_bit(v);
            self.registers.update_a(bcd);
        } else {
            self.registers.update_carry_bit(diff >= 0);
            self.registers.update_overflow_bit(v);
            self.registers.update_a(result);
        }
    }

    fn ora(&mut self, data: u8) {
        self.registers.update_a(self.registers.a | data);
    }

    fn eor(&mut self, data: u8) {
        self.registers.update_a(self.registers.a ^ data);
    }

    fn and(&mut self, data: u8) {
        self.registers.update_a(self.registers.a & data);
    }

    fn inc(&mut self, data: u8) -> u8 {
        let result = data.wrapping_add(1);
        self.registers.update_zn_flags(result);
        result
    }

    fn dec(&mut self, data: u8) -> u8 {
        let result = data.wrapping_sub(1);
        self.registers.update_zn_flags(result);
        result
    }

    fn asl(&mut self, data: u8) -> u8 {
        let carry = (data & 0x80) != 0;
        let result = data << 1;
        self.registers.update_carry_bit(carry);
        self.registers.update_zn_flags(result);
        result
    }

    fn rol(&mut self, data: u8) -> u8 {
        let carry = (data & 0x80) != 0;
        let result = if self.registers.get_carry_bit() {
            (data << 1) | 0x01
        } else {
            (data << 1) & !0x01
        };
        self.registers.update_zn_flags(result);
        self.registers.update_carry_bit(carry);
        result
    }

    fn lsr(&mut self, data: u8) -> u8 {
        let carry = (data & 0x01) != 0;
        let result = data >> 1;
        self.registers.update_zn_flags(result);
        self.registers.update_carry_bit(carry);
        result
    }

    fn ror(&mut self, data: u8) -> u8 {
        let carry = (data & 0x01) != 0;
        let result = if self.registers.get_carry_bit() {
            (data >> 1) | 0x80
        } else {
            (data >> 1) & !0x80
        };
        self.registers.update_zn_flags(result);
        self.registers.update_carry_bit(carry);
        result
    }

    fn cmp(&mut self, data: u8) {
        let (result, carry_1) = self.registers.a.overflowing_add(!data);
        let (result, carry_2) = result.overflowing_add(1);
        self.registers.update_carry_bit(carry_1 || carry_2);
        self.registers.update_zn_flags(result);
    }

    fn cpx(&mut self, data: u8) {
        let (result, carry_1) = self.registers.x.overflowing_add(!data);
        let (result, carry_2) = result.overflowing_add(1);
        self.registers.update_carry_bit(carry_1 || carry_2);
        self.registers.update_zn_flags(result);
    }

    fn cpy(&mut self, data: u8) {
        let (result, carry_1) = self.registers.y.overflowing_add(!data);
        let (result, carry_2) = result.overflowing_add(1);
        self.registers.update_carry_bit(carry_1 || carry_2);
        self.registers.update_zn_flags(result);
    }

    fn bit(&mut self, data: u8) {
        self.registers.update_zero_bit(self.registers.a & data == 0);
        self.registers
            .update_negative_bit(data & (1 << STATUS_NEGATIVE_BIT) != 0);
        self.registers
            .update_overflow_bit(data & (1 << STATUS_OVERFLOW_BIT) != 0);
    }

    fn branch_on_condition(&mut self, cond: bool) {
        let offset = self.next() as i8;
        if cond {
            self.clock_cycle();

            let old = self.registers.pc;
            let new = (old as i16 + offset as i16) as u16;

            if (old ^ new) & 0xFF00 != 0 {
                self.clock_cycle();
            }

            self.registers.pc = new;
        }
    }

    fn push_stack(&mut self, data: u8) {
        self.write_memory(0x0100 + self.registers.s as u16, data);
        self.registers.s = self.registers.s.wrapping_sub(1);
    }

    fn pull_stack(&mut self) -> u8 {
        self.registers.s = self.registers.s.wrapping_add(1);
        self.read_memory(0x0100 + self.registers.s as u16)
    }
}

impl CPU {
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
    fn adc_preindexed_indirect_zeropage_x(&mut self) {
        let data = self.read_preindexed_indirect_zeropage_x();
        self.adc(data);
    }
    fn adc_postindexed_indirect_zeropage_y(&mut self) {
        let data = self.read_postindexed_indirect_zeropage_y();
        self.adc(data);
    }

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
    fn sbc_preindexed_indirect_zeropage_x(&mut self) {
        let data = self.read_preindexed_indirect_zeropage_x();
        self.sbc(data);
    }
    fn sbc_postindexed_indirect_zeropage_y(&mut self) {
        let data = self.read_postindexed_indirect_zeropage_y();
        self.sbc(data);
    }

    fn eor_immediate(&mut self) {
        let data = self.next();
        self.eor(data);
    }
    fn eor_zeropage(&mut self) {
        let data = self.read_zeropage();
        self.eor(data);
    }
    fn eor_zeropage_x(&mut self) {
        let data = self.read_zeropage_x();
        self.eor(data);
    }
    fn eor_absolute(&mut self) {
        let data = self.read_absolute();
        self.eor(data);
    }
    fn eor_absolute_x(&mut self) {
        let data = self.read_absolute_x();
        self.eor(data);
    }
    fn eor_absolute_y(&mut self) {
        let data = self.read_absolute_y();
        self.eor(data);
    }
    fn eor_preindexed_indirect_zeropage_x(&mut self) {
        let data = self.read_preindexed_indirect_zeropage_x();
        self.eor(data);
    }
    fn eor_postindexed_indirect_zeropage_y(&mut self) {
        let data = self.read_postindexed_indirect_zeropage_y();
        self.eor(data);
    }

    fn and_immediate(&mut self) {
        let data = self.next();
        self.and(data);
    }
    fn and_zeropage(&mut self) {
        let data = self.read_zeropage();
        self.and(data);
    }
    fn and_zeropage_x(&mut self) {
        let data = self.read_zeropage_x();
        self.and(data);
    }
    fn and_absolute(&mut self) {
        let data = self.read_absolute();
        self.and(data);
    }
    fn and_absolute_x(&mut self) {
        let data = self.read_absolute_x();
        self.and(data);
    }
    fn and_absolute_y(&mut self) {
        let data = self.read_absolute_y();
        self.and(data);
    }
    fn and_preindexed_indirect_zeropage_x(&mut self) {
        let data = self.read_preindexed_indirect_zeropage_x();
        self.and(data);
    }
    fn and_postindexed_indirect_zeropage_y(&mut self) {
        let data = self.read_postindexed_indirect_zeropage_y();
        self.and(data);
    }

    fn ora_immediate(&mut self) {
        let data = self.next();
        self.ora(data);
    }
    fn ora_zeropage(&mut self) {
        let data = self.read_zeropage();
        self.ora(data);
    }
    fn ora_zeropage_x(&mut self) {
        let data = self.read_zeropage_x();
        self.ora(data);
    }
    fn ora_absolute(&mut self) {
        let data = self.read_absolute();
        self.ora(data);
    }
    fn ora_absolute_x(&mut self) {
        let data = self.read_absolute_x();
        self.ora(data);
    }
    fn ora_absolute_y(&mut self) {
        let data = self.read_absolute_y();
        self.ora(data);
    }
    fn ora_preindexed_indirect_zeropage_x(&mut self) {
        let data = self.read_preindexed_indirect_zeropage_x();
        self.ora(data);
    }
    fn ora_postindexed_indirect_zeropage_y(&mut self) {
        let data = self.read_postindexed_indirect_zeropage_y();
        self.ora(data);
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
    fn lda_preindexed_indirect_zeropage_x(&mut self) {
        let data = self.read_preindexed_indirect_zeropage_x();
        self.registers.update_a(data);
    }
    fn lda_postindexed_indirect_zeropage_y(&mut self) {
        let data = self.read_postindexed_indirect_zeropage_y();
        self.registers.update_a(data);
    }

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
        self.write_zeropage(self.registers.a);
    }
    fn sta_zeropage_x(&mut self) {
        self.write_zeropage_x(self.registers.a);
    }
    fn sta_absolute(&mut self) {
        self.write_absolute(self.registers.a);
    }
    fn sta_absolute_x(&mut self) {
        let addr = self.addr_absolute_x(Access::Write);
        self.write_memory(addr, self.registers.a);
    }
    fn sta_absolute_y(&mut self) {
        let addr = self.addr_absolute_y(Access::Write);
        self.write_memory(addr, self.registers.a);
    }
    fn sta_preindexed_indirect_zeropage_x(&mut self) {
        let addr = self.addr_preindexed_indirect_zeropage_x();
        self.write_memory(addr, self.registers.a);
    }
    fn sta_postindexed_indirect_zeropage_y(&mut self) {
        let addr = self.addr_postindexed_indirect_zeropage_y(Access::Write);
        self.write_memory(addr, self.registers.a);
    }

    fn stx_zeropage(&mut self) {
        self.write_zeropage(self.registers.x);
    }
    fn stx_zeropage_y(&mut self) {
        self.write_zeropage_y(self.registers.x);
    }
    fn stx_absolute(&mut self) {
        self.write_absolute(self.registers.x);
    }

    fn sty_zeropage(&mut self) {
        self.write_zeropage(self.registers.y);
    }
    fn sty_zeropage_x(&mut self) {
        self.write_zeropage_x(self.registers.y);
    }
    fn sty_absolute(&mut self) {
        self.write_absolute(self.registers.y);
    }

    fn jmp_absolute(&mut self) {
        let instr_addr = self.registers.pc - 1;
        let lo = self.next();
        let hi = self.next();
        let data = u16::from(lo) | (u16::from(hi) << 8);
        if data == instr_addr {
            panic!("Trapped at {:#x}", instr_addr);
        }
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

    fn jsr(&mut self) {
        let target_lo = self.next();
        let target_hi = self.next();
        let target_addr = u16::from(target_lo) | (u16::from(target_hi) << 8);

        let ret = self.registers.pc.wrapping_sub(1);
        let ret_lo = (ret & 0x00FF) as u8;
        let ret_hi = (ret >> 8) as u8;

        self.push_stack(ret_hi);
        self.push_stack(ret_lo);

        self.clock_cycle();
        self.registers.pc = target_addr;
    }

    fn rts(&mut self) {
        self.clock_cycle();

        let ret_lo = self.pull_stack();
        let ret_hi = self.pull_stack();
        let ret_addr = u16::from(ret_lo) | (u16::from(ret_hi) << 8);

        self.registers.pc = ret_addr.wrapping_add(1);
        self.clock_cycle();

        self.clock_cycle();
    }

    fn cmp_immediate(&mut self) {
        let data = self.next();
        self.cmp(data);
    }
    fn cmp_zeropage(&mut self) {
        let data = self.read_zeropage();
        self.cmp(data);
    }
    fn cmp_zeropage_x(&mut self) {
        let data = self.read_zeropage_x();
        self.cmp(data);
    }
    fn cmp_absolute(&mut self) {
        let data = self.read_absolute();
        self.cmp(data);
    }
    fn cmp_absolute_x(&mut self) {
        let data = self.read_absolute_x();
        self.cmp(data);
    }
    fn cmp_absolute_y(&mut self) {
        let data = self.read_absolute_y();
        self.cmp(data);
    }
    fn cmp_preindexed_indirect_zeropage_x(&mut self) {
        let data = self.read_preindexed_indirect_zeropage_x();
        self.cmp(data);
    }
    fn cmp_postindexed_indirect_zeropage_y(&mut self) {
        let data = self.read_postindexed_indirect_zeropage_y();
        self.cmp(data);
    }

    fn cpx_immediate(&mut self) {
        let data = self.next();
        self.cpx(data);
    }
    fn cpx_zeropage(&mut self) {
        let data = self.read_zeropage();
        self.cpx(data);
    }
    fn cpx_absolute(&mut self) {
        let data = self.read_absolute();
        self.cpx(data);
    }

    fn cpy_immediate(&mut self) {
        let data = self.next();
        self.cpy(data);
    }
    fn cpy_zeropage(&mut self) {
        let data = self.read_zeropage();
        self.cpy(data);
    }
    fn cpy_absolute(&mut self) {
        let data = self.read_absolute();
        self.cpy(data);
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

    fn sec(&mut self) {
        self.registers.update_carry_bit(true);
        self.clock_cycle();
    }
    fn sed(&mut self) {
        self.registers.update_decimal_bit(true);
        self.clock_cycle();
    }
    fn sei(&mut self) {
        self.registers.update_interupt_bit(true);
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

    fn inx(&mut self) {
        self.registers.update_x(self.registers.x.wrapping_add(1));
        self.clock_cycle();
    }

    fn iny(&mut self) {
        self.registers.update_y(self.registers.y.wrapping_add(1));
        self.clock_cycle();
    }

    fn pha(&mut self) {
        self.clock_cycle();
        self.push_stack(self.registers.a);
    }

    fn php(&mut self) {
        self.clock_cycle();
        self.push_stack(self.registers.p | (1 << STATUS_BREAK_BIT) | (1 << STATUS_IGNORED_BIT));
    }

    fn pla(&mut self) {
        self.clock_cycle();
        let data = self.pull_stack();
        self.registers.update_a(data);
    }

    fn plp(&mut self) {
        self.clock_cycle();
        let data = self.pull_stack();
        self.registers.p = data & !(1 << STATUS_BREAK_BIT) & !(1 << STATUS_IGNORED_BIT);
    }

    fn brk(&mut self) {
        self.next();

        let ret_lo = (self.registers.pc & 0x00FF) as u8;
        let ret_hi = (self.registers.pc >> 8) as u8;

        self.push_stack(ret_hi);
        self.push_stack(ret_lo);
        self.push_stack(self.registers.p | (1 << STATUS_BREAK_BIT) | (1 << STATUS_IGNORED_BIT));

        self.registers.update_interupt_bit(true);

        let pc_lo = self.read_memory(INTERRUPT_VECTOR_IRQ_LO);
        let pc_hi = self.read_memory(INTERRUPT_VECTOR_IRQ_HI);
        self.registers.pc = u16::from(pc_lo) | (u16::from(pc_hi) << 8);
    }

    fn rti(&mut self) {
        self.clock_cycle();

        self.registers.p =
            self.pull_stack() & !(1 << STATUS_BREAK_BIT) & !(1 << STATUS_IGNORED_BIT);

        let ret_lo = self.pull_stack();
        let ret_hi = self.pull_stack();
        self.registers.pc = u16::from(ret_lo) | (u16::from(ret_hi) << 8);

        self.clock_cycle();
    }

    fn bit_zeropage(&mut self) {
        let data = self.read_zeropage();
        self.bit(data);
    }

    fn bit_absolute(&mut self) {
        let data = self.read_absolute();
        self.bit(data);
    }

    fn asl_accumulator(&mut self) {
        let result = self.asl(self.registers.a);
        self.registers.update_a(result);
    }
    fn asl_zeropage(&mut self) {
        let addr = self.addr_zeropage();
        let data = self.read_memory(addr);
        let result = self.asl(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }
    fn asl_zeropage_x(&mut self) {
        let addr = self.addr_zeropage_x();
        let data = self.read_memory(addr);
        let result = self.asl(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }
    fn asl_absolute(&mut self) {
        let addr = self.addr_absolute();
        let data = self.read_memory(addr);
        let result = self.asl(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }
    fn asl_absolute_x(&mut self) {
        let addr = self.addr_absolute_x(Access::ReadModify);
        let data = self.read_memory(addr);
        let result = self.asl(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }

    fn lsr_accumulator(&mut self) {
        let result = self.lsr(self.registers.a);
        self.registers.update_a(result);
    }
    fn lsr_zeropage(&mut self) {
        let addr = self.addr_zeropage();
        let data = self.read_memory(addr);
        let result = self.lsr(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }
    fn lsr_zeropage_x(&mut self) {
        let addr = self.addr_zeropage_x();
        let data = self.read_memory(addr);
        let result = self.lsr(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }
    fn lsr_absolute(&mut self) {
        let addr = self.addr_absolute();
        let data = self.read_memory(addr);
        let result = self.lsr(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }
    fn lsr_absolute_x(&mut self) {
        let addr = self.addr_absolute_x(Access::ReadModify);
        let data = self.read_memory(addr);
        let result = self.lsr(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }

    fn rol_accumulator(&mut self) {
        let result = self.rol(self.registers.a);
        self.registers.update_a(result);
    }
    fn rol_zeropage(&mut self) {
        let addr = self.addr_zeropage();
        let data = self.read_memory(addr);
        let result = self.rol(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }
    fn rol_zeropage_x(&mut self) {
        let addr = self.addr_zeropage_x();
        let data = self.read_memory(addr);
        let result = self.rol(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }
    fn rol_absolute(&mut self) {
        let addr = self.addr_absolute();
        let data = self.read_memory(addr);
        let result = self.rol(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }
    fn rol_absolute_x(&mut self) {
        let addr = self.addr_absolute_x(Access::ReadModify);
        let data = self.read_memory(addr);
        let result = self.rol(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }

    fn ror_accumulator(&mut self) {
        let result = self.ror(self.registers.a);
        self.registers.update_a(result);
    }
    fn ror_zeropage(&mut self) {
        let addr = self.addr_zeropage();
        let data = self.read_memory(addr);
        let result = self.ror(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }
    fn ror_zeropage_x(&mut self) {
        let addr = self.addr_zeropage_x();
        let data = self.read_memory(addr);
        let result = self.ror(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }
    fn ror_absolute(&mut self) {
        let addr = self.addr_absolute();
        let data = self.read_memory(addr);
        let result = self.ror(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }
    fn ror_absolute_x(&mut self) {
        let addr = self.addr_absolute_x(Access::ReadModify);
        let data = self.read_memory(addr);
        let result = self.ror(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }

    fn inc_zeropage(&mut self) {
        let addr = self.addr_zeropage();
        let data = self.read_memory(addr);
        let result = self.inc(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }
    fn inc_zeropage_x(&mut self) {
        let addr = self.addr_zeropage_x();
        let data = self.read_memory(addr);
        let result = self.inc(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }
    fn inc_absolute(&mut self) {
        let addr = self.addr_absolute();
        let data = self.read_memory(addr);
        let result = self.inc(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }
    fn inc_absolute_x(&mut self) {
        let addr = self.addr_absolute_x(Access::ReadModify);
        let data = self.read_memory(addr);
        let result = self.inc(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }

    fn dec_zeropage(&mut self) {
        let addr = self.addr_zeropage();
        let data = self.read_memory(addr);
        let result = self.dec(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }
    fn dec_zeropage_x(&mut self) {
        let addr = self.addr_zeropage_x();
        let data = self.read_memory(addr);
        let result = self.dec(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }
    fn dec_absolute(&mut self) {
        let addr = self.addr_absolute();
        let data = self.read_memory(addr);
        let result = self.dec(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }
    fn dec_absolute_x(&mut self) {
        let addr = self.addr_absolute_x(Access::ReadModify);
        let data = self.read_memory(addr);
        let result = self.dec(data);
        self.clock_cycle();
        self.write_memory(addr, result);
    }
}

impl CPU {
    pub fn new(memory: RAM, clock_speed: u32) -> Self {
        Self {
            registers: Registers::new(),
            memory,
            clock_speed,
        }
    }

    pub fn run(&mut self) {
        loop {
            println!("{}", self.registers);
            let instruction = self.next();
            println!("interpreting {instruction:2X}");
            println!();

            match instruction {
                0xEA => self.nop(),
                0x69 => self.adc_immediate(),
                0x65 => self.adc_zeropage(),
                0x75 => self.adc_zeropage_x(),
                0x6D => self.adc_absolute(),
                0x7D => self.adc_absolute_x(),
                0x79 => self.adc_absolute_y(),
                0x61 => self.adc_preindexed_indirect_zeropage_x(),
                0x71 => self.adc_postindexed_indirect_zeropage_y(),

                0xE9 => self.sbc_immediate(),
                0xE5 => self.sbc_zeropage(),
                0xF5 => self.sbc_zeropage_x(),
                0xED => self.sbc_absolute(),
                0xFD => self.sbc_absolute_x(),
                0xF9 => self.sbc_absolute_y(),
                0xE1 => self.sbc_preindexed_indirect_zeropage_x(),
                0xF1 => self.sbc_postindexed_indirect_zeropage_y(),

                0x29 => self.and_immediate(),
                0x25 => self.and_zeropage(),
                0x35 => self.and_zeropage_x(),
                0x2D => self.and_absolute(),
                0x3D => self.and_absolute_x(),
                0x39 => self.and_absolute_y(),
                0x21 => self.and_preindexed_indirect_zeropage_x(),
                0x31 => self.and_postindexed_indirect_zeropage_y(),

                0x09 => self.ora_immediate(),
                0x05 => self.ora_zeropage(),
                0x15 => self.ora_zeropage_x(),
                0x0D => self.ora_absolute(),
                0x1D => self.ora_absolute_x(),
                0x19 => self.ora_absolute_y(),
                0x01 => self.ora_preindexed_indirect_zeropage_x(),
                0x11 => self.ora_postindexed_indirect_zeropage_y(),

                0x49 => self.eor_immediate(),
                0x45 => self.eor_zeropage(),
                0x55 => self.eor_zeropage_x(),
                0x4D => self.eor_absolute(),
                0x5D => self.eor_absolute_x(),
                0x59 => self.eor_absolute_y(),
                0x41 => self.eor_preindexed_indirect_zeropage_x(),
                0x51 => self.eor_postindexed_indirect_zeropage_y(),

                0x4C => self.jmp_absolute(),
                0x6C => self.jmp_indirect(),

                0x20 => self.jsr(),
                0x60 => self.rts(),

                0xA9 => self.lda_immediate(),
                0xA5 => self.lda_zeropage(),
                0xB5 => self.lda_zeropage_x(),
                0xAD => self.lda_absolute(),
                0xBD => self.lda_absolute_x(),
                0xB9 => self.lda_absolute_y(),
                0xA1 => self.lda_preindexed_indirect_zeropage_x(),
                0xB1 => self.lda_postindexed_indirect_zeropage_y(),

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
                0x9D => self.sta_absolute_x(),
                0x99 => self.sta_absolute_y(),
                0x81 => self.sta_preindexed_indirect_zeropage_x(),
                0x91 => self.sta_postindexed_indirect_zeropage_y(),

                0x86 => self.stx_zeropage(),
                0x96 => self.stx_zeropage_y(),
                0x8E => self.stx_absolute(),

                0x84 => self.sty_zeropage(),
                0x94 => self.sty_zeropage_x(),
                0x8C => self.sty_absolute(),

                0xC9 => self.cmp_immediate(),
                0xC5 => self.cmp_zeropage(),
                0xD5 => self.cmp_zeropage_x(),
                0xCD => self.cmp_absolute(),
                0xDD => self.cmp_absolute_x(),
                0xD9 => self.cmp_absolute_y(),
                0xC1 => self.cmp_preindexed_indirect_zeropage_x(),
                0xD1 => self.cmp_postindexed_indirect_zeropage_y(),

                0xE0 => self.cpx_immediate(),
                0xE4 => self.cpx_zeropage(),
                0xEC => self.cpx_absolute(),

                0xC0 => self.cpy_immediate(),
                0xC4 => self.cpy_zeropage(),
                0xCC => self.cpy_absolute(),

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

                0x38 => self.sec(),
                0xF8 => self.sed(),
                0x78 => self.sei(),

                0xCA => self.dex(),
                0x88 => self.dey(),
                0xE8 => self.inx(),
                0xC8 => self.iny(),

                0x48 => self.pha(),
                0x08 => self.php(),
                0x68 => self.pla(),
                0x28 => self.plp(),

                0x00 => self.brk(),
                0x40 => self.rti(),

                0x24 => self.bit_zeropage(),
                0x2C => self.bit_absolute(),

                0x0A => self.asl_accumulator(),
                0x06 => self.asl_zeropage(),
                0x16 => self.asl_zeropage_x(),
                0x0E => self.asl_absolute(),
                0x1E => self.asl_absolute_x(),

                0x4A => self.lsr_accumulator(),
                0x46 => self.lsr_zeropage(),
                0x56 => self.lsr_zeropage_x(),
                0x4E => self.lsr_absolute(),
                0x5E => self.lsr_absolute_x(),

                0x2A => self.rol_accumulator(),
                0x26 => self.rol_zeropage(),
                0x36 => self.rol_zeropage_x(),
                0x2E => self.rol_absolute(),
                0x3E => self.rol_absolute_x(),

                0x6A => self.ror_accumulator(),
                0x66 => self.ror_zeropage(),
                0x76 => self.ror_zeropage_x(),
                0x6E => self.ror_absolute(),
                0x7E => self.ror_absolute_x(),

                0xE6 => self.inc_zeropage(),
                0xF6 => self.inc_zeropage_x(),
                0xEE => self.inc_absolute(),
                0xFE => self.inc_absolute_x(),

                0xC6 => self.dec_zeropage(),
                0xD6 => self.dec_zeropage_x(),
                0xCE => self.dec_absolute(),
                0xDE => self.dec_absolute_x(),

                x => {
                    unreachable!("invalid instruction: {:X}", x);
                }
            }
        }
    }
}
