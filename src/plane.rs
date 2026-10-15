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
    /// Set when the plane is perpendicular to an axis and both edges run
    /// along the other two axes; enables a fast intersection path.
    axis: Option<usize>,
}

impl Plane {
    pub fn new(q: Point3, u: Vec3, v: Vec3) -> Self {
        let n = cross(u, v);
        let normal = unit_vector(n);
        let d = dot(normal, q);
        let w = n / dot(n, n);

        // The shape is contained in the box spanned by its two diagonals.
        let bbox = Aabb::from_points(q, q + u + v).union(Aabb::from_points(q + u, q + v));

        let axis = axis_aligned(&n, &u, &v);

        Self {
            q,
            u,
            v,
            w,
            normal,
            d,
            bbox,
            axis,
        }
    }

    pub fn bbox(&self) -> Aabb {
        self.bbox
    }

    /// Ray/plane intersection. Returns `(t, point, alpha, beta)` if the ray
    /// crosses the infinite plane within `ray_t`, where `alpha` and `beta` are
    /// the hit point's coordinates along `u` and `v`.
    pub fn hit(&self, r: &Ray, ray_t: Interval) -> Option<(f64, Point3, f64, f64)> {
        if let Some(axis) = self.axis {
            return self.hit_axis_aligned(r, ray_t, axis);
        }

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

    /// The axis-aligned case: the plane equation reduces to a single
    /// coordinate, so the two planar coordinates can be tested directly
    /// against the shape's extent, and the cross products are only needed
    /// once a hit is certain.
    fn hit_axis_aligned(
        &self,
        r: &Ray,
        ray_t: Interval,
        axis: usize,
    ) -> Option<(f64, Point3, f64, f64)> {
        let n = self.normal[axis];
        let denom = n * r.direction[axis];
        if denom.abs() < 1e-8 {
            return None;
        }

        let t = (self.d - n * r.origin[axis]) / denom;
        if !ray_t.contains(t) {
            return None;
        }

        // Reject clearly-outside points before computing alpha and beta. The
        // margin keeps rounding from turning a boundary hit into a miss.
        const MARGIN: f64 = 1e-6;
        let point = r.at(t);
        for other in [(axis + 1) % 3, (axis + 2) % 3] {
            let extent = self.bbox.axis_interval(other);
            let coordinate = point[other];
            if coordinate < extent.min - MARGIN || coordinate > extent.max + MARGIN {
                return None;
            }
        }

        let planar_hit = point - self.q;
        let alpha = dot(self.w, cross(planar_hit, self.v));
        let beta = dot(self.w, cross(self.u, planar_hit));

        Some((t, point, alpha, beta))
    }
}

/// Returns the axis a vector runs along, if it has exactly one nonzero
/// component.
fn single_axis(v: &Vec3) -> Option<usize> {
    let mut found = None;
    for axis in 0..3 {
        if v[axis] != 0.0 {
            if found.is_some() {
                return None;
            }
            found = Some(axis);
        }
    }
    found
}

/// The axis perpendicular to the plane, when the normal runs along one axis
/// and both edges run along the other two.
fn axis_aligned(n: &Vec3, u: &Vec3, v: &Vec3) -> Option<usize> {
    let axis = single_axis(n)?;
    if single_axis(u)? == axis || single_axis(v)? == axis {
        return None;
    }
    Some(axis)
}
