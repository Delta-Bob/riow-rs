use std::sync::atomic::{AtomicUsize, Ordering};

use image::{ImageBuffer, RgbImage};
use rayon::prelude::*;

use crate::background::{Background, BackgroundPdf, Sky};
use crate::color::{Color, ToneMapping, linear_to_srgb};
use crate::common::{self, degrees_to_radians, random_f64, random_f64_range};
use crate::hittable::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::material::{SampledPdf, Scatter};
use crate::pdf::{HittablePdf, Pdf};
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3, cross, random_in_unit_disk, unit_vector};

const OUTPUT_PATH: &str = "image.png";

/// Paths are terminated by Russian roulette once they are this many bounces deep.
const RR_START_DEPTH: i32 = 4;
/// Lower bound on the Russian roulette survival probability, so that the
/// throughput can never explode.
const RR_MIN_SURVIVAL: f64 = 0.05;
/// Mixes a row index into its RNG seed to decorrelate adjacent rows.
const ROW_SEED_MIX: u64 = 0x9E37_79B9_7F4A_7C15;

/// A light-sampling strategy for a shading point: explicit lights and/or a
/// samplable background (a sun), each sampled with equal probability.
struct LightSampler<'a> {
    hittable: Option<HittablePdf<'a>>,
    background: Option<BackgroundPdf<'a>>,
}

impl LightSampler<'_> {
    fn value(&self, direction: &Vec3) -> f64 {
        match (&self.hittable, &self.background) {
            (Some(hittable), Some(background)) => {
                0.5 * hittable.value(direction) + 0.5 * background.value(direction)
            }
            (Some(hittable), None) => hittable.value(direction),
            (None, Some(background)) => background.value(direction),
            (None, None) => 0.0,
        }
    }

    fn generate(&self) -> Vec3 {
        match (&self.hittable, &self.background) {
            (Some(hittable), Some(background)) => {
                if random_f64() < 0.5 {
                    hittable.generate()
                } else {
                    background.generate()
                }
            }
            (Some(hittable), None) => hittable.generate(),
            (None, Some(background)) => background.generate(),
            (None, None) => Vec3::new(1.0, 0.0, 0.0),
        }
    }
}

/// All the knobs that define a camera. `Default` is the standard 16:9 setup
/// used by most of the demo scenes.
#[derive(Clone, Copy)]
pub struct CameraSettings {
    pub aspect_ratio: f64,
    pub image_width: i32,
    pub samples_per_pixel: i32,
    pub max_depth: i32,
    pub background: Background,
    pub vfov: f64,
    pub lookfrom: Point3,
    pub lookat: Point3,
    pub vup: Vec3,
    pub defocus_angle: f64,
    pub focus_dist: f64,
    /// Exposure multiplier applied before tone mapping. The default matches
    /// the radiance scale of `Sky::daylight()` and the demo lights.
    pub exposure: f64,
    pub tone_mapping: ToneMapping,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            aspect_ratio: 16.0 / 9.0,
            image_width: 400,
            samples_per_pixel: 100,
            max_depth: 50,
            background: Background::Sky(Sky::daylight()),
            vfov: 20.0,
            lookfrom: Point3::new(13.0, 2.0, 3.0),
            lookat: Point3::new(0.0, 0.0, 0.0),
            vup: Vec3::new(0.0, 1.0, 0.0),
            defocus_angle: 0.0,
            focus_dist: 10.0,
            exposure: 0.2,
            tone_mapping: ToneMapping::Aces,
        }
    }
}

pub struct Camera {
    settings: CameraSettings,

    // Derived from the settings at construction time.
    image_height: i32,
    pixel_samples_scale: f64,
    center: Point3,
    pixel00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

impl Camera {
    pub fn new(settings: CameraSettings) -> Self {
        let image_height = ((settings.image_width as f64 / settings.aspect_ratio) as i32).max(1);
        let pixel_samples_scale = 1.0 / settings.samples_per_pixel as f64;
        let center = settings.lookfrom;

        // Determine viewport dimensions.
        let theta = degrees_to_radians(settings.vfov);
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * settings.focus_dist;
        let viewport_width = viewport_height * (settings.image_width as f64 / image_height as f64);

        // Camera frame basis vectors.
        let w = unit_vector(settings.lookfrom - settings.lookat);
        let u = unit_vector(cross(settings.vup, w));
        let v = cross(w, u);

        // Vectors across the horizontal and down the vertical viewport edges.
        let viewport_u = u * viewport_width;
        let viewport_v = v * -viewport_height;

        // Pixel-to-pixel delta vectors.
        let pixel_delta_u = viewport_u / settings.image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        // Location of the upper-left pixel.
        let viewport_upper_left =
            center - (w * settings.focus_dist) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel00_loc = viewport_upper_left + (pixel_delta_u + pixel_delta_v) * 0.5;

        // Defocus disk basis vectors.
        let defocus_radius =
            settings.focus_dist * degrees_to_radians(settings.defocus_angle / 2.0).tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        Self {
            settings,
            image_height,
            pixel_samples_scale,
            center,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
            defocus_disk_u,
            defocus_disk_v,
        }
    }

