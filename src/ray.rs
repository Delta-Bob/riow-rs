use crate::vec3::{Vec3, Point3};

pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
    pub tm: f64,
}

impl Ray {
    pub fn default() -> Ray {
        Ray {
            origin: Point3::new(0.0, 0.0, 0.0),
            direction: Vec3::new(0.0, 0.0, 0.0),
            tm: 0.0,
        }
    }

    pub fn new(origin: Point3, direction: Vec3) -> Ray {
        Ray { origin, direction, tm: 0.0 }
    }

    pub fn new_with_time(origin: Point3, direction: Vec3, tm: f64) -> Ray {
        Ray { origin, direction, tm }
    }

    pub fn time(&self) -> f64 {
        self.tm
    }
    
    pub fn at(&self, t: f64) -> Point3 {
        self.origin + self.direction * t
    }
}