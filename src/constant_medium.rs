use std::sync::Arc;

use crate::aabb::Aabb;
use crate::color::Color;
use crate::common::random_f64;
use crate::hittable::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::material::{HenyeyGreenstein, Isotropic, Material};
use crate::ray::Ray;
use crate::vec3::Vec3;

/// A participating medium of constant density inside a boundary object.
pub struct ConstantMedium {
    boundary: Arc<dyn Hittable>,
    neg_inv_density: f64,
    phase_function: Arc<dyn Material>,
}

impl ConstantMedium {
    /// A medium with a custom phase function.
    pub fn new(
        boundary: Arc<dyn Hittable>,
        density: f64,
        phase_function: Arc<dyn Material>,
    ) -> Self {
        Self {
            boundary,
            neg_inv_density: -1.0 / density,
            phase_function,
        }
    }

    /// A medium that scatters isotropically.
    pub fn new_with_color(boundary: Arc<dyn Hittable>, density: f64, albedo: Color) -> Self {
        Self::new(boundary, density, Arc::new(Isotropic::new(albedo)))
    }

    /// A medium with a Henyey-Greenstein phase function; `g > 0` scatters
    /// forward, `g < 0` backward.
    pub fn new_with_color_and_g(
        boundary: Arc<dyn Hittable>,
        density: f64,
        albedo: Color,
        g: f64,
    ) -> Self {
        Self::new(
            boundary,
            density,
            Arc::new(HenyeyGreenstein::new(albedo, g)),
        )
    }
}

impl Hittable for ConstantMedium {
    fn hit<'a>(&'a self, ray: &Ray, ray_t: Interval, rec: &mut HitRecord<'a>) -> bool {
        let mut rec1 = HitRecord::new();
        let mut rec2 = HitRecord::new();

        if !self.boundary.hit(ray, Interval::universe(), &mut rec1) {
            return false;
        }

        if !self.boundary.hit(
            ray,
            Interval::new(rec1.t + 0.0001, f64::INFINITY),
            &mut rec2,
        ) {
            return false;
        }

        if rec1.t < ray_t.min {
            rec1.t = ray_t.min;
        }
        if rec2.t > ray_t.max {
            rec2.t = ray_t.max;
        }
        if rec1.t >= rec2.t {
            return false;
        }
        if rec1.t < 0.0 {
            rec1.t = 0.0;
        }

        let ray_length = ray.direction.length();
        let distance_inside_boundary = (rec2.t - rec1.t) * ray_length;
        let hit_distance = self.neg_inv_density * random_f64().ln();

        if hit_distance > distance_inside_boundary {
            return false;
        }

        rec.t = rec1.t + hit_distance / ray_length;
        rec.p = ray.at(rec.t);
        rec.normal = Vec3::new(1.0, 0.0, 0.0); // Arbitrary.
        rec.front_face = true; // Also arbitrary.
        rec.mat = Some(self.phase_function.as_ref());

        true
    }

    fn bounding_box(&self) -> Aabb {
        self.boundary.bounding_box()
    }
}
