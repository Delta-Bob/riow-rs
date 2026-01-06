use crate::vec3::{Vec3, Point3};

pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
}

impl Ray {
    pub fn default() -> Ray {
        Ray {
            origin: Point3::new(0.0, 0.0, 0.0),
            direction: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn new(origin: Point3, direction: Vec3) -> Ray {
        Ray { origin, direction }
    }
    
    pub fn at(&self, t: f64) -> Point3 {
        self.origin + self.direction * t
    }
}