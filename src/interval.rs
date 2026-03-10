use crate::common;

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
    pub fn empty() -> Self {
        Self {
            min: common::INFINITY,
            max: -common::INFINITY,
        }
    }

    pub fn _universe() -> Self {
        Self {
            min: -common::INFINITY,
            max: common::INFINITY,
        }
    }

    pub fn from_intervals(a: &Interval, b: &Interval) -> Self {
        Self {
            min: if a.min <= b.min { a.min } else { b.min },
            max: if a.max >= b.max { a.max } else { b.max },
        }
    }

    pub fn new(min: f64, max: f64) -> Self { Self {min, max} }

    pub fn size(&self) -> f64 {
        self.max - self.min
    }

    pub fn contains(&self, x: f64) -> bool {
        self.min <= x && x <= self.max
    }

    pub fn surrounds(&self, x: f64) -> bool {
        self.min < x && x < self.max
    }

    pub fn clamp(&self, x: f64) -> f64 {
        if x < self.min {return self.min};
        if x > self.max {return self.max};
        return x;
    }

    pub fn expand(&self, delta: f64) -> Self {
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

impl std::ops::Add<Interval> for f64 {
    type Output = Interval;
    fn add(self, ival: Interval) -> Interval {
        ival + self
    }
}