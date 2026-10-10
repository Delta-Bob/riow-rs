//! Plane math shared by `Quad` and `Triangle`.

use crate::aabb::Aabb;
use crate::interval::Interval;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3, cross, dot, unit_vector};

/// A parallelogram in 3D: corner `q` plus edge vectors `u` and `v`.
///
/// Both `Quad` and `Triangle` live in such a plane; they differ only in how
/// they test whether a planar hit point lies inside the shape.
pub struct Plane {
    pub q: Point3,
    pub u: Vec3,
    pub v: Vec3,
    /// `n / dot(n, n)` where `n = cross(u, v)`: lets us solve for the planar
    /// hit coordinates with a couple of dot products.
    w: Vec3,
    pub normal: Vec3,
    /// Offset of the plane equation: `dot(normal, q)`.
    d: f64,
    bbox: Aabb,
}

impl Plane {
    pub fn new(q: Point3, u: Vec3, v: Vec3) -> Self {
        let n = cross(u, v);
        let normal = unit_vector(n);
        let d = dot(normal, q);
        let w = n / dot(n, n);

        // The shape is contained in the box spanned by its two diagonals.
        let bbox = Aabb::from_points(q, q + u + v).union(Aabb::from_points(q + u, q + v));

        Self {
            q,
            u,
            v,
            w,
            normal,
            d,
            bbox,
        }
    }

    pub fn bbox(&self) -> Aabb {
        self.bbox
    }

    /// Ray/plane intersection. Returns `(t, point, alpha, beta)` if the ray
    /// crosses the infinite plane within `ray_t`, where `alpha` and `beta` are
    /// the hit point's coordinates along `u` and `v`.
    pub fn hit(&self, r: &Ray, ray_t: Interval) -> Option<(f64, Point3, f64, f64)> {
        let denom = dot(self.normal, r.direction);

        // No hit if the ray is parallel to the plane.
        if denom.abs() < 1e-8 {
            return None;
        }

        // No hit if the intersection is outside the ray interval.
        let t = (self.d - dot(self.normal, r.origin)) / denom;
        if !ray_t.contains(t) {
            return None;
        }

        let point = r.at(t);
        let planar_hit = point - self.q;
        let alpha = dot(self.w, cross(planar_hit, self.v));
        let beta = dot(self.w, cross(self.u, planar_hit));

        Some((t, point, alpha, beta))
    }
}
