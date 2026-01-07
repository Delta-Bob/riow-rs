mod color;
mod common;
mod hittable;
mod hittable_list;
mod ray;
mod sphere;
mod vec3;
mod interval;
mod camera;
mod material;
 
use camera::Camera;
use hittable_list::HittableList;
use sphere::Sphere;
use vec3::Point3;
use material::*;
use color::Color;

use std::sync::Arc;

use crate::material::Metal;

fn main() {
    // World
    let mut world = HittableList::new();

    let material_ground = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    let material_center = Arc::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    let material_left   = Arc::new(Metal::new(Color::new(0.8, 0.8, 0.8)));
    let material_right  = Arc::new(Metal::new(Color::new(0.8, 0.6, 0.2)));

    world.add(Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, Some(material_ground))));
    world.add(Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.2), 0.5, Some(material_center))));
    world.add(Box::new(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, Some(material_left))));
    world.add(Box::new(Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, Some(material_right))));


    let aspect_ratio        = 16.0 / 9.0;
    let image_width         = 400;
    let samples_per_pixel   = 500;
    let max_depth           = 50;

    // Camera
    let camera = Camera::new(aspect_ratio, image_width, samples_per_pixel, max_depth);
    camera.render(&world);
}
