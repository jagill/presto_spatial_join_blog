use crate::models;
use crate::models::Polygon;
use rstar::{Point, RTreeObject};
pub use rstar::{RTree, AABB};

impl Point for models::Point {
    type Scalar = f64;
    const DIMENSIONS: usize = 2;

    fn generate(generator: impl Fn(usize) -> Self::Scalar) -> Self {
        models::Point {
            x: generator(0),
            y: generator(1),
        }
    }

    fn nth(&self, index: usize) -> Self::Scalar {
        match index {
            0 => self.x,
            1 => self.y,
            _ => unreachable!(),
        }
    }

    fn nth_mut(&mut self, index: usize) -> &mut Self::Scalar {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            _ => unreachable!(),
        }
    }
}

impl RTreeObject for Polygon {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let env = self.envelope;
        AABB::from_corners([env.min_x, env.min_y], [env.max_x, env.max_y])
    }
}

pub fn build_rtree(polygons: &[Polygon]) -> RTree<Polygon> {
    RTree::bulk_load(polygons.to_vec())
}
