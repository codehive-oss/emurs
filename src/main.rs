struct Registers {
    a: u8,
    x: u8,
    y: u8,
    sp: u8,
    pc: u16,
    p: u8,
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
    reg: Registers,
    ram: Memory,
}

impl Emulator {}

fn main() {
    println!("Hello, world!");
}
