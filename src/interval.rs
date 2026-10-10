//! A 1D interval, used for ray parameter ranges and bounding-box slabs.

#[derive(Clone, Copy)]
pub struct Interval {
    pub min: f64,
    pub max: f64,
}

impl Default for Interval {
    fn default() -> Self {
        Self::empty()
    }
}

impl Interval {
    /// The empty interval (min > max), absorbing under union.
    pub fn empty() -> Self {
        Self {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }

    /// The interval containing every real number.
    pub fn universe() -> Self {
        Self {
            min: f64::NEG_INFINITY,
            max: f64::INFINITY,
        }
    }

    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    /// The smallest interval containing both `self` and `other`.
    pub fn union(self, other: Interval) -> Self {
        Self {
            min: if self.min <= other.min {
                self.min
            } else {
                other.min
            },
            max: if self.max >= other.max {
                self.max
            } else {
                other.max
            },
        }
    }

    pub fn size(self) -> f64 {
        self.max - self.min
    }

    pub fn contains(self, x: f64) -> bool {
        self.min <= x && x <= self.max
    }

    pub fn surrounds(self, x: f64) -> bool {
        self.min < x && x < self.max
    }

    pub fn clamp(self, x: f64) -> f64 {
        if x < self.min {
            self.min
        } else if x > self.max {
            self.max
        } else {
            x
        }
    }

    pub fn expand(self, delta: f64) -> Self {
        let padding = delta / 2.0;
        Self {
            min: self.min - padding,
            max: self.max + padding,
        }
    }
}

impl std::ops::Add<f64> for Interval {
    type Output = Interval;

    fn add(self, displacement: f64) -> Interval {
        Interval::new(self.min + displacement, self.max + displacement)
    }
}
