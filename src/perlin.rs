use crate::common::random_f64;
use crate::vec3::*;

pub struct Perlin {
    point_count: usize,
    randvec: [Vec3; 256],
    perm_x: [usize; 256],
    perm_y: [usize; 256],
    perm_z: [usize; 256],
}

impl Perlin {
    pub fn new() -> Self {
        let mut perlin = Perlin {
            point_count: 256,
            randvec: [Vec3::default(); 256],
            perm_x: [0; 256],
            perm_y: [0; 256],
            perm_z: [0; 256],
        };

        for i in 0..perlin.point_count {
            perlin.randvec[i] = unit_vector(random(-1.0, 1.0));
        }

        Self::generate_perm(&mut perlin.perm_x, perlin.point_count);
        Self::generate_perm(&mut perlin.perm_y, perlin.point_count);
        Self::generate_perm(&mut perlin.perm_z, perlin.point_count);

        perlin
    }

    pub fn noise(&self, p: Point3) -> f64 {
        let u = p.x() - p.x().floor();
        let v = p.y() - p.y().floor();
        let w = p.z() - p.z().floor();

        let i = (p.x().floor() as i32) as usize;
        let j = (p.y().floor() as i32) as usize;
        let k = (p.z().floor() as i32) as usize;
        let mut c = [[[Vec3::default(); 2]; 2]; 2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    c[di][dj][dk] = self.randvec[
                        self.perm_x[(i + di) & 255] ^
                        self.perm_y[(j + dj) & 255] ^
                        self.perm_z[(k + dk) & 255]
                    ];
                }
            }
        }

        Self::perlin_interp(c, u, v, w)
    }

    pub fn turb(&self, p: Point3, depth: i32) -> f64 {
        let mut accum = 0.0;
        let mut temp_p = p;
        let mut weight = 1.0;

        for _i in 0..depth {
            accum += weight * self.noise(temp_p);
            weight *= 0.5;
            temp_p = temp_p * 2.0;
        }

        accum.abs()
    }

    fn perlin_interp(c: [[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);
        let mut accum = 0.0;

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let weight_v = Vec3::new(u - i as f64, v - j as f64, w - k as f64);
                    accum += (i as f64 * uu + (1.0 - i as f64) * (1.0 - uu)) *
                             (j as f64 * vv + (1.0 - j as f64) * (1.0 - vv)) *
                             (k as f64 * ww + (1.0 - k as f64) * (1.0 - ww)) *
                             dot(c[i][j][k], weight_v);
                }
            }
        }

        accum
    }

    fn generate_perm(p: &mut [usize; 256], point_count: usize) {
        for i in 0..point_count {
            p[i] = i;
        }

        Self::permute(p, point_count);
    }

    fn permute(p: &mut [usize; 256], n: usize) {
        for i in (1..n).rev() {
            let target = (random_f64() * (i as f64 + 1.0)) as usize;
            p.swap(i, target);
        }
    }
}