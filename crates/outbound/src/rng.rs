/// `SplitMix64`.
///
/// Small enough to keep in-tree, and the property that matters is reproducibility: one
/// seed rebuilds one world exactly, in the window and headless alike. World generation
/// used the renderer's RNG, which meant no world could be built without opening a
/// window first.
pub(crate) struct Rng(u64);

impl Rng {
    pub(crate) const fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub(crate) const fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform over `0..limit`, modulo bias included. Negligible while `limit` is tiny
    /// against 2^64, and world generation is not somewhere it matters.
    pub(crate) const fn below(&mut self, limit: u64) -> u64 {
        if limit == 0 {
            return 0;
        }
        self.next_u64() % limit
    }

    pub(crate) fn between(&mut self, low: i64, high: i64) -> i64 {
        let span = high.saturating_sub(low).saturating_add(1).max(1);
        let span = u64::try_from(span).unwrap_or(1);
        low.saturating_add(i64::try_from(self.below(span)).unwrap_or(0))
    }

    pub(crate) fn position(&mut self, extent: i64) -> (i64, i64) {
        (self.between(0, extent), self.between(0, extent))
    }
}
