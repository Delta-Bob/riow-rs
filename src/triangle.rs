use std::sync::Arc;

use crate::aabb::Aabb;
use crate::hittable::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::material::Material;
use crate::plane::Plane;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

pub struct Triangle {
    plane: Plane,
    mat: Option<Arc<dyn Material>>,
}

impl Triangle {
    /// A triangle with corner `q` and edge vectors `u` and `v`.
    pub fn new(q: Point3, u: Vec3, v: Vec3, mat: Option<Arc<dyn Material>>) -> Self {
        Self {
            plane: Plane::new(q, u, v),
            mat,
        }
    }
}

impl Hittable for Triangle {
    fn bounding_box(&self) -> Aabb {
        self.plane.bbox()
    }

    fn hit<'a>(&'a self, r: &Ray, ray_t: Interval, rec: &mut HitRecord<'a>) -> bool {
        let Some((t, point, alpha, beta)) = self.plane.hit(r, ray_t) else {
            return false;
        };

        // Inside the triangle when the barycentric coordinates are non-negative
        // and sum to at most one.
        if alpha < 0.0 || beta < 0.0 || alpha + beta > 1.0 {
            return false;
        }

        rec.t = t;
        rec.p = point;
        rec.u = alpha;
        rec.v = beta;
        rec.mat = self.mat.as_deref();
        rec.set_face_normal(r, self.plane.normal);

        true
    }
}
