use std::{fs::File, io::Read};

pub struct RAM(Box<[u8; 0x10000]>);

impl RAM {
    pub fn new() -> Self {
        Self(Box::new([0; 0x10000]))
    }

    pub fn load(&mut self, path: &str) -> Result<(), anyhow::Error> {
        let mut file = File::open(path)?;
        file.read(&mut *self.0)?;
        Ok(())
    }

    #[inline]
    pub fn read(&self, a: u16) -> u8 {
        self.0[a as usize]
    }

    #[inline]
    pub fn write(&mut self, a: u16, v: u8) {
        self.0[a as usize] = v;
    }
}
