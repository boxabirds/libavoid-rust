//! Visibility graph computation for pathfinding
//!
//! This module implements visibility graph algorithms that determine which
//! vertices can "see" each other without obstruction.

use crate::geometry::{Point, Edge, Polygon, PolygonInterface};
use crate::obstacle::Obstacle;
use std::collections::HashMap;

/// Vertex information in the visibility graph
#[derive(Debug, Clone)]
pub struct VertexInfo {
    /// The point location
    pub point: Point,
    /// Unique vertex ID
    pub id: u32,
    /// Edges to other visible vertices
    pub edges: Vec<EdgeInfo>,
    /// Whether this vertex is active
    pub active: bool,
}

/// Edge information in the visibility graph
#[derive(Debug, Clone, Copy)]
pub struct EdgeInfo {
    /// Target vertex ID
    pub target_id: u32,
    /// Edge weight (distance)
    pub weight: f64,
    /// Whether this is an orthogonal edge
    pub orthogonal: bool,
}

impl VertexInfo {
    /// Creates a new vertex
    pub fn new(point: Point, id: u32) -> Self {
        VertexInfo {
            point,
            id,
            edges: Vec::new(),
            active: true,
        }
    }

    /// Adds an edge to another vertex
    pub fn add_edge(&mut self, target_id: u32, weight: f64, orthogonal: bool) {
        self.edges.push(EdgeInfo {
            target_id,
            weight,
            orthogonal,
        });
    }

    /// Removes all edges to a specific vertex
    pub fn remove_edges_to(&mut self, target_id: u32) {
        self.edges.retain(|e| e.target_id != target_id);
    }
}

/// Visibility graph for pathfinding
#[derive(Debug)]
pub struct VisibilityGraph {
    /// Map from vertex ID to vertex info
    vertices: HashMap<u32, VertexInfo>,
    /// Next available vertex ID
    next_vertex_id: u32,
}

impl VisibilityGraph {
    /// Creates a new empty visibility graph
    pub fn new() -> Self {
        VisibilityGraph {
            vertices: HashMap::new(),
            next_vertex_id: 1,
        }
    }

    /// Adds a vertex to the graph
    pub fn add_vertex(&mut self, point: Point) -> u32 {
        let id = self.next_vertex_id;
        self.next_vertex_id += 1;

        let vertex = VertexInfo::new(point, id);
        self.vertices.insert(id, vertex);
        id
    }

    /// Adds a vertex with a specific ID
    pub fn add_vertex_with_id(&mut self, point: Point, id: u32) {
        let vertex = VertexInfo::new(point, id);
        self.vertices.insert(id, vertex);

        if id >= self.next_vertex_id {
            self.next_vertex_id = id + 1;
        }
    }

    /// Removes a vertex from the graph
    pub fn remove_vertex(&mut self, id: u32) {
        self.vertices.remove(&id);

        // Remove all edges pointing to this vertex
        for vertex in self.vertices.values_mut() {
            vertex.remove_edges_to(id);
        }
    }

    /// Gets a vertex by ID
    pub fn get_vertex(&self, id: u32) -> Option<&VertexInfo> {
        self.vertices.get(&id)
    }

    /// Gets a mutable vertex by ID
    pub fn get_vertex_mut(&mut self, id: u32) -> Option<&mut VertexInfo> {
        self.vertices.get_mut(&id)
    }

    /// Adds an edge between two vertices
    pub fn add_edge(&mut self, from_id: u32, to_id: u32, orthogonal: bool) {
        if let (Some(from), Some(to)) = (self.vertices.get(&from_id), self.vertices.get(&to_id)) {
            let distance = from.point.distance(&to.point);

            if let Some(from_vertex) = self.vertices.get_mut(&from_id) {
                from_vertex.add_edge(to_id, distance, orthogonal);
            }
        }
    }

    /// Computes visibility between a vertex and all other vertices
    pub fn compute_vertex_visibility(
        &mut self,
        vertex_id: u32,
        obstacles: &[&dyn Obstacle],
    ) {
        let vertex = match self.vertices.get(&vertex_id) {
            Some(v) => v.clone(),
            None => return,
        };

        // Check visibility to all other vertices
        for other_id in self.vertices.keys().copied().collect::<Vec<_>>() {
            if other_id == vertex_id {
                continue;
            }

            let other = match self.vertices.get(&other_id) {
                Some(v) => v,
                None => continue,
            };

            if self.is_visible(&vertex.point, &other.point, obstacles) {
                let distance = vertex.point.distance(&other.point);
                let orthogonal = is_orthogonal(&vertex.point, &other.point);

                if let Some(v) = self.vertices.get_mut(&vertex_id) {
                    v.add_edge(other_id, distance, orthogonal);
                }
            }
        }
    }

    /// Checks if two points are visible to each other
    fn is_visible(&self, p1: &Point, p2: &Point, obstacles: &[&dyn Obstacle]) -> bool {
        let edge = Edge::new(*p1, *p2);

        // Check if the edge intersects any obstacle
        for obstacle in obstacles {
            if !obstacle.is_active() {
                continue;
            }

            if self.edge_intersects_polygon(&edge, obstacle.polygon()) {
                return false;
            }
        }

        true
    }

