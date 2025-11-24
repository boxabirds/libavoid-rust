//! Channel-based orthogonal routing
//!
//! This module implements VPSC-based nudging of orthogonal routes.
//! Routes are divided into segments, which are then positioned using
//! Variable Placement with Separation Constraints.
//!
//! Ported from libavoid orthogonal.cpp

use crate::geometry::{Polygon, PolygonInterface, Point};
use crate::vpsc::IncSolver;

// ============================================================================
// Constants
// ============================================================================

/// Weight for free (shiftable) segments
const FREE_WEIGHT: f64 = 0.00001;
/// Weight for fixed segments
const FIXED_WEIGHT: f64 = 100000.0;

/// Minimum gap between parallel segments
const DEFAULT_NUDGE_DISTANCE: f64 = 4.0;

// ============================================================================
// Shift Segment
// ============================================================================

/// A segment of a route that can be shifted perpendicular to its direction
#[derive(Debug, Clone)]
pub struct ShiftSegment {
    /// Route index
    pub route_idx: usize,
    /// Start point index in route
    pub low_idx: usize,
    /// End point index in route
    pub high_idx: usize,
    /// Dimension: 0 = horizontal (shifts in Y), 1 = vertical (shifts in X)
    pub dimension: usize,
    /// Minimum allowed position
    pub min_limit: f64,
    /// Maximum allowed position
    pub max_limit: f64,
    /// Whether this segment is fixed
    pub fixed: bool,
    /// Variable index in VPSC solver (set during solve)
    pub variable_idx: Option<usize>,
    /// Current position
    pub position: f64,
}

impl ShiftSegment {
    /// Create a new shiftable segment
    pub fn new(
        route_idx: usize,
        low_idx: usize,
        high_idx: usize,
        dimension: usize,
        position: f64,
        min_limit: f64,
        max_limit: f64,
    ) -> Self {
        ShiftSegment {
            route_idx,
            low_idx,
            high_idx,
            dimension,
            min_limit,
            max_limit,
            fixed: false,
            variable_idx: None,
            position,
        }
    }

    /// Create a fixed segment
    pub fn fixed(
        route_idx: usize,
        low_idx: usize,
        high_idx: usize,
        dimension: usize,
        position: f64,
    ) -> Self {
        ShiftSegment {
            route_idx,
            low_idx,
            high_idx,
            dimension,
            min_limit: position,
            max_limit: position,
            fixed: true,
            variable_idx: None,
            position,
        }
    }

    /// Get the perpendicular coordinate (the one that can shift)
    pub fn perp_coord(&self, routes: &[Polygon]) -> f64 {
        let route = &routes[self.route_idx];
        let point = route.at(self.low_idx);
        if self.dimension == 0 {
            point.y
        } else {
            point.x
        }
    }

    /// Get the range of the segment in the parallel dimension
    pub fn range(&self, routes: &[Polygon]) -> (f64, f64) {
        let route = &routes[self.route_idx];
        let low = route.at(self.low_idx);
        let high = route.at(self.high_idx);

        if self.dimension == 0 {
            // Horizontal segment
            (low.x.min(high.x), low.x.max(high.x))
        } else {
            // Vertical segment
            (low.y.min(high.y), low.y.max(high.y))
        }
    }

    /// Check if this segment overlaps with another in the parallel dimension
    pub fn overlaps(&self, other: &ShiftSegment, routes: &[Polygon]) -> bool {
        if self.dimension != other.dimension {
            return false;
        }

        let (self_min, self_max) = self.range(routes);
        let (other_min, other_max) = other.range(routes);

        // Check for overlap
        self_max > other_min && other_max > self_min
    }
}

// ============================================================================
// Channel Router
// ============================================================================

/// VPSC-based channel router for orthogonal routes
pub struct ChannelRouter {
    /// Nudge distance (minimum separation)
    pub nudge_distance: f64,
}

impl ChannelRouter {
    pub fn new() -> Self {
        ChannelRouter {
            nudge_distance: DEFAULT_NUDGE_DISTANCE,
        }
    }

    pub fn with_nudge_distance(nudge_distance: f64) -> Self {
        ChannelRouter { nudge_distance }
    }

    /// Nudge routes to spread overlapping segments
    pub fn nudge_routes(&self, routes: &mut [Polygon]) {
        self.nudge_routes_with_obstacles(routes, &[]);
    }

    /// Nudge routes to spread overlapping segments, respecting obstacles
    pub fn nudge_routes_with_obstacles(&self, routes: &mut [Polygon], obstacles: &[Polygon]) {
        // Process horizontal segments (shift in Y)
        self.nudge_dimension_with_obstacles(routes, 0, obstacles);
        // Process vertical segments (shift in X)
        self.nudge_dimension_with_obstacles(routes, 1, obstacles);
    }

