//! Axis-aligned bounding boxes.

use crate::interval::Interval;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

#[derive(Clone, Copy, Default)]
pub struct Aabb {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
}

impl Aabb {
    pub fn empty() -> Self {
        Self {
            x: Interval::empty(),
            y: Interval::empty(),
            z: Interval::empty(),
        }
    }

    /// Builds a box from three intervals, padding any degenerate axis.
    pub fn new(x: Interval, y: Interval, z: Interval) -> Self {
        let mut bbox = Self { x, y, z };
        bbox.pad_to_minimums();
        bbox
    }

    /// Builds the box spanning the two points.
    pub fn from_points(a: Point3, b: Point3) -> Self {
        let x = if a.x() < b.x() {
            Interval::new(a.x(), b.x())
        } else {
            Interval::new(b.x(), a.x())
        };
        let y = if a.y() < b.y() {
            Interval::new(a.y(), b.y())
        } else {
            Interval::new(b.y(), a.y())
        };
        let z = if a.z() < b.z() {
            Interval::new(a.z(), b.z())
        } else {
            Interval::new(b.z(), a.z())
        };

        Self { x, y, z }
    }

    /// The smallest box containing both `self` and `other`.
    pub fn union(self, other: Aabb) -> Self {
        Self {
            x: self.x.union(other.x),
            y: self.y.union(other.y),
            z: self.z.union(other.z),
        }
    }

    pub fn axis_interval(&self, n: usize) -> Interval {
        match n {
            1 => self.y,
            2 => self.z,
            _ => self.x,
        }
    }

    /// Returns true if the ray intersects this box within `ray_t`.
    ///
    /// The `t0 > t1` form of the swap keeps axis-parallel rays that lie
    /// exactly on a slab boundary (`0 * inf = NaN`) from being rejected;
    /// `f64::max`/`min` ignore NaN, which keeps those axes unconstrained.
    pub fn hit(&self, r: &Ray, ray_t: Interval) -> bool {
        self.intersect(r, ray_t.min, ray_t.max).is_some()
    }

    /// The distance along the ray at which it enters this box, if it crosses
    /// the box within `[t_min, t_max]`.
    pub fn intersect(&self, r: &Ray, t_min: f64, t_max: f64) -> Option<f64> {
        let mut t_near = f64::NEG_INFINITY;
        let mut t_far = f64::INFINITY;

        for axis in 0..3 {
            let interval = self.axis_interval(axis);
            let adinv = r.inv_dir(axis);

            let t0 = (interval.min - r.origin[axis]) * adinv;
            let t1 = (interval.max - r.origin[axis]) * adinv;

            let (lo, hi) = if t0 > t1 { (t1, t0) } else { (t0, t1) };
            t_near = t_near.max(lo);
            t_far = t_far.min(hi);
        }

        if t_near > t_max || t_far < t_min {
            return None;
        }

        Some(t_near.max(t_min))
    }

    /// The center of the box.
    pub fn centroid(&self) -> Point3 {
        Point3::new(
            self.x.min + self.x.size() / 2.0,
            self.y.min + self.y.size() / 2.0,
            self.z.min + self.z.size() / 2.0,
        )
    }

    /// The area of the box surface, used by the surface-area heuristic.
    pub fn surface_area(&self) -> f64 {
        let x = self.x.size();
        let y = self.y.size();
        let z = self.z.size();
        2.0 * (x * y + y * z + z * x)
    }

    /// The index (0, 1 or 2) of the longest axis of this box.
    pub fn longest_axis(&self) -> usize {
        if self.x.size() > self.y.size() && self.x.size() > self.z.size() {
            0
        } else if self.y.size() > self.z.size() {
            1
        } else {
            2
        }
    }

    fn pad_to_minimums(&mut self) {
        let delta = 0.0001;
        if self.x.size() < delta {
            self.x = self.x.expand(delta);
        }
        if self.y.size() < delta {
            self.y = self.y.expand(delta);
        }
        if self.z.size() < delta {
            self.z = self.z.expand(delta);
        }
    }
}

impl std::ops::Add<Vec3> for Aabb {
    type Output = Aabb;

    fn add(self, offset: Vec3) -> Aabb {
        Aabb::new(
            self.x + offset.x(),
            self.y + offset.y(),
            self.z + offset.z(),
        )
    }
}
