use std::sync::Arc;

use crate::aabb::Aabb;
use crate::common::random_f64;
use crate::hittable::{HitRecord, Hittable};
use crate::hittable_list::HittableList;
use crate::interval::Interval;
use crate::material::Material;
use crate::plane::Plane;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3, cross, dot};

pub struct Quad {
    plane: Plane,
    mat: Option<Arc<dyn Material>>,
    /// Surface area, precomputed for the light-sampling density.
    area: f64,
}

impl Quad {
    /// A parallelogram with corner `q` and edge vectors `u` and `v`.
    pub fn new(q: Point3, u: Vec3, v: Vec3, mat: Option<Arc<dyn Material>>) -> Self {
        Self {
            plane: Plane::new(q, u, v),
            mat,
            area: cross(u, v).length(),
        }
    }

    /// An axis-aligned box built from six quads, spanning the corners `a` and `b`.
    pub fn box_shape(a: Point3, b: Point3, mat: Option<Arc<dyn Material>>) -> HittableList {
        let mut sides = HittableList::new();

        let min = Point3::new(a.x().min(b.x()), a.y().min(b.y()), a.z().min(b.z()));
        let max = Point3::new(a.x().max(b.x()), a.y().max(b.y()), a.z().max(b.z()));

        let dx = Vec3::new(max.x() - min.x(), 0.0, 0.0);
        let dy = Vec3::new(0.0, max.y() - min.y(), 0.0);
        let dz = Vec3::new(0.0, 0.0, max.z() - min.z());

        sides.add(Box::new(Quad::new(
            Point3::new(min.x(), min.y(), max.z()),
            dx,
            dy,
            mat.clone(),
        )));
        sides.add(Box::new(Quad::new(
            Point3::new(max.x(), min.y(), max.z()),
            -dz,
            dy,
            mat.clone(),
        )));
        sides.add(Box::new(Quad::new(
            Point3::new(max.x(), min.y(), min.z()),
            -dx,
            dy,
            mat.clone(),
        )));
        sides.add(Box::new(Quad::new(
            Point3::new(min.x(), min.y(), min.z()),
            dz,
            dy,
            mat.clone(),
        )));
        sides.add(Box::new(Quad::new(
            Point3::new(min.x(), max.y(), max.z()),
            dx,
            -dz,
            mat.clone(),
        )));
        sides.add(Box::new(Quad::new(
            Point3::new(min.x(), min.y(), min.z()),
            dx,
            dz,
            mat,
        )));

        sides
    }
}

impl Hittable for Quad {
    fn bounding_box(&self) -> Aabb {
        self.plane.bbox()
    }

    fn hit<'a>(&'a self, r: &Ray, ray_t: Interval, rec: &mut HitRecord<'a>) -> bool {
        let Some((t, point, alpha, beta)) = self.plane.hit(r, ray_t) else {
            return false;
        };

        let unit = Interval::new(0.0, 1.0);
        if !unit.contains(alpha) || !unit.contains(beta) {
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

    fn pdf_value(&self, origin: &Point3, direction: &Vec3, _time: f64) -> f64 {
        let mut rec = HitRecord::new();
        let ray = Ray::new(*origin, *direction, 0.0);
        if !self.hit(&ray, Interval::new(0.001, f64::INFINITY), &mut rec) {
            return 0.0;
        }

        let distance_squared = rec.t * rec.t * direction.length_squared();
        let cosine = (dot(*direction, rec.normal) / direction.length()).abs();

        distance_squared / (cosine * self.area)
    }

    fn random(&self, origin: &Point3, _time: f64) -> Vec3 {
        let p = self.plane.q + (self.plane.u * random_f64()) + (self.plane.v * random_f64());
        p - *origin
    }
}
