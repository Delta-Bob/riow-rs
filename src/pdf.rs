//! Probability density functions used to sample scattered ray directions.

use std::f64::consts::PI;

use crate::common::random_f64;
use crate::hittable::Hittable;
use crate::vec3::{Point3, Vec3, cross, dot, unit_vector};

pub trait Pdf {
    fn value(&self, direction: &Vec3) -> f64;
    fn generate(&self) -> Vec3;
}

/// An orthonormal basis built around a given normal.
pub struct Onb {
    axis: [Vec3; 3],
}

impl Onb {
    pub fn build_from_w(n: &Vec3) -> Self {
        let w = unit_vector(*n);
        let a = if w.x().abs() > 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };
        let v = unit_vector(cross(w, a));
        let u = cross(w, v);
        Self { axis: [u, v, w] }
    }

    pub fn u(&self) -> Vec3 {
        self.axis[0]
    }

    pub fn v(&self) -> Vec3 {
        self.axis[1]
    }

    pub fn w(&self) -> Vec3 {
        self.axis[2]
    }

    /// Transforms a vector from basis coordinates to world coordinates.
    pub fn local(&self, a: &Vec3) -> Vec3 {
        self.u() * a.x() + self.v() * a.y() + self.w() * a.z()
    }
}

fn random_cosine_dir() -> Vec3 {
    let r1 = random_f64();
    let r2 = random_f64();

    let phi = 2.0 * PI * r1;
    let x = phi.cos() * r2.sqrt();
    let y = phi.sin() * r2.sqrt();
    let z = (1.0 - r2).sqrt();

    Vec3::new(x, y, z)
}

/// Cosine-weighted density around a normal.
pub struct CosinePdf {
    uvw: Onb,
}

impl CosinePdf {
    pub fn new(w: &Vec3) -> Self {
        Self {
            uvw: Onb::build_from_w(w),
        }
    }
}

impl Pdf for CosinePdf {
    fn value(&self, direction: &Vec3) -> f64 {
        let cosine_theta = dot(unit_vector(*direction), self.uvw.w());
        f64::max(0.0, cosine_theta / PI)
    }

    fn generate(&self) -> Vec3 {
        self.uvw.local(&random_cosine_dir())
    }
}

/// GGX/Trowbridge-Reitz normal distribution function, `alpha = roughness²`.
pub fn ggx_d(n_dot_h: f64, alpha: f64) -> f64 {
    let a2 = alpha * alpha;
    let denom = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    a2 / (PI * denom * denom)
}

/// Smith masking function for the GGX distribution.
pub fn ggx_g1(n_dot_x: f64, alpha: f64) -> f64 {
    let a2 = alpha * alpha;
    2.0 * n_dot_x / (n_dot_x + (a2 + (1.0 - a2) * n_dot_x * n_dot_x).sqrt())
}

/// Half-vector density for GGX microfacets, sampled proportional to `D(n·h)`.
pub struct GgxPdf {
    normal: Vec3,
    view: Vec3,
    alpha: f64,
}

impl GgxPdf {
    /// `normal` is the shading normal, `view` the unit vector toward the
    /// camera, and `roughness` is in `[0, 1]`.
    pub fn new(normal: Vec3, view: Vec3, roughness: f64) -> Self {
        Self {
            normal,
            view,
            alpha: roughness * roughness,
        }
    }
}

impl Pdf for GgxPdf {
    fn value(&self, direction: &Vec3) -> f64 {
        let light = unit_vector(*direction);
        let half_sum = self.view + light;
        if half_sum.length_squared() < 1e-12 {
            return 0.0;
        }
        let half = unit_vector(half_sum);
        let n_dot_h = dot(self.normal, half);
        let v_dot_h = dot(self.view, half);
        if n_dot_h <= 0.0 || v_dot_h <= 0.0 {
            return 0.0;
        }

        ggx_d(n_dot_h, self.alpha) * n_dot_h / (4.0 * v_dot_h)
    }

    fn generate(&self) -> Vec3 {
        loop {
            // Sample a microfacet normal proportional to D(n·h)(n·h).
            let u1 = random_f64();
            let u2 = random_f64();
            let phi = 2.0 * PI * u2;
            let tan_theta_sq = self.alpha * self.alpha * u1 / (1.0 - u1);
            let cos_theta = 1.0 / (1.0 + tan_theta_sq).sqrt();
            let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

            let half_local = Vec3::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta);
            let half = Onb::build_from_w(&self.normal).local(&half_local);
            let light = half * (2.0 * dot(self.view, half)) - self.view;

            if dot(self.normal, light) > 0.0 {
                return light;
            }
        }
    }
}

/// Henyey-Greenstein phase function; `g > 0` scatters forward.
pub fn hg_phase(g: f64, cos_theta: f64) -> f64 {
    let denom = 1.0 + g * g - 2.0 * g * cos_theta;
    (1.0 - g * g) / (4.0 * PI * denom.powf(1.5))
}

/// Density that samples the Henyey-Greenstein phase function.
pub struct HenyeyGreensteinPdf {
    /// Propagation direction of the incoming ray.
    forward: Vec3,
    g: f64,
}

impl HenyeyGreensteinPdf {
    pub fn new(forward: Vec3, g: f64) -> Self {
        Self {
            forward,
            g: g.clamp(-0.95, 0.95),
        }
    }
}

impl Pdf for HenyeyGreensteinPdf {
    fn value(&self, direction: &Vec3) -> f64 {
        let cos_theta = dot(self.forward, unit_vector(*direction));
        hg_phase(self.g, cos_theta)
    }

    fn generate(&self) -> Vec3 {
        // Sample cos(theta) with the inverse-CDF method.
        let u = random_f64();
        let cos_theta = if self.g.abs() < 1e-3 {
            1.0 - 2.0 * u
        } else {
            let t = (1.0 - self.g * self.g) / (1.0 - self.g + 2.0 * self.g * u);
            (1.0 + self.g * self.g - t * t) / (2.0 * self.g)
        };
        let cos_theta = cos_theta.clamp(-1.0, 1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let phi = 2.0 * PI * random_f64();
        let local = Vec3::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta);
        Onb::build_from_w(&self.forward).local(&local)
    }
}

/// Density over the surface area of a set of objects, seen from `origin`.
pub struct HittablePdf<'a> {
    origin: Point3,
    objects: &'a dyn Hittable,
    time: f64,
}

impl<'a> HittablePdf<'a> {
    pub fn new(objects: &'a dyn Hittable, origin: Point3, time: f64) -> Self {
        Self {
            origin,
            objects,
            time,
        }
    }
}

impl Pdf for HittablePdf<'_> {
    fn value(&self, direction: &Vec3) -> f64 {
        self.objects.pdf_value(&self.origin, direction, self.time)
    }

    fn generate(&self) -> Vec3 {
        self.objects.random(&self.origin, self.time)
    }
}

/// Uniform density over the whole sphere.
pub struct SpherePdf;

impl Pdf for SpherePdf {
    fn value(&self, _direction: &Vec3) -> f64 {
        1.0 / (4.0 * PI)
    }

    fn generate(&self) -> Vec3 {
        crate::vec3::random_unit_vector()
    }
}
