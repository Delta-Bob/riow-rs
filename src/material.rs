use std::f64::consts::PI;

use crate::color::Color;
use crate::common::random_f64;
use crate::hittable::HitRecord;
use crate::pdf::{CosinePdf, GgxPdf, HenyeyGreensteinPdf, Pdf, SpherePdf, ggx_d, ggx_g1, hg_phase};
use crate::ray::Ray;
use crate::texture::{SolidColor, Texture};
use crate::vec3::{Vec3, dot, reflect, refract, unit_vector};

/// How a material scatters an incoming ray.
pub enum Scatter {
    /// Delta BSDF: the outgoing ray is fully determined (mirror, glass).
    Specular { attenuation: Color, ray: Ray },
    /// Non-delta BSDF: the outgoing direction is drawn from `pdf`.
    Sampled { pdf: Box<dyn Pdf> },
}

pub trait Material: Send + Sync {
    fn scatter(&self, _r_in: &Ray, _rec: &HitRecord) -> Option<Scatter> {
        None
    }

    /// The material's BSDF times the cosine of the outgoing direction: the
    /// material factor of the rendering equation, used by the camera's
    /// estimator as `eval / pdf`.
    fn eval(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> Color {
        Color::new(0.0, 0.0, 0.0)
    }

    fn emitted(&self, _u: f64, _v: f64, _p: Vec3) -> Color {
        Color::new(0.0, 0.0, 0.0)
    }
}

/// Matte material with a configurable albedo texture.
pub struct Lambertian {
    tex: Box<dyn Texture>,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self::from_texture(Box::new(SolidColor::new(albedo)))
    }

    pub fn from_texture(tex: Box<dyn Texture>) -> Self {
        Self { tex }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord) -> Option<Scatter> {
        Some(Scatter::Sampled {
            pdf: Box::new(CosinePdf::new(&rec.normal)),
        })
    }

    fn eval(&self, _r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> Color {
        let cosine = dot(rec.normal, unit_vector(scattered.direction));
        self.tex.value(rec.u, rec.v, rec.p) * (f64::max(0.0, cosine) / PI)
    }
}

/// Metal with a GGX microfacet distribution and Schlick Fresnel.
pub struct Metal {
    albedo: Color,
    roughness: f64,
}

impl Metal {
    /// `roughness` runs from 0 (a perfect mirror) to 1 (fully rough).
    pub fn new(albedo: Color, roughness: f64) -> Self {
        Self {
            albedo,
            roughness: roughness.clamp(0.0, 1.0),
        }
    }

    fn alpha(&self) -> f64 {
        self.roughness * self.roughness
    }
}

/// Schlick's approximation of the Fresnel reflectance of a conductor.
fn fresnel(f0: Color, cos_theta: f64) -> Color {
    let one = Color::new(1.0, 1.0, 1.0);
    f0 + (one - f0) * (1.0 - cos_theta).powi(5)
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<Scatter> {
        // Smooth metal is a perfect mirror.
        if self.roughness < 1e-3 {
            let direction = reflect(unit_vector(r_in.direction), rec.normal);
            let cos_theta = dot(rec.normal, -unit_vector(r_in.direction)).max(0.0);
            return Some(Scatter::Specular {
                attenuation: fresnel(self.albedo, cos_theta),
                ray: Ray::new(rec.p, direction, r_in.time),
            });
        }

        let view = -unit_vector(r_in.direction);
        Some(Scatter::Sampled {
            pdf: Box::new(GgxPdf::new(rec.normal, view, self.roughness)),
        })
    }

    fn eval(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> Color {
        let view = -unit_vector(r_in.direction);
        let light = unit_vector(scattered.direction);
        let n_dot_v = dot(rec.normal, view);
        let n_dot_l = dot(rec.normal, light);
        if n_dot_v <= 0.0 || n_dot_l <= 0.0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        let half_sum = view + light;
        if half_sum.length_squared() < 1e-12 {
            return Color::new(0.0, 0.0, 0.0);
        }
        let half = unit_vector(half_sum);

        let fresnel = fresnel(self.albedo, dot(view, half).max(0.0));
        let d = ggx_d(dot(rec.normal, half), self.alpha());
        let g = ggx_g1(n_dot_v, self.alpha()) * ggx_g1(n_dot_l, self.alpha());

        // BSDF times cosine: F * D * G / (4 * (n·v)).
        fresnel * (d * g / (4.0 * n_dot_v))
    }
}

/// Clear glass-like material, with reflection at grazing angles.
pub struct Dielectric {
    refraction_index: f64,
}

impl Dielectric {
    pub fn new(refraction_index: f64) -> Self {
        Self { refraction_index }
    }

    /// Schlick's approximation of the Fresnel reflectance.
    fn reflectance(cosine: f64, refraction_index: f64) -> f64 {
        let r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
        let r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
    }
}

impl Material for Dielectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<Scatter> {
        let ri = if rec.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction = unit_vector(r_in.direction);
        let cos_theta = dot(-unit_direction, rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = ri * sin_theta > 1.0;
        let direction = if cannot_refract || Self::reflectance(cos_theta, ri) > random_f64() {
            reflect(unit_direction, rec.normal)
        } else {
            refract(unit_direction, rec.normal, ri)
        };

        Some(Scatter::Specular {
            attenuation: Color::new(1.0, 1.0, 1.0),
            ray: Ray::new(rec.p, direction, r_in.time),
        })
    }
}

/// A material that emits light.
pub struct DiffuseLight {
    tex: Box<dyn Texture>,
}

impl DiffuseLight {
    pub fn new(emit: Color) -> Self {
        Self {
            tex: Box::new(SolidColor::new(emit)),
        }
    }
}

impl Material for DiffuseLight {
    fn emitted(&self, u: f64, v: f64, p: Vec3) -> Color {
        self.tex.value(u, v, p)
    }
}

/// Material for volumetric media: scatters uniformly in all directions.
pub struct Isotropic {
    tex: Box<dyn Texture>,
}

impl Isotropic {
    pub fn new(albedo: Color) -> Self {
        Self {
            tex: Box::new(SolidColor::new(albedo)),
        }
    }
}

impl Material for Isotropic {
    fn scatter(&self, _r_in: &Ray, _rec: &HitRecord) -> Option<Scatter> {
        Some(Scatter::Sampled {
            pdf: Box::new(SpherePdf),
        })
    }

    fn eval(&self, _r_in: &Ray, rec: &HitRecord, _scattered: &Ray) -> Color {
        self.tex.value(rec.u, rec.v, rec.p) * (1.0 / (4.0 * PI))
    }
}

/// Volumetric material with a Henyey-Greenstein phase function; `g > 0`
/// scatters forward, `g < 0` backward.
pub struct HenyeyGreenstein {
    tex: Box<dyn Texture>,
    g: f64,
}

impl HenyeyGreenstein {
    pub fn new(albedo: Color, g: f64) -> Self {
        Self {
            tex: Box::new(SolidColor::new(albedo)),
            g: g.clamp(-0.95, 0.95),
        }
    }
}

impl Material for HenyeyGreenstein {
    fn scatter(&self, r_in: &Ray, _rec: &HitRecord) -> Option<Scatter> {
        Some(Scatter::Sampled {
            pdf: Box::new(HenyeyGreensteinPdf::new(
                unit_vector(r_in.direction),
                self.g,
            )),
        })
    }

    fn eval(&self, r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> Color {
        let cos_theta = dot(
            unit_vector(r_in.direction),
            unit_vector(scattered.direction),
        );
        self.tex.value(rec.u, rec.v, rec.p) * hg_phase(self.g, cos_theta)
    }
}
