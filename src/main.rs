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
mod aabb;
mod bvh;
mod texture;
mod rtw_stb_image;
 
use camera::Camera;
use hittable_list::HittableList;
use sphere::Sphere;
use vec3::{Point3, Vec3};
use material::*;
use color::Color;

use std::sync::Arc;
use std::time::Instant;

fn bouncing_spheres() {
    // World
    let mut world = HittableList::new();

    let checker = Arc::new(Lambertian::from_texture(
        Box::new(texture::CheckerTexture::from_colors(
            0.32,
            Color::new(0.2, 0.3, 0.1),
            Color::new(0.9, 0.9, 0.9),
        ))
    ));

    world.add(Box::new(Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, Some(checker))));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = common::random_f64();
            let center = Point3::new(
                a as f64 + 0.9 * common::random_f64(),
                0.2,
                b as f64 + 0.9 * common::random_f64(),
            );

            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    // diffuse
                    let albedo = Color::random() * Color::random();
                    let sphere_material = Arc::new(Lambertian::new(albedo));
                    let center2 = center + vec3::Vec3::new(0.0, common::random_f64_range(0.0, 0.5), 0.0);
                    world.add(Box::new(Sphere::new_moving(center, center2, 0.2, Some(sphere_material))));
                } else if choose_mat < 0.95 {
                    // metal
                    let albedo = Color::random_range(0.5, 1.0);
                    let fuzz = common::random_f64_range(0.0, 0.5);
                    let sphere_material = Arc::new(Metal::new(albedo, fuzz));
                    world.add(Box::new(Sphere::new(center, 0.2, Some(sphere_material))));
                } else {
                    // glass
                    let sphere_material = Arc::new(Dielectric::new(1.5));
                    world.add(Box::new(Sphere::new(center, 0.2, Some(sphere_material))));
                }
            }
        }
    }

    let material1 = Arc::new(Dielectric::new(1.5));
    world.add(Box::new(Sphere::new(Point3::new(0.0, 1.0, 0.0), 1.0, Some(material1))));

    let material2 = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    world.add(Box::new(Sphere::new(Point3::new(-4.0, 1.0, 0.0), 1.0, Some(material2))));

    let material3 = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));
    world.add(Box::new(Sphere::new(Point3::new(4.0, 1.0, 0.0), 1.0, Some(material3))));

    let world = bvh::BvhNode::new(world);

    // Camera settings
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;
    let samples_per_pixel = 100;
    let max_depth = 50;

    let vfov = 20.0;
    
    let lookfrom = Point3::new(13.0, 2.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = vec3::Vec3::new(0.0, 1.0, 0.0);

    let defocus_angle = 0.6;
    let focus_dist = 10.0;

    // Camera
    let camera = Camera::new(
        aspect_ratio,
        image_width,
        samples_per_pixel,
        max_depth,
        vfov,
        lookfrom,
        lookat,
        vup,
        defocus_angle,
        focus_dist,
    );

    // Render
    let start = Instant::now();
    camera.render(&world);
    let duration = start.elapsed();
    eprintln!("Render time: {:?}", duration);
}

fn checkered_spheres() {
    let mut world = HittableList::new();

    let checker = Arc::new(Lambertian::from_texture(
        Box::new(texture::CheckerTexture::from_colors(
            0.32,
            Color::new(0.2, 0.3, 0.1),
            Color::new(0.9, 0.9, 0.9),
        ))
    ));

    world.add(Box::new(Sphere::new(Point3::new(0.0, -10.0, 0.0), 10.0, Some(checker.clone()))));
    world.add(Box::new(Sphere::new(Point3::new(0.0, 10.0, 0.0), 10.0, Some(checker))));

    let world = bvh::BvhNode::new(world);

        // Camera settings
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;
    let samples_per_pixel = 100;
    let max_depth = 50;

    let vfov = 20.0;
    
    let lookfrom = Point3::new(13.0, 2.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = vec3::Vec3::new(0.0, 1.0, 0.0);

    let defocus_angle = 0.0;
    let focus_dist = 10.0;

    // Camera
    let camera = Camera::new(
        aspect_ratio,
        image_width,
        samples_per_pixel,
        max_depth,
        vfov,
        lookfrom,
        lookat,
        vup,
        defocus_angle,
        focus_dist,
    );

    // Render
    let start = Instant::now();
    camera.render(&world);
    let duration = start.elapsed();
    eprintln!("Render time: {:?}", duration);
}

fn earth() {
    let mut world = HittableList::new();

    let earth_texture = texture::ImageTexture::new("earthmap.jpg");
    let earth_surface = Arc::new(Lambertian::from_texture(Box::new(earth_texture)));
    let globe = Sphere::new(Point3::new(0.0, 0.0, 0.0), 2.0, Some(earth_surface));

    world.add(Box::new(globe));

    let world = bvh::BvhNode::new(world);

    let aspect_ratio      = 16.0 / 9.0;
    let image_width       = 400;
    let samples_per_pixel = 100;
    let max_depth         = 50;

    let vfov     = 20.0;
    let lookfrom = Point3::new(0.0, 0.0, 12.0);
    let lookat   = Point3::new(0.0, 0.0, 0.0);
    let vup      = Vec3::new(0.0, 1.0, 0.0);

    let defocus_angle = 0.0;

    let camera = Camera::new(
        aspect_ratio,
        image_width,
        samples_per_pixel,
        max_depth,
        vfov,
        lookfrom,
        lookat,
        vup,
        defocus_angle,
        10.0,
    );

    camera.render(&world);    
}

fn main() {
    match 3 {
        1 => bouncing_spheres(),
        2 => checkered_spheres(),
        3 => earth(),
        _ => unreachable!(),
    }
}