    /// Renders `world` to `image.png`, using `lights` for direct light sampling.
    ///
    /// Each image row is seeded independently, so a render is reproducible
    /// regardless of how rows are scheduled across threads.
    pub fn render(&self, world: &dyn Hittable, lights: &dyn Hittable) {
        let width = self.settings.image_width;
        let height = self.image_height;

        eprintln!(
            "Rendering {width}x{height} ({} samples per pixel, max depth {})",
            self.settings.samples_per_pixel, self.settings.max_depth
        );

        let lines_remaining = AtomicUsize::new(height as usize);

        let pixel_rows: Vec<Vec<Color>> = (0..height)
            .into_par_iter()
            .map(|j| {
                common::set_seed(common::DEFAULT_SEED ^ (j as u64).wrapping_mul(ROW_SEED_MIX));

                let remaining = lines_remaining.fetch_sub(1, Ordering::Relaxed);
                eprint!("\rScanlines remaining: {remaining}   ");

                let samples = self.settings.samples_per_pixel as usize;
                let x_strata = common::shuffled(samples);
                let y_strata = common::shuffled(samples);

                (0..width)
                    .map(|i| self.sample_pixel(i, j, &x_strata, &y_strata, world, lights))
                    .collect()
            })
            .collect();

        let mut image: RgbImage = ImageBuffer::new(width as u32, height as u32);
        for (j, row) in pixel_rows.iter().enumerate() {
            for (i, color) in row.iter().enumerate() {
                image.put_pixel(i as u32, j as u32, self.encode_pixel(*color));
            }
        }

        image.save(OUTPUT_PATH).unwrap();
        eprintln!("\rSaved output to {OUTPUT_PATH}");
    }

    /// Averages jittered stratified samples for pixel `(i, j)`. The stratum
    /// permutations are shared across the row, which costs nothing because
    /// every stratum is sampled either way.
    fn sample_pixel(
        &self,
        i: i32,
        j: i32,
        x_strata: &[usize],
        y_strata: &[usize],
        world: &dyn Hittable,
        lights: &dyn Hittable,
    ) -> Color {
        let samples = self.settings.samples_per_pixel;
        let mut pixel_color = Color::new(0.0, 0.0, 0.0);

        for k in 0..samples as usize {
            let offset = Vec3::new(
                (x_strata[k] as f64 + random_f64()) / samples as f64 - 0.5,
                (y_strata[k] as f64 + random_f64()) / samples as f64 - 0.5,
                0.0,
            );

            let ray = self.get_ray(i, j, offset);
            pixel_color += self.ray_color(
                &ray,
                self.settings.max_depth,
                world,
                lights,
                Color::new(1.0, 1.0, 1.0),
                1.0,
            );
        }

        pixel_color * self.pixel_samples_scale
    }

    /// A camera ray from the defocus disk toward `offset` within pixel `(i, j)`.
    fn get_ray(&self, i: i32, j: i32, offset: Vec3) -> Ray {
        let pixel_sample = self.pixel00_loc
            + self.pixel_delta_u * (i as f64 + offset.x())
            + self.pixel_delta_v * (j as f64 + offset.y());

        let ray_origin = if self.settings.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };

        let ray_direction = pixel_sample - ray_origin;
        let ray_time = random_f64_range(0.0, 1.0);

        Ray::new(ray_origin, ray_direction, ray_time)
    }

    fn defocus_disk_sample(&self) -> Point3 {
        let p = random_in_unit_disk();
        self.center + (self.defocus_disk_u * p[0]) + (self.defocus_disk_v * p[1])
    }

