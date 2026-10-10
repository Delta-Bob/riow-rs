use crate::vec3::Vec3;

/// A linear RGB color, stored as a `Vec3`.
pub type Color = Vec3;

/// Encodes one linear-light component with the sRGB transfer curve.
pub fn linear_to_srgb(linear: f64) -> f64 {
    if linear <= 0.003_130_8 {
        12.92 * linear
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

/// Decodes one sRGB-encoded component back to linear light.
pub fn srgb_to_linear(srgb: f64) -> f64 {
    if srgb <= 0.040_45 {
        srgb / 12.92
    } else {
        ((srgb + 0.055) / 1.055).powf(2.4)
    }
}

/// HDR-to-LDR mapping applied to each pixel before the sRGB encode.
///
/// Only `Aces` is used by the demo scenes; the others are camera options.
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum ToneMapping {
    /// No mapping; values above 1 are clipped.
    None,
    /// `x / (1 + x)`: simple and smooth.
    Reinhard,
    /// The ACES filmic curve (Narkowicz approximation), used by film.
    Aces,
}

impl ToneMapping {
    pub fn apply(self, x: f64) -> f64 {
        match self {
            ToneMapping::None => x,
            ToneMapping::Reinhard => x / (1.0 + x),
            ToneMapping::Aces => {
                let (a, b, c, d, e) = (2.51, 0.03, 2.43, 0.59, 0.14);
                (x * (a * x + b)) / (x * (c * x + d) + e)
            }
        }
    }
}
