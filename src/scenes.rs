//! The demo scenes. Pick one with `SCENE` in `main.rs`.

use std::sync::Arc;

use crate::background::Background;
use crate::bvh::BvhNode;
use crate::camera::{Camera, CameraSettings};
use crate::color::Color;
use crate::common::{random_f64, random_f64_range};
use crate::constant_medium::ConstantMedium;
use crate::hittable::{Hittable, RotateY, Translate};
use crate::hittable_list::HittableList;
use crate::material::{Dielectric, DiffuseLight, Lambertian, Metal};
use crate::quad::Quad;
use crate::sphere::Sphere;
use crate::texture::{CheckerTexture, ImageTexture, NoiseTexture};
use crate::triangle::Triangle;
use crate::vec3::{Point3, Vec3};

/// Hundreds of small spheres (diffuse, metal and glass) on a checkered ground,
/// with motion blur and depth of field. The "Ray Tracing in One Weekend" cover.
pub fn bouncing_spheres() {
    let mut world = HittableList::new();

    let checker = Arc::new(Lambertian::from_texture(Box::new(
        CheckerTexture::from_colors(0.32, Color::new(0.2, 0.3, 0.1), Color::new(0.9, 0.9, 0.9)),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        Some(checker),
    )));

    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random_f64();
            let center = Point3::new(
                a as f64 + 0.9 * random_f64(),
                0.2,
                b as f64 + 0.9 * random_f64(),
            );

            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    // Diffuse
                    let albedo = Color::random() * Color::random();
                    let sphere_material = Arc::new(Lambertian::new(albedo));
                    let center2 = center + Vec3::new(0.0, random_f64_range(0.0, 0.5), 0.0);
                    world.add(Box::new(Sphere::new_moving(
                        center,
                        center2,
                        0.2,
                        Some(sphere_material),
                    )));
                } else if choose_mat < 0.95 {
                    // Metal
                    let albedo = Color::random_range(0.5, 1.0);
                    let roughness = random_f64_range(0.0, 0.5);
                    let sphere_material = Arc::new(Metal::new(albedo, roughness));
                    world.add(Box::new(Sphere::new(center, 0.2, Some(sphere_material))));
                } else {
                    // Glass
                    let sphere_material = Arc::new(Dielectric::new(1.5));
                    world.add(Box::new(Sphere::new(center, 0.2, Some(sphere_material))));
                }
            }
        }
    }

    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        Some(Arc::new(Dielectric::new(1.5))),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        Some(Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)))),
    )));
    world.add(Box::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        Some(Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0))),
    )));

    let world = BvhNode::new(world);

    let camera = Camera::new(CameraSettings {
        defocus_angle: 0.6,
        ..Default::default()
    });
    camera.render(&world, &HittableList::new());
}

/// Two large checkered spheres.
pub fn checkered_spheres() {
    let mut world = HittableList::new();

    let checker = Arc::new(Lambertian::from_texture(Box::new(
        CheckerTexture::from_colors(0.32, Color::new(0.2, 0.3, 0.1), Color::new(0.9, 0.9, 0.9)),
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

    let world = BvhNode::new(world);

    let camera = Camera::new(CameraSettings {
        image_width: 1920,
        samples_per_pixel: 500,
        ..Default::default()
    });
    camera.render(&world, &HittableList::new());
}

/// A textured globe, using `earthmap.jpg`.
pub fn earth() {
    let mut world = HittableList::new();

    let earth_texture = ImageTexture::new("earthmap.jpg");
    let earth_surface = Arc::new(Lambertian::from_texture(Box::new(earth_texture)));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 0.0, 0.0),
        2.0,
        Some(earth_surface),
    )));

    let world = BvhNode::new(world);

    let camera = Camera::new(CameraSettings {
        lookfrom: Point3::new(0.0, 0.0, 12.0),
        ..Default::default()
    });
    camera.render(&world, &HittableList::new());
}

/// Two spheres with a marble-like Perlin noise texture.
pub fn perlin_spheres() {
    let mut world = HittableList::new();

    let pertext = Arc::new(Lambertian::from_texture(Box::new(NoiseTexture::new(4.0))));
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

    let world = BvhNode::new(world);

    let camera = Camera::new(CameraSettings {
        image_width: 3840,
        samples_per_pixel: 2,
        ..Default::default()
    });
    camera.render(&world, &HittableList::new());
}