    /// Nudge segments in one dimension, respecting obstacles
    fn nudge_dimension_with_obstacles(&self, routes: &mut [Polygon], dimension: usize, obstacles: &[Polygon]) {
        // Build shift segments with obstacle-aware limits
        let mut segments = self.build_shift_segments_with_obstacles(routes, dimension, obstacles);

        if segments.is_empty() {
            return;
        }

        // Build constraints and solve
        let (solver, seg_var_map) = self.build_vpsc_problem(&segments, routes);

        // Apply solved positions
        self.apply_positions(&mut segments, &solver, &seg_var_map);

        // Update routes with new positions
        self.update_routes(routes, &segments, dimension);
    }

    /// Build shift segments from routes
    fn build_shift_segments(&self, routes: &[Polygon], dimension: usize) -> Vec<ShiftSegment> {
        let mut segments = Vec::new();

        for (route_idx, route) in routes.iter().enumerate() {
            if route.size() < 2 {
                continue;
            }

            for i in 0..route.size() - 1 {
                let p1 = route.at(i);
                let p2 = route.at(i + 1);

                let is_horizontal = (p1.y - p2.y).abs() < 1e-6;
                let is_vertical = (p1.x - p2.x).abs() < 1e-6;

                // Dimension 0: horizontal segments (can shift in Y)
                // Dimension 1: vertical segments (can shift in X)
                if (dimension == 0 && is_horizontal) || (dimension == 1 && is_vertical) {
                    let position = if dimension == 0 { p1.y } else { p1.x };

                    // Compute limits based on neighboring segments
                    let (min_limit, max_limit) = self.compute_limits(route, i, dimension);

                    let segment = if min_limit >= max_limit {
                        ShiftSegment::fixed(route_idx, i, i + 1, dimension, position)
                    } else {
                        ShiftSegment::new(
                            route_idx, i, i + 1, dimension, position, min_limit, max_limit,
                        )
                    };

                    segments.push(segment);
                }
            }
        }

        segments
    }

    /// Compute movement limits for a segment
    fn compute_limits(&self, route: &Polygon, seg_idx: usize, dimension: usize) -> (f64, f64) {
        let p1 = route.at(seg_idx);
        let p2 = route.at(seg_idx + 1);

        // Get perpendicular coordinate
        let perp = if dimension == 0 { p1.y } else { p1.x };

        // By default, allow significant movement
        let default_range = 500.0;
        let mut min_limit = perp - default_range;
        let mut max_limit = perp + default_range;

        // Constrain by endpoints if at start/end of route
        if seg_idx == 0 {
            // First segment - constrain to not pass start point
            if dimension == 0 {
                min_limit = min_limit.max(p1.y - self.nudge_distance);
                max_limit = max_limit.min(p1.y + self.nudge_distance);
            } else {
                min_limit = min_limit.max(p1.x - self.nudge_distance);
                max_limit = max_limit.min(p1.x + self.nudge_distance);
            }
        }

        if seg_idx + 2 >= route.size() {
            // Last segment - constrain to not pass end point
            if dimension == 0 {
                min_limit = min_limit.max(p2.y - self.nudge_distance);
                max_limit = max_limit.min(p2.y + self.nudge_distance);
            } else {
                min_limit = min_limit.max(p2.x - self.nudge_distance);
                max_limit = max_limit.min(p2.x + self.nudge_distance);
            }
        }

        (min_limit, max_limit)
    }

