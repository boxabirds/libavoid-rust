//! Orthogonal (rectilinear) connector routing
//!
//! This module implements algorithms for routing connectors using only
//! horizontal and vertical line segments.

use crate::geometry::{Point, Polygon, PolygonInterface};
use crate::obstacle::Obstacle;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

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

// ============================================================================
// Orthogonal A* Routing (Task 23-24)
// ============================================================================

/// Node ID for orthogonal graph
type NodeId = u64;

/// State for A* priority queue
#[derive(Clone)]
struct AStarState {
    cost: f64,
    node_id: NodeId,
    position: Point,
    direction: Option<Direction>,
}

impl PartialEq for AStarState {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for AStarState {}

impl Ord for AStarState {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap (lower cost = higher priority)
        other.cost.partial_cmp(&self.cost).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for AStarState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Encodes a position and direction into a node ID
fn encode_node(x: i32, y: i32, dir: Option<Direction>) -> NodeId {
    let dir_bits: u64 = match dir {
        None => 0,
        Some(Direction::North) => 1,
        Some(Direction::South) => 2,
        Some(Direction::East) => 3,
        Some(Direction::West) => 4,
    };
    ((x as u64 & 0xFFFFFF) << 32) | ((y as u64 & 0xFFFFFF) << 8) | dir_bits
}

/// Grid resolution for discretization
const GRID_RESOLUTION: f64 = 1.0;

/// Advanced orthogonal router using A* on an implicit grid
pub struct OrthogonalAStarRouter {
    bend_penalty: f64,
    segment_penalty: f64,
    nudge_distance: f64,
}

impl OrthogonalAStarRouter {
    pub fn new() -> Self {
        OrthogonalAStarRouter {
            bend_penalty: 50.0,
            segment_penalty: 1.0,
            nudge_distance: 4.0,
        }
    }

    pub fn with_penalties(bend_penalty: f64, segment_penalty: f64, nudge_distance: f64) -> Self {
        OrthogonalAStarRouter {
            bend_penalty,
            segment_penalty,
            nudge_distance,
        }
    }

    /// Routes using A* on an implicit orthogonal grid
    pub fn route_astar(
        &self,
        start: Point,
        goal: Point,
        obstacles: &[&dyn Obstacle],
    ) -> Polygon {
        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<NodeId, (NodeId, Point)> = HashMap::new();
        let mut g_score: HashMap<NodeId, f64> = HashMap::new();

        // Start with all four directions
        for dir in [Direction::North, Direction::South, Direction::East, Direction::West] {
            let start_node = encode_node(
                (start.x / GRID_RESOLUTION) as i32,
                (start.y / GRID_RESOLUTION) as i32,
                Some(dir),
            );
            g_score.insert(start_node, 0.0);
            open_set.push(AStarState {
                cost: self.heuristic(&start, &goal),
                node_id: start_node,
                position: start,
                direction: Some(dir),
            });
        }

        let goal_tolerance = GRID_RESOLUTION * 2.0;

        while let Some(current) = open_set.pop() {
            // Check if we reached the goal
            if current.position.distance(&goal) < goal_tolerance {
                return self.reconstruct_path(&came_from, current.node_id, start, goal);
            }

            let current_g = *g_score.get(&current.node_id).unwrap_or(&f64::INFINITY);

            // Explore neighbors in orthogonal directions
            for next_dir in [Direction::North, Direction::South, Direction::East, Direction::West] {
                let (dx, dy) = match next_dir {
                    Direction::North => (0.0, -GRID_RESOLUTION),
                    Direction::South => (0.0, GRID_RESOLUTION),
                    Direction::East => (GRID_RESOLUTION, 0.0),
                    Direction::West => (-GRID_RESOLUTION, 0.0),
                };

                let next_pos = Point::new(current.position.x + dx, current.position.y + dy);

                // Skip if blocked by obstacle
                if self.is_blocked(&current.position, &next_pos, obstacles) {
                    continue;
                }

                let next_node = encode_node(
                    (next_pos.x / GRID_RESOLUTION) as i32,
                    (next_pos.y / GRID_RESOLUTION) as i32,
                    Some(next_dir),
                );

                // Compute cost
                let move_cost = GRID_RESOLUTION * self.segment_penalty;
                let bend_cost = if current.direction != Some(next_dir) {
                    self.bend_penalty
                } else {
                    0.0
                };
                let tentative_g = current_g + move_cost + bend_cost;

                if tentative_g < *g_score.get(&next_node).unwrap_or(&f64::INFINITY) {
                    came_from.insert(next_node, (current.node_id, current.position));
                    g_score.insert(next_node, tentative_g);

                    let f = tentative_g + self.heuristic(&next_pos, &goal);
                    open_set.push(AStarState {
                        cost: f,
                        node_id: next_node,
                        position: next_pos,
                        direction: Some(next_dir),
                    });
                }
            }
        }

        // Fallback: simple L-shape route
        self.simple_l_route(start, goal)
    }

