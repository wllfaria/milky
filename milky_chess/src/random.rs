#[derive(Debug)]
pub struct Random {
    state: u32,
}

impl Default for Random {
    fn default() -> Self {
        Self::new()
    }
}

impl Random {
    pub fn new() -> Self {
        Self { state: 1804289383 }
    }

    pub fn gen_u32(&mut self) -> u32 {
        let mut number = self.state;

        number ^= number << 13;
        number ^= number >> 17;
        number ^= number << 5;

        self.state = number;
        number
    }

    pub fn gen_u64(&mut self) -> u64 {
        let n1 = self.gen_u32() as u64 & 0xFFFF;
        let n2 = self.gen_u32() as u64 & 0xFFFF;
        let n3 = self.gen_u32() as u64 & 0xFFFF;
        let n4 = self.gen_u32() as u64 & 0xFFFF;

        n1 | (n2 << 16) | (n3 << 32) | (n4 << 48)
    }

    pub fn gen_magic_number_candidate(&mut self) -> u64 {
        self.gen_u64() & self.gen_u64() & self.gen_u64()
    }
}