/// Five colored quads forming an open box.
pub fn quads() {
    let mut world = HittableList::new();

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

    let world = BvhNode::new(world);

    let camera = Camera::new(CameraSettings {
        aspect_ratio: 1.0,
        vfov: 80.0,
        lookfrom: Point3::new(0.0, 0.0, 9.0),
        ..Default::default()
    });
    camera.render(&world, &HittableList::new());
}

/// A colored tetrahedron above a ground plane, showing off `Triangle`.
pub fn triangles() {
    let mut world = HittableList::new();

    // Ground (a large sphere acting as a plane).
    let ground_mat = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.8)));
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, -100.5, 0.0),
        100.0,
        Some(ground_mat),
    )));

    // Face materials.
    let red = Arc::new(Lambertian::new(Color::new(0.85, 0.15, 0.15)));
    let green = Arc::new(Lambertian::new(Color::new(0.15, 0.85, 0.15)));
    let blue = Arc::new(Lambertian::new(Color::new(0.15, 0.15, 0.85)));
    let yellow = Arc::new(Lambertian::new(Color::new(0.9, 0.85, 0.2)));

    // Tetrahedron vertices (centered near the origin).
    let v0 = Point3::new(0.0, 1.0, 0.0);
    let v1 = Point3::new(-1.0, 0.0, 1.0);
    let v2 = Point3::new(1.0, 0.0, 1.0);
    let v3 = Point3::new(0.0, 0.0, -1.0);

    // Three side faces and a base face.
    world.add(Box::new(Triangle::new(v0, v1 - v0, v2 - v0, Some(red))));
    world.add(Box::new(Triangle::new(v0, v2 - v0, v3 - v0, Some(green))));
    world.add(Box::new(Triangle::new(v0, v3 - v0, v1 - v0, Some(blue))));
    world.add(Box::new(Triangle::new(v1, v3 - v1, v2 - v1, Some(yellow))));

    let world = BvhNode::new(world);

    let lookfrom = Point3::new(3.0, 2.0, 5.0);
    let lookat = Point3::new(0.0, 0.3, 0.0);

    let camera = Camera::new(CameraSettings {
        image_width: 1920,
        samples_per_pixel: 500,
        vfov: 40.0,
        lookfrom,
        lookat,
        focus_dist: (lookfrom - lookat).length(),
        ..Default::default()
    });
    camera.render(&world, &HittableList::new());
}