    /// Build shift segments with obstacle awareness
    fn build_shift_segments_with_obstacles(
        &self,
        routes: &[Polygon],
        dimension: usize,
        obstacles: &[Polygon],
    ) -> Vec<ShiftSegment> {
        let mut segments = Vec::new();

        // Precompute obstacle bounding boxes
        let obstacle_bounds: Vec<(f64, f64, f64, f64)> = obstacles
            .iter()
            .map(|obs| Self::polygon_bounds(obs))
            .collect();

        for (route_idx, route) in routes.iter().enumerate() {
            if route.size() < 2 {
                continue;
            }

            for i in 0..route.size() - 1 {
                let p1 = route.at(i);
                let p2 = route.at(i + 1);

                let is_horizontal = (p1.y - p2.y).abs() < 1e-6;
                let is_vertical = (p1.x - p2.x).abs() < 1e-6;

                // Dimension 0: horizontal segments (can shift in Y)
                // Dimension 1: vertical segments (can shift in X)
                if (dimension == 0 && is_horizontal) || (dimension == 1 && is_vertical) {
                    let position = if dimension == 0 { p1.y } else { p1.x };

                    // Compute base limits from route structure
                    let (mut min_limit, mut max_limit) = self.compute_limits(route, i, dimension);

                    // Constrain by obstacles
                    let seg_min_par = if dimension == 0 { p1.x.min(p2.x) } else { p1.y.min(p2.y) };
                    let seg_max_par = if dimension == 0 { p1.x.max(p2.x) } else { p1.y.max(p2.y) };

                    for &(obs_min_x, obs_min_y, obs_max_x, obs_max_y) in &obstacle_bounds {
                        // Check if segment overlaps obstacle in parallel dimension
                        let (obs_min_par, obs_max_par, obs_min_perp, obs_max_perp) = if dimension == 0 {
                            (obs_min_x, obs_max_x, obs_min_y, obs_max_y)
                        } else {
                            (obs_min_y, obs_max_y, obs_min_x, obs_max_x)
                        };

                        // If segment's parallel range overlaps obstacle's parallel range
                        if seg_max_par > obs_min_par && seg_min_par < obs_max_par {
                            // Obstacle constrains movement in perpendicular direction
                            if position < obs_min_perp {
                                // Segment is below/left of obstacle - can't move past obstacle's min
                                max_limit = max_limit.min(obs_min_perp - self.nudge_distance);
                            } else if position > obs_max_perp {
                                // Segment is above/right of obstacle - can't move past obstacle's max
                                min_limit = min_limit.max(obs_max_perp + self.nudge_distance);
                            }
                            // If segment is inside obstacle... it shouldn't move at all
                            // (but this case shouldn't happen with proper routing)
                        }
                    }

                    let segment = if min_limit >= max_limit {
                        ShiftSegment::fixed(route_idx, i, i + 1, dimension, position)
                    } else {
                        ShiftSegment::new(
                            route_idx, i, i + 1, dimension, position, min_limit, max_limit,
                        )
                    };

                    segments.push(segment);
                }
            }
        }

        segments
    }

    /// Compute bounding box of a polygon
    fn polygon_bounds(poly: &Polygon) -> (f64, f64, f64, f64) {
        if poly.size() == 0 {
            return (0.0, 0.0, 0.0, 0.0);
        }

        let first = poly.at(0);
        let mut min_x = first.x;
        let mut min_y = first.y;
        let mut max_x = first.x;
        let mut max_y = first.y;

        for i in 1..poly.size() {
            let p = poly.at(i);
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        (min_x, min_y, max_x, max_y)
    }

    /// Build VPSC problem from segments
    fn build_vpsc_problem(
        &self,
        segments: &[ShiftSegment],
        routes: &[Polygon],
    ) -> (IncSolver, Vec<Option<usize>>) {
        let mut solver = IncSolver::new();
        let mut seg_var_map = Vec::with_capacity(segments.len());

        // Create variables for each segment
        for segment in segments {
            let weight = if segment.fixed { FIXED_WEIGHT } else { FREE_WEIGHT };
            let var_idx = solver.add_variable(segment.position, weight);
            seg_var_map.push(Some(var_idx));
        }

        // Sort segments by position for consistent ordering
        let mut segment_order: Vec<usize> = (0..segments.len()).collect();
        segment_order.sort_by(|&a, &b| {
            let pos_a = segments[a].position;
            let pos_b = segments[b].position;
            pos_a.partial_cmp(&pos_b).unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.cmp(&b)) // Tie-breaker by index
        });

        // Create CHAIN constraints between adjacent overlapping segments
        // This avoids the bug where O(n²) pairwise constraints cause
        // variables to be merged at wrong offsets
        for window in segment_order.windows(2) {
            let i = window[0];
            let j = window[1];

            if segments[i].overlaps(&segments[j], routes) {
                let var_i = seg_var_map[i].unwrap();
                let var_j = seg_var_map[j].unwrap();
                solver.add_constraint(var_i, var_j, self.nudge_distance);
            }
        }

        // Solve
        solver.solve();

        (solver, seg_var_map)
    }

    /// Apply solved positions to segments
    fn apply_positions(
        &self,
        segments: &mut [ShiftSegment],
        solver: &IncSolver,
        seg_var_map: &[Option<usize>],
    ) {
        for (i, segment) in segments.iter_mut().enumerate() {
            if let Some(var_idx) = seg_var_map[i] {
                let new_pos = solver.get_position(var_idx);
                // Clamp to limits
                segment.position = new_pos.max(segment.min_limit).min(segment.max_limit);
            }
        }
    }

