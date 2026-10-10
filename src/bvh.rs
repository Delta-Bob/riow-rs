use std::sync::Arc;

use crate::aabb::{Aabb, compare_by_axis};
use crate::hittable::{HitRecord, Hittable};
use crate::hittable_list::HittableList;
use crate::interval::Interval;
use crate::ray::Ray;

/// A bounding volume hierarchy over a set of objects.
pub struct BvhNode {
    left: Arc<dyn Hittable>,
    right: Arc<dyn Hittable>,
    bbox: Aabb,
}

impl BvhNode {
    pub fn new(list: HittableList) -> Self {
        let mut objects = Vec::from(list);
        assert!(!objects.is_empty(), "BvhNode requires at least one object");
        Self::build(&mut objects)
    }

    /// Recursively splits `objects` along the longest axis of their bounding
    /// box, in place, until each half fits in a leaf.
    fn build(objects: &mut [Arc<dyn Hittable>]) -> Self {
        let mut bbox = Aabb::empty();
        for object in objects.iter() {
            bbox = bbox.union(object.bounding_box());
        }

        let axis = bbox.longest_axis();
        objects.sort_by(|a, b| compare_by_axis(a, b, axis));

        match objects.len() {
            1 => {
                let object = objects[0].clone();
                Self {
                    left: object.clone(),
                    right: object,
                    bbox,
                }
            }
            2 => Self {
                left: objects[0].clone(),
                right: objects[1].clone(),
                bbox,
            },
            _ => {
                let (left_objects, right_objects) = objects.split_at_mut(objects.len() / 2);
                let left: Arc<dyn Hittable> = Arc::new(Self::build(left_objects));
                let right: Arc<dyn Hittable> = Arc::new(Self::build(right_objects));
                let bbox = left.bounding_box().union(right.bounding_box());

                Self { left, right, bbox }
            }
        }
    }
}

impl Hittable for BvhNode {
    fn hit<'a>(&'a self, r: &Ray, ray_t: Interval, rec: &mut HitRecord<'a>) -> bool {
        if !self.bbox.hit(r, ray_t) {
            return false;
        }

        let hit_left = self.left.hit(r, ray_t, rec);
        let hit_right = self.right.hit(
            r,
            Interval::new(ray_t.min, if hit_left { rec.t } else { ray_t.max }),
            rec,
        );

        hit_left || hit_right
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}
