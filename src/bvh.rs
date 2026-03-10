use crate::aabb::{AABB, box_x_compare, box_y_compare, box_z_compare,};
use crate::hittable::*;
use crate::hittable_list::HittableList;
use crate::interval::Interval;
use crate::ray::Ray;

use std::sync::Arc;

pub struct BvhNode {
    left: Arc<dyn Hittable>,
    right: Arc<dyn Hittable>,
    bbox: AABB,
}

impl BvhNode {
    pub fn new(list: HittableList) -> Self {
        let len = list.objects.len();
        let objects: Vec<Arc<dyn Hittable>> = list.objects
            .into_iter()
            .map(|obj| Arc::from(obj))
            .collect();
        Self::new_from_vec(objects, 0, len)
    }

    pub fn new_from_vec(
        mut objects: Vec<Arc<dyn Hittable>>,
        start: usize,
        end: usize,
    ) -> Self {

        let mut bbox = AABB::empty();
        for object_index in start..end {
            bbox = AABB::from_boxes(&bbox, &objects[object_index].bounding_box());
        }

        let axis = bbox.longest_axis();

        let comparator = match axis {
            0 => box_x_compare,
            1 => box_y_compare,
            _ => box_z_compare,
        };

        objects[start..end].sort_by(comparator);
        
        let object_span = end - start;

        if object_span == 1 {
            let obj = objects[start].clone();
            let bbox = AABB::from_boxes(
                &obj.bounding_box(),
                &obj.bounding_box(),
            );
            BvhNode {
                left: obj.clone(),
                right: obj,
                bbox,
            }
        } else if object_span == 2 {
            let left = objects[start].clone();
            let right = objects[start + 1].clone();
            let bbox = AABB::from_boxes(&left.bounding_box(), &right.bounding_box());
            BvhNode {
                left,
                right,
                bbox,
            }
        } else {
            let mid = start + object_span / 2;
            let left = Arc::new(Self::new_from_vec(objects.clone(), start, mid));
            let right = Arc::new(Self::new_from_vec(objects, mid, end));
            let bbox = AABB::from_boxes(&left.bounding_box(), &right.bounding_box());

            BvhNode {
                left,
                right,
                bbox,
            }
        }
    }
}

impl Hittable for BvhNode {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
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

    fn bounding_box(&self) -> AABB {
        self.bbox.clone()
    }
}