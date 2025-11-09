//! Orthogonal (rectilinear) connector routing
//!
//! This module implements algorithms for routing connectors using only
//! horizontal and vertical line segments.

use crate::geometry::{Point, Polygon, PolygonInterface};
use crate::obstacle::Obstacle;

/// Direction for orthogonal routing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

impl Direction {
    /// Returns the opposite direction
    pub fn opposite(&self) -> Direction {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::East => Direction::West,
            Direction::West => Direction::East,
        }
    }

    /// Returns whether this direction is horizontal
    pub fn is_horizontal(&self) -> bool {
        matches!(self, Direction::East | Direction::West)
    }

    /// Returns whether this direction is vertical
    pub fn is_vertical(&self) -> bool {
        matches!(self, Direction::North | Direction::South)
    }
}

/// Orthogonal routing context
pub struct OrthogonalRouter {
    /// Routing penalty for bends
    bend_penalty: f64,
    /// Routing penalty for segment length
    segment_penalty: f64,
}

impl OrthogonalRouter {
    /// Creates a new orthogonal router
    pub fn new() -> Self {
        OrthogonalRouter {
            bend_penalty: 50.0,
            segment_penalty: 1.0,
        }
    }

    /// Creates an orthogonal router with custom penalties
    pub fn with_penalties(bend_penalty: f64, segment_penalty: f64) -> Self {
        OrthogonalRouter {
            bend_penalty,
            segment_penalty,
        }
    }

    /// Routes an orthogonal path between two points
    pub fn route_orthogonal(
        &self,
        start: Point,
        end: Point,
        obstacles: &[&dyn Obstacle],
    ) -> Polygon {
        // Simple orthogonal routing: try horizontal-then-vertical and vertical-then-horizontal
        let path1 = self.route_h_v(start, end, obstacles);
        let path2 = self.route_v_h(start, end, obstacles);

        // Return the path with lower cost
        let cost1 = self.compute_path_cost(&path1);
        let cost2 = self.compute_path_cost(&path2);

        if cost1 <= cost2 {
            path1
        } else {
            path2
        }
    }

    /// Routes horizontal-then-vertical
    fn route_h_v(&self, start: Point, end: Point, obstacles: &[&dyn Obstacle]) -> Polygon {
        let mut poly = Polygon::with_capacity(3);
        poly.push(start);

        let mid = Point::new(end.x, start.y);

        // Check if direct path is clear
        if self.is_path_clear(&start, &mid, obstacles) && self.is_path_clear(&mid, &end, obstacles)
        {
            if (start.x - end.x).abs() > 1e-6 {
                poly.push(mid);
            }
            poly.push(end);
        } else {
            // Simple fallback: add offset
            let offset = 20.0;
            let mid1 = Point::new(start.x + offset, start.y);
            let mid2 = Point::new(start.x + offset, end.y);
            let mid3 = Point::new(end.x, end.y);

            poly.push(mid1);
            poly.push(mid2);
            poly.push(mid3);
            poly.push(end);
        }

        poly
    }

    /// Routes vertical-then-horizontal
    fn route_v_h(&self, start: Point, end: Point, obstacles: &[&dyn Obstacle]) -> Polygon {
        let mut poly = Polygon::with_capacity(3);
        poly.push(start);

        let mid = Point::new(start.x, end.y);

        // Check if direct path is clear
        if self.is_path_clear(&start, &mid, obstacles) && self.is_path_clear(&mid, &end, obstacles)
        {
            if (start.y - end.y).abs() > 1e-6 {
                poly.push(mid);
            }
            poly.push(end);
        } else {
            // Simple fallback: add offset
            let offset = 20.0;
            let mid1 = Point::new(start.x, start.y + offset);
            let mid2 = Point::new(end.x, start.y + offset);
            let mid3 = Point::new(end.x, end.y);

            poly.push(mid1);
            poly.push(mid2);
            poly.push(mid3);
            poly.push(end);
        }

        poly
    }

    /// Checks if a path between two points is clear of obstacles
    fn is_path_clear(&self, from: &Point, to: &Point, obstacles: &[&dyn Obstacle]) -> bool {
        for obstacle in obstacles {
            if !obstacle.is_active() {
                continue;
            }

            // Check if the line segment intersects the obstacle's bounding box
            let bbox = obstacle.polygon().bounding_rect();

            if self.segment_intersects_box(from, to, &bbox) {
                return false;
            }
        }

        true
    }

