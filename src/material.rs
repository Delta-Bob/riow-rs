use crate::color::Color;
use crate::common::random_f64;
use crate::hittable::HitRecord;
use crate::ray::Ray;
use crate::texture::Texture;
use crate::vec3::{Vec3, dot, random_unit_vector, reflect, refract, unit_vector};
use crate::pdf::{Pdf, CosinePdf, SpherePdf};

pub struct ScatterRecord<'a> {
    pub attenuation: Color,
    pub specular_ray: Option<Ray>,
    pub is_specular: bool,
    pub pdf: Option<Box<dyn Pdf + 'a>>,
}

pub trait Material: Send + Sync {
    fn scatter(&self, _r_in: &Ray, _rec: &HitRecord) -> Option<ScatterRecord> {
        None
    }

    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        0.0
    }

    fn emitted(&self, _u: f64, _v: f64, _p: Vec3) -> Color {
        Color::new(0.0, 0.0, 0.0)
    }
}

pub struct Lambertian {
    tex: Box<dyn Texture>,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Lambertian {
        Lambertian {
            tex: Box::new(crate::texture::SolidColor::new(albedo)),
        }
    }

    pub fn from_texture(tex: Box<dyn Texture>) -> Lambertian {
        Lambertian { tex }
    }
}

impl Material for Lambertian {
    fn scatter(&'_ self, _r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord<'_>> {
        let attenuation = self.tex.value(rec.u, rec.v, rec.p);
        let pdf = Box::new(CosinePdf::new(&rec.normal));
        Some(ScatterRecord {
            attenuation,
            specular_ray: None,
            is_specular: false,
            pdf: Some(pdf),
        })
    }

    fn scattering_pdf(&self, _r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        let cosine = dot(rec.normal, unit_vector(scattered.direction));
        f64::max(0.0, cosine / crate::common::PI)
    }
}

pub struct Metal {
    albedo: Color,
    fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Metal {
        Metal {
            albedo,
            fuzz: if fuzz < 1.0 { fuzz } else { 1.0 },
        }
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let mut reflected = reflect(r_in.direction, rec.normal);
        reflected = unit_vector(reflected) + (random_unit_vector() * self.fuzz);
        let specular_ray = Ray::new_with_time(rec.p, reflected, r_in.time());

        Some(ScatterRecord {
            attenuation: self.albedo,
            specular_ray: Some(specular_ray),
            is_specular: true,
            pdf: None,
        })
    }
}

pub struct Dielectric {
    refraction_index: f64,
}

impl Dielectric {
    pub fn new(refraction_index: f64) -> Dielectric {
        Dielectric { refraction_index }
    }

    pub fn reflectance(cosine: f64, refraction_index: f64) -> f64 {
        let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
        r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
    }
}

impl Material for Dielectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord> {
        let attenuation = Color::new(1.0, 1.0, 1.0);
        let ri = if rec.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction = unit_vector(r_in.direction);
        let cos_theta = (dot(-unit_direction, rec.normal)).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = ri * sin_theta > 1.0;
        let direction: Vec3;

        if cannot_refract || Dielectric::reflectance(cos_theta, ri) > random_f64() {
            direction = reflect(unit_direction, rec.normal);
        } else {
            direction = refract(unit_direction, rec.normal, ri);
        }

        let specular_ray = Ray::new_with_time(rec.p, direction, r_in.time());

        Some(ScatterRecord {
            attenuation,
            specular_ray: Some(specular_ray),
            is_specular: true,
            pdf: None,
        })
    }
}

pub struct DiffuseLight {
    tex: Box<dyn Texture>,
}

impl DiffuseLight {
    pub fn new(emit: Color) -> Self {
        Self {
            tex: Box::new(crate::texture::SolidColor::new(emit)),
        }
    }

    pub fn _from_texture(tex: Box<dyn Texture>) -> Self {
        Self { tex }
    }
}

impl Material for DiffuseLight {
    fn scatter(&self, _r_in: &Ray, _rec: &HitRecord) -> Option<ScatterRecord> {
        None
    }

    fn emitted(&self, u: f64, v: f64, p: Vec3) -> Color {
        self.tex.value(u, v, p)
    }
}

pub struct Isotropic {
    tex: Box<dyn Texture>,
}

impl Isotropic {
    pub fn new(albedo: Color) -> Self {
        Self {
            tex: Box::new(crate::texture::SolidColor::new(albedo)),
        }
    }

    pub fn from_texture(tex: Box<dyn Texture>) -> Self {
        Self { tex }
    }
}

impl Material for Isotropic {
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord) -> Option<ScatterRecord<'_>> {
        let attenuation = self.tex.value(rec.u, rec.v, rec.p);
        let pdf = Box::new(SpherePdf::new());
        Some(ScatterRecord {
            attenuation,
            specular_ray: None,
            is_specular: false,
            pdf: Some(pdf),
        })
    }

    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        1.0 / (4.0 * crate::common::PI)
    }
}
