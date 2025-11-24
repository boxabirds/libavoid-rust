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

    /// Converts to ConnDirFlags bit
    pub fn to_conn_dir_flag(&self) -> u32 {
        use crate::connector::{CONN_DIR_UP, CONN_DIR_DOWN, CONN_DIR_LEFT, CONN_DIR_RIGHT};
        match self {
            Direction::North => CONN_DIR_UP,
            Direction::South => CONN_DIR_DOWN,
            Direction::East => CONN_DIR_RIGHT,
            Direction::West => CONN_DIR_LEFT,
        }
    }

    /// Returns all directions allowed by the given ConnDirFlags
    pub fn from_conn_dir_flags(flags: u32) -> Vec<Direction> {
        use crate::connector::{CONN_DIR_UP, CONN_DIR_DOWN, CONN_DIR_LEFT, CONN_DIR_RIGHT};
        let mut dirs = Vec::new();
        if flags & CONN_DIR_UP != 0 {
            dirs.push(Direction::North);
        }
        if flags & CONN_DIR_DOWN != 0 {
            dirs.push(Direction::South);
        }
        if flags & CONN_DIR_LEFT != 0 {
            dirs.push(Direction::West);
        }
        if flags & CONN_DIR_RIGHT != 0 {
            dirs.push(Direction::East);
        }
        dirs
    }

    /// Checks if this direction is allowed by the given ConnDirFlags
    pub fn is_allowed_by(&self, flags: u32) -> bool {
        flags & self.to_conn_dir_flag() != 0
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
    /// C++ ref: libavoid/router.cpp - bendPenalty default = 0.0
    pub fn new() -> Self {
        OrthogonalRouter {
            bend_penalty: 0.0,  // Match C++ default
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
    ///
    /// This is used as a simple L-shaped routing when visibility graph
    /// is not available. It tries horizontal-then-vertical and
    /// vertical-then-horizontal paths and returns the better one.
    pub fn route_orthogonal(
        &self,
        start: Point,
        end: Point,
        obstacles: &[&dyn Obstacle],
    ) -> Polygon {
        // Try simple L-shaped paths
        let path1 = self.route_h_v_simple(start, end, obstacles);
        let path2 = self.route_v_h_simple(start, end, obstacles);

        match (path1, path2) {
            (Some(p1), Some(p2)) => {
                // Both valid - return the one with lower cost
                let cost1 = self.compute_path_cost(&p1);
                let cost2 = self.compute_path_cost(&p2);
                if cost1 <= cost2 { p1 } else { p2 }
            }
            (Some(p1), None) => p1,
            (None, Some(p2)) => p2,
            (None, None) => {
                // Both simple paths blocked - return simple L route
                // (The visibility graph should handle complex routing)
                let mut poly = Polygon::with_capacity(3);
                poly.push(start);
                if (start.x - end.x).abs() > 1e-6 {
                    poly.push(Point::new(end.x, start.y));
                }
                poly.push(end);
                poly
            }
        }
    }

    /// Try simple horizontal-then-vertical route, returns None if blocked
    fn route_h_v_simple(&self, start: Point, end: Point, obstacles: &[&dyn Obstacle]) -> Option<Polygon> {
        let mid = Point::new(end.x, start.y);

        // Check if both segments are clear
        if !self.is_path_clear(&start, &mid, obstacles) {
            return None;
        }
        if !self.is_path_clear(&mid, &end, obstacles) {
            return None;
        }

        let mut poly = Polygon::with_capacity(3);
        poly.push(start);
        if (start.x - end.x).abs() > 1e-6 {
            poly.push(mid);
        }
        poly.push(end);
        Some(poly)
    }

    /// Try simple vertical-then-horizontal route, returns None if blocked
    fn route_v_h_simple(&self, start: Point, end: Point, obstacles: &[&dyn Obstacle]) -> Option<Polygon> {
        let mid = Point::new(start.x, end.y);

        // Check if both segments are clear
        if !self.is_path_clear(&start, &mid, obstacles) {
            return None;
        }
        if !self.is_path_clear(&mid, &end, obstacles) {
            return None;
        }

        let mut poly = Polygon::with_capacity(3);
        poly.push(start);
        if (start.y - end.y).abs() > 1e-6 {
            poly.push(mid);
        }
        poly.push(end);
        Some(poly)
    }

    /// Routes horizontal-then-vertical
    #[allow(dead_code)] // Alternative routing strategy
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
    #[allow(dead_code)] // Alternative routing strategy
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

// NOTE: OrthogonalAStarRouter has been removed.
// Orthogonal routing now uses the visibility graph approach in orthogonal_visgraph.rs

// ============================================================================
// Route Nudging (Task 25)
// ============================================================================

/// Result of nudging operation
#[derive(Debug)]
pub struct NudgeResult {
    /// The nudged route
    pub route: Polygon,
    /// Amount of nudging applied
    pub nudge_amount: f64,
}

/// Nudges overlapping orthogonal routes apart
pub fn nudge_routes(routes: &mut [Polygon], nudge_distance: f64) {
    if routes.len() < 2 {
        return;
    }

    // Find overlapping horizontal segments
    let mut h_segments: Vec<(usize, usize, f64, f64, f64)> = Vec::new(); // (route_idx, seg_idx, y, x_min, x_max)

    for (route_idx, route) in routes.iter().enumerate() {
        for seg_idx in 0..route.size().saturating_sub(1) {
            let p1 = route.at(seg_idx);
            let p2 = route.at(seg_idx + 1);

            // Horizontal segment
            if (p1.y - p2.y).abs() < 0.001 {
                let x_min = p1.x.min(p2.x);
                let x_max = p1.x.max(p2.x);
                h_segments.push((route_idx, seg_idx, p1.y, x_min, x_max));
            }
        }
    }

    // Group overlapping horizontal segments
    for i in 0..h_segments.len() {
        for j in (i + 1)..h_segments.len() {
            let (ri, si, yi, xi_min, xi_max) = h_segments[i];
            let (rj, sj, yj, xj_min, xj_max) = h_segments[j];

            // Check if same y and overlapping x range
            if (yi - yj).abs() < 0.001 && xi_max > xj_min && xj_max > xi_min && ri != rj {
                // Nudge one route up, one down
                let offset = nudge_distance / 2.0;

                if let Some(p) = routes[ri].ps.get_mut(si) {
                    p.y -= offset;
                }
                if si + 1 < routes[ri].size() {
                    if let Some(p) = routes[ri].ps.get_mut(si + 1) {
                        p.y -= offset;
                    }
                }

                if let Some(p) = routes[rj].ps.get_mut(sj) {
                    p.y += offset;
                }
                if sj + 1 < routes[rj].size() {
                    if let Some(p) = routes[rj].ps.get_mut(sj + 1) {
                        p.y += offset;
                    }
                }
            }
        }
    }

    // Similar for vertical segments
    let mut v_segments: Vec<(usize, usize, f64, f64, f64)> = Vec::new();

    for (route_idx, route) in routes.iter().enumerate() {
        for seg_idx in 0..route.size().saturating_sub(1) {
            let p1 = route.at(seg_idx);
            let p2 = route.at(seg_idx + 1);

            // Vertical segment
            if (p1.x - p2.x).abs() < 0.001 {
                let y_min = p1.y.min(p2.y);
                let y_max = p1.y.max(p2.y);
                v_segments.push((route_idx, seg_idx, p1.x, y_min, y_max));
            }
        }
    }

    for i in 0..v_segments.len() {
        for j in (i + 1)..v_segments.len() {
            let (ri, si, xi, yi_min, yi_max) = v_segments[i];
            let (rj, sj, xj, yj_min, yj_max) = v_segments[j];

            if (xi - xj).abs() < 0.001 && yi_max > yj_min && yj_max > yi_min && ri != rj {
                let offset = nudge_distance / 2.0;

                if let Some(p) = routes[ri].ps.get_mut(si) {
                    p.x -= offset;
                }
                if si + 1 < routes[ri].size() {
                    if let Some(p) = routes[ri].ps.get_mut(si + 1) {
                        p.x -= offset;
                    }
                }

                if let Some(p) = routes[rj].ps.get_mut(sj) {
                    p.x += offset;
                }
                if sj + 1 < routes[rj].size() {
                    if let Some(p) = routes[rj].ps.get_mut(sj + 1) {
                        p.x += offset;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connector::{CONN_DIR_UP, CONN_DIR_DOWN, CONN_DIR_LEFT, CONN_DIR_RIGHT, CONN_DIR_ALL};

    #[test]
    fn test_direction() {
        assert_eq!(Direction::North.opposite(), Direction::South);
        assert_eq!(Direction::East.opposite(), Direction::West);
        assert!(Direction::East.is_horizontal());
        assert!(Direction::North.is_vertical());
    }

    #[test]
    fn test_direction_to_conn_dir_flag() {
        assert_eq!(Direction::North.to_conn_dir_flag(), CONN_DIR_UP);
        assert_eq!(Direction::South.to_conn_dir_flag(), CONN_DIR_DOWN);
        assert_eq!(Direction::East.to_conn_dir_flag(), CONN_DIR_RIGHT);
        assert_eq!(Direction::West.to_conn_dir_flag(), CONN_DIR_LEFT);
    }

    #[test]
    fn test_direction_from_conn_dir_flags() {
        let dirs = Direction::from_conn_dir_flags(CONN_DIR_UP | CONN_DIR_RIGHT);
        assert_eq!(dirs.len(), 2);
        assert!(dirs.contains(&Direction::North));
        assert!(dirs.contains(&Direction::East));

        let all_dirs = Direction::from_conn_dir_flags(CONN_DIR_ALL);
        assert_eq!(all_dirs.len(), 4);
    }

    #[test]
    fn test_direction_is_allowed_by() {
        assert!(Direction::North.is_allowed_by(CONN_DIR_UP));
        assert!(!Direction::North.is_allowed_by(CONN_DIR_DOWN));
        assert!(Direction::East.is_allowed_by(CONN_DIR_ALL));
    }

    // NOTE: test_astar_with_direction_constraints removed with OrthogonalAStarRouter

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
