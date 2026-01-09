use crate::color::Color;
use crate::vec3::{Vec3, Point3};

pub trait Texture: Send + Sync {
    fn value(&self, u: f64, v: f64, p: Vec3) -> Color;
}

pub struct SolidColor {
    albedo: Color,
}

impl SolidColor {
    pub fn new(color: Color) -> Self {
        Self { albedo: color }
    }

    pub fn from_rgb(r: f64, g: f64, b: f64) -> Self {
        Self { albedo: Color::new(r, g, b) }
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: Vec3) -> Color {
        self.albedo
    }
}

pub struct CheckerTexture {
    inv_scale: f64,
    even: Box<dyn Texture>,
    odd: Box<dyn Texture>,
}

impl CheckerTexture {
    pub fn new(scale: f64, even : Box<dyn Texture>, odd: Box<dyn Texture>) -> Self {
        Self { inv_scale: 1.0 / scale, even, odd }
    }

    pub fn from_colors(scale: f64, c1: Color , c2: Color) -> Self {
        Self {
            inv_scale: scale,
            even: Box::new(SolidColor::new(c1)),
            odd: Box::new(SolidColor::new(c2)),
        }
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: f64, v: f64, p: Point3) -> Color {
        let x_integer = (self.inv_scale * p.x()).floor() as i32;
        let y_integer = (self.inv_scale * p.y()).floor() as i32;
        let z_integer = (self.inv_scale * p.z()).floor() as i32;

        let is_even = (x_integer + y_integer + z_integer) % 2 == 0;

        if is_even {
            self.even.value(u, v, p)
        } else {
            self.odd.value(u, v, p)
        } 
    }
}