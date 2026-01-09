use crate::hittable::*;
use crate::interval::*;
use crate::ray::*;
use crate::vec3::*;
use crate::material::Material;
use crate::aabb::AABB;

use std::sync::Arc;

pub struct Sphere {
    center0: Point3,   // center at t=0
    motion: Vec3,      // delta center over shutter [0,1]
    radius: f64,
    mat: Option<Arc<dyn Material>>,
}

impl Sphere {
    // Stationary sphere
    pub fn new(center: Point3, radius: f64, mat: Option<Arc<dyn Material>>) -> Sphere {
        Sphere {
            center0: center,
            motion: Vec3::new(0.0, 0.0, 0.0),
            radius: radius.max(0.0),
            mat,
        }
    }

    // Moving sphere: center moves linearly from center1 to center2 over time [0,1]
    pub fn new_moving(center1: Point3, center2: Point3, radius: f64, mat: Option<Arc<dyn Material>>) -> Sphere {
        Sphere {
            center0: center1,
            motion: center2 - center1,
            radius: radius.max(0.0),
            mat,
        }
    }

    #[inline]
    fn center_at(&self, time: f64) -> Point3 {
        // Equivalent to: center_ray.at(time) when center_ray = Ray(center1, center2-center1, 0)
        self.center0 + self.motion * time
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        let center = self.center_at(r.time());
        let oc = r.origin - center;
        let a = r.direction.length_squared();
        let half_b = dot(oc, r.direction);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return false;
        }

        let sqrt_d = f64::sqrt(discriminant);

        // Find the nearest root that lies in the acceptable range
        let mut root = (-half_b - sqrt_d) / a;
        if !ray_t.surrounds(root) {
            root = (-half_b + sqrt_d) / a;
            if !ray_t.surrounds(root) {
                return false;
            }
        }

        rec.t = root;
        rec.p = r.at(rec.t);
        let outward_normal = (rec.p - center) / self.radius;
        rec.set_face_normal(r, outward_normal);
        rec.mat = self.mat.clone();

        true
    }

    fn bounding_box(&self) -> AABB {
        let rvec = Vec3::new(self.radius, self.radius, self.radius);

        let box0 = AABB::new_point3(self.center_at(0.0) - rvec, self.center_at(0.0) + rvec);

        let box1 = AABB::new_point3(
            self.center_at(1.0) - rvec,
            self.center_at(1.0) + rvec
        );

        AABB::from_boxes(&box0, &box1)
    }
}