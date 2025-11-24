//! Graph algorithms for pathfinding
//!
//! This module implements A* and other pathfinding algorithms used to find
//! optimal routes through the visibility graph.

use crate::geometry::{Point, Polygon};
use crate::visibility::{VisibilityGraph, VertexId, VertInf, EdgeInf};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;

// ============================================================================
// Constants
// ============================================================================

/// Default segment penalty (cost per path segment to encourage fewer bends)
pub const DEFAULT_SEGMENT_PENALTY: f64 = 1.0;

/// Default angle penalty (cost for non-straight angles)
pub const DEFAULT_ANGLE_PENALTY: f64 = 50.0;

/// Default crossing penalty (cost for crossing another connector)
pub const DEFAULT_CROSSING_PENALTY: f64 = 200.0;

/// Default reverse direction penalty
pub const DEFAULT_REVERSE_PENALTY: f64 = 100.0;

// ============================================================================
// Path Result
// ============================================================================

/// A* path finding result
#[derive(Debug, Clone)]
pub struct PathResult {
    /// The path as a sequence of vertex IDs
    pub path: Vec<VertexId>,
    /// Total cost of the path
    pub cost: f64,
}

// ============================================================================
// Search Node
// ============================================================================

/// A node in the search priority queue
#[derive(Debug, Clone)]
struct SearchNode {
    vertex_id: VertexId,
    g_score: f64,              // Cost from start to this node
    f_score: f64,              // Estimated total cost (g + heuristic)
    prev_direction: Option<Point>, // Direction we came from (for angle penalty)
}

impl PartialEq for SearchNode {
    fn eq(&self, other: &Self) -> bool {
        self.f_score == other.f_score
    }
}

impl Eq for SearchNode {}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Reverse ordering for min-heap
        other.f_score.partial_cmp(&self.f_score)
    }
}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

// ============================================================================
// Path Finder
// ============================================================================

/// Pathfinding context for A* search with configurable cost penalties
pub struct PathFinder {
    /// Heuristic weight (1.0 = A*, 0.0 = Dijkstra)
    heuristic_weight: f64,
    /// Cost per path segment (encourages fewer bends)
    segment_penalty: f64,
    /// Cost for non-straight angles (encourages straighter paths)
    angle_penalty: f64,
    /// Cost for crossing another connector
    crossing_penalty: f64,
    /// Cost for routing away from destination
    reverse_penalty: f64,
}

impl PathFinder {
    /// Creates a new pathfinder with default parameters
    pub fn new() -> Self {
        PathFinder {
            heuristic_weight: 1.0,
            segment_penalty: DEFAULT_SEGMENT_PENALTY,
            angle_penalty: DEFAULT_ANGLE_PENALTY,
            crossing_penalty: DEFAULT_CROSSING_PENALTY,
            reverse_penalty: DEFAULT_REVERSE_PENALTY,
        }
    }

    /// Creates a pathfinder with custom parameters
    pub fn with_parameters(
        segment_penalty: f64,
        angle_penalty: f64,
        crossing_penalty: f64,
        reverse_penalty: f64,
    ) -> Self {
        PathFinder {
            heuristic_weight: 1.0,
            segment_penalty,
            angle_penalty,
            crossing_penalty,
            reverse_penalty,
        }
    }

    /// Creates a pathfinder with custom heuristic weight
    pub fn with_heuristic_weight(weight: f64) -> Self {
        PathFinder {
            heuristic_weight: weight,
            ..Self::new()
        }
    }

    /// Sets the segment penalty
    pub fn set_segment_penalty(&mut self, penalty: f64) {
        self.segment_penalty = penalty;
    }

    /// Sets the angle penalty
    pub fn set_angle_penalty(&mut self, penalty: f64) {
        self.angle_penalty = penalty;
    }

    /// Sets the crossing penalty
    pub fn set_crossing_penalty(&mut self, penalty: f64) {
        self.crossing_penalty = penalty;
    }

    /// Sets the reverse direction penalty
    pub fn set_reverse_penalty(&mut self, penalty: f64) {
        self.reverse_penalty = penalty;
    }

