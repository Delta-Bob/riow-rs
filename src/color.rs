use std::io::Write;

use crate::vec3::Vec3;
use crate::interval::{self, Interval};

pub type Color = Vec3;

pub fn linear_to_gamma(linear_component: f64) -> f64 {
    if linear_component > 0.0 {
        return linear_component.sqrt()
    }

    return 0.0
}

pub fn write_color(out: &mut impl Write, pixel_color: Color) {
    let mut r = pixel_color.x();
    let mut g = pixel_color.y();
    let mut b = pixel_color.z();

    r = linear_to_gamma(r);
    g = linear_to_gamma(g);
    b = linear_to_gamma(b);

    let intensity = interval::Interval { min: 0.000, max: 0.999 };
    let rbyte = (255.999 * intensity.clamp(r)) as i32;
    let gbyte = (255.999 * intensity.clamp(g)) as i32;
    let bbyte = (255.999 * intensity.clamp(b)) as i32;

    writeln!(out, "{} {} {}\n", rbyte, gbyte, bbyte).unwrap();
}