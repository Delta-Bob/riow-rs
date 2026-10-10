use std::f64::consts::PI;
use std::sync::Arc;

use crate::aabb::Aabb;
use crate::common::random_f64;
use crate::hittable::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::material::Material;
use crate::pdf::Onb;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3, dot, random_unit_vector};

pub struct Sphere {
    /// Center at time 0.
    center0: Point3,
    /// Center displacement over the shutter interval `[0, 1]`.
    motion: Vec3,
    radius: f64,
    mat: Option<Arc<dyn Material>>,
}

impl Sphere {
    /// A stationary sphere.
    pub fn new(center: Point3, radius: f64, mat: Option<Arc<dyn Material>>) -> Self {
        Self {
            center0: center,
            motion: Vec3::new(0.0, 0.0, 0.0),
            radius: radius.max(0.0),
            mat,
        }
    }

    /// A sphere moving linearly from `center1` to `center2` over time `[0, 1]`.
    pub fn new_moving(
        center1: Point3,
        center2: Point3,
        radius: f64,
        mat: Option<Arc<dyn Material>>,
    ) -> Self {
        Self {
            center0: center1,
            motion: center2 - center1,
            radius: radius.max(0.0),
            mat,
        }
    }

    fn center_at(&self, time: f64) -> Point3 {
        self.center0 + self.motion * time
    }
}

impl Hittable for Sphere {
    fn hit<'a>(&'a self, r: &Ray, ray_t: Interval, rec: &mut HitRecord<'a>) -> bool {
        let center = self.center_at(r.time);
        let oc = r.origin - center;
        let a = r.direction.length_squared();
        let half_b = dot(oc, r.direction);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return false;
        }

        let sqrt_d = f64::sqrt(discriminant);

        // Find the nearest root that lies in the acceptable range.
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
        rec.mat = self.mat.as_deref();

        let (u, v) = get_sphere_uv(&outward_normal);
        rec.u = u;
        rec.v = v;

        true
    }

    fn bounding_box(&self) -> Aabb {
        let rvec = Vec3::new(self.radius, self.radius, self.radius);

        let box0 = Aabb::from_points(self.center_at(0.0) - rvec, self.center_at(0.0) + rvec);
        let box1 = Aabb::from_points(self.center_at(1.0) - rvec, self.center_at(1.0) + rvec);

        box0.union(box1)
    }

    fn pdf_value(&self, origin: &Point3, direction: &Vec3, time: f64) -> f64 {
        let center = self.center_at(time);
        let distance_squared = (*origin - center).length_squared();

        // From inside, we sample uniformly over the sphere.
        if distance_squared <= self.radius * self.radius {
            return 1.0 / (4.0 * PI);
        }

        let mut rec = HitRecord::new();
        let ray = Ray::new(*origin, *direction, time);
        if !self.hit(&ray, Interval::new(0.001, f64::INFINITY), &mut rec) {
            return 0.0;
        }

        let cos_theta_max = (1.0 - self.radius * self.radius / distance_squared).sqrt();
        let solid_angle = 2.0 * PI * (1.0 - cos_theta_max);

        1.0 / solid_angle
    }

    fn random(&self, origin: &Point3, time: f64) -> Vec3 {
        let center = self.center_at(time);
        let direction = center - *origin;
        let distance_squared = direction.length_squared();

        if distance_squared <= self.radius * self.radius {
            return random_unit_vector();
        }

        let uvw = Onb::build_from_w(&direction);

        let r1 = random_f64();
        let r2 = random_f64();

        let z = 1.0 + r2 * ((1.0 - self.radius * self.radius / distance_squared).sqrt() - 1.0);
        let phi = 2.0 * PI * r1;
        let x = phi.cos() * (1.0 - z * z).sqrt();
        let y = phi.sin() * (1.0 - z * z).sqrt();

        uvw.local(&Vec3::new(x, y, z))
    }
}

/// Maps a point on the unit sphere to texture coordinates `(u, v)`.
fn get_sphere_uv(p: &Point3) -> (f64, f64) {
    let theta = f64::acos(-p.y());
    let phi = f64::atan2(-p.z(), p.x()) + PI;

    let u = phi / (2.0 * PI);
    let v = theta / PI;
    (u, v)
}
