//! A raytracer in the style of "Ray Tracing in One Weekend", in Rust.
//!
//! Run with `cargo run --release`; the image is written to `image.png`.

mod aabb;
mod background;
mod bvh;
mod camera;
mod color;
mod common;
mod constant_medium;
mod hittable;
mod hittable_list;
mod interval;
mod material;
mod pdf;
mod perlin;
mod plane;
mod quad;
mod ray;
mod rtw_image;
mod scenes;
mod sphere;
mod texture;
mod triangle;
mod vec3;

use std::time::Instant;

/// Which demo scene to render (1-10). See `scenes.rs` for the full list.
const SCENE: u8 = 9;

fn main() {
    // Seeds scene construction; each image row seeds itself during rendering.
    common::set_seed(common::DEFAULT_SEED);

    let start = Instant::now();

    match SCENE {
        1 => scenes::bouncing_spheres(),
        2 => scenes::checkered_spheres(),
        3 => scenes::earth(),
        4 => scenes::perlin_spheres(),
        5 => scenes::quads(),
        6 => scenes::triangles(),
        7 => scenes::simple_light(),
        8 => scenes::cornell_smoke(2400, 1000, 32),
        9 => scenes::cornell_box(200, 500, 50),
        10 => scenes::final_scene(320, 500, 32),
        _ => panic!("SCENE must be between 1 and 10 (got {SCENE})"),
    }

    eprintln!("Render time: {:?}", start.elapsed());
}
