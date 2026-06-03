mod aabb;
mod bvh;
mod camera;
mod color;
mod common;
mod constant_medium;
mod hittable;
mod hittable_list;
mod interval;
mod material;
mod perlin;
mod quad;
mod ray;
mod rtw_stb_image;
mod sphere;
mod texture;
mod triangle;
mod vec3;
mod pdf;

use crate::quad::Quad;
use crate::triangle::Triangle;
use camera::Camera;
use color::Color;
use hittable_list::HittableList;
use material::*;
use sphere::Sphere;
use std::sync::Arc;
use std::time::Instant;
use vec3::{Point3, Vec3};

fn bouncing_spheres() {
    // World
    let mut world = HittableList::default();

    let checker = Arc::new(Lambertian::from_texture(Box::new(
        texture::CheckerTexture::from_colors(
            0.32,
            Color::new(0.2, 0.3, 0.1),
            Color::new(0.9, 0.9, 0.9),
        ),
    )));

    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        Some(checker),
    )));

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
                    let center2 =
                        center + vec3::Vec3::new(0.0, common::random_f64_range(0.0, 0.5), 0.0);
                    world.add(Box::new(Sphere::new_moving(
                        center,
                        center2,
                        0.2,
                        Some(sphere_material),
                    )));
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
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        Some(material1),
    )));

    let material2 = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    world.add(Box::new(Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        Some(material2),
    )));

    let material3 = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));
    world.add(Box::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        Some(material3),
    )));

    let world = bvh::BvhNode::new(world);

    // Camera settings
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;
    let samples_per_pixel = 100;
    let max_depth = 50;
    let background = Color::new(0.70, 0.80, 1.00);

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
        background,
        vfov,
        lookfrom,
        lookat,
        vup,
        defocus_angle,
        focus_dist,
    );

    // Render
    let empty_lights = HittableList::new();
    camera.render(&world, &empty_lights);
}

fn checkered_spheres() {
    let mut world = HittableList::default();

    let checker = Arc::new(Lambertian::from_texture(Box::new(
        texture::CheckerTexture::from_colors(
            0.32,
            Color::new(0.2, 0.3, 0.1),
            Color::new(0.9, 0.9, 0.9),
        ),
    )));

    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -10.0, 0.0),
        10.0,
        Some(checker.clone()),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 10.0, 0.0),
        10.0,
        Some(checker),
    )));

    let world = bvh::BvhNode::new(world);

    // Camera settings
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 1920;
    let samples_per_pixel = 500;
    let max_depth = 50;
    let background = Color::new(0.70, 0.80, 1.00);

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
        background,
        vfov,
        lookfrom,
        lookat,
        vup,
        defocus_angle,
        focus_dist,
    );

    // Render
    let start = Instant::now();
    let empty_lights = HittableList::new();
    camera.render(&world, &empty_lights);
    let duration = start.elapsed();
    eprintln!("Render time: {:?}", duration);
}

fn earth() {
    let mut world = HittableList::default();

    let earth_texture = texture::ImageTexture::new("earthmap.jpg");
    let earth_surface = Arc::new(Lambertian::from_texture(Box::new(earth_texture)));
    let globe = Sphere::new(Point3::new(0.0, 0.0, 0.0), 2.0, Some(earth_surface));

    world.add(Box::new(globe));

    let world = bvh::BvhNode::new(world);

    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;
    let samples_per_pixel = 100;
    let max_depth = 50;
    let background = Color::new(0.70, 0.80, 1.00);

    let vfov = 20.0;
    let lookfrom = Point3::new(0.0, 0.0, 12.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = Vec3::new(0.0, 1.0, 0.0);

    let defocus_angle = 0.0;

    let camera = Camera::new(
        aspect_ratio,
        image_width,
        samples_per_pixel,
        max_depth,
        background,
        vfov,
        lookfrom,
        lookat,
        vup,
        defocus_angle,
        10.0,
    );

    let empty_lights = HittableList::new();
    camera.render(&world, &empty_lights);
}

fn perlin_spheres() {
    let mut world = HittableList::default();

    let pertext = Arc::new(Lambertian::from_texture(Box::new(
        texture::NoiseTexture::new(4.0),
    )));

    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        Some(pertext.clone()),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 2.0, 0.0),
        2.0,
        Some(pertext),
    )));

    let world = bvh::BvhNode::new(world);

    let aspect_ratio = 16.0 / 9.0;
    let image_width = 3840;
    let samples_per_pixel = 2;
    let max_depth = 50;
    let background = Color::new(0.70, 0.80, 1.00);

    let vfov = 20.0;
    let lookfrom = Point3::new(13.0, 2.0, 3.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = Vec3::new(0.0, 1.0, 0.0);

    let defocus_angle = 0.0;

    let camera = Camera::new(
        aspect_ratio,
        image_width,
        samples_per_pixel,
        max_depth,
        background,
        vfov,
        lookfrom,
        lookat,
        vup,
        defocus_angle,
        10.0,
    );

    let empty_lights = HittableList::new();
    camera.render(&world, &empty_lights);
}