    /// Checks if a line segment intersects a bounding box
    fn segment_intersects_box(
        &self,
        from: &Point,
        to: &Point,
        bbox: &crate::geometry::Box,
    ) -> bool {
        // Check if segment is horizontal
        if (from.y - to.y).abs() < 1e-6 {
            let y = from.y;
            let x_min = from.x.min(to.x);
            let x_max = from.x.max(to.x);

            return y >= bbox.min.y && y <= bbox.max.y && x_max >= bbox.min.x && x_min <= bbox.max.x;
        }

        // Check if segment is vertical
        if (from.x - to.x).abs() < 1e-6 {
            let x = from.x;
            let y_min = from.y.min(to.y);
            let y_max = from.y.max(to.y);

            return x >= bbox.min.x && x <= bbox.max.x && y_max >= bbox.min.y && y_min <= bbox.max.y;
        }

        false
    }

    /// Computes the cost of a path
    fn compute_path_cost(&self, path: &Polygon) -> f64 {
        if path.size() < 2 {
            return 0.0;
        }

        let mut cost = 0.0;
        let mut prev_dir: Option<Direction> = None;

        for i in 0..path.size() - 1 {
            let p1 = path.at(i);
            let p2 = path.at(i + 1);

            // Add segment length cost
            let dist = p1.distance(p2);
            cost += dist * self.segment_penalty;

            // Determine direction
            let dir = if (p1.x - p2.x).abs() < 1e-6 {
                if p2.y > p1.y {
                    Direction::North
                } else {
                    Direction::South
                }
            } else if p2.x > p1.x {
                Direction::East
            } else {
                Direction::West
            };

            // Add bend cost
            if let Some(prev) = prev_dir {
                if prev != dir {
                    cost += self.bend_penalty;
                }
            }

            prev_dir = Some(dir);
        }

        cost
    }

    /// Simplifies an orthogonal path by removing redundant points
    pub fn simplify_orthogonal_path(&self, path: &mut Polygon) {
        if path.size() < 3 {
            return;
        }

        let mut simplified = vec![*path.at(0)];

        for i in 1..path.size() - 1 {
            let prev = simplified.last().unwrap();
            let curr = path.at(i);
            let next = path.at(i + 1);

            // Check if current point is on the line between prev and next
            let on_horizontal = (prev.y - curr.y).abs() < 1e-6 && (curr.y - next.y).abs() < 1e-6;
            let on_vertical = (prev.x - curr.x).abs() < 1e-6 && (curr.x - next.x).abs() < 1e-6;

            if !on_horizontal && !on_vertical {
                simplified.push(*curr);
            }
        }

        simplified.push(*path.at(path.size() - 1));

        path.ps = simplified;
    }
}

impl Default for OrthogonalRouter {
    fn default() -> Self {
        OrthogonalRouter::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction() {
        assert_eq!(Direction::North.opposite(), Direction::South);
        assert_eq!(Direction::East.opposite(), Direction::West);
        assert!(Direction::East.is_horizontal());
        assert!(Direction::North.is_vertical());
    }

    #[test]
    fn test_orthogonal_routing() {
        let router = OrthogonalRouter::new();
        let start = Point::new(0.0, 0.0);
        let end = Point::new(10.0, 10.0);

        let path = router.route_orthogonal(start, end, &[]);

        assert!(path.size() >= 2);
        assert_eq!(*path.at(0), start);
        assert_eq!(*path.at(path.size() - 1), end);
    }

    #[test]
    fn test_path_simplification() {
        let router = OrthogonalRouter::new();
        let mut path = Polygon::new();

        // Path with redundant point
        path.push(Point::new(0.0, 0.0));
        path.push(Point::new(5.0, 0.0));
        path.push(Point::new(10.0, 0.0));
        path.push(Point::new(10.0, 10.0));

        router.simplify_orthogonal_path(&mut path);

        // Should remove the middle point on the horizontal segment
        assert_eq!(path.size(), 3);
        assert_eq!(*path.at(0), Point::new(0.0, 0.0));
        assert_eq!(*path.at(1), Point::new(10.0, 0.0));
        assert_eq!(*path.at(2), Point::new(10.0, 10.0));
    }
}
