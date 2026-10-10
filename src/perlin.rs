//! Perlin noise for procedural textures.

use crate::common::random_f64;
use crate::vec3::{Point3, Vec3, dot, unit_vector};

const POINT_COUNT: usize = 256;

pub struct Perlin {
    randvec: [Vec3; POINT_COUNT],
    perm_x: [usize; POINT_COUNT],
    perm_y: [usize; POINT_COUNT],
    perm_z: [usize; POINT_COUNT],
}

impl Perlin {
    pub fn new() -> Self {
        let mut randvec = [Vec3::default(); POINT_COUNT];
        for v in &mut randvec {
            *v = unit_vector(Vec3::random_range(-1.0, 1.0));
        }

        Self {
            randvec,
            perm_x: Self::generate_perm(),
            perm_y: Self::generate_perm(),
            perm_z: Self::generate_perm(),
        }
    }

    /// Returns a noise value in `[-1, 1]` at the point `p`.
    pub fn noise(&self, p: Point3) -> f64 {
        let u = p.x() - p.x().floor();
        let v = p.y() - p.y().floor();
        let w = p.z() - p.z().floor();

        let i = (p.x().floor() as i32) as usize;
        let j = (p.y().floor() as i32) as usize;
        let k = (p.z().floor() as i32) as usize;
        let mut c = [[[Vec3::default(); 2]; 2]; 2];

        for (di, plane) in c.iter_mut().enumerate() {
            for (dj, row) in plane.iter_mut().enumerate() {
                for (dk, corner) in row.iter_mut().enumerate() {
                    *corner = self.randvec[self.perm_x[(i + di) & 255]
                        ^ self.perm_y[(j + dj) & 255]
                        ^ self.perm_z[(k + dk) & 255]];
                }
            }
        }

        Self::perlin_interp(c, u, v, w)
    }

    /// Sums several octaves of `noise` for a fractal ("turbulence") pattern.
    pub fn turb(&self, p: Point3, depth: i32) -> f64 {
        let mut accum = 0.0;
        let mut temp_p = p;
        let mut weight = 1.0;

        for _ in 0..depth {
            accum += weight * self.noise(temp_p);
            weight *= 0.5;
            temp_p = temp_p * 2.0;
        }

        accum.abs()
    }

    fn perlin_interp(c: [[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        // Hermite smoothing.
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);

        let mut accum = 0.0;
        for (i, plane) in c.iter().enumerate() {
            for (j, row) in plane.iter().enumerate() {
                for (k, corner) in row.iter().enumerate() {
                    let weight_v = Vec3::new(u - i as f64, v - j as f64, w - k as f64);
                    accum += (i as f64 * uu + (1.0 - i as f64) * (1.0 - uu))
                        * (j as f64 * vv + (1.0 - j as f64) * (1.0 - vv))
                        * (k as f64 * ww + (1.0 - k as f64) * (1.0 - ww))
                        * dot(*corner, weight_v);
                }
            }
        }

        accum
    }

    fn generate_perm() -> [usize; POINT_COUNT] {
        let mut p = [0; POINT_COUNT];
        for (i, slot) in p.iter_mut().enumerate() {
            *slot = i;
        }

        Self::permute(&mut p);
        p
    }

    fn permute(p: &mut [usize; POINT_COUNT]) {
        for i in (1..POINT_COUNT).rev() {
            let target = (random_f64() * (i as f64 + 1.0)) as usize;
            p.swap(i, target);
        }
    }
}