fn quads() {
    let mut world = HittableList::default();

    let left_red = Arc::new(Lambertian::new(Color::new(1.0, 0.2, 0.2)));
    let back_green = Arc::new(Lambertian::new(Color::new(0.2, 1.0, 0.2)));
    let right_blue = Arc::new(Lambertian::new(Color::new(0.2, 0.2, 1.0)));
    let upper_orange = Arc::new(Lambertian::new(Color::new(1.0, 0.5, 0.0)));
    let lower_teal = Arc::new(Lambertian::new(Color::new(0.2, 0.8, 0.8)));

    world.add(Box::new(Quad::new(
        Point3::new(-3.0, -2.0, 5.0),
        Vec3::new(0.0, 0.0, -4.0),
        Vec3::new(0.0, 4.0, 0.0),
        Some(left_red),
    )));
    world.add(Box::new(Quad::new(
        Point3::new(-2.0, -2.0, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 4.0, 0.0),
        Some(back_green),
    )));
    world.add(Box::new(Quad::new(
        Point3::new(3.0, -2.0, 1.0),
        Vec3::new(0.0, 0.0, 4.0),
        Vec3::new(0.0, 4.0, 0.0),
        Some(right_blue),
    )));
    world.add(Box::new(Quad::new(
        Point3::new(-2.0, 3.0, 1.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 4.0),
        Some(upper_orange),
    )));
    world.add(Box::new(Quad::new(
        Point3::new(-2.0, -3.0, 5.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -4.0),
        Some(lower_teal),
    )));

    let world = bvh::BvhNode::new(world);

    let aspect_ratio = 1.0;
    let image_width = 400;
    let samples_per_pixel = 100;
    let max_depth = 50;
    let background = Color::new(0.70, 0.80, 1.00);

    let vfov = 80.0;
    let lookfrom = Point3::new(0.0, 0.0, 9.0);
    let lookat = Point3::new(0.0, 0.0, 0.0);
    let vup = Vec3::new(0.0, 1.0, 0.0);

    let defocus_angle = 0.0;

    let camera = Camera::new(
        aspect_ratio,
        image_width,
        samples_per_pixel,
        max_depth,
        background,
        vfov,
        lookfrom,
        lookat,
        vup,
        defocus_angle,
        10.0,
    );

    let empty_lights = HittableList::new();
    camera.render(&world, &empty_lights);
}
fn triangles() {
    // Build a coherent triangular demo: a colored tetrahedron above a ground plane.
    let mut world = HittableList::default();

    // Ground (large sphere acting as a ground plane)
    let ground_mat = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.8)));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -100.5, 0.0),
        100.0,
        Some(ground_mat),
    )));

    // Face materials
    let red = Arc::new(Lambertian::new(Color::new(0.85, 0.15, 0.15)));
    let green = Arc::new(Lambertian::new(Color::new(0.15, 0.85, 0.15)));
    let blue = Arc::new(Lambertian::new(Color::new(0.15, 0.15, 0.85)));
    let yellow = Arc::new(Lambertian::new(Color::new(0.9, 0.85, 0.2)));

    // Tetrahedron vertices (centered near the origin)
    let v0 = Point3::new(0.0, 1.0, 0.0);
    let v1 = Point3::new(-1.0, 0.0, 1.0);
    let v2 = Point3::new(1.0, 0.0, 1.0);
    let v3 = Point3::new(0.0, 0.0, -1.0);

    // Three side faces and a base face
    world.add(Box::new(Triangle::new(v0, v1 - v0, v2 - v0, Some(red))));
    world.add(Box::new(Triangle::new(v0, v2 - v0, v3 - v0, Some(green))));
    world.add(Box::new(Triangle::new(v0, v3 - v0, v1 - v0, Some(blue))));
    world.add(Box::new(Triangle::new(v1, v3 - v1, v2 - v1, Some(yellow))));

    let world = bvh::BvhNode::new(world);

    // Camera: angle and distance chosen to frame the tetrahedron nicely
    let lookfrom = Point3::new(3.0, 2.0, 5.0);
    let lookat = Point3::new(0.0, 0.3, 0.0);
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 1920;
    let samples_per_pixel = 500;
    let max_depth = 50;
    let background = Color::new(0.70, 0.80, 1.00);
    let vfov = 40.0;
    let vup = Vec3::new(0.0, 1.0, 0.0);
    let defocus_angle = 0.0;
    let focus_dist = (lookfrom - lookat).length();

    let camera = Camera::new(
        aspect_ratio,
        image_width,
        samples_per_pixel,
        max_depth,
        background,
        vfov,
        lookfrom,
        lookat,
        vup,
        defocus_angle,
        focus_dist,
    );

    let empty_lights = HittableList::new();
    camera.render(&world, &empty_lights);
}

