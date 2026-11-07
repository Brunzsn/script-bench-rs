//! Deterministic pseudo-random numbers shared by every engine.
//!
//! The benchmark data must be identical for every engine and every iteration,
//! so the host `rand` function uses this small SplitMix64 generator instead of
//! an OS-seeded RNG. The state is reset before each validation and benchmark
//! run by [`crate::harness::run_cases`].

use std::cell::Cell;

/// Seed used for every benchmark run.
pub const SEED: u64 = 0x5EED_1BAD_B002_D00D;

/// A tiny deterministic PRNG (SplitMix64).
#[derive(Debug, Clone, Copy)]
pub struct DetRng {
    state: u64,
}

impl DetRng {
    /// Creates a generator initialized with [`SEED`].
    pub const fn new() -> Self {
        Self { state: SEED }
    }

    /// Returns the next 64-bit value.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Returns a value in `0..n`.
    ///
    /// Modulo is intentional: `rand::random_range` consumes a variable amount
    /// of entropy depending on rejection sampling, which would make the
    /// sequence engine-dependent.
    pub fn below(&mut self, n: u32) -> u32 {
        assert!(n > 0, "upper bound must be greater than zero");
        (self.next_u64() % u64::from(n)) as u32
    }
}

impl Default for DetRng {
    fn default() -> Self {
        Self::new()
    }
}

thread_local! {
    static RNG: Cell<DetRng> = const { Cell::new(DetRng::new()) };
}

/// Resets the thread-local generator to its initial state.
pub fn reset() {
    RNG.with(|rng| rng.set(DetRng::new()));
}

/// Returns a deterministic value in `0..n` from the thread-local generator.
pub fn rand_below(n: u32) -> u32 {
    RNG.with(|rng| {
        let mut state = rng.get();
        let value = state.below(n);
        rng.set(state);
        value
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_restarts_the_sequence() {
        reset();
        let first: Vec<u32> = (0..8).map(|_| rand_below(16)).collect();
        reset();
        let second: Vec<u32> = (0..8).map(|_| rand_below(16)).collect();
        assert_eq!(first, second);
    }
}
