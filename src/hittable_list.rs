use std::sync::Arc;

use crate::aabb::Aabb;
use crate::common::random_int_range;
use crate::hittable::{HitRecord, Hittable};
use crate::interval::Interval;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// A list of hittable objects, tested in insertion order.
#[derive(Default)]
pub struct HittableList {
    objects: Vec<Box<dyn Hittable>>,
    bbox: Aabb,
}

impl HittableList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, object: Box<dyn Hittable>) {
        self.bbox = self.bbox.union(object.bounding_box());
        self.objects.push(object);
    }

    /// Consumes the list and returns the contained objects.
    pub fn into_objects(self) -> Vec<Box<dyn Hittable>> {
        self.objects
    }
}

impl Hittable for HittableList {
    fn hit<'a>(&'a self, ray: &Ray, ray_t: Interval, rec: &mut HitRecord<'a>) -> bool {
        // The stored box bounds every child, so rays that miss it can skip
        // the whole list.
        if !self.bbox.hit(ray, ray_t) {
            return false;
        }

        let mut temp_rec = HitRecord::new();
        let mut hit_anything = false;
        let mut closest_so_far = ray_t.max;

        for object in &self.objects {
            if object.hit(ray, Interval::new(ray_t.min, closest_so_far), &mut temp_rec) {
                hit_anything = true;
                closest_so_far = temp_rec.t;
                rec.clone_from(&temp_rec);
            }
        }

        hit_anything
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }

    fn pdf_value(&self, origin: &Point3, direction: &Vec3, time: f64) -> f64 {
        let weight = 1.0 / self.objects.len() as f64;
        self.objects
            .iter()
            .map(|object| weight * object.pdf_value(origin, direction, time))
            .sum()
    }

    fn random(&self, origin: &Point3, time: f64) -> Vec3 {
        let count = self.objects.len();
        if count == 0 {
            return Vec3::new(1.0, 0.0, 0.0);
        }
        let index = random_int_range(0, count as i32 - 1) as usize;
        self.objects[index].random(origin, time)
    }

    fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}

impl From<HittableList> for Vec<Arc<dyn Hittable>> {
    fn from(list: HittableList) -> Self {
        list.into_objects().into_iter().map(Arc::from).collect()
    }
}
