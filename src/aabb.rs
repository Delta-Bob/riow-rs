use crate::interval::Interval;
use crate::vec3::{Point3, Vec3};
use crate::ray::Ray;
use crate::hittable::Hittable;

use std::sync::Arc;

#[derive(Clone)]
pub struct AABB {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
}

impl Default for AABB {
    fn default() -> Self {
        AABB::new(Interval::default(), Interval::default(), Interval::default())
    }
}

impl AABB {
    pub fn empty() -> Self {
        Self {
            x: Interval::empty(),
            y: Interval::empty(),
            z: Interval::empty(),
        }
    }

    pub fn new(x: Interval, y: Interval, z: Interval) -> Self {
        let mut aabb = Self { x, y, z };
        aabb.pad_to_minimums();
        aabb
    }

    pub fn new_point3(a: Point3, b: Point3) -> Self {
        let x = if a.x() < b.x() {
            Interval::new(a.x(), b.x())
        } else {
            Interval::new(b.x(), a.x())
        };
        let y = if a.y() < b.y() {
            Interval::new(a.y(), b.y())
        } else {
            Interval::new(b.y(), a.y())
        };
        let z = if a.z() < b.z() {
            Interval::new(a.z(), b.z())
        } else {
            Interval::new(b.z(), a.z())
        };

        Self { x, y, z }
    }

    pub fn from_boxes(box0: &AABB, box1: &AABB) -> Self {
        Self {
            x: Interval::from_intervals(&box0.x, &box1.x),
            y: Interval::from_intervals(&box0.y, &box1.y),
            z: Interval::from_intervals(&box0.z, &box1.z),
        }
    }

    pub fn axis_interval(&self, n: usize) -> &Interval {
        match n {
            1 => &self.y,
            2 => &self.z,
            _ => &self.x,
        }
    }

    pub fn hit(&self, r: &Ray, ray_t: Interval) -> bool {
        let ray_orig = r.origin;
        let ray_dir = r.direction;

        for a in 0..3 {
            let ax = self.axis_interval(a);
            let adinv = 1.0 / ray_dir[a];

            let t0 = (ax.min - ray_orig[a]) * adinv;
            let t1 = (ax.max - ray_orig[a]) * adinv;

            if t0 < t1 {
                if t1 < ray_t.min || t0 > ray_t.max {
                    return false;
                }
            } else {
                if t0 < ray_t.min || t1 > ray_t.max {
                    return false;
                }
            }
        }
        true
    }

    pub fn longest_axis(&self) -> i32 {
        // Returns the index of the longest axis of the bounding box

        if self.x.size() > self.y.size() && self.x.size() > self.z.size() {
            0
        } else if self.y.size() > self.z.size() {
            1
        } else {
            2
        }
    }

    fn pad_to_minimums(&mut self) {
        let delta = 0.0001;
        if self.x.size() < delta { self.x = self.x.expand(delta); }
        if self.y.size() < delta { self.y = self.y.expand(delta); }
        if self.z.size() < delta { self.z = self.z.expand(delta); }
    }
}

impl std::ops::Add<Vec3> for AABB {
    type Output = AABB;
    fn add(self, offset: Vec3) -> AABB {
        AABB::new(self.x + offset.x(), self.y + offset.y(), self.z + offset.z())
    }
}

impl std::ops::Add<AABB> for Vec3 {
    type Output = AABB;
    fn add(self, bbox: AABB) -> AABB {
        bbox + self
    }
}

pub fn box_compare(a: &Arc<dyn Hittable>, b: &Arc<dyn Hittable>, axis_index: usize) -> std::cmp::Ordering {
    let a_bbox = a.bounding_box();
    let b_bbox = b.bounding_box();
    let a_axis_interval = a_bbox.axis_interval(axis_index);
    let b_axis_interval = b_bbox.axis_interval(axis_index);
    a_axis_interval.min.partial_cmp(&b_axis_interval.min).unwrap_or(std::cmp::Ordering::Equal)
}

pub fn box_x_compare(a: &Arc<dyn Hittable>, b: &Arc<dyn Hittable>) -> std::cmp::Ordering {
    box_compare(a, b, 0)
}

pub fn box_y_compare(a: &Arc<dyn Hittable>, b: &Arc<dyn Hittable>) -> std::cmp::Ordering {
    box_compare(a, b, 1)
}

pub fn box_z_compare(a: &Arc<dyn Hittable>, b: &Arc<dyn Hittable>) -> std::cmp::Ordering {
    box_compare(a, b, 2)
}