// XorShift64 gets stuck forever if initialized with a zero state, so we fall
// back to a golden-ratio-derived constant commonly used for hash/PRNG seeding.
const DEFAULT_RNG_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct XorShift64 {
    pub(crate) state: u64,
}

impl XorShift64 {
    pub fn new(seed: u64) -> Self {
        let state = if seed == 0 { DEFAULT_RNG_SEED } else { seed };
        Self { state }
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub fn next_index(&mut self, upper_bound: usize) -> usize {
        if upper_bound <= 1 {
            return 0;
        }
        (self.next_u64() % upper_bound as u64) as usize
    }
}
