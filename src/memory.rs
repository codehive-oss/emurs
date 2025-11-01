use std::cell::RefCell;
use std::rc::Rc;

pub trait Memory {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
}

pub struct Ram(Vec<u8>);

impl Ram {
    pub fn new(size: usize) -> Self {
        Self(vec![0; size])
    }
    
    pub fn size(&self) -> usize {
        self.0.len()
    }
}

impl Memory for Ram {
    #[inline]
    fn read(&self, a: u16) -> u8 {
        self.0[a as usize]
    }

    #[inline]
    fn write(&mut self, a: u16, v: u8) {
        self.0[a as usize] = v;
    }
}

pub struct DummyMemory {
    last_read_addr: RefCell<u16>,
    last_write_addr: u16,
    last_write_value: u8
}

impl DummyMemory {
    pub fn new() -> Self {
        DummyMemory {
            last_read_addr: RefCell::new(0),
            last_write_addr: 0,
            last_write_value: 0
        }
    }

    pub fn last_read_addr(&self) -> u16 {
        *self.last_read_addr.borrow()
    }

    pub fn last_write_addr(&self) -> u16 {
        self.last_write_addr
    }

    pub fn last_write_value(&self) -> u8 {
        self.last_write_value
    }
}

impl Memory for DummyMemory {
    fn read(&self, addr: u16) -> u8 {
        *self.last_read_addr.borrow_mut() = addr;
        (addr & 0xFF) as u8
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.last_write_addr = addr;
        self.last_write_value = data;
    }
}

impl<T: Memory> Memory for Rc<RefCell<T>> {
    fn read(&self, addr: u16) -> u8 {
        self.as_ref().borrow().read(addr)
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.as_ref().borrow_mut().write(addr, data);
    }
}
