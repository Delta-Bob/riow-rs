use crate::vec3::{Point3, Vec3};

pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
    /// Time of the ray within the shutter interval `[0, 1]`, for motion blur.
    pub time: f64,
    /// Componentwise reciprocal of `direction`, precomputed because every
    /// bounding-box test needs it on all three axes.
    inv_dir: Vec3,
}

impl Ray {
    pub fn new(origin: Point3, direction: Vec3, time: f64) -> Self {
        let inv_dir = Vec3::new(
            1.0 / direction.x(),
            1.0 / direction.y(),
            1.0 / direction.z(),
        );

        Self {
            origin,
            direction,
            time,
            inv_dir,
        }
    }

    /// The reciprocal of the direction component along `axis`.
    pub fn inv_dir(&self, axis: usize) -> f64 {
        self.inv_dir[axis]
    }

    pub fn at(&self, t: f64) -> Point3 {
        self.origin + self.direction * t
    }
}