    fn heuristic(&self, from: &Point, to: &Point) -> f64 {
        // Manhattan distance for orthogonal routing
        (from.x - to.x).abs() + (from.y - to.y).abs()
    }

    fn is_blocked(&self, from: &Point, to: &Point, obstacles: &[&dyn Obstacle]) -> bool {
        for obstacle in obstacles {
            if !obstacle.is_active() {
                continue;
            }
            let bbox = obstacle.polygon().bounding_rect();

            // Check if segment passes through obstacle bbox (with buffer)
            let buffer = 1.0;
            let min_x = bbox.min.x - buffer;
            let max_x = bbox.max.x + buffer;
            let min_y = bbox.min.y - buffer;
            let max_y = bbox.max.y + buffer;

            // Horizontal segment
            if (from.y - to.y).abs() < 0.001 {
                let y = from.y;
                let x_min = from.x.min(to.x);
                let x_max = from.x.max(to.x);
                if y > min_y && y < max_y && x_max > min_x && x_min < max_x {
                    return true;
                }
            }
            // Vertical segment
            else if (from.x - to.x).abs() < 0.001 {
                let x = from.x;
                let y_min = from.y.min(to.y);
                let y_max = from.y.max(to.y);
                if x > min_x && x < max_x && y_max > min_y && y_min < max_y {
                    return true;
                }
            }
        }
        false
    }

    fn reconstruct_path(
        &self,
        came_from: &HashMap<NodeId, (NodeId, Point)>,
        goal_node: NodeId,
        start: Point,
        goal: Point,
    ) -> Polygon {
        let mut path = vec![goal];
        let mut current = goal_node;

        while let Some((prev_node, prev_pos)) = came_from.get(&current) {
            path.push(*prev_pos);
            current = *prev_node;
        }

        path.push(start);
        path.reverse();

        // Simplify path by removing collinear points
        let mut poly = Polygon::new();
        for point in path {
            poly.push(point);
        }
        self.simplify_orthogonal(&mut poly);
        poly
    }

    fn simple_l_route(&self, start: Point, goal: Point) -> Polygon {
        let mut poly = Polygon::new();
        poly.push(start);

        if (start.x - goal.x).abs() > 0.001 {
            poly.push(Point::new(goal.x, start.y));
        }

        poly.push(goal);
        poly
    }

    fn simplify_orthogonal(&self, path: &mut Polygon) {
        if path.size() < 3 {
            return;
        }

        let mut simplified = vec![*path.at(0)];

        for i in 1..path.size() - 1 {
            let prev = simplified.last().unwrap();
            let curr = path.at(i);
            let next = path.at(i + 1);

            // Keep point if direction changes
            let h1 = (prev.y - curr.y).abs() < 0.001;
            let h2 = (curr.y - next.y).abs() < 0.001;

            if h1 != h2 {
                simplified.push(*curr);
            }
        }

        simplified.push(*path.at(path.size() - 1));
        path.ps = simplified;
    }
}

impl Default for OrthogonalAStarRouter {
    fn default() -> Self {
        Self::new()
    }
}

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
