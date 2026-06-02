use std::io::Write;

use crate::vec3::Vec3;
use crate::interval::{self};

pub type Color = Vec3;

pub fn linear_to_gamma(linear_component: f64) -> f64 {
    if linear_component > 0.0 {
        return linear_component.sqrt()
    }

    return 0.0
}