    /// Finds the shortest path from start to goal using A* with cost penalties
    pub fn find_path(
        &self,
        graph: &VisibilityGraph,
        start_id: VertexId,
        goal_id: VertexId,
    ) -> Option<Vec<VertexId>> {
        self.find_path_with_result(graph, start_id, goal_id)
            .map(|r| r.path)
    }

    /// Finds the shortest path and returns the full result with cost
    pub fn find_path_with_result(
        &self,
        graph: &VisibilityGraph,
        start_id: VertexId,
        goal_id: VertexId,
    ) -> Option<PathResult> {
        let start_vertex = graph.get_vertex(start_id)?;
        let goal_vertex = graph.get_vertex(goal_id)?;
        let goal_point = goal_vertex.point;

        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<VertexId, (VertexId, Option<Point>)> = HashMap::new();
        let mut g_score: HashMap<VertexId, f64> = HashMap::new();
        let mut closed_set: HashSet<VertexId> = HashSet::new();

        g_score.insert(start_id, 0.0);

        let h = self.heuristic(&start_vertex.point, &goal_point);
        open_set.push(SearchNode {
            vertex_id: start_id,
            g_score: 0.0,
            f_score: h,
            prev_direction: None,
        });

        while let Some(current) = open_set.pop() {
            if current.vertex_id == goal_id {
                let path = self.reconstruct_path(&came_from, goal_id);
                return Some(PathResult {
                    path,
                    cost: current.g_score,
                });
            }

            if closed_set.contains(&current.vertex_id) {
                continue;
            }

            closed_set.insert(current.vertex_id);

            let current_vertex = match graph.get_vertex(current.vertex_id) {
                Some(v) => v,
                None => continue,
            };

            // Iterate over all edges (both regular and orthogonal)
            for edge in current_vertex.all_edges() {
                if closed_set.contains(&edge.target_id) {
                    continue;
                }

                let target_vertex = match graph.get_vertex(edge.target_id) {
                    Some(v) => v,
                    None => continue,
                };

                // Calculate the full edge cost including penalties
                let edge_cost = self.compute_edge_cost(
                    current_vertex,
                    target_vertex,
                    edge,
                    &goal_point,
                    current.prev_direction.as_ref(),
                );

                let tentative_g = current.g_score + edge_cost;
                let previous_g = g_score.get(&edge.target_id).copied().unwrap_or(f64::INFINITY);

                if tentative_g < previous_g {
                    // Calculate direction for angle penalty in next iteration
                    let new_direction = Point::new(
                        target_vertex.point.x - current_vertex.point.x,
                        target_vertex.point.y - current_vertex.point.y,
                    );

                    came_from.insert(edge.target_id, (current.vertex_id, Some(new_direction)));
                    g_score.insert(edge.target_id, tentative_g);

                    let h = self.heuristic(&target_vertex.point, &goal_point);
                    let f = tentative_g + h;

                    open_set.push(SearchNode {
                        vertex_id: edge.target_id,
                        g_score: tentative_g,
                        f_score: f,
                        prev_direction: Some(new_direction),
                    });
                }
            }
        }

        None // No path found
    }

    /// Finds a path with checkpoints (waypoints that must be visited)
    pub fn find_path_with_checkpoints(
        &self,
        graph: &VisibilityGraph,
        start_id: VertexId,
        goal_id: VertexId,
        checkpoints: &[VertexId],
    ) -> Option<PathResult> {
        if checkpoints.is_empty() {
            return self.find_path_with_result(graph, start_id, goal_id);
        }

        let mut full_path: Vec<VertexId> = Vec::new();
        let mut total_cost = 0.0;
        let mut current_start = start_id;

        // Route through each checkpoint in order
        for &checkpoint in checkpoints {
            let result = self.find_path_with_result(graph, current_start, checkpoint)?;

            // Add path (skip first point if not first segment to avoid duplicates)
            if full_path.is_empty() {
                full_path.extend(result.path);
            } else {
                full_path.extend(result.path.into_iter().skip(1));
            }
            total_cost += result.cost;
            current_start = checkpoint;
        }

        // Route from last checkpoint to goal
        let final_result = self.find_path_with_result(graph, current_start, goal_id)?;
        full_path.extend(final_result.path.into_iter().skip(1));
        total_cost += final_result.cost;

        Some(PathResult {
            path: full_path,
            cost: total_cost,
        })
    }

