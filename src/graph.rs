//! Graph algorithms for pathfinding
//!
//! This module implements A* and other pathfinding algorithms used to find
//! optimal routes through the visibility graph.

use crate::geometry::{Point, Polygon, PolygonInterface};
use crate::visibility::VisibilityGraph;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;

/// A node in the search priority queue
#[derive(Debug, Clone)]
struct SearchNode {
    vertex_id: u32,
    g_score: f64, // Cost from start to this node
    f_score: f64, // Estimated total cost (g + heuristic)
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

/// Pathfinding context for A* search
pub struct PathFinder {
    /// Heuristic weight (1.0 = A*, 0.0 = Dijkstra)
    heuristic_weight: f64,
}

impl PathFinder {
    /// Creates a new pathfinder
    pub fn new() -> Self {
        PathFinder {
            heuristic_weight: 1.0,
        }
    }

    /// Creates a pathfinder with custom heuristic weight
    pub fn with_heuristic_weight(weight: f64) -> Self {
        PathFinder {
            heuristic_weight: weight,
        }
    }

    /// Finds the shortest path from start to goal using A*
    pub fn find_path(
        &self,
        graph: &VisibilityGraph,
        start_id: u32,
        goal_id: u32,
    ) -> Option<Vec<u32>> {
        let start_vertex = graph.get_vertex(start_id)?;
        let goal_vertex = graph.get_vertex(goal_id)?;

        let mut open_set = BinaryHeap::new();
        let mut came_from: HashMap<u32, u32> = HashMap::new();
        let mut g_score: HashMap<u32, f64> = HashMap::new();
        let mut closed_set: HashSet<u32> = HashSet::new();

        g_score.insert(start_id, 0.0);

        let h = self.heuristic(&start_vertex.point, &goal_vertex.point);
        open_set.push(SearchNode {
            vertex_id: start_id,
            g_score: 0.0,
            f_score: h,
        });

        while let Some(current) = open_set.pop() {
            if current.vertex_id == goal_id {
                return Some(self.reconstruct_path(&came_from, goal_id));
            }

            if closed_set.contains(&current.vertex_id) {
                continue;
            }

            closed_set.insert(current.vertex_id);

            let current_vertex = match graph.get_vertex(current.vertex_id) {
                Some(v) => v,
                None => continue,
            };

            for edge in &current_vertex.edges {
                if closed_set.contains(&edge.target_id) {
                    continue;
                }

                let tentative_g = current.g_score + edge.weight;

                let previous_g = g_score.get(&edge.target_id).copied().unwrap_or(f64::INFINITY);

                if tentative_g < previous_g {
                    came_from.insert(edge.target_id, current.vertex_id);
                    g_score.insert(edge.target_id, tentative_g);

                    if let Some(target_vertex) = graph.get_vertex(edge.target_id) {
                        let h = self.heuristic(&target_vertex.point, &goal_vertex.point);
                        let f = tentative_g + h;

                        open_set.push(SearchNode {
                            vertex_id: edge.target_id,
                            g_score: tentative_g,
                            f_score: f,
                        });
                    }
                }
            }
        }

        None // No path found
    }

    /// Reconstructs the path from the came_from map
    fn reconstruct_path(&self, came_from: &HashMap<u32, u32>, mut current: u32) -> Vec<u32> {
        let mut path = vec![current];

        while let Some(&prev) = came_from.get(&current) {
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
    pub fn path_to_polygon(&self, graph: &VisibilityGraph, path: &[u32]) -> Option<Polygon> {
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

/// Computes the cost of a path through the graph
pub fn compute_path_cost(graph: &VisibilityGraph, path: &[u32]) -> f64 {
    if path.len() < 2 {
        return 0.0;
    }

    let mut total_cost = 0.0;

    for i in 0..path.len() - 1 {
        let from_id = path[i];
        let to_id = path[i + 1];

        if let Some(from_vertex) = graph.get_vertex(from_id) {
            if let Some(edge) = from_vertex.edges.iter().find(|e| e.target_id == to_id) {
                total_cost += edge.weight;
            } else if let Some(to_vertex) = graph.get_vertex(to_id) {
                // If no edge exists, use direct distance
                total_cost += from_vertex.point.distance(&to_vertex.point);
            }
        }
    }

    total_cost
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pathfinder_creation() {
        let pf = PathFinder::new();
        assert_eq!(pf.heuristic_weight, 1.0);
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
}