    /// The radiance along `ray`, with `beta` the throughput accumulated so
    /// far and `emission_weight` the MIS weight to apply to radiance arriving
    /// at the first hit. `depth` counts the bounces still allowed.
    fn ray_color(
        &self,
        ray: &Ray,
        depth: i32,
        world: &dyn Hittable,
        lights: &dyn Hittable,
        beta: Color,
        emission_weight: f64,
    ) -> Color {
        if depth <= 0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        let mut rec = HitRecord::new();
        if !world.hit(ray, Interval::new(0.001, f64::INFINITY), &mut rec) {
            let radiance = self
                .settings
                .background
                .radiance(unit_vector(ray.direction));
            return beta * emission_weight * radiance;
        }

        let material = rec.mat.unwrap();
        let emission = beta * emission_weight * material.emitted(rec.u, rec.v, rec.p);

        // Roulette the throughput accumulated *before* this vertex. Testing
        // the post-BSDF throughput instead would kill (and then explode) the
        // huge light-sampling estimators, producing fireflies.
        let Some(beta) = self.roulette(beta, depth) else {
            return emission;
        };

        let Some(scatter) = material.scatter(ray, &rec) else {
            return emission;
        };

        match scatter {
            Scatter::Specular {
                attenuation,
                ray: specular_ray,
            } => {
                let beta = beta * attenuation;
                // Delta BSDFs have no MIS weight: only the BSDF can sample them.
                emission + self.ray_color(&specular_ray, depth - 1, world, lights, beta, 1.0)
            }
            Scatter::Sampled { pdf } => {
                let material_pdf = &pdf;
                let sampler = self.light_sampler(lights, &rec, ray.time);

                // Direct lighting: sample the light strategy and add its
                // contribution, weighted with the balance heuristic.
                let direct = match &sampler {
                    Some(sampler) => {
                        self.direct_light(sampler, material_pdf, &rec, ray, beta, world)
                    }
                    None => Color::new(0.0, 0.0, 0.0),
                };

                // Continuation: sample the material's own distribution. The
                // continuation factor stays the plain BSDF/PDF ratio (paths
                // decay with the albedo); the MIS weight for any light hit
                // further along the path is carried down and applied there.
                let direction = material_pdf.generate();
                let pdf_value = material_pdf.value(&direction);
                if pdf_value < 1e-15 {
                    return emission + direct;
                }

                let scattered = Ray::new(rec.p, direction, ray.time);
                let beta = beta * material.eval(ray, &rec, &scattered) / pdf_value;

                let light_pdf_value = sampler.as_ref().map_or(0.0, |s| s.value(&direction));
                let weight = if light_pdf_value > 0.0 {
                    pdf_value / (pdf_value + light_pdf_value)
                } else {
                    1.0
                };

                emission
                    + direct
                    + self.ray_color(&scattered, depth - 1, world, lights, beta, weight)
            }
        }
    }

    /// The light-sampling strategy for the current shading point: explicit
    /// lights and/or a samplable background (a sun). Returns `None` when
    /// neither exists, in which case paths are traced by BSDF sampling alone.
    fn light_sampler<'a>(
        &'a self,
        lights: &'a dyn Hittable,
        rec: &HitRecord,
        time: f64,
    ) -> Option<LightSampler<'a>> {
        let hittable = if lights.is_empty() {
            None
        } else {
            Some(HittablePdf::new(lights, rec.p, time))
        };
        let background = self.settings.background.sampler();

        if hittable.is_none() && background.is_none() {
            return None;
        }

        Some(LightSampler {
            hittable,
            background,
        })
    }

    /// The next-event contribution at a shading point: sample the light
    /// strategy, trace a shadow ray, and weight the result with the balance
    /// heuristic against the material's own density.
    fn direct_light(
        &self,
        light_pdf: &LightSampler,
        material_pdf: &SampledPdf,
        rec: &HitRecord,
        ray: &Ray,
        beta: Color,
        world: &dyn Hittable,
    ) -> Color {
        let direction = light_pdf.generate();
        let light_pdf_value = light_pdf.value(&direction);
        if light_pdf_value < 1e-15 {
            return Color::new(0.0, 0.0, 0.0);
        }

        let scattered = Ray::new(rec.p, direction, ray.time);
        let eval = rec.mat.unwrap().eval(ray, rec, &scattered);
        if eval.max_component() <= 0.0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        let radiance = {
            let mut shadow_rec = HitRecord::new();
            if world.hit(
                &scattered,
                Interval::new(0.001, f64::INFINITY),
                &mut shadow_rec,
            ) {
                let material = shadow_rec.mat.unwrap();
                material.emitted(shadow_rec.u, shadow_rec.v, shadow_rec.p)
            } else {
                self.settings
                    .background
                    .radiance(unit_vector(scattered.direction))
            }
        };

        let bsdf_pdf_value = material_pdf.value(&direction);
        let weight = light_pdf_value / (light_pdf_value + bsdf_pdf_value);

        beta * eval * radiance * (weight / light_pdf_value)
    }

    /// Russian roulette: once paths are deep enough, terminate them with
    /// probability `1 - q` and scale the throughput by `1 / q`, which keeps
    /// the estimator unbiased. Returns `None` when the path should stop.
    fn roulette(&self, beta: Color, depth: i32) -> Option<Color> {
        if self.settings.max_depth - depth < RR_START_DEPTH {
            return Some(beta);
        }

        let survival = beta.max_component().clamp(RR_MIN_SURVIVAL, 1.0);
        if survival >= 1.0 {
            return Some(beta);
        }
        if random_f64() >= survival {
            return None;
        }

        Some(beta / survival)
    }

    /// Applies exposure, tone mapping and the sRGB transfer curve to one pixel.
    fn encode_pixel(&self, color: Color) -> image::Rgb<u8> {
        let encode = |linear: f64| {
            let exposed = (linear * self.settings.exposure).max(0.0);
            let mapped = self.settings.tone_mapping.apply(exposed);
            (linear_to_srgb(mapped).clamp(0.0, 1.0) * 255.999) as u8
        };
        image::Rgb([encode(color.x()), encode(color.y()), encode(color.z())])
    }
}