    /// Computes the full cost of traversing an edge including all penalties
    fn compute_edge_cost(
        &self,
        from: &VertInf,
        to: &VertInf,
        edge: &EdgeInf,
        goal: &Point,
        prev_direction: Option<&Point>,
    ) -> f64 {
        // Base cost is the edge distance
        let mut cost = edge.distance;

        // Segment penalty (each edge adds a fixed cost)
        cost += self.segment_penalty;

        // Angle penalty (cost for turning)
        if let Some(prev_dir) = prev_direction {
            let current_dir = Point::new(
                to.point.x - from.point.x,
                to.point.y - from.point.y,
            );
            cost += self.compute_angle_penalty(prev_dir, &current_dir);
        }

        // Crossing penalty (cost for crossing other connectors)
        if !edge.using_connectors.is_empty() {
            cost += self.crossing_penalty * edge.connector_count() as f64;
        }

        // Reverse direction penalty (cost for moving away from goal)
        cost += self.compute_reverse_penalty(&from.point, &to.point, goal);

        cost
    }

    /// Computes the angle penalty between two direction vectors
    fn compute_angle_penalty(&self, prev_dir: &Point, current_dir: &Point) -> f64 {
        // Normalize the direction vectors
        let prev_len = (prev_dir.x * prev_dir.x + prev_dir.y * prev_dir.y).sqrt();
        let curr_len = (current_dir.x * current_dir.x + current_dir.y * current_dir.y).sqrt();

        if prev_len < 1e-10 || curr_len < 1e-10 {
            return 0.0;
        }

        let prev_norm = Point::new(prev_dir.x / prev_len, prev_dir.y / prev_len);
        let curr_norm = Point::new(current_dir.x / curr_len, current_dir.y / curr_len);

        // Dot product gives us cos(angle)
        let dot = prev_norm.x * curr_norm.x + prev_norm.y * curr_norm.y;

        // Clamp to valid range
        let dot = dot.max(-1.0).min(1.0);

        // Angle in radians
        let angle = dot.acos();

        // Use logarithmic scaling like C++ libavoid (makepath.cpp:450-470)
        // This gives less penalty to moderate turns, more to sharp turns
        // C++: angleWeight * std::log10(1.0 + (angle / M_PI))
        let normalized_angle = angle / std::f64::consts::PI;
        let angle_cost = self.angle_penalty * (1.0 + normalized_angle).log10();

        angle_cost
    }

    /// Computes penalty for moving away from the goal
    fn compute_reverse_penalty(&self, from: &Point, to: &Point, goal: &Point) -> f64 {
        let dist_from_goal_before = from.distance(goal);
        let dist_from_goal_after = to.distance(goal);

        // If we're moving away from the goal, apply penalty
        if dist_from_goal_after > dist_from_goal_before {
            let reverse_amount = dist_from_goal_after - dist_from_goal_before;
            self.reverse_penalty * (reverse_amount / dist_from_goal_before).min(1.0)
        } else {
            0.0
        }
    }

    /// Reconstructs the path from the came_from map
    fn reconstruct_path(
        &self,
        came_from: &HashMap<VertexId, (VertexId, Option<Point>)>,
        mut current: VertexId,
    ) -> Vec<VertexId> {
        let mut path = vec![current];

        while let Some(&(prev, _)) = came_from.get(&current) {
            path.push(prev);
            current = prev;
        }

        path.reverse();
        path
    }

    /// Heuristic function for A* (Euclidean distance)
    fn heuristic(&self, from: &Point, to: &Point) -> f64 {
        from.distance(to) * self.heuristic_weight
    }

    /// Converts a path of vertex IDs to a polygon route
    pub fn path_to_polygon(&self, graph: &VisibilityGraph, path: &[VertexId]) -> Option<Polygon> {
        if path.is_empty() {
            return None;
        }

        let mut polygon = Polygon::with_capacity(path.len());

        for &vertex_id in path {
            if let Some(vertex) = graph.get_vertex(vertex_id) {
                polygon.push(vertex.point);
            } else {
                return None;
            }
        }

        Some(polygon)
    }
}

