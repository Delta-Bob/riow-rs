use std::sync::Arc;

use crate::aabb::Aabb;
use crate::hittable::{HitRecord, Hittable};
use crate::hittable_list::HittableList;
use crate::interval::Interval;
use crate::ray::Ray;

/// Maximum number of primitives in a leaf.
const MAX_LEAF: usize = 4;
/// Number of bins per axis used by the SAH sweep.
const BINS: usize = 16;
/// Traversal stack depth. SAH with the median fallback keeps the tree
/// balanced, so this is far more than any scene needs.
const STACK_SIZE: usize = 64;

/// One node of the flat tree. Internal nodes store the index of their right
/// child (the left child is always the next node); leaves store a range of
/// primitives.
struct Node {
    bbox: Aabb,
    /// Right child index for internal nodes, first primitive index for leaves.
    offset: u32,
    /// Primitive count for leaves; zero for internal nodes.
    count: u32,
}

/// A bounding volume hierarchy over a set of objects, stored as a flat array.
///
/// The tree is built with a binned surface-area heuristic and traversed
/// front to back, so nodes farther along the ray are skipped as soon as a
/// closer hit has been found.
pub struct BvhNode {
    nodes: Vec<Node>,
    prims: Vec<Arc<dyn Hittable>>,
}

impl BvhNode {
    pub fn new(list: HittableList) -> Self {
        let mut prims: Vec<Arc<dyn Hittable>> = list.into();
        assert!(!prims.is_empty(), "BvhNode requires at least one object");

        let mut nodes = Vec::with_capacity(2 * prims.len());
        build(&mut nodes, &mut prims, 0, 0);

        Self { nodes, prims }
    }
}

/// Recursively builds the subtree covering `prims[base..]`, returning its
/// node index. The left child always lands at `index + 1`.
fn build(
    nodes: &mut Vec<Node>,
    prims: &mut [Arc<dyn Hittable>],
    base: usize,
    depth: usize,
) -> u32 {
    let mut bbox = Aabb::empty();
    for prim in prims.iter() {
        bbox = bbox.union(prim.bounding_box());
    }

    let index = nodes.len() as u32;
    nodes.push(Node {
        bbox,
        offset: 0,
        count: 0,
    });

    let split = if prims.len() > MAX_LEAF && depth < STACK_SIZE {
        split_sah(prims, &bbox)
    } else {
        None
    };

    let Some(mid) = split else {
        nodes[index as usize] = Node {
            bbox,
            offset: base as u32,
            count: prims.len() as u32,
        };
        return index;
    };

    let (left, right) = prims.split_at_mut(mid);
    build(nodes, left, base, depth + 1);
    let right_index = build(nodes, right, base + mid, depth + 1);

    nodes[index as usize] = Node {
        bbox,
        offset: right_index,
        count: 0,
    };
    index
}

