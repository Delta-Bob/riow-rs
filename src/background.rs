//! Environment backgrounds: what a ray sees when it escapes the scene.

use std::f64::consts::PI;

use crate::color::Color;
use crate::common::random_f64;
use crate::pdf::{Onb, Pdf};
use crate::vec3::{Vec3, dot, random_unit_vector, unit_vector};

/// A procedural daytime sky: a vertical gradient plus a sun disk.
#[derive(Clone, Copy)]
pub struct Sky {
    /// Radiance looking straight up.
    pub zenith: Color,
    /// Radiance at the horizon.
    pub horizon: Color,
    /// Unit direction toward the sun.
    pub sun_direction: Vec3,
    /// Angular radius of the sun, in radians.
    pub sun_angular_radius: f64,
    /// Radiance of the sun disk.
    pub sun_radiance: Color,
}

impl Sky {
    /// A clear blue sky with a bright sun, calibrated so that the demo scenes
    /// are well exposed at the default camera exposure.
    pub fn daylight() -> Self {
        Self {
            zenith: Color::new(0.15, 0.30, 0.70),
            horizon: Color::new(0.70, 0.80, 1.00),
            sun_direction: unit_vector(Vec3::new(0.6, 0.7, 0.3)),
            sun_angular_radius: 0.01,
            sun_radiance: Color::new(50_000.0, 48_000.0, 45_000.0),
        }
    }

    fn sun_cos_max(&self) -> f64 {
        self.sun_angular_radius.cos()
    }

    fn sun_is_visible(&self) -> bool {
        self.sun_radiance.max_component() > 0.0
    }

    /// Radiance arriving from `direction` (which must be a unit vector).
    pub fn radiance(&self, direction: Vec3) -> Color {
        let t = direction.y().max(0.0).powf(0.5);
        let mut color = self.horizon * (1.0 - t) + self.zenith * t;

        if self.sun_is_visible() {
            // A hard edge: the radiance support must match the cone that
            // `BackgroundPdf` samples exactly, or rays that graze the edge
            // produce fireflies. The camera's pixel samples antialias it.
            let cos_theta = dot(direction, self.sun_direction);
            if cos_theta >= self.sun_cos_max() {
                color += self.sun_radiance;
            }
        }

        color
    }
}

/// What a ray sees when it escapes the scene.
#[derive(Clone, Copy)]
pub enum Background {
    /// A flat radiance in every direction.
    Uniform(Color),
    /// A procedural sky.
    Sky(Sky),
}

impl Background {
    pub fn radiance(&self, direction: Vec3) -> Color {
        match self {
            Background::Uniform(color) => *color,
            Background::Sky(sky) => sky.radiance(direction),
        }
    }

    /// A `Pdf` that samples the bright parts of this background, when there
    /// are any worth sampling.
    pub fn sampler(&self) -> Option<BackgroundPdf<'_>> {
        match self {
            Background::Uniform(color) if color.max_component() > 0.0 => {
                Some(BackgroundPdf { background: self })
            }
            Background::Sky(sky) if sky.sun_is_visible() => {
                Some(BackgroundPdf { background: self })
            }
            _ => None,
        }
    }
}

/// Samples a `Background`: uniformly over the sphere for flat colors, or over
/// the sun cone for skies.
pub struct BackgroundPdf<'a> {
    background: &'a Background,
}

impl Pdf for BackgroundPdf<'_> {
    fn value(&self, direction: &Vec3) -> f64 {
        match self.background {
            Background::Uniform(_) => 1.0 / (4.0 * PI),
            Background::Sky(sky) => {
                let cos_theta = dot(unit_vector(*direction), sky.sun_direction);
                if cos_theta >= sky.sun_cos_max() {
                    1.0 / (2.0 * PI * (1.0 - sky.sun_cos_max()))
                } else {
                    0.0
                }
            }
        }
    }

    fn generate(&self) -> Vec3 {
        match self.background {
            Background::Uniform(_) => random_unit_vector(),
            Background::Sky(sky) => {
                // Uniform within the cone around the sun.
                let cos_theta = 1.0 - random_f64() * (1.0 - sky.sun_cos_max());
                let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
                let phi = 2.0 * PI * random_f64();
                let local = Vec3::new(sin_theta * phi.cos(), sin_theta * phi.sin(), cos_theta);
                Onb::build_from_w(&sky.sun_direction).local(&local)
            }
        }
    }
}
