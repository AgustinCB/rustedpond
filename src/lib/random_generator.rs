pub struct RandomGenerator([usize; 2]);
impl RandomGenerator {
    #[inline]
    pub fn new(fseed: usize, sseed: usize) -> RandomGenerator {
        RandomGenerator([fseed, sseed])
    }

    #[inline]
    pub fn generate_integer(&mut self) -> usize {
        let mut x = self.0[0];
        let y = self.0[1];
        self.0[0] = y;
        x ^= x << 23;
        self.0[1] = x ^ y ^ (x >> 17) ^ (y >> 26);
        self.0[1] + y
    }

    #[inline]
    pub fn generate_boolean(&mut self) -> bool {
        (self.generate_integer() & 0x80) > 0
    }
}