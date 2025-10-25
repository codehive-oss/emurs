pub struct Ram(Vec<u8>);

impl Ram {
    pub fn new(size: usize) -> Self {
        Self(vec![0; size])
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
