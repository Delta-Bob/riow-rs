use crate::hittable::*;
use crate::vec3::*;
use crate::material::Material;
use crate::aabb::AABB;
use crate::ray::Ray;
use crate::interval::Interval;

use std::sync::Arc;

pub struct Triangle {
    Q: Point3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    mat: Option<Arc<dyn Material>>,
    bbox: AABB,
    normal: Vec3,
    D: f64,
}

impl Triangle {
    pub fn new(Q: Point3, u: Vec3, v: Vec3, mat: Option<Arc<dyn Material>>) -> Triangle {
        let bbox = AABB::new_point3(Q, Q + u + v);
        let n = cross(u, v);
        let normal = unit_vector(n);
        let D = dot(normal, Q);
        let w = n / dot(n, n);

        let mut triangle = Triangle {
            Q,
            u,
            v,
            w,
            mat,
            bbox,
            normal,
            D,
        };
        triangle.set_bounding_box();
        triangle
    }

    fn set_bounding_box(&mut self) {
        let bbox_diagonal1 = AABB::new_point3(self.Q, self.Q + self.u + self.v);
        let bbox_diagonal2 = AABB::new_point3(self.Q + self.u, self.Q + self.v);
        self.bbox = AABB::from_boxes(&bbox_diagonal1, &bbox_diagonal2);
    }

    fn is_interior(&self, a: f64, b: f64, rec: &mut HitRecord) -> bool {
        if a < 0.0 || b < 0.0 || a + b > 1.0 {
            return false;
        }

        rec.u = a;
        rec.v = b;
        true
    }
}

impl Hittable for Triangle {
    fn bounding_box(&self) -> AABB {
        self.bbox.clone()
    }

    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        let denom = dot(self.normal, r.direction);

        // No hit if the ray is parallel to the plane
        if denom.abs() < 1e-8 {
            return false;
        }

        // Return false if the hit point parameter t is outside the ray interval
        let t = (self.D - dot(self.normal, r.origin)) / denom;
        if !ray_t.contains(t) {
            return false;
        }

        // Determine if the hit point lies within the planar shape using its plane coordinates
        let intersection = r.at(t);
        let planar_hitpt_vector = intersection - self.Q;
        let alpha = dot(self.w, cross(planar_hitpt_vector, self.v));
        let beta = dot(self.w, cross(self.u, planar_hitpt_vector));

        if !self.is_interior(alpha, beta, rec) {
            return false;
        }
        

        rec.t = t;
        rec.p = intersection;
        rec.mat = self.mat.clone();
        rec.set_face_normal(r, self.normal);

        return true;
    }
}