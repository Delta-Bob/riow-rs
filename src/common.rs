//! Small shared helpers: angle conversion and random-number generation.

use std::cell::RefCell;

use rand::seq::SliceRandom;
use rand::{Rng, SeedableRng, rngs::SmallRng};

/// Seed for reproducible renders. Scene construction and rendering both
/// derive from it, so changing it gives a different noise pattern.
pub const DEFAULT_SEED: u64 = 0x5EED;

thread_local! {
    static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::seed_from_u64(DEFAULT_SEED));
}

/// Reseeds this thread's random number generator.
pub fn set_seed(seed: u64) {
    RNG.with(|rng| *rng.borrow_mut() = SmallRng::seed_from_u64(seed));
}

/// Converts degrees to radians.
pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

/// Returns a random real in `[0, 1)`.
pub fn random_f64() -> f64 {
    RNG.with(|rng| rng.borrow_mut().random())
}

/// Returns a random real in `[min, max)`.
pub fn random_f64_range(min: f64, max: f64) -> f64 {
    min + (max - min) * random_f64()
}

/// Returns a random integer in `[min, max]`.
pub fn random_int_range(min: i32, max: i32) -> i32 {
    RNG.with(|rng| rng.borrow_mut().random_range(min..=max))
}

/// Returns a shuffled copy of `0..n`.
pub fn shuffled(n: usize) -> Vec<usize> {
    let mut values: Vec<usize> = (0..n).collect();
    RNG.with(|rng| values.shuffle(&mut *rng.borrow_mut()));
    values
}
