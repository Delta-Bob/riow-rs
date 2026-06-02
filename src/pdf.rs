use crate::vec3::{Point3, Vec3, cross, dot, unit_vector};
use crate::common::{random_f64, PI};
use crate::hittable::Hittable;

pub struct Onb {
    pub axis: [Vec3; 3],
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
    pub fn u(&self) -> Vec3 { self.axis[0] }
    pub fn v(&self) -> Vec3 { self.axis[1] }
    pub fn w(&self) -> Vec3 { self.axis[2] }

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

pub trait Pdf {
    fn value(&self, direction: &Vec3) -> f64;
    fn generate(&self) -> Vec3;
}

pub struct HittablePdf<'a> {
    pub origin: Point3,
    pub objects: &'a dyn Hittable,
}

impl<'a> HittablePdf<'a> {
    pub fn new(objects: &'a dyn Hittable, origin: Point3) -> Self {
        Self { origin, objects }
    }
}

impl<'a> Pdf for HittablePdf<'a> {
    fn value(&self, direction: &Vec3) -> f64 {
        self.objects.pdf_value(&self.origin, direction)
    }

    fn generate(&self) -> Vec3 {
        self.objects.random(&self.origin)
    }
}

pub struct SpherePdf {}

impl SpherePdf {
    pub fn new() -> Self {
        Self {}
    }
}

impl Pdf for SpherePdf {
    fn value(&self, _direction: &Vec3) -> f64 {
        1.0 / (4.0 * PI)
    }

    fn generate(&self) -> Vec3 {
        crate::vec3::random_unit_vector()
    }
}

pub struct MixturePdf<'a> {
    pub p0: &'a dyn Pdf,
    pub p1: &'a dyn Pdf,
}

impl<'a> MixturePdf<'a> {
    pub fn new(p0: &'a dyn Pdf, p1: &'a dyn Pdf) -> Self {
        Self { p0, p1 }
    }
}

impl<'a> Pdf for MixturePdf<'a> {
    fn value(&self, direction: &Vec3) -> f64 {
        0.5 * self.p0.value(direction) + 0.5 * self.p1.value(direction)
    }

    fn generate(&self) -> Vec3 {
        if crate::common::random_f64() < 0.5 {
            self.p0.generate()
        } else {
            self.p1.generate()
        }
    }
}