fn simple_light() {
    let mut world = HittableList::default();

    let pertext = Arc::new(Lambertian::from_texture(Box::new(
        texture::NoiseTexture::new(4.0),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        Some(pertext.clone()),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 2.0, 0.0),
        2.0,
        Some(pertext),
    )));

    let difflight = Arc::new(DiffuseLight::new(Color::new(4.0, 4.0, 4.0)));
    world.add(Box::new(Quad::new(
        Point3::new(3.0, 1.0, -2.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        Some(difflight.clone()),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 7.0, 0.0),
        2.0,
        Some(difflight.clone()),
    )));

    let world = bvh::BvhNode::new(world);

    let aspect_ratio = 16.0 / 9.0;
    let image_width = 1920;
    let samples_per_pixel = 200;
    let max_depth = 50;
    let background = Color::new(0.0, 0.0, 0.0);

    let vfov = 20.0;
    let lookfrom = Point3::new(26.0, 3.0, 6.0);
    let lookat = Point3::new(0.0, 2.0, 0.0);
    let vup = Vec3::new(0.0, 1.0, 0.0);

    let defocus_angle = 0.0;

    let camera = Camera::new(
        aspect_ratio,
        image_width,
        samples_per_pixel,
        max_depth,
        background,
        vfov,
        lookfrom,
        lookat,
        vup,
        defocus_angle,
        10.0,
    );

    let mut lights = HittableList::new();
    lights.add(Box::new(Quad::new(
        Point3::new(3.0, 1.0, -2.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        None,
    )));
    lights.add(Box::new(Sphere::new(
        Point3::new(0.0, 7.0, 0.0),
        2.0,
        None,
    )));
    camera.render(&world, &lights);
}

fn cornell_box() {
    let mut world = HittableList::default();

    let red = Arc::new(Lambertian::new(Color::new(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(Color::new(0.12, 0.45, 0.15)));
    let light = Arc::new(DiffuseLight::new(Color::new(15.0, 15.0, 15.0)));
    let metal = Arc::new(Metal::new(Vec3::new(1.0, 1.0, 1.0), 10.0));

    // Cornell box walls
    world.add(Box::new(Quad::new(
        Point3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Some(green),
    )));
    world.add(Box::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Some(red),
    )));
    world.add(Box::new(Quad::new(
        Point3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        Some(light),
    )));
    world.add(Box::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Some(white.clone()),
    )));
    world.add(Box::new(Quad::new(
        Point3::new(555.0, 555.0, 555.0),
        Vec3::new(-555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -555.0),
        Some(white.clone()),
    )));
    world.add(Box::new(Quad::new(
        Point3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Some(white.clone()),
    )));

    let box1: Arc<dyn hittable::Hittable> = Arc::new(Quad::box_shape(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 330.0, 165.0),
        Some(metal),
    ));
    let box1 = Arc::new(hittable::RotateY::new(box1, 15.0));
    let box1 = hittable::Translate::new(box1, Vec3::new(265.0, 0.0, 295.0));
    world.add(Box::new(box1));

    let box2: Arc<dyn hittable::Hittable> = Arc::new(Quad::box_shape(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 165.0, 165.0),
        Some(white.clone()),
    ));
    let box2 = Arc::new(hittable::RotateY::new(box2, -18.0));
    let box2 = hittable::Translate::new(box2, Vec3::new(130.0, 0.0, 65.0));
    world.add(Box::new(box2));

    let world = bvh::BvhNode::new(world);

    let aspect_ratio = 1.0;
    let image_width = 720;
    let samples_per_pixel = 100;
    let max_depth = 10;
    let background = Color::new(0.0, 0.0, 0.0);

    let vfov = 40.0;
    let lookfrom = Point3::new(278.0, 278.0, -800.0);
    let lookat = Point3::new(278.0, 278.0, 0.0);
    let vup = Vec3::new(0.0, 1.0, 0.0);

    let defocus_angle = 0.0;
    let focus_dist = 10.0;

    let camera = Camera::new(
        aspect_ratio,
        image_width,
        samples_per_pixel,
        max_depth,
        background,
        vfov,
        lookfrom,
        lookat,
        vup,
        defocus_angle,
        focus_dist,
    );

    let mut lights = HittableList::new();
    lights.add(Box::new(Quad::new(
        Point3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        None,
    )));

    camera.render(&world, &lights);
}

