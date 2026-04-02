use crate::{math::vec3::Vec3, sort::quick_sort::quick_sort};

pub fn simple_polygon_line(points: &mut Vec<Vec3>) {
  quick_sort(points);
}
