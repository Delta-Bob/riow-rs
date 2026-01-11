use crate::common::random_f64;
use crate::vec3::Point3;

pub struct Perlin {
    point_count: usize,
    randfloat: [f64; 256],
    perm_x: [usize; 256],
    perm_y: [usize; 256],
    perm_z: [usize; 256],
}

impl Perlin {
    pub fn new() -> Self {
        let mut perlin = Perlin {
            point_count: 256,
            randfloat: [0.0; 256],
            perm_x: [0; 256],
            perm_y: [0; 256],
            perm_z: [0; 256],
        };

        for i in 0..256 {
            perlin.randfloat[i] = random_f64();
        }

        Self::generate_perm(&mut perlin.perm_x, perlin.point_count);
        Self::generate_perm(&mut perlin.perm_y, perlin.point_count);
        Self::generate_perm(&mut perlin.perm_z, perlin.point_count);

        perlin
    }

    pub fn noise(&self, p: Point3) -> f64 {
        let i = ((4.0 * p.x()) as i32 & 255) as usize;
        let j = ((4.0 * p.y()) as i32 & 255) as usize;
        let k = ((4.0 * p.z()) as i32 & 255) as usize;

        self.randfloat[self.perm_x[i] ^ self.perm_y[j] ^ self.perm_z[k]]
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