fn cornell_smoke() {
    let mut world = HittableList::default();

    let red = Arc::new(Lambertian::new(Color::new(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(Color::new(0.12, 0.45, 0.15)));
    let light = Arc::new(DiffuseLight::new(Color::new(7.0, 7.0, 7.0)));

    // Cornell box walls
    world.add(Box::new(Quad::new(
        Point3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Some(green),
    )));
    world.add(Box::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Some(red),
    )));
    world.add(Box::new(Quad::new(
        Point3::new(113.0, 554.0, 127.0),
        Vec3::new(330.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 305.0),
        Some(light.clone()),
    )));
    world.add(Box::new(Quad::new(
        Point3::new(0.0, 555.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Some(white.clone()),
    )));
    world.add(Box::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Some(white.clone()),
    )));
    world.add(Box::new(Quad::new(
        Point3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Some(white.clone()),
    )));

    let box1: Arc<dyn hittable::Hittable> = Arc::new(Quad::box_shape(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 330.0, 165.0),
        Some(white.clone()),
    ));
    let box1 = Arc::new(hittable::RotateY::new(box1, 15.0));
    let box1 = Arc::new(hittable::Translate::new(box1, Vec3::new(265.0, 0.0, 295.0)));
    world.add(Box::new(constant_medium::ConstantMedium::new_with_color(
        box1,
        0.01,
        Color::new(0.0, 0.0, 0.0),
    )));

    let box2: Arc<dyn hittable::Hittable> = Arc::new(Quad::box_shape(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 165.0, 165.0),
        Some(white.clone()),
    ));
    let box2 = Arc::new(hittable::RotateY::new(box2, -18.0));
    let box2 = Arc::new(hittable::Translate::new(box2, Vec3::new(130.0, 0.0, 65.0)));
    world.add(Box::new(constant_medium::ConstantMedium::new_with_color(
        box2,
        0.01,
        Color::new(1.0, 1.0, 1.0),
    )));

    let world = bvh::BvhNode::new(world);

    let aspect_ratio = 1.0;
    let image_width = 720;
    let samples_per_pixel = 200;
    let max_depth = 50;
    let background = Color::new(0.0, 0.0, 0.0);

    let vfov = 40.0;
    let lookfrom = Point3::new(278.0, 278.0, -800.0);
    let lookat = Point3::new(278.0, 278.0, 0.0);
    let vup = Vec3::new(0.0, 1.0, 0.0);

    let defocus_angle = 0.0;
    let focus_dist = 10.0;

    let camera = Camera::new(
        aspect_ratio,
        image_width,
        samples_per_pixel,
        max_depth,
        background,
        vfov,
        lookfrom,
        lookat,
        vup,
        defocus_angle,
        focus_dist,
    );

    let mut lights = HittableList::new();
    lights.add(Box::new(Quad::new(
        Point3::new(113.0, 554.0, 127.0),
        Vec3::new(330.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 305.0),
        None,
    )));
    camera.render(&world, &lights);
}

