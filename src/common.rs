pub use std::f64::consts::PI;
pub use std::f64::INFINITY;
use rand::Rng;

pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

pub fn random_f64() -> f64 {
    // Returns a random real in [0,1).
    let mut rng = rand::rng();
    rng.random::<f64>()
}

pub fn random_f64_range(min: f64, max: f64) -> f64 {
    // Returns a random real in [min,max).
    min + (max - min) * random_f64()
}

pub fn random_int_range(min: i32, max: i32) -> i32 {
    // Returns a random integer in [min,max].
    let mut rng = rand::rng();
    let n: i32 = rng.random_range(min..=max);
    n
}