    /// Update routes with new segment positions
    fn update_routes(&self, routes: &mut [Polygon], segments: &[ShiftSegment], dimension: usize) {
        for segment in segments {
            let route = &mut routes[segment.route_idx];
            let new_pos = segment.position;

            // Update both endpoints of the segment
            if dimension == 0 {
                // Horizontal segment - update Y
                route.ps[segment.low_idx].y = new_pos;
                route.ps[segment.high_idx].y = new_pos;
            } else {
                // Vertical segment - update X
                route.ps[segment.low_idx].x = new_pos;
                route.ps[segment.high_idx].x = new_pos;
            }
        }
    }
}

impl Default for ChannelRouter {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    fn make_route(points: &[(f64, f64)]) -> Polygon {
        let mut poly = Polygon::new();
        for &(x, y) in points {
            poly.push(Point::new(x, y));
        }
        poly
    }

    #[test]
    fn test_no_overlap() {
        let router = ChannelRouter::new();
        let mut routes = vec![
            make_route(&[(0.0, 0.0), (100.0, 0.0)]),
            make_route(&[(0.0, 50.0), (100.0, 50.0)]),
        ];

        router.nudge_routes(&mut routes);

        // Routes should stay at their positions (no overlap)
        assert!((routes[0].at(0).y - 0.0).abs() < 1.0);
        assert!((routes[1].at(0).y - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_overlapping_horizontal() {
        let router = ChannelRouter::new();
        let mut routes = vec![
            make_route(&[(0.0, 0.0), (100.0, 0.0)]),
            make_route(&[(0.0, 0.0), (100.0, 0.0)]),
        ];

        router.nudge_routes(&mut routes);

        // Routes should be pushed apart
        let y1 = routes[0].at(0).y;
        let y2 = routes[1].at(0).y;
        let gap = (y1 - y2).abs();

        assert!(
            gap >= DEFAULT_NUDGE_DISTANCE - 0.1,
            "Gap {} should be >= {}",
            gap,
            DEFAULT_NUDGE_DISTANCE
        );
    }

    #[test]
    fn test_overlapping_vertical() {
        let router = ChannelRouter::new();
        let mut routes = vec![
            make_route(&[(0.0, 0.0), (0.0, 100.0)]),
            make_route(&[(0.0, 0.0), (0.0, 100.0)]),
        ];

        router.nudge_routes(&mut routes);

        // Routes should be pushed apart
        let x1 = routes[0].at(0).x;
        let x2 = routes[1].at(0).x;
        let gap = (x1 - x2).abs();

        assert!(
            gap >= DEFAULT_NUDGE_DISTANCE - 0.1,
            "Gap {} should be >= {}",
            gap,
            DEFAULT_NUDGE_DISTANCE
        );
    }

    #[test]
    fn test_three_overlapping() {
        let router = ChannelRouter::new();
        let mut routes = vec![
            make_route(&[(0.0, 0.0), (100.0, 0.0)]),
            make_route(&[(0.0, 0.0), (100.0, 0.0)]),
            make_route(&[(0.0, 0.0), (100.0, 0.0)]),
        ];

        router.nudge_routes(&mut routes);

        // All routes should be pushed apart
        let y1 = routes[0].at(0).y;
        let y2 = routes[1].at(0).y;
        let y3 = routes[2].at(0).y;

        let mut ys = vec![y1, y2, y3];
        ys.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let gap1 = ys[1] - ys[0];
        let gap2 = ys[2] - ys[1];

        assert!(
            gap1 >= DEFAULT_NUDGE_DISTANCE - 0.1,
            "Gap1 {} should be >= {}",
            gap1,
            DEFAULT_NUDGE_DISTANCE
        );
        assert!(
            gap2 >= DEFAULT_NUDGE_DISTANCE - 0.1,
            "Gap2 {} should be >= {}",
            gap2,
            DEFAULT_NUDGE_DISTANCE
        );
    }

    #[test]
    fn test_l_shaped_route() {
        let router = ChannelRouter::new();
        let mut routes = vec![
            make_route(&[(0.0, 0.0), (50.0, 0.0), (50.0, 100.0)]),
            make_route(&[(0.0, 0.0), (50.0, 0.0), (50.0, 100.0)]),
        ];

        router.nudge_routes(&mut routes);

        // Both horizontal and vertical segments should be nudged
        // Check horizontal segments
        let y1_h = routes[0].at(0).y;
        let y2_h = routes[1].at(0).y;

        // Check vertical segments
        let x1_v = routes[0].at(1).x;
        let x2_v = routes[1].at(1).x;

        // At least one dimension should have separation
        let h_gap = (y1_h - y2_h).abs();
        let v_gap = (x1_v - x2_v).abs();

        assert!(
            h_gap >= DEFAULT_NUDGE_DISTANCE - 0.1 || v_gap >= DEFAULT_NUDGE_DISTANCE - 0.1,
            "At least one dimension should be nudged: h_gap={}, v_gap={}",
            h_gap,
            v_gap
        );
    }
}