    /// Checks if an edge intersects a polygon
    fn edge_intersects_polygon(&self, edge: &Edge, polygon: &Polygon) -> bool {
        let n = polygon.size();
        if n < 2 {
            return false;
        }

        // Check intersection with each polygon edge
        for i in 0..n {
            let j = (i + 1) % n;
            let p1 = polygon.at(i);
            let p2 = polygon.at(j);

            if segments_intersect(&edge.a, &edge.b, p1, p2) {
                return true;
            }
        }

        false
    }

    /// Returns an iterator over all vertices
    pub fn vertices(&self) -> impl Iterator<Item = &VertexInfo> {
        self.vertices.values()
    }

    /// Returns the number of vertices
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Clears all vertices and edges
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.next_vertex_id = 1;
    }
}

impl Default for VisibilityGraph {
    fn default() -> Self {
        VisibilityGraph::new()
    }
}

/// Checks if two points form an orthogonal (horizontal or vertical) line
fn is_orthogonal(p1: &Point, p2: &Point) -> bool {
    const EPSILON: f64 = 1e-6;
    (p1.x - p2.x).abs() < EPSILON || (p1.y - p2.y).abs() < EPSILON
}

/// Checks if two line segments intersect
fn segments_intersect(a1: &Point, a2: &Point, b1: &Point, b2: &Point) -> bool {
    /// Computes the cross product of vectors (p2-p1) and (p3-p1)
    fn ccw(p1: &Point, p2: &Point, p3: &Point) -> f64 {
        (p2.x - p1.x) * (p3.y - p1.y) - (p2.y - p1.y) * (p3.x - p1.x)
    }

    let ccw1 = ccw(a1, a2, b1);
    let ccw2 = ccw(a1, a2, b2);
    let ccw3 = ccw(b1, b2, a1);
    let ccw4 = ccw(b1, b2, a2);

    // Check if the segments straddle each other
    if ccw1 * ccw2 < 0.0 && ccw3 * ccw4 < 0.0 {
        return true;
    }

    // Check for collinear cases
    const EPSILON: f64 = 1e-10;
    if ccw1.abs() < EPSILON && on_segment(a1, b1, a2) {
        return true;
    }
    if ccw2.abs() < EPSILON && on_segment(a1, b2, a2) {
        return true;
    }
    if ccw3.abs() < EPSILON && on_segment(b1, a1, b2) {
        return true;
    }
    if ccw4.abs() < EPSILON && on_segment(b1, a2, b2) {
        return true;
    }

    false
}

/// Checks if point q lies on segment pr
fn on_segment(p: &Point, q: &Point, r: &Point) -> bool {
    q.x <= p.x.max(r.x)
        && q.x >= p.x.min(r.x)
        && q.y <= p.y.max(r.y)
        && q.y >= p.y.min(r.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility_graph_creation() {
        let mut graph = VisibilityGraph::new();
        let v1 = graph.add_vertex(Point::new(0.0, 0.0));
        let v2 = graph.add_vertex(Point::new(10.0, 10.0));

        assert_eq!(graph.vertex_count(), 2);
        assert!(graph.get_vertex(v1).is_some());
        assert!(graph.get_vertex(v2).is_some());
    }

    #[test]
    fn test_add_edge() {
        let mut graph = VisibilityGraph::new();
        let v1 = graph.add_vertex(Point::new(0.0, 0.0));
        let v2 = graph.add_vertex(Point::new(10.0, 0.0));

        graph.add_edge(v1, v2, true);

        let vertex = graph.get_vertex(v1).unwrap();
        assert_eq!(vertex.edges.len(), 1);
        assert_eq!(vertex.edges[0].target_id, v2);
        assert_eq!(vertex.edges[0].weight, 10.0);
        assert!(vertex.edges[0].orthogonal);
    }

    #[test]
    fn test_is_orthogonal() {
        assert!(is_orthogonal(
            &Point::new(0.0, 0.0),
            &Point::new(10.0, 0.0)
        ));
        assert!(is_orthogonal(
            &Point::new(0.0, 0.0),
            &Point::new(0.0, 10.0)
        ));
        assert!(!is_orthogonal(
            &Point::new(0.0, 0.0),
            &Point::new(10.0, 10.0)
        ));
    }

    #[test]
    fn test_segments_intersect() {
        let a1 = Point::new(0.0, 0.0);
        let a2 = Point::new(10.0, 10.0);
        let b1 = Point::new(0.0, 10.0);
        let b2 = Point::new(10.0, 0.0);

        assert!(segments_intersect(&a1, &a2, &b1, &b2));

        let c1 = Point::new(0.0, 0.0);
        let c2 = Point::new(5.0, 5.0);
        let d1 = Point::new(10.0, 0.0);
        let d2 = Point::new(15.0, 5.0);

        assert!(!segments_intersect(&c1, &c2, &d1, &d2));
    }
}
