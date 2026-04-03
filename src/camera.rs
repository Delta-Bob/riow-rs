use crate::color::Color;
use crate::hittable::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::pdf::{Pdf, HittablePdf, MixturePdf};
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3, unit_vector, cross};
use crate::common::{self, degrees_to_radians, INFINITY};

use std::io::Write;
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Camera {
    pub aspect_ratio: f64,
    pub image_width: i32,
    pub samples_per_pixel: i32,
    pub max_depth: i32,
    pub background: Color,

    pub vfov: f64,
    pub lookfrom: Point3,
    pub lookat: Point3,
    pub vup: Vec3,

    pub defocus_angle: f64,
    pub focus_dist: f64,

    // Private fields
    image_height: i32,
    pixel_samples_scale: f64,
    center: Point3,
    pixel00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

impl Camera {
    pub fn new(aspect_ratio: f64, image_width: i32, samples_per_pixel: i32, max_depth: i32, background: Color, vfov: f64, lookfrom: Point3, lookat: Point3, vup: Vec3, defocus_angle: f64, focus_dist: f64) -> Self {
        let mut camera = Camera {
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
            image_height: 0,
            pixel_samples_scale: 0.0,
            center: Point3::new(0.0, 0.0, 0.0),
            pixel00_loc: Point3::new(0.0, 0.0, 0.0),
            pixel_delta_u: Vec3::new(0.0, 0.0, 0.0),
            pixel_delta_v: Vec3::new(0.0, 0.0, 0.0),
            u: Vec3::new(0.0, 0.0, 0.0),
            v: Vec3::new(0.0, 0.0, 0.0),
            w: Vec3::new(0.0, 0.0, 0.0),
            defocus_disk_u: Vec3::new(0.0, 0.0, 0.0),
            defocus_disk_v: Vec3::new(0.0, 0.0, 0.0),
        };
        camera.initialize();
        camera
    }

    pub fn render(&self, world: &(dyn Hittable + Sync), lights: &(dyn Hittable + Sync)) {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();

        writeln!(handle, "P3\n{} {}\n255", self.image_width, self.image_height).unwrap();

        let lines_remaining = AtomicUsize::new(self.image_height as usize);

        let pixel_rows: Vec<Vec<Color>> = (0..self.image_height).into_par_iter().map(|j| {
            // Print progress (Atomic decrement is thread-safe)
            let remaining = lines_remaining.fetch_sub(1, Ordering::Relaxed);

            eprint!("\rScanlines remaining: {}   ", remaining);

            let mut row_pixels = Vec::with_capacity(self.image_width as usize);

            for i in 0..self.image_width {
                let mut pixel_color = Color::new(0.0, 0.0, 0.0);
                for _ in 0..self.samples_per_pixel {
                    let r = self.get_ray(i, j);
                    pixel_color += self.ray_color(&r, self.max_depth, world, lights);
                }
                // Instead of writing to file, we push to a temporary vector
                row_pixels.push(pixel_color * self.pixel_samples_scale);
            }
            row_pixels
        }).collect(); // This collects all threads' results back into order

        eprintln!("\rDone.                 ");

        for row in pixel_rows {
            for pixel in row {
                crate::color::write_color(&mut handle, pixel);
            }
        }
    }

    fn initialize(&mut self) {
        self.image_height = (self.image_width as f64 / self.aspect_ratio) as i32;
        if self.image_height < 1 {
            self.image_height = 1;
        }

        self.pixel_samples_scale = 1.0 / self.samples_per_pixel as f64;
        
        self.center = self.lookfrom;

        // Determine viewport dimensions
        let theta = degrees_to_radians(self.vfov);
        let h = (theta/2.0).tan();
        let viewport_height = 2.0 * h * self.focus_dist;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);

        self.w = unit_vector(self.lookfrom - self.lookat);
        self.u = unit_vector(cross(self.vup, self.w));
        self.v = cross(self.w, self.u);

        // Calculate the vectors across the horizontal and down the vertical viewport edges
        let viewport_u = self.u * viewport_width;
        let viewport_v = self.v * -viewport_height;

        // Calculate the horizontal and vertical delta vectors from pixel to pixel
        self.pixel_delta_u = viewport_u / (self.image_width as f64);
        self.pixel_delta_v = viewport_v / (self.image_height as f64);

        // Calculate the location of the upper left pixel
        let viewport_upper_left = self.center - (self.w * self.focus_dist) - viewport_u / 2.0 - viewport_v / 2.0;
        self.pixel00_loc = viewport_upper_left + (self.pixel_delta_u + self.pixel_delta_v) * 0.5;

        // Calculate the camera defocus disk basis vectors
        let defocus_radius = self.focus_dist * degrees_to_radians(self.defocus_angle / 2.0).tan();
        self.defocus_disk_u = self.u * defocus_radius;
        self.defocus_disk_v = self.v * defocus_radius;
    }

    fn get_ray(&self, i: i32, j: i32) -> Ray {
        // Construct a camera ray origination from the defocus disk and directed at a randomly
        // samples point around the pixel i, j.

        let offset = self.sample_square();
        let pixel_sample = self.pixel00_loc
            + self.pixel_delta_u * (i as f64 + offset.x())
            + self.pixel_delta_v * (j as f64 + offset.y());

        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };

        let ray_direction = pixel_sample - ray_origin;
        let ray_time = common::random_f64_range(0.0, 1.0);

        Ray::new_with_time(ray_origin, ray_direction, ray_time)
    }

    fn sample_square(&self) -> Vec3 {
        Vec3::new(common::random_f64() - 0.5, common::random_f64() - 0.5, 0.0)
    }

    fn ray_color(&self, r: &Ray, depth: i32, world: &(dyn Hittable + Sync), lights: &(dyn Hittable + Sync)) -> Color {
        if depth <= 0 {
            return Color::new(0.0, 0.0 , 0.0)
        }

        let mut rec = HitRecord::new();

        if !world.hit(r, Interval { min: 0.001, max: INFINITY }, &mut rec) {
            return self.background;
        }

        let color_from_emission = rec.mat.as_ref().unwrap().emitted(rec.u, rec.v, rec.p);

        let scatter_rec = if let Some(sr) = rec.mat.as_ref().unwrap().scatter(r, &rec) {
            sr
        } else {
            return color_from_emission;
        };

        if scatter_rec.is_specular {
            let spec_ray = scatter_rec.specular_ray.unwrap();
            return color_from_emission 
                + scatter_rec.attenuation * self.ray_color(&spec_ray, depth - 1, world, lights);
        }

        let material_pdf = scatter_rec.pdf.unwrap();
        let pdf_val: f64;
        let scattered_dir: Vec3;
        
        if lights.is_empty() {
            scattered_dir = material_pdf.generate();
            pdf_val = material_pdf.value(&scattered_dir);
        } else {
            let light_pdf = HittablePdf::new(lights, rec.p);
            let mixed_pdf = MixturePdf::new(&light_pdf, material_pdf.as_ref());
            scattered_dir = mixed_pdf.generate();
            pdf_val = mixed_pdf.value(&scattered_dir);
        }

        let scattered = Ray::new_with_time(rec.p, scattered_dir, r.time());
        let scattering_pdf_val = rec.mat.as_ref().unwrap().scattering_pdf(r, &rec, &scattered);

        let scatter_color = if pdf_val < 1e-15 {
            Color::new(0.0, 0.0, 0.0)
        } else {
            (scatter_rec.attenuation * scattering_pdf_val * self.ray_color(&scattered, depth - 1, world, lights)) / pdf_val
        };

        return color_from_emission + scatter_color;
    }

    fn defocus_disk_sample(&self) -> Point3 {
        let p = crate::vec3::random_in_unit_disk();
        self.center + (self.defocus_disk_u * p[0]) + (self.defocus_disk_v * p[1])
    }
}