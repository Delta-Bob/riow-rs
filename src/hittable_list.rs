use crate::hittable::*;
use crate::interval::*;
use crate::ray::*;
use crate::aabb::AABB;
use crate::vec3::{Point3, Vec3};

#[derive(Default)]
pub struct HittableList {
    pub objects: Vec<Box<dyn Hittable>>,
    pub bbox: AABB,
}

impl HittableList {
    pub fn new() -> HittableList {
        Default::default()
    }

    pub fn add(&mut self, object: Box<dyn Hittable>) {
        self.bbox = AABB::from_boxes(&self.bbox, &object.bounding_box());
        self.objects.push(object); 
    }
}

impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        let mut temp_rec = HitRecord::new();
        let mut hit_anything = false;
        let mut closest_so_far = ray_t.max;
 
        for object in &self.objects {
            if object.hit(ray, Interval::new(ray_t.min, closest_so_far), &mut temp_rec) {
                hit_anything = true;
                closest_so_far = temp_rec.t;
                *rec = temp_rec.clone();
            }
        }
 
        hit_anything
    }

    fn bounding_box(&self) -> AABB {
        self.bbox.clone()
    }

    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        let weight = 1.0 / self.objects.len() as f64;
        let sum: f64 = self.objects.iter().map(|obj| weight * obj.pdf_value(origin, direction)).sum();
        sum
    }

    fn random(&self, origin: &Point3) -> Vec3 {
        let int_size = self.objects.len();
        if int_size == 0 {
            return Vec3::new(1.0, 0.0, 0.0);
        }
        let index = crate::common::random_int_range(0, int_size as i32 - 1) as usize;
        self.objects[index].random(origin)
    }

    fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}