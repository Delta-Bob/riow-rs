//! The `Hittable` trait, `HitRecord`, and the instance transforms.

use std::sync::Arc;

use crate::aabb::Aabb;
use crate::interval::Interval;
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3, dot};

/// A surface hit. `mat` borrows from the object that was hit, so recording a
/// hit never touches a reference count.
#[derive(Clone, Default)]
pub struct HitRecord<'a> {
    pub p: Point3,
    pub normal: Vec3,
    pub mat: Option<&'a dyn Material>,
    pub t: f64,
    pub u: f64,
    pub v: f64,
    pub front_face: bool,
}

impl<'a> HitRecord<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets `normal` so that it always points against the incoming ray.
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: Vec3) {
        self.front_face = dot(r.direction, outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }
}

pub trait Hittable: Send + Sync {
    fn hit<'a>(&'a self, ray: &Ray, ray_t: Interval, rec: &mut HitRecord<'a>) -> bool;

    fn bounding_box(&self) -> Aabb;

    /// Probability density of a ray with the given origin, direction and time
    /// hitting this object.
    fn pdf_value(&self, _origin: &Point3, _direction: &Vec3, _time: f64) -> f64 {
        0.0
    }

    /// Generates a random direction from `origin` toward this object.
    fn random(&self, _origin: &Point3, _time: f64) -> Vec3 {
        Vec3::new(1.0, 0.0, 0.0)
    }

    /// True when this object contains no light sources; lets the camera skip
    /// mixture-PDF sampling in scenes without lights.
    fn is_empty(&self) -> bool {
        false
    }
}

impl Hittable for Arc<dyn Hittable> {
    fn hit<'a>(&'a self, ray: &Ray, ray_t: Interval, rec: &mut HitRecord<'a>) -> bool {
        self.as_ref().hit(ray, ray_t, rec)
    }

    fn bounding_box(&self) -> Aabb {
        self.as_ref().bounding_box()
    }

    fn pdf_value(&self, origin: &Point3, direction: &Vec3, time: f64) -> f64 {
        self.as_ref().pdf_value(origin, direction, time)
    }

    fn random(&self, origin: &Point3, time: f64) -> Vec3 {
        self.as_ref().random(origin, time)
    }

    fn is_empty(&self) -> bool {
        self.as_ref().is_empty()
    }
}

/// Moves an object by a fixed offset.
pub struct Translate {
    object: Arc<dyn Hittable>,
    offset: Vec3,
    bbox: Aabb,
}

impl Translate {
    pub fn new(object: Arc<dyn Hittable>, offset: Vec3) -> Self {
        let bbox = object.bounding_box() + offset;
        Self {
            object,
            offset,
            bbox,
        }
    }
}

impl Hittable for Translate {
    fn hit<'a>(&'a self, ray: &Ray, ray_t: Interval, rec: &mut HitRecord<'a>) -> bool {
        // Reject rays that miss the world-space bounds before transforming.
        if !self.bbox.hit(ray, ray_t) {
            return false;
        }

        // Move the ray backwards by the offset.
        let offset_ray = Ray::new(ray.origin - self.offset, ray.direction, ray.time);

        // Determine whether an intersection exists along the offset ray.
        if !self.object.hit(&offset_ray, ray_t, rec) {
            return false;
        }

        // Move the intersection point forwards by the offset.
        rec.p += self.offset;
        true
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}

/// Rotates an object around the y axis.
pub struct RotateY {
    object: Arc<dyn Hittable>,
    sin_theta: f64,
    cos_theta: f64,
    bbox: Aabb,
}

impl RotateY {
    pub fn new(object: Arc<dyn Hittable>, angle: f64) -> Self {
        let radians = crate::common::degrees_to_radians(angle);
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();
        let bbox = object.bounding_box();

        // Compute the bounding box of the rotated object by rotating its corners.
        let mut min = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

        for x in [bbox.x.min, bbox.x.max] {
            for y in [bbox.y.min, bbox.y.max] {
                for z in [bbox.z.min, bbox.z.max] {
                    let new_x = cos_theta * x + sin_theta * z;
                    let new_z = -sin_theta * x + cos_theta * z;
                    let corner = Vec3::new(new_x, y, new_z);

                    for c in 0..3 {
                        if corner[c] < min[c] {
                            min[c] = corner[c];
                        }
                        if corner[c] > max[c] {
                            max[c] = corner[c];
                        }
                    }
                }
            }
        }

        let bbox = Aabb::from_points(min, max);
        Self {
            object,
            sin_theta,
            cos_theta,
            bbox,
        }
    }
}

impl Hittable for RotateY {
    fn hit<'a>(&'a self, r: &Ray, ray_t: Interval, rec: &mut HitRecord<'a>) -> bool {
        // Reject rays that miss the world-space bounds before transforming.
        if !self.bbox.hit(r, ray_t) {
            return false;
        }

        // Transform the ray from world space to object space.
        let origin = Point3::new(
            self.cos_theta * r.origin.x() - self.sin_theta * r.origin.z(),
            r.origin.y(),
            self.sin_theta * r.origin.x() + self.cos_theta * r.origin.z(),
        );

        let direction = Vec3::new(
            self.cos_theta * r.direction.x() - self.sin_theta * r.direction.z(),
            r.direction.y(),
            self.sin_theta * r.direction.x() + self.cos_theta * r.direction.z(),
        );

        let rotated_ray = Ray::new(origin, direction, r.time);

        // Determine whether an intersection exists in object space.
        if !self.object.hit(&rotated_ray, ray_t, rec) {
            return false;
        }

        // Transform the intersection from object space back to world space.
        rec.p = Point3::new(
            self.cos_theta * rec.p.x() + self.sin_theta * rec.p.z(),
            rec.p.y(),
            -self.sin_theta * rec.p.x() + self.cos_theta * rec.p.z(),
        );

        rec.normal = Vec3::new(
            self.cos_theta * rec.normal.x() + self.sin_theta * rec.normal.z(),
            rec.normal.y(),
            -self.sin_theta * rec.normal.x() + self.cos_theta * rec.normal.z(),
        );

        true
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}
