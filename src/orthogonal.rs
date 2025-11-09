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
    /// Nudging distance for separating overlapping segments
    nudge_distance: f64,
}

impl OrthogonalRouter {
    /// Creates a new orthogonal router
    pub fn new() -> Self {
        OrthogonalRouter {
            bend_penalty: 50.0,
            segment_penalty: 1.0,
            nudge_distance: 4.0,
        }
    }

    /// Creates an orthogonal router with custom penalties
    pub fn with_penalties(bend_penalty: f64, segment_penalty: f64) -> Self {
        OrthogonalRouter {
            bend_penalty,
            segment_penalty,
            nudge_distance: 4.0,
        }
    }

    /// Sets the nudging distance
    pub fn set_nudge_distance(&mut self, distance: f64) {
        self.nudge_distance = distance;
    }

    /// Nudges a route to avoid overlapping with existing routes
    pub fn nudge_route(&self, route: &mut Polygon, existing_routes: &[&Polygon]) {
        if route.size() < 2 {
            return;
        }

        // For each segment in the route
        for i in 0..route.size() - 1 {
            let p1 = *route.at(i);
            let p2 = *route.at(i + 1);

            // Check if this segment is orthogonal
            let is_horizontal = (p1.y - p2.y).abs() < 1e-6;
            let is_vertical = (p1.x - p2.x).abs() < 1e-6;

            if !is_horizontal && !is_vertical {
                continue;
            }

            // Check for overlap with existing routes
            for existing in existing_routes {
                if self.segments_overlap(&p1, &p2, existing) {
                    // Nudge perpendicular to segment direction
                    let nudge = if is_horizontal {
                        Point::new(0.0, self.nudge_distance)
                    } else {
                        Point::new(self.nudge_distance, 0.0)
                    };

                    // Apply nudge to this segment
                    route.set_point(i, p1 + nudge);
                    route.set_point(i + 1, p2 + nudge);
                    break;
                }
            }
        }
    }

    /// Checks if a segment overlaps with any segment in an existing route
    fn segments_overlap(&self, p1: &Point, p2: &Point, existing: &Polygon) -> bool {
        if existing.size() < 2 {
            return false;
        }

        let is_horizontal = (p1.y - p2.y).abs() < 1e-6;

        for i in 0..existing.size() - 1 {
            let e1 = existing.at(i);
            let e2 = existing.at(i + 1);

            let e_is_horizontal = (e1.y - e2.y).abs() < 1e-6;

            // Only check segments with same orientation
            if is_horizontal != e_is_horizontal {
                continue;
            }

            if is_horizontal {
                // Check horizontal overlap
                if (p1.y - e1.y).abs() < self.nudge_distance {
                    let min_x1 = p1.x.min(p2.x);
                    let max_x1 = p1.x.max(p2.x);
                    let min_x2 = e1.x.min(e2.x);
                    let max_x2 = e1.x.max(e2.x);

                    if min_x1 <= max_x2 && max_x1 >= min_x2 {
                        return true;
                    }
                }
            } else {
                // Check vertical overlap
                if (p1.x - e1.x).abs() < self.nudge_distance {
                    let min_y1 = p1.y.min(p2.y);
                    let max_y1 = p1.y.max(p2.y);
                    let min_y2 = e1.y.min(e2.y);
                    let max_y2 = e1.y.max(e2.y);

                    if min_y1 <= max_y2 && max_y1 >= min_y2 {
                        return true;
                    }
                }
            }
        }

        false
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
            // Find clear channel using binary search
            let offset = self.find_clear_channel(start, end, obstacles, true);
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
            // Find clear channel using binary search
            let offset = self.find_clear_channel(start, end, obstacles, false);
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

    /// Find a clear channel by scanning for gaps between obstacles
    fn find_clear_channel(&self, start: Point, end: Point, obstacles: &[&dyn Obstacle], horizontal: bool) -> f64 {
        // Scan obstacles to find channels
        let mut channels = Vec::new();

        if horizontal {
            // Find vertical channels (gaps along X axis)
            let min_x = start.x.min(end.x);
            let max_x = start.x.max(end.x);

            // Collect obstacle boundaries in X range
            let mut boundaries = vec![min_x, max_x];
            for obstacle in obstacles {
                let bbox = obstacle.polygon().bounding_rect();
                if bbox.max.x >= min_x && bbox.min.x <= max_x {
                    boundaries.push(bbox.min.x);
                    boundaries.push(bbox.max.x);
                }
            }
            boundaries.sort_by(|a, b| a.partial_cmp(b).unwrap());

            // Find gaps between boundaries
            for i in 0..boundaries.len() - 1 {
                let gap_center = (boundaries[i] + boundaries[i + 1]) / 2.0;
                let gap_x = gap_center - start.x;

                // Check if this channel is clear
                let test_start = Point::new(gap_center, start.y);
                let test_end = Point::new(gap_center, end.y);
                if self.is_path_clear(&test_start, &test_end, obstacles) {
                    channels.push(gap_x);
                }
            }
        } else {
            // Find horizontal channels (gaps along Y axis)
            let min_y = start.y.min(end.y);
            let max_y = start.y.max(end.y);

            let mut boundaries = vec![min_y, max_y];
            for obstacle in obstacles {
                let bbox = obstacle.polygon().bounding_rect();
                if bbox.max.y >= min_y && bbox.min.y <= max_y {
                    boundaries.push(bbox.min.y);
                    boundaries.push(bbox.max.y);
                }
            }
            boundaries.sort_by(|a, b| a.partial_cmp(b).unwrap());

            for i in 0..boundaries.len() - 1 {
                let gap_center = (boundaries[i] + boundaries[i + 1]) / 2.0;
                let gap_y = gap_center - start.y;

                let test_start = Point::new(start.x, gap_center);
                let test_end = Point::new(end.x, gap_center);
                if self.is_path_clear(&test_start, &test_end, obstacles) {
                    channels.push(gap_y);
                }
            }
        }

        // Return the first clear channel, or small offset if none found
        channels.into_iter().next().unwrap_or(20.0)
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
