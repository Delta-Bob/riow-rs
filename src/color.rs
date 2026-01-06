use std::io::Write;

use crate::vec3::Vec3;

pub type Color = Vec3;

pub fn write_color(out: &mut impl Write, pixel_color: Color) {
    let r = pixel_color.x();
    let g = pixel_color.y();
    let b = pixel_color.z();

    let rbyte = (255.999 * r) as i32;
    let gbyte = (255.999 * g) as i32;
    let bbyte = (255.999 * b) as i32;

    writeln!(out, "{} {} {}\n", rbyte, gbyte, bbyte).unwrap();
}