/// Chooses a split plane with the binned surface-area heuristic. Returns the
/// number of primitives that go to the left child, or `None` when the best
/// split is no cheaper than testing every primitive in a leaf.
fn split_sah(prims: &mut [Arc<dyn Hittable>], bbox: &Aabb) -> Option<usize> {
    let mut centroid_bbox = Aabb::empty();
    for prim in prims.iter() {
        let c = prim.bounding_box().centroid();
        centroid_bbox = centroid_bbox.union(Aabb::from_points(c, c));
    }

    let axis = centroid_bbox.longest_axis();
    let interval = centroid_bbox.axis_interval(axis);
    let extent = interval.size();
    if !extent.is_finite() || extent <= 0.0 {
        return median_split(prims, axis);
    }

    let mut bins = [(); BINS].map(|_| Bin {
        bbox: Aabb::empty(),
        count: 0,
    });
    for prim in prims.iter() {
        let c = prim.bounding_box().centroid()[axis];
        let bin = (((c - interval.min) / extent) * BINS as f64) as usize;
        let bin = bin.min(BINS - 1);
        bins[bin].count += 1;
        bins[bin].bbox = bins[bin].bbox.union(prim.bounding_box());
    }

    // Cost of splitting before bin `b`: prefix areas from the left, suffix
    // areas from the right, weighted by the primitive counts.
    let mut left_area = [0.0; BINS];
    let mut left_count = [0u32; BINS];
    let mut acc = Aabb::empty();
    let mut count = 0;
    for b in 0..BINS - 1 {
        acc = acc.union(bins[b].bbox);
        count += bins[b].count;
        left_area[b] = acc.surface_area();
        left_count[b] = count;
    }

    let mut best_cost = f64::INFINITY;
    let mut best_bin = 0;
    let mut acc = Aabb::empty();
    let mut count = 0;
    for b in (1..BINS).rev() {
        acc = acc.union(bins[b].bbox);
        count += bins[b].count;
        if left_count[b - 1] == 0 || count == 0 {
            continue;
        }
        let cost = left_area[b - 1] * left_count[b - 1] as f64 + acc.surface_area() * count as f64;
        if cost < best_cost {
            best_cost = cost;
            best_bin = b;
        }
    }

    if !best_cost.is_finite() || best_cost >= bbox.surface_area() * prims.len() as f64 {
        return None;
    }

    // Reorder by the split axis and cut at the first primitive in the right
    // half of the best bin.
    let min = interval.min;
    let mut keyed: Vec<(f64, Arc<dyn Hittable>)> = prims
        .iter()
        .map(|prim| (prim.bounding_box().centroid()[axis], prim.clone()))
        .collect();
    keyed.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut mid = 0;
    for (i, (key, _)) in keyed.iter().enumerate() {
        let bin = ((((key - min) / extent) * BINS as f64) as usize).min(BINS - 1);
        if bin >= best_bin {
            mid = i;
            break;
        }
        mid = i + 1;
    }

    if mid == 0 || mid == prims.len() {
        return median_split(prims, axis);
    }

    for (slot, (_, prim)) in prims.iter_mut().zip(keyed) {
        *slot = prim;
    }

    Some(mid)
}

/// Splits the primitives in half by their bounding-box minimum on `axis`.
fn median_split(prims: &mut [Arc<dyn Hittable>], axis: usize) -> Option<usize> {
    if prims.len() <= MAX_LEAF {
        return None;
    }

    prims.sort_by(|a, b| {
        let a = a.bounding_box().axis_interval(axis).min;
        let b = b.bounding_box().axis_interval(axis).min;
        a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal)
    });

    Some(prims.len() / 2)
}

struct Bin {
    bbox: Aabb,
    count: u32,
}

impl Hittable for BvhNode {
    fn hit<'a>(&'a self, r: &Ray, ray_t: Interval, rec: &mut HitRecord<'a>) -> bool {
        let mut stack = [0u32; STACK_SIZE];
        let mut stack_len = 0;
        let mut index = 0u32;
        let mut closest = ray_t.max;
        let mut hit_anything = false;

        loop {
            let node = &self.nodes[index as usize];
            let entered = node.bbox.intersect(r, ray_t.min, closest).is_some();

            if entered {
                if node.count > 0 {
                    let first = node.offset as usize;
                    for prim in &self.prims[first..first + node.count as usize] {
                        if prim.hit(r, Interval::new(ray_t.min, closest), rec) {
                            hit_anything = true;
                            closest = rec.t;
                        }
                    }
                } else {
                    let left = index + 1;
                    let right = node.offset;
                    let left_t = self.nodes[left as usize].bbox.intersect(r, ray_t.min, closest);
                    let right_t = self.nodes[right as usize].bbox.intersect(r, ray_t.min, closest);

                    let nearer = match (left_t, right_t) {
                        (Some(a), Some(b)) => {
                            if a <= b { Some((left, right)) } else { Some((right, left)) }
                        }
                        (Some(_), None) => Some((left, 0)),
                        (None, Some(_)) => Some((right, 0)),
                        (None, None) => None,
                    };

                    if let Some((near, far)) = nearer {
                        if far != 0 {
                            stack[stack_len] = far;
                            stack_len += 1;
                        }
                        index = near;
                        continue;
                    }
                }
            }

            if stack_len == 0 {
                return hit_anything;
            }
            stack_len -= 1;
            index = stack[stack_len];
        }
    }

    fn bounding_box(&self) -> Aabb {
        self.nodes[0].bbox
    }
}