impl Default for PathFinder {
    fn default() -> Self {
        PathFinder::new()
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Computes the cost of a path through the graph
pub fn compute_path_cost(graph: &VisibilityGraph, path: &[VertexId]) -> f64 {
    if path.len() < 2 {
        return 0.0;
    }

    let mut total_cost = 0.0;

    for i in 0..path.len() - 1 {
        let from_id = path[i];
        let to_id = path[i + 1];

        if let Some(from_vertex) = graph.get_vertex(from_id) {
            if let Some(edge) = from_vertex.all_edges().find(|e| e.target_id == to_id) {
                total_cost += edge.distance;
            } else if let Some(to_vertex) = graph.get_vertex(to_id) {
                // If no edge exists, use direct distance
                total_cost += from_vertex.point.distance(&to_vertex.point);
            }
        }
    }

    total_cost
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::PolygonInterface;

    #[test]
    fn test_pathfinder_creation() {
        let pf = PathFinder::new();
        assert_eq!(pf.heuristic_weight, 1.0);
        assert_eq!(pf.segment_penalty, DEFAULT_SEGMENT_PENALTY);
        assert_eq!(pf.angle_penalty, DEFAULT_ANGLE_PENALTY);
    }

    #[test]
    fn test_pathfinder_with_parameters() {
        let pf = PathFinder::with_parameters(2.0, 100.0, 500.0, 50.0);
        assert_eq!(pf.segment_penalty, 2.0);
        assert_eq!(pf.angle_penalty, 100.0);
        assert_eq!(pf.crossing_penalty, 500.0);
        assert_eq!(pf.reverse_penalty, 50.0);
    }

    #[test]
    fn test_simple_path() {
        let mut graph = VisibilityGraph::new();

        let v1 = graph.add_vertex(Point::new(0.0, 0.0));
        let v2 = graph.add_vertex(Point::new(10.0, 0.0));
        let v3 = graph.add_vertex(Point::new(20.0, 0.0));

        graph.add_edge(v1, v2, true);
        graph.add_edge(v2, v3, true);

        let pf = PathFinder::new();
        let path = pf.find_path(&graph, v1, v3);

        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 3);
        assert_eq!(path[0], v1);
        assert_eq!(path[2], v3);
    }

    #[test]
    fn test_path_with_result() {
        let mut graph = VisibilityGraph::new();

        let v1 = graph.add_vertex(Point::new(0.0, 0.0));
        let v2 = graph.add_vertex(Point::new(10.0, 0.0));

        graph.add_edge(v1, v2, true);

        let pf = PathFinder::new();
        let result = pf.find_path_with_result(&graph, v1, v2);

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.path.len(), 2);
        assert!(result.cost > 10.0); // Distance + segment penalty
    }

    #[test]
    fn test_no_path() {
        let mut graph = VisibilityGraph::new();

        let v1 = graph.add_vertex(Point::new(0.0, 0.0));
        let v2 = graph.add_vertex(Point::new(10.0, 0.0));

        // No edges - no path

        let pf = PathFinder::new();
        let path = pf.find_path(&graph, v1, v2);

        assert!(path.is_none());
    }

    #[test]
    fn test_path_to_polygon() {
        let mut graph = VisibilityGraph::new();

        let v1 = graph.add_vertex(Point::new(0.0, 0.0));
        let v2 = graph.add_vertex(Point::new(10.0, 0.0));

        let pf = PathFinder::new();
        let poly = pf.path_to_polygon(&graph, &[v1, v2]);

        assert!(poly.is_some());
        let poly = poly.unwrap();
        assert_eq!(poly.size(), 2);
        assert_eq!(poly.at(0).x, 0.0);
        assert_eq!(poly.at(1).x, 10.0);
    }

    #[test]
    fn test_angle_penalty_straight() {
        let pf = PathFinder::new();

        // Same direction should have no penalty
        let prev = Point::new(1.0, 0.0);
        let curr = Point::new(1.0, 0.0);
        let penalty = pf.compute_angle_penalty(&prev, &curr);
        assert!(penalty < 0.01);
    }

