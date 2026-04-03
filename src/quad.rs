use crate::aabb::AABB;
use crate::hittable::*;
use crate::hittable_list::HittableList;
use crate::interval::Interval;
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::*;

use std::sync::Arc;

pub struct Quad {
    q: Point3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    mat: Option<Arc<dyn Material>>,
    bbox: AABB,
    normal: Vec3,
    d: f64,
}

impl Quad {
    pub fn new(q: Point3, u: Vec3, v: Vec3, mat: Option<Arc<dyn Material>>) -> Quad {
        let bbox = AABB::new_point3(q, q + u + v);
        let n = cross(u, v);
        let normal = unit_vector(n);
        let d = dot(normal, q);
        let w = n / dot(n, n);

        let mut quad = Quad {
            q,
            u,
            v,
            w,
            mat,
            bbox,
            normal,
            d,
        };
        quad.set_bounding_box();
        quad
    }

    fn set_bounding_box(&mut self) {
        let bbox_diagonal1 = AABB::new_point3(self.q, self.q + self.u + self.v);
        let bbox_diagonal2 = AABB::new_point3(self.q + self.u, self.q + self.v);
        self.bbox = AABB::from_boxes(&bbox_diagonal1, &bbox_diagonal2);
    }

    fn is_interior(&self, a: f64, b: f64, rec: &mut HitRecord) -> bool {
        let unit_interval = Interval::new(0.0, 1.0);

        if !unit_interval.contains(a) || !unit_interval.contains(b) {
            return false;
        }

        rec.u = a;
        rec.v = b;
        true
    }

    pub fn box_shape(a: Point3, b: Point3, mat: Option<Arc<dyn Material>>) -> HittableList {
        let mut sides = HittableList::new();

        let min = Point3::new(a.x().min(b.x()), a.y().min(b.y()), a.z().min(b.z()));
        let max = Point3::new(a.x().max(b.x()), a.y().max(b.y()), a.z().max(b.z()));

        let dx = Vec3::new(max.x() - min.x(), 0.0, 0.0);
        let dy = Vec3::new(0.0, max.y() - min.y(), 0.0);
        let dz = Vec3::new(0.0, 0.0, max.z() - min.z());

        sides.add(Box::new(Quad::new(Point3::new(min.x(), min.y(), max.z()), dx, dy, mat.clone())));
        sides.add(Box::new(Quad::new(Point3::new(max.x(), min.y(), max.z()), -dz, dy, mat.clone())));
        sides.add(Box::new(Quad::new(Point3::new(max.x(), min.y(), min.z()), -dx, dy, mat.clone())));
        sides.add(Box::new(Quad::new(Point3::new(min.x(), min.y(), min.z()), dz, dy, mat.clone())));
        sides.add(Box::new(Quad::new(Point3::new(min.x(), max.y(), max.z()), dx, -dz, mat.clone())));
        sides.add(Box::new(Quad::new(Point3::new(min.x(), min.y(), min.z()), dx, dz, mat.clone())));

        sides
    }
}

impl Hittable for Quad {
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
        let t = (self.d - dot(self.normal, r.origin)) / denom;
        if !ray_t.contains(t) {
            return false;
        }

        // Determine if the hit point lies within the planar shape using its plane coordinates
        let intersection = r.at(t);
        let planar_hitpt_vector = intersection - self.q;
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

    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        let mut rec = HitRecord::new();
        if !self.hit(&Ray::new_with_time(*origin, *direction, 0.0), Interval::new(0.001, crate::common::INFINITY), &mut rec) {
            return 0.0;
        }

        let distance_squared = rec.t * rec.t * direction.length_squared();
        let cosine = (dot(*direction, rec.normal) / direction.length()).abs();
        let area = cross(self.u, self.v).length();

        distance_squared / (cosine * area)
    }

    fn random(&self, origin: &Point3) -> Vec3 {
        let p = self.q + (self.u * crate::common::random_f64()) + (self.v * crate::common::random_f64());
        p - *origin
    }
}
