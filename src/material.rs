use crate::common::random_f64;
use crate::ray::Ray;
use crate::hittable::HitRecord;
use crate::color::Color;
use crate::vec3::{random_unit_vector, reflect, unit_vector, dot, refract, Vec3};
use crate::texture::Texture;

pub trait Material: Send + Sync {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        false
    }
}

pub struct Lambertian {
    tex: Box<dyn Texture>,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Lambertian {
        Lambertian { tex: Box::new(crate::texture::SolidColor::new(albedo)) }
    }

    pub fn from_texture(tex: Box<dyn Texture>) -> Lambertian {
        Lambertian { tex  }
    }
}

impl Material for Lambertian {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        let mut scatter_direction = rec.normal + random_unit_vector();
        
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        *scattered = Ray::new_with_time(rec.p, scatter_direction, r_in.time());
        *attenuation = self.tex.value(0.0, 0.0, rec.p);
        true
    }
}

pub struct Metal {
    albedo: Color,
    fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Metal {
        Metal { albedo, fuzz: if fuzz < 1.0 { fuzz } else { 1.0 } }
    }
}

impl Material for Metal {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        let mut reflected = reflect(r_in.direction, rec.normal);
        reflected = unit_vector(reflected) + (random_unit_vector() * self.fuzz);
        *scattered = Ray::new_with_time(rec.p, reflected, r_in.time());
        *attenuation = self.albedo;
        return dot(scattered.direction, rec.normal) >  0.0;
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
        // Use Schlick's approximation for reflectance.
        let mut r0 = (1.0 - refraction_index) / (1.0 + refraction_index);
        r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powf(5.0)
    }
}

impl Material for Dielectric {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        *attenuation = Color::new(1.0, 1.0, 1.0);
        let ri = if rec.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction = unit_vector(r_in.direction);
        let cos_theta = (dot(-unit_direction, rec.normal)).min(1.0);
        let sin_theta = (1.0 - cos_theta*cos_theta).sqrt();

        let cannot_refract = ri * sin_theta > 1.0;
        let direction: Vec3;

        if cannot_refract || Dielectric::reflectance(cos_theta, ri) > random_f64() {
            direction = reflect(unit_direction, rec.normal);
        } else {
            direction = refract(unit_direction, rec.normal, ri);
        }

        *scattered = Ray::new_with_time(rec.p, direction, r_in.time());

        true
    }
}