    #[test]
    fn test_angle_penalty_90_degree() {
        let pf = PathFinder::new();

        // 90 degree turn with logarithmic scaling
        // angle/π = 0.5, so log10(1.5) ≈ 0.176, penalty ≈ 8.8
        let prev = Point::new(1.0, 0.0);
        let curr = Point::new(0.0, 1.0);
        let penalty = pf.compute_angle_penalty(&prev, &curr);
        let expected = DEFAULT_ANGLE_PENALTY * (1.0f64 + 0.5f64).log10();
        assert!((penalty - expected).abs() < 0.1);
    }

    #[test]
    fn test_angle_penalty_180_degree() {
        let pf = PathFinder::new();

        // 180 degree turn with logarithmic scaling
        // angle/π = 1.0, so log10(2.0) ≈ 0.301, penalty ≈ 15.05
        let prev = Point::new(1.0, 0.0);
        let curr = Point::new(-1.0, 0.0);
        let penalty = pf.compute_angle_penalty(&prev, &curr);
        let expected = DEFAULT_ANGLE_PENALTY * (1.0f64 + 1.0f64).log10();
        assert!((penalty - expected).abs() < 0.1);
    }

    #[test]
    fn test_path_with_checkpoints() {
        let mut graph = VisibilityGraph::new();

        let v1 = graph.add_vertex(Point::new(0.0, 0.0));
        let v2 = graph.add_vertex(Point::new(10.0, 0.0));
        let v3 = graph.add_vertex(Point::new(20.0, 0.0));
        let v4 = graph.add_vertex(Point::new(30.0, 0.0));

        graph.add_edge(v1, v2, true);
        graph.add_edge(v2, v3, true);
        graph.add_edge(v3, v4, true);

        let pf = PathFinder::new();
        let result = pf.find_path_with_checkpoints(&graph, v1, v4, &[v2, v3]);

        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.path.len(), 4);
        assert_eq!(result.path, vec![v1, v2, v3, v4]);
    }

    #[test]
    fn test_logarithmic_angle_penalty() {
        let pf = PathFinder::new();

        // Test logarithmic scaling (C++ parity)
        // log10(1 + 0) = 0
        // log10(1 + 0.5) ≈ 0.176
        // log10(1 + 1.0) ≈ 0.301

        // 0° turn (same direction)
        let prev = Point::new(1.0, 0.0);
        let curr = Point::new(1.0, 0.0);
        let penalty_0 = pf.compute_angle_penalty(&prev, &curr);
        assert!(penalty_0.abs() < 0.1, "0° turn should have ~0 penalty");

        // 90° turn (angle/π = 0.5)
        let prev = Point::new(1.0, 0.0);
        let curr = Point::new(0.0, 1.0);
        let penalty_90 = pf.compute_angle_penalty(&prev, &curr);
        let expected_90 = DEFAULT_ANGLE_PENALTY * (1.0f64 + 0.5f64).log10(); // ≈ 8.8
        assert!((penalty_90 - expected_90).abs() < 0.1,
            "90° turn should use log scale: got {}, expected {}",
            penalty_90, expected_90);

        // 180° turn (angle/π = 1.0)
        let prev = Point::new(1.0, 0.0);
        let curr = Point::new(-1.0, 0.0);
        let penalty_180 = pf.compute_angle_penalty(&prev, &curr);
        let expected_180 = DEFAULT_ANGLE_PENALTY * (1.0f64 + 1.0f64).log10(); // ≈ 15.05
        assert!((penalty_180 - expected_180).abs() < 0.1,
            "180° turn should use log scale: got {}, expected {}",
            penalty_180, expected_180);

        // Verify logarithmic property: 90° penalty is proportionally higher
        // With logarithmic scaling, moderate turns get relatively MORE penalty compared to sharp turns
        // (as a ratio) than they would with linear scaling
        // Linear: 90° = 0.5x of 180°, Logarithmic: 90° ≈ 0.585x of 180°
        assert!(penalty_90 > penalty_180 / 2.0,
            "Logarithmic scaling: 90° penalty should be > half of 180° penalty (ratio: {:.3})",
            penalty_90 / penalty_180);
    }
}