/// Perlin-noise spheres lit by a rectangular light and a spherical light.
pub fn simple_light() {
    let mut world = HittableList::new();

    let pertext = Arc::new(Lambertian::from_texture(Box::new(NoiseTexture::new(4.0))));
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

    let difflight = Arc::new(DiffuseLight::new(Color::new(40.0, 40.0, 40.0)));
    let light_quad: Arc<dyn Hittable> = Arc::new(Quad::new(
        Point3::new(3.0, 1.0, -2.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        Some(difflight.clone()),
    ));
    let light_sphere: Arc<dyn Hittable> = Arc::new(Sphere::new(
        Point3::new(0.0, 7.0, 0.0),
        2.0,
        Some(difflight),
    ));

    world.add(Box::new(light_quad.clone()));
    world.add(Box::new(light_sphere.clone()));

    let world = BvhNode::new(world);

    let camera = Camera::new(CameraSettings {
        image_width: 1920,
        samples_per_pixel: 200,
        background: Background::Uniform(Color::new(0.0, 0.0, 0.0)),
        lookfrom: Point3::new(26.0, 3.0, 6.0),
        lookat: Point3::new(0.0, 2.0, 0.0),
        ..Default::default()
    });

    let mut lights = HittableList::new();
    lights.add(Box::new(light_quad));
    lights.add(Box::new(light_sphere));
    camera.render(&world, &lights);
}

/// The classic Cornell box, with two rotated metal/white boxes.
pub fn cornell_box(image_width: i32, samples_per_pixel: i32, max_depth: i32) {
    let mut world = HittableList::new();

    let red = Arc::new(Lambertian::new(Color::new(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(Color::new(0.12, 0.45, 0.15)));
    let light_mat = Arc::new(DiffuseLight::new(Color::new(60.0, 60.0, 60.0)));
    let metal = Arc::new(Metal::new(Vec3::new(1.0, 1.0, 1.0), 0.4));

    let light_quad: Arc<dyn Hittable> = Arc::new(Quad::new(
        Point3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        Some(light_mat),
    ));

    // Cornell box walls.
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
    world.add(Box::new(light_quad.clone()));
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

    let box1: Arc<dyn Hittable> = Arc::new(Quad::box_shape(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 330.0, 165.0),
        Some(metal),
    ));
    let box1 = Arc::new(RotateY::new(box1, 15.0));
    let box1 = Translate::new(box1, Vec3::new(265.0, 0.0, 295.0));
    world.add(Box::new(box1));

    let box2: Arc<dyn Hittable> = Arc::new(Quad::box_shape(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 165.0, 165.0),
        Some(white),
    ));
    let box2 = Arc::new(RotateY::new(box2, -18.0));
    let box2 = Translate::new(box2, Vec3::new(130.0, 0.0, 65.0));
    world.add(Box::new(box2));

    let world = BvhNode::new(world);

    let camera = Camera::new(CameraSettings {
        aspect_ratio: 1.0,
        image_width,
        samples_per_pixel,
        max_depth,
        background: Background::Uniform(Color::new(0.0, 0.0, 0.0)),
        vfov: 40.0,
        lookfrom: Point3::new(278.0, 278.0, -800.0),
        lookat: Point3::new(278.0, 278.0, 0.0),
        ..Default::default()
    });

    let mut lights = HittableList::new();
    lights.add(Box::new(light_quad));
    camera.render(&world, &lights);
}

/// The Cornell box with two boxes filled with smoke.
pub fn cornell_smoke(image_width: i32, samples_per_pixel: i32, max_depth: i32) {
    let mut world = HittableList::new();

    let red = Arc::new(Lambertian::new(Color::new(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(Color::new(0.12, 0.45, 0.15)));
    let light_mat = Arc::new(DiffuseLight::new(Color::new(20.0, 20.0, 20.0)));

    let light_quad: Arc<dyn Hittable> = Arc::new(Quad::new(
        Point3::new(113.0, 554.0, 127.0),
        Vec3::new(330.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 305.0),
        Some(light_mat),
    ));

    // Cornell box walls.
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
    world.add(Box::new(light_quad.clone()));
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

    let box1: Arc<dyn Hittable> = Arc::new(Quad::box_shape(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 330.0, 165.0),
        Some(white.clone()),
    ));
    let box1 = Arc::new(RotateY::new(box1, 15.0));
    let box1 = Arc::new(Translate::new(box1, Vec3::new(265.0, 0.0, 295.0)));
    world.add(Box::new(ConstantMedium::new_with_color_and_g(
        box1,
        0.01,
        Color::new(0.0, 0.0, 0.0),
        0.5,
    )));

    let box2: Arc<dyn Hittable> = Arc::new(Quad::box_shape(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 165.0, 165.0),
        Some(white.clone()),
    ));
    let box2 = Arc::new(RotateY::new(box2, -18.0));
    let box2 = Arc::new(Translate::new(box2, Vec3::new(130.0, 0.0, 65.0)));
    world.add(Box::new(ConstantMedium::new_with_color_and_g(
        box2,
        0.01,
        Color::new(1.0, 1.0, 1.0),
        0.5,
    )));

    let world = BvhNode::new(world);

    let camera = Camera::new(CameraSettings {
        image_width,
        samples_per_pixel,
        max_depth,
        background: Background::Uniform(Color::new(0.0, 0.0, 0.0)),
        vfov: 40.0,
        lookfrom: Point3::new(278.0, 278.0, -800.0),
        lookat: Point3::new(278.0, 278.0, 0.0),
        ..Default::default()
    });

    let mut lights = HittableList::new();
    lights.add(Box::new(light_quad));
    camera.render(&world, &lights);
}

/// The "final scene" from the book: a huge ground of boxes, spheres of every
/// material, volumes, textures and a misty atmosphere.
pub fn final_scene(image_width: i32, samples_per_pixel: i32, max_depth: i32) {
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
            let y1 = random_f64_range(1.0, 101.0);
            let z1 = z0 + w;

            let box_shape = Quad::box_shape(
                Point3::new(x0, y0, z0),
                Point3::new(x1, y1, z1),
                Some(ground.clone()),
            );
            boxes1.add(Box::new(box_shape));
        }
    }

    let mut world = HittableList::new();
    world.add(Box::new(BvhNode::new(boxes1)));

    let light_mat = Arc::new(DiffuseLight::new(Color::new(7.0, 7.0, 7.0)));
    let light_quad: Arc<dyn Hittable> = Arc::new(Quad::new(
        Point3::new(123.0, 554.0, 147.0),
        Vec3::new(300.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 265.0),
        Some(light_mat),
    ));
    world.add(Box::new(light_quad.clone()));

    // Moving sphere.
    let center1 = Point3::new(400.0, 400.0, 200.0);
    let center2 = center1 + Vec3::new(30.0, 0.0, 0.0);
    let sphere_material = Arc::new(Lambertian::new(Color::new(0.7, 0.3, 0.1)));
    world.add(Box::new(Sphere::new_moving(
        center1,
        center2,
        50.0,
        Some(sphere_material),
    )));

    // Glass sphere.
    world.add(Box::new(Sphere::new(
        Point3::new(260.0, 150.0, 45.0),
        50.0,
        Some(Arc::new(Dielectric::new(1.5))),
    )));

    // Metal sphere.
    world.add(Box::new(Sphere::new(
        Point3::new(0.0, 150.0, 145.0),
        50.0,
        Some(Arc::new(Metal::new(Color::new(0.8, 0.8, 0.9), 0.5))),
    )));

    // Smoke sphere.
    let boundary: Arc<dyn Hittable> = Arc::new(Sphere::new(
        Point3::new(360.0, 150.0, 145.0),
        70.0,
        Some(Arc::new(Dielectric::new(1.5))),
    ));
    world.add(Box::new(ConstantMedium::new_with_color_and_g(
        boundary,
        0.2,
        Color::new(0.2, 0.4, 0.9),
        0.5,
    )));

    // Large outer sphere for an atmosphere effect.
    let boundary_outer: Arc<dyn Hittable> = Arc::new(Sphere::new(
        Point3::new(0.0, 0.0, 0.0),
        5000.0,
        Some(Arc::new(Dielectric::new(1.5))),
    ));
    world.add(Box::new(ConstantMedium::new_with_color(
        boundary_outer,
        0.00002,
        Color::new(1.0, 1.0, 1.0),
    )));

    // Earth sphere.
    let earth_texture = ImageTexture::new("earthmap.jpg");
    let earth_mat = Arc::new(Lambertian::from_texture(Box::new(earth_texture)));
    world.add(Box::new(Sphere::new(
        Point3::new(400.0, 200.0, 400.0),
        100.0,
        Some(earth_mat),
    )));

    // Perlin noise sphere.
    let perlin_mat = Arc::new(Lambertian::from_texture(Box::new(NoiseTexture::new(0.2))));
    world.add(Box::new(Sphere::new(
        Point3::new(220.0, 280.0, 300.0),
        80.0,
        Some(perlin_mat),
    )));

    // Cluster of small random spheres.
    let mut boxes2 = HittableList::new();
    let white = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    for _ in 0..1000 {
        let random_point = Vec3::random_range(0.0, 165.0);
        boxes2.add(Box::new(Sphere::new(
            random_point,
            10.0,
            Some(white.clone()),
        )));
    }

    let boxes2 = Arc::new(BvhNode::new(boxes2));
    let boxes2 = Arc::new(RotateY::new(boxes2, 15.0));
    let boxes2 = Translate::new(boxes2, Vec3::new(-100.0, 270.0, 395.0));
    world.add(Box::new(boxes2));

    let world = BvhNode::new(world);

    let camera = Camera::new(CameraSettings {
        image_width,
        samples_per_pixel,
        max_depth,
        vfov: 40.0,
        lookfrom: Point3::new(478.0, 278.0, -600.0),
        lookat: Point3::new(278.0, 278.0, 0.0),
        ..Default::default()
    });

    let mut lights = HittableList::new();
    lights.add(Box::new(light_quad));
    camera.render(&world, &lights);
}