fn final_scene(image_width: usize, samples_per_pixel: usize, max_depth: usize) {
    let mut boxes1 = HittableList::new();
    let ground = Arc::new(Lambertian::new(Color::new(0.48, 0.83, 0.53)));

    let boxes_per_side = 20;
    for i in 0..boxes_per_side {
        for j in 0..boxes_per_side {
            let w = 100.0;
            let x0 = -1000.0 + i as f64 * w;
            let z0 = -1000.0 + j as f64 * w;
            let y0 = 0.0;
            let x1 = x0 + w;
            let y1 = common::random_f64_range(1.0, 101.0);
            let z1 = z0 + w;

            let box_shape = Quad::box_shape(
                Point3::new(x0, y0, z0),
                Point3::new(x1, y1, z1),
                Some(ground.clone()),
            );
            boxes1.add(Box::new(box_shape));
        }
    }

    let mut world = HittableList::default();
    world.add(Box::new(bvh::BvhNode::new(boxes1)));

    let light = Arc::new(DiffuseLight::new(Color::new(7.0, 7.0, 7.0)));
    world.add(Box::new(Quad::new(
        Point3::new(123.0, 554.0, 147.0),
        Vec3::new(300.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 265.0),
        Some(light.clone()),
    )));

    // Moving sphere
    let center1 = Point3::new(400.0, 400.0, 200.0);
    let center2 = center1 + Vec3::new(30.0, 0.0, 0.0);
    let sphere_material = Arc::new(Lambertian::new(Color::new(0.7, 0.3, 0.1)));
    world.add(Box::new(Sphere::new_moving(
        center1,
        center2,
        50.0,
        Some(sphere_material),
    )));

    // Glass sphere
    world.add(Box::new(Sphere::new(
        Point3::new(260.0, 150.0, 45.0),
        50.0,
        Some(Arc::new(Dielectric::new(1.5))),
    )));

    // Metal sphere
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 150.0, 145.0),
        50.0,
        Some(Arc::new(Metal::new(Color::new(0.8, 0.8, 0.9), 1.0))),
    )));

    // Smoke spheres
    let boundary = Sphere::new(
        Point3::new(360.0, 150.0, 145.0),
        70.0,
        Some(Arc::new(Dielectric::new(1.5))),
    );
    let boundary_arc: Arc<dyn hittable::Hittable> = Arc::new(boundary);
    world.add(Box::new(constant_medium::ConstantMedium::new_with_color(
        boundary_arc,
        0.2,
        Color::new(0.2, 0.4, 0.9),
    )));

    // Large outer sphere for atmosphere effect
    let boundary_outer = Sphere::new(
        Point3::new(0.0, 0.0, 0.0),
        5000.0,
        Some(Arc::new(Dielectric::new(1.5))),
    );
    let boundary_outer_arc: Arc<dyn hittable::Hittable> = Arc::new(boundary_outer);
    world.add(Box::new(constant_medium::ConstantMedium::new_with_color(
        boundary_outer_arc,
        0.0001,
        Color::new(1.0, 1.0, 1.0),
    )));

    // Earth sphere
    let earth_texture = texture::ImageTexture::new("earthmap.jpg");
    let earth_mat = Arc::new(Lambertian::from_texture(Box::new(earth_texture)));
    world.add(Box::new(Sphere::new(
        Point3::new(400.0, 200.0, 400.0),
        100.0,
        Some(earth_mat),
    )));

    // Perlin noise sphere
    let perlin_tex = texture::NoiseTexture::new(0.2);
    let perlin_mat = Arc::new(Lambertian::from_texture(Box::new(perlin_tex)));
    world.add(Box::new(Sphere::new(
        Point3::new(220.0, 280.0, 300.0),
        80.0,
        Some(perlin_mat),
    )));

    // Cluster of small random spheres
    let mut boxes2 = HittableList::new();
    let white = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let ns = 1000;
    for _ in 0..ns {
        let random_point = vec3::random(0.0, 165.0);
        boxes2.add(Box::new(Sphere::new(
            Point3::new(random_point.x(), random_point.y(), random_point.z()),
            10.0,
            Some(white.clone()),
        )));
    }

    let boxes2_bvh = Arc::new(bvh::BvhNode::new(boxes2));
    let boxes2_rotated = Arc::new(hittable::RotateY::new(boxes2_bvh, 15.0));
    let boxes2_final = hittable::Translate::new(boxes2_rotated, Vec3::new(-100.0, 270.0, 395.0));
    world.add(Box::new(boxes2_final));

    let world = bvh::BvhNode::new(world);

    let aspect_ratio = 16.0 / 9.0;
    let background = Color::new(0.0, 0.0, 0.0);

    let vfov = 40.0;
    let lookfrom = Point3::new(478.0, 278.0, -600.0);
    let lookat = Point3::new(278.0, 278.0, 0.0);
    let vup = Vec3::new(0.0, 1.0, 0.0);

    let defocus_angle = 0.0;

    let camera = Camera::new(
        aspect_ratio,
        image_width as i32,
        samples_per_pixel as i32,
        max_depth as i32,
        background,
        vfov,
        lookfrom,
        lookat,
        vup,
        defocus_angle,
        10.0,
    );

    let mut lights = HittableList::new();
    lights.add(Box::new(Quad::new(
        Point3::new(123.0, 554.0, 147.0),
        Vec3::new(300.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 265.0),
        None,
    )));
    camera.render(&world, &lights);
}

// Choose demo scene here
fn main() {
    let start = Instant::now();

    match 10 {
        1 => bouncing_spheres(),
        2 => checkered_spheres(),
        3 => earth(),
        4 => perlin_spheres(),
        5 => quads(),
        6 => triangles(),
        7 => simple_light(),
        8 => cornell_smoke(),
        9 => cornell_box(),
        10 => final_scene(1280, 10, 32),
        _ => unreachable!(),
    }

    let duration = start.elapsed();
    eprintln!("Render time: {:?}", duration);
}
