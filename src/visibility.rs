//! Visibility graph computation for pathfinding
//!
//! This module implements visibility graph algorithms that determine which
//! vertices can "see" each other without obstruction.

use crate::geometry::{Point, Polygon, PolygonInterface, segment_intersects_polygon_interior};
use crate::obstacle::Obstacle;
use std::collections::{HashMap, HashSet};

// ============================================================================
// Type Aliases
// ============================================================================

/// Vertex ID in visibility graph
pub type VertexId = u32;

/// Edge ID in visibility graph
pub type EdgeId = u32;

/// Obstacle ID reference
pub type ObstacleId = u32;

/// Connector ID reference
pub type ConnectorId = u32;

// ============================================================================
// Enums
// ============================================================================

/// Type of vertex in the visibility graph
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertexType {
    /// Normal vertex (generic)
    Normal,
    /// Shape corner vertex
    ShapeCorner,
    /// Connector endpoint vertex
    ConnectorEnd,
    /// Checkpoint/waypoint vertex
    Checkpoint,
    /// Dummy vertex for orthogonal routing
    OrthogonalDummy,
}

impl Default for VertexType {
    fn default() -> Self {
        VertexType::Normal
    }
}

/// Direction for orthogonal edges
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

// ============================================================================
// Search State (for A*)
// ============================================================================

/// A* search state for a vertex
#[derive(Debug, Clone, Default)]
pub struct SearchState {
    /// Cost from source (g-score)
    pub g_score: f64,
    /// Estimated total cost (f-score = g + h)
    pub f_score: f64,
    /// Previous vertex in path
    pub came_from: Option<VertexId>,
    /// Direction we arrived from (for angle penalties)
    pub came_from_direction: Option<Point>,
    /// Search generation (for reuse without clearing)
    pub generation: u32,
}

impl SearchState {
    /// Creates a new search state
    pub fn new() -> Self {
        SearchState {
            g_score: f64::INFINITY,
            f_score: f64::INFINITY,
            came_from: None,
            came_from_direction: None,
            generation: 0,
        }
    }

    /// Resets the search state for a new search
    pub fn reset(&mut self) {
        self.g_score = f64::INFINITY;
        self.f_score = f64::INFINITY;
        self.came_from = None;
        self.came_from_direction = None;
    }

    /// Checks if this state is valid for the current generation
    pub fn is_current(&self, current_generation: u32) -> bool {
        self.generation == current_generation
    }
}

// ============================================================================
// Edge Information
// ============================================================================

/// Edge information in the visibility graph
#[derive(Debug, Clone)]
pub struct EdgeInf {
    /// Unique edge ID
    pub id: EdgeId,
    /// Target vertex ID
    pub target_id: VertexId,
    /// Edge weight (distance)
    pub distance: f64,
    /// Whether this is an orthogonal edge (H or V)
    pub orthogonal: bool,
    /// Direction for orthogonal edges
    pub direction: Option<Direction>,
    /// Connectors currently using this edge
    pub using_connectors: HashSet<ConnectorId>,
    /// Edge is blocked by an obstacle
    pub blocked: bool,
}

impl EdgeInf {
    /// Creates a new edge
    pub fn new(id: EdgeId, target_id: VertexId, distance: f64) -> Self {
        EdgeInf {
            id,
            target_id,
            distance,
            orthogonal: false,
            direction: None,
            using_connectors: HashSet::new(),
            blocked: false,
        }
    }

    /// Creates an orthogonal edge
    pub fn orthogonal(id: EdgeId, target_id: VertexId, distance: f64, direction: Direction) -> Self {
        EdgeInf {
            id,
            target_id,
            distance,
            orthogonal: true,
            direction: Some(direction),
            using_connectors: HashSet::new(),
            blocked: false,
        }
    }

    /// Marks this edge as used by a connector
    pub fn add_connector(&mut self, conn_id: ConnectorId) {
        self.using_connectors.insert(conn_id);
    }

    /// Removes a connector from this edge
    pub fn remove_connector(&mut self, conn_id: ConnectorId) {
        self.using_connectors.remove(&conn_id);
    }

    /// Returns the number of connectors using this edge
    pub fn connector_count(&self) -> usize {
        self.using_connectors.len()
    }
}

// ============================================================================
// Vertex Information
// ============================================================================

/// Vertex information in the visibility graph
#[derive(Debug, Clone)]
pub struct VertInf {
    /// Unique vertex ID
    pub id: VertexId,
    /// The point location
    pub point: Point,
    /// Type of vertex
    pub vertex_type: VertexType,
    /// Owning obstacle ID (for shape corner vertices)
    pub obstacle_id: Option<ObstacleId>,
    /// Shape edge index before this vertex (for corners)
    pub shape_edge_before: Option<usize>,
    /// Shape edge index after this vertex (for corners)
    pub shape_edge_after: Option<usize>,
    /// Connection pin ID (for endpoint vertices)
    pub connection_pin: Option<u32>,
    /// Visibility edges to other vertices
    pub edges: Vec<EdgeInf>,
    /// Orthogonal-only edges (for orthogonal routing)
    pub orthogonal_edges: Vec<EdgeInf>,
    /// A* search state
    pub search_state: SearchState,
    /// Whether this vertex is active
    pub active: bool,
}

impl VertInf {
    /// Creates a new basic vertex
    pub fn new(id: VertexId, point: Point) -> Self {
        VertInf {
            id,
            point,
            vertex_type: VertexType::Normal,
            obstacle_id: None,
            shape_edge_before: None,
            shape_edge_after: None,
            connection_pin: None,
            edges: Vec::new(),
            orthogonal_edges: Vec::new(),
            search_state: SearchState::new(),
            active: true,
        }
    }

    /// Creates a shape corner vertex
    pub fn shape_corner(
        id: VertexId,
        point: Point,
        obstacle_id: ObstacleId,
        edge_before: usize,
        edge_after: usize,
    ) -> Self {
        VertInf {
            id,
            point,
            vertex_type: VertexType::ShapeCorner,
            obstacle_id: Some(obstacle_id),
            shape_edge_before: Some(edge_before),
            shape_edge_after: Some(edge_after),
            connection_pin: None,
            edges: Vec::new(),
            orthogonal_edges: Vec::new(),
            search_state: SearchState::new(),
            active: true,
        }
    }

    /// Creates a connector endpoint vertex
    pub fn connector_end(id: VertexId, point: Point, pin_id: Option<u32>) -> Self {
        VertInf {
            id,
            point,
            vertex_type: VertexType::ConnectorEnd,
            obstacle_id: None,
            shape_edge_before: None,
            shape_edge_after: None,
            connection_pin: pin_id,
            edges: Vec::new(),
            orthogonal_edges: Vec::new(),
            search_state: SearchState::new(),
            active: true,
        }
    }

    /// Creates a checkpoint vertex
    pub fn checkpoint(id: VertexId, point: Point) -> Self {
        VertInf {
            id,
            point,
            vertex_type: VertexType::Checkpoint,
            obstacle_id: None,
            shape_edge_before: None,
            shape_edge_after: None,
            connection_pin: None,
            edges: Vec::new(),
            orthogonal_edges: Vec::new(),
            search_state: SearchState::new(),
            active: true,
        }
    }

    /// Adds an edge to another vertex
    pub fn add_edge(&mut self, edge: EdgeInf) {
        if edge.orthogonal {
            self.orthogonal_edges.push(edge);
        } else {
            self.edges.push(edge);
        }
    }

    /// Adds a simple edge (backwards compatible)
    pub fn add_simple_edge(&mut self, target_id: VertexId, weight: f64, orthogonal: bool) {
        // Generate a simple edge ID from target
        let edge = EdgeInf {
            id: target_id, // Simple ID scheme
            target_id,
            distance: weight,
            orthogonal,
            direction: None,
            using_connectors: HashSet::new(),
            blocked: false,
        };
        self.add_edge(edge);
    }

    /// Removes all edges to a specific vertex
    pub fn remove_edges_to(&mut self, target_id: VertexId) {
        self.edges.retain(|e| e.target_id != target_id);
        self.orthogonal_edges.retain(|e| e.target_id != target_id);
    }

    /// Gets all edges (both regular and orthogonal)
    pub fn all_edges(&self) -> impl Iterator<Item = &EdgeInf> {
        self.edges.iter().chain(self.orthogonal_edges.iter())
    }

    /// Resets search state for a new search
    pub fn reset_search(&mut self) {
        self.search_state.reset();
    }
}

// ============================================================================
// Legacy type alias for backwards compatibility
// ============================================================================

/// Backwards compatible alias
pub type VertexInfo = VertInf;

/// Backwards compatible EdgeInfo
#[derive(Debug, Clone, Copy)]
pub struct EdgeInfo {
    /// Target vertex ID
    pub target_id: u32,
    /// Edge weight (distance)
    pub weight: f64,
    /// Whether this is an orthogonal edge
    pub orthogonal: bool,
}

// ============================================================================
// Visibility Graph
// ============================================================================

/// Visibility graph for pathfinding
#[derive(Debug)]
pub struct VisibilityGraph {
    /// Map from vertex ID to vertex info
    vertices: HashMap<VertexId, VertInf>,
    /// Next available vertex ID
    next_vertex_id: VertexId,
    /// Next available edge ID
    next_edge_id: EdgeId,
    /// Current search generation (incremented each search)
    search_generation: u32,
}

impl VisibilityGraph {
    /// Creates a new empty visibility graph
    pub fn new() -> Self {
        VisibilityGraph {
            vertices: HashMap::new(),
            next_vertex_id: 1,
            next_edge_id: 1,
            search_generation: 0,
        }
    }

    /// Adds a vertex to the graph
    pub fn add_vertex(&mut self, point: Point) -> VertexId {
        let id = self.next_vertex_id;
        self.next_vertex_id += 1;

        let vertex = VertInf::new(id, point);
        self.vertices.insert(id, vertex);
        id
    }

    /// Adds a vertex with a specific ID
    pub fn add_vertex_with_id(&mut self, point: Point, id: VertexId) {
        let vertex = VertInf::new(id, point);
        self.vertices.insert(id, vertex);

        if id >= self.next_vertex_id {
            self.next_vertex_id = id + 1;
        }
    }

    /// Adds a shape corner vertex
    pub fn add_shape_corner(
        &mut self,
        point: Point,
        obstacle_id: ObstacleId,
        edge_before: usize,
        edge_after: usize,
    ) -> VertexId {
        let id = self.next_vertex_id;
        self.next_vertex_id += 1;

        let vertex = VertInf::shape_corner(id, point, obstacle_id, edge_before, edge_after);
        self.vertices.insert(id, vertex);
        id
    }

    /// Adds a connector endpoint vertex
    pub fn add_connector_end(&mut self, point: Point, pin_id: Option<u32>) -> VertexId {
        let id = self.next_vertex_id;
        self.next_vertex_id += 1;

        let vertex = VertInf::connector_end(id, point, pin_id);
        self.vertices.insert(id, vertex);
        id
    }

    /// Removes a vertex from the graph
    pub fn remove_vertex(&mut self, id: VertexId) {
        self.vertices.remove(&id);

        // Remove all edges pointing to this vertex
        for vertex in self.vertices.values_mut() {
            vertex.remove_edges_to(id);
        }
    }

    /// Gets a vertex by ID
    pub fn get_vertex(&self, id: VertexId) -> Option<&VertInf> {
        self.vertices.get(&id)
    }

    /// Gets a mutable vertex by ID
    pub fn get_vertex_mut(&mut self, id: VertexId) -> Option<&mut VertInf> {
        self.vertices.get_mut(&id)
    }

    /// Adds an edge between two vertices
    pub fn add_edge(&mut self, from_id: VertexId, to_id: VertexId, orthogonal: bool) {
        // First, get the necessary information without holding borrows
        let edge_info = {
            let from = self.vertices.get(&from_id);
            let to = self.vertices.get(&to_id);

            match (from, to) {
                (Some(from_vertex), Some(to_vertex)) => {
                    let distance = from_vertex.point.distance(&to_vertex.point);
                    let direction = if orthogonal {
                        Some(compute_direction(&from_vertex.point, &to_vertex.point))
                    } else {
                        None
                    };
                    Some((distance, direction))
                }
                _ => None,
            }
        };

        // Now apply the edge with a fresh mutable borrow
        if let Some((distance, direction)) = edge_info {
            let edge_id = self.next_edge_id;
            self.next_edge_id += 1;

            if let Some(from_vertex) = self.vertices.get_mut(&from_id) {
                let edge = EdgeInf {
                    id: edge_id,
                    target_id: to_id,
                    distance,
                    orthogonal,
                    direction,
                    using_connectors: HashSet::new(),
                    blocked: false,
                };
                from_vertex.add_edge(edge);
            }
        }
    }

    /// Computes visibility between a vertex and all other vertices.
    /// Adds bidirectional edges so pathfinding can traverse in both directions.
    pub fn compute_vertex_visibility(
        &mut self,
        vertex_id: VertexId,
        obstacles: &[&dyn Obstacle],
    ) {
        let vertex = match self.vertices.get(&vertex_id) {
            Some(v) => v.clone(),
            None => return,
        };

        // Collect all visible edges first without holding mutable borrows
        // Store: (other_id, other_point, distance, orthogonal, forward_direction, reverse_direction)
        let mut edges_to_add: Vec<(VertexId, Point, f64, bool, Option<Direction>, Option<Direction>)> = Vec::new();

        // Check visibility to all other vertices
        for other_id in self.vertices.keys().copied().collect::<Vec<_>>() {
            if other_id == vertex_id {
                continue;
            }

            let other_point = match self.vertices.get(&other_id) {
                Some(v) => v.point,
                None => continue,
            };

            if self.is_visible(&vertex.point, &other_point, obstacles) {
                let distance = vertex.point.distance(&other_point);
                let orthogonal = is_orthogonal(&vertex.point, &other_point);
                let forward_direction = if orthogonal {
                    Some(compute_direction(&vertex.point, &other_point))
                } else {
                    None
                };
                let reverse_direction = if orthogonal {
                    Some(compute_direction(&other_point, &vertex.point))
                } else {
                    None
                };
                edges_to_add.push((other_id, other_point, distance, orthogonal, forward_direction, reverse_direction));
            }
        }

        // Now add all the edges (bidirectional)
        for (target_id, _target_point, distance, orthogonal, forward_direction, reverse_direction) in edges_to_add {
            // Add forward edge: vertex_id -> target_id
            let forward_edge_id = self.next_edge_id;
            self.next_edge_id += 1;

            if let Some(v) = self.vertices.get_mut(&vertex_id) {
                let edge = EdgeInf {
                    id: forward_edge_id,
                    target_id,
                    distance,
                    orthogonal,
                    direction: forward_direction,
                    using_connectors: HashSet::new(),
                    blocked: false,
                };
                v.add_edge(edge);
            }

            // Add reverse edge: target_id -> vertex_id
            // Only add if target doesn't already have an edge to vertex_id
            let has_reverse = self.vertices.get(&target_id)
                .map(|v| v.all_edges().any(|e| e.target_id == vertex_id))
                .unwrap_or(false);

            if !has_reverse {
                let reverse_edge_id = self.next_edge_id;
                self.next_edge_id += 1;

                if let Some(v) = self.vertices.get_mut(&target_id) {
                    let edge = EdgeInf {
                        id: reverse_edge_id,
                        target_id: vertex_id,
                        distance,
                        orthogonal,
                        direction: reverse_direction,
                        using_connectors: HashSet::new(),
                        blocked: false,
                    };
                    v.add_edge(edge);
                }
            }
        }
    }

    /// Checks if two points are visible to each other.
    /// Uses proper polygon interior intersection test.
    pub fn is_visible(&self, p1: &Point, p2: &Point, obstacles: &[&dyn Obstacle]) -> bool {
        // Check if the edge intersects any obstacle interior
        for obstacle in obstacles {
            if !obstacle.is_active() {
                continue;
            }

            // Use the proper polygon intersection test that handles
            // corner touches correctly (corner touches are allowed for visibility)
            if segment_intersects_polygon_interior(p1, p2, obstacle.polygon()) {
                return false;
            }
        }

        true
    }

    /// Returns an iterator over all vertices
    pub fn vertices(&self) -> impl Iterator<Item = &VertInf> {
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
        self.next_edge_id = 1;
    }

    /// Finds a vertex at the given position (within tolerance)
    pub fn find_vertex_at(&self, point: &Point) -> Option<VertexId> {
        const TOLERANCE: f64 = 1e-6;
        for vertex in self.vertices.values() {
            if (vertex.point.x - point.x).abs() < TOLERANCE
                && (vertex.point.y - point.y).abs() < TOLERANCE
            {
                return Some(vertex.id);
            }
        }
        None
    }

    /// Removes all edges from and to a vertex (but keeps the vertex)
    pub fn remove_edges_for_vertex(&mut self, vertex_id: VertexId) {
        // Remove outgoing edges from this vertex
        if let Some(vertex) = self.vertices.get_mut(&vertex_id) {
            vertex.edges.clear();
            vertex.orthogonal_edges.clear();
        }

        // Remove incoming edges to this vertex from all other vertices
        for (id, vertex) in self.vertices.iter_mut() {
            if *id != vertex_id {
                vertex.remove_edges_to(vertex_id);
            }
        }
    }

    /// Prepares for a new A* search by incrementing the generation counter
    pub fn prepare_for_search(&mut self) {
        self.search_generation += 1;
    }

    /// Gets the current search generation
    pub fn search_generation(&self) -> u32 {
        self.search_generation
    }

    /// Resets all vertex search states (alternative to generation-based approach)
    pub fn reset_search_states(&mut self) {
        for vertex in self.vertices.values_mut() {
            vertex.reset_search();
        }
    }

    /// Computes visibility using Lee's sweep algorithm (O(n log n)).
    /// This is more efficient than the naive O(n²) approach for dense graphs.
    ///
    /// Based on: Lee, D.-T. (1978). Proximity and reachability in the plane.
    /// PhD thesis, Department of Electrical Engineering, University of Illinois.
    pub fn compute_vertex_visibility_sweep(
        &mut self,
        vertex_id: VertexId,
        obstacles: &[&dyn Obstacle],
    ) {
        use crate::geometry::{
            rotational_angle, vec_dir, point_on_line, ray_intersect_point,
            segments_intersect, in_valid_region, DO_INTERSECT, VEC_DIR_AHEAD, VEC_DIR_BEHIND,
        };
        use std::collections::BTreeSet;

        let center = match self.vertices.get(&vertex_id) {
            Some(v) => v.point,
            None => return,
        };

        // Collect all other vertices sorted by angle from center
        let mut sorted_vertices: Vec<SweepPoint> = Vec::new();

        for (&other_id, other_vertex) in &self.vertices {
            if other_id == vertex_id {
                continue;
            }

            let relative = Point::new(
                other_vertex.point.x - center.x,
                other_vertex.point.y - center.y,
            );
            let angle = rotational_angle(&relative);
            let distance = center.distance(&other_vertex.point);

            sorted_vertices.push(SweepPoint {
                vertex_id: other_id,
                point: other_vertex.point,
                angle,
                distance,
            });
        }

        // Sort by angle, then by distance
        sorted_vertices.sort_by(|a, b| {
            a.angle.partial_cmp(&b.angle)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.distance.partial_cmp(&b.distance)
                    .unwrap_or(std::cmp::Ordering::Equal))
        });

        // Collect obstacle edges for the sweep
        let mut obstacle_edges: Vec<SweepEdge> = Vec::new();
        for obstacle in obstacles {
            if !obstacle.is_active() {
                continue;
            }
            let poly = obstacle.polygon();
            let n = poly.size();
            if n < 2 {
                continue;
            }

            for i in 0..n {
                let p1 = *poly.at(i);
                let p2 = *poly.at((i + 1) % n);
                obstacle_edges.push(SweepEdge {
                    p1,
                    p2,
                    obstacle_id: obstacle.id(),
                });
            }
        }

        // Initialize the sweep tree with edges crossing the initial ray (positive x-axis)
        let x_axis = Point::new(center.x + 1e10, center.y);
        let mut active_edges: Vec<SweepEdge> = Vec::new();

        for edge in &obstacle_edges {
            // Check if edge crosses the initial ray
            if segments_intersect(&center, &x_axis, &edge.p1, &edge.p2) {
                active_edges.push(edge.clone());
            }
        }

        // Find edges each vertex belongs to (for add/remove during sweep)
        let mut vertex_edges: HashMap<(u32, u32), Vec<SweepEdge>> = HashMap::new();
        for edge in &obstacle_edges {
            // We'll track edges by their endpoint vertex IDs if available
            // For now, we use position-based matching
        }

        // Sweep through all vertices
        let mut edges_to_add: Vec<(VertexId, f64)> = Vec::new();

        for sweep_point in &sorted_vertices {
            // Update active edges: remove edges ending at this angle, add edges starting
            // For a proper sweep, we'd need to track edge events more carefully
            // This is a simplified version

            // Check visibility: is this point closer than the nearest blocking edge?
            let visible = self.sweep_check_visible(
                &center,
                &sweep_point.point,
                sweep_point.distance,
                &active_edges,
            );

            if visible {
                edges_to_add.push((sweep_point.vertex_id, sweep_point.distance));
            }

            // Update active edges for points on obstacle boundaries
            // (Add edges starting at this point, remove edges ending at this point)
            self.sweep_update_active_edges(
                &center,
                &sweep_point.point,
                sweep_point.angle,
                &obstacle_edges,
                &mut active_edges,
            );
        }

        // Add all visible edges (bidirectional)
        for (target_id, distance) in edges_to_add {
            let forward_edge_id = self.next_edge_id;
            self.next_edge_id += 1;

            if let Some(v) = self.vertices.get_mut(&vertex_id) {
                let edge = EdgeInf {
                    id: forward_edge_id,
                    target_id,
                    distance,
                    orthogonal: false,
                    direction: None,
                    using_connectors: HashSet::new(),
                    blocked: false,
                };
                v.add_edge(edge);
            }

            // Add reverse edge if not already present
            let has_reverse = self.vertices.get(&target_id)
                .map(|v| v.all_edges().any(|e| e.target_id == vertex_id))
                .unwrap_or(false);

            if !has_reverse {
                let reverse_edge_id = self.next_edge_id;
                self.next_edge_id += 1;

                if let Some(v) = self.vertices.get_mut(&target_id) {
                    let edge = EdgeInf {
                        id: reverse_edge_id,
                        target_id: vertex_id,
                        distance,
                        orthogonal: false,
                        direction: None,
                        using_connectors: HashSet::new(),
                        blocked: false,
                    };
                    v.add_edge(edge);
                }
            }
        }
    }

    /// Helper: Check if a point is visible given the current active edges
    fn sweep_check_visible(
        &self,
        center: &Point,
        target: &Point,
        target_distance: f64,
        active_edges: &[SweepEdge],
    ) -> bool {
        use crate::geometry::{ray_intersect_point, DO_INTERSECT};

        if active_edges.is_empty() {
            return true;
        }

        // Find the closest blocking edge along the ray to target
        let mut closest_dist = f64::INFINITY;

        for edge in active_edges {
            // Skip if target is on this edge
            if edge.p1.equals(target) || edge.p2.equals(target) {
                continue;
            }

            // Find where the ray from center to target intersects this edge
            let (result, intersection) = ray_intersect_point(&edge.p1, &edge.p2, center, target);

            if result == DO_INTERSECT {
                let dist = center.distance(&intersection);
                if dist < closest_dist {
                    closest_dist = dist;
                }
            }
        }

        // Target is visible if it's closer than any blocking edge
        target_distance < closest_dist - 1e-9
    }

    /// Helper: Update active edges during sweep
    fn sweep_update_active_edges(
        &self,
        center: &Point,
        current_point: &Point,
        _current_angle: f64,
        all_edges: &[SweepEdge],
        active_edges: &mut Vec<SweepEdge>,
    ) {
        use crate::geometry::{vec_dir, VEC_DIR_AHEAD, VEC_DIR_BEHIND};

        // For each edge connected to current_point, add or remove from active set
        for edge in all_edges {
            let is_p1 = edge.p1.equals(current_point);
            let is_p2 = edge.p2.equals(current_point);

            if !is_p1 && !is_p2 {
                continue;
            }

            // Determine if the other endpoint is ahead or behind the sweep ray
            let other = if is_p1 { &edge.p2 } else { &edge.p1 };
            let x_ray = Point::new(center.x + 1e10, center.y);
            let dir = vec_dir(center, current_point, other);

            if dir == VEC_DIR_BEHIND {
                // Edge is ending, remove from active set
                active_edges.retain(|e| !(e.p1.equals(&edge.p1) && e.p2.equals(&edge.p2)));
            } else if dir == VEC_DIR_AHEAD {
                // Edge is starting, add to active set
                if !active_edges.iter().any(|e| e.p1.equals(&edge.p1) && e.p2.equals(&edge.p2)) {
                    active_edges.push(edge.clone());
                }
            }
        }
    }
}

/// Point sorted by angle for sweep algorithm
#[derive(Debug, Clone)]
struct SweepPoint {
    vertex_id: VertexId,
    point: Point,
    angle: f64,
    distance: f64,
}

/// Edge in the sweep tree
#[derive(Debug, Clone)]
struct SweepEdge {
    p1: Point,
    p2: Point,
    obstacle_id: u32,
}

impl Default for VisibilityGraph {
    fn default() -> Self {
        VisibilityGraph::new()
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Checks if two points form an orthogonal (horizontal or vertical) line
pub fn is_orthogonal(p1: &Point, p2: &Point) -> bool {
    const EPSILON: f64 = 1e-6;
    (p1.x - p2.x).abs() < EPSILON || (p1.y - p2.y).abs() < EPSILON
}

/// Computes the direction from p1 to p2 (for orthogonal edges)
fn compute_direction(p1: &Point, p2: &Point) -> Direction {
    const EPSILON: f64 = 1e-6;

    if (p1.x - p2.x).abs() < EPSILON {
        // Vertical
        if p2.y > p1.y {
            Direction::Down // Y increases downward typically
        } else {
            Direction::Up
        }
    } else {
        // Horizontal
        if p2.x > p1.x {
            Direction::Right
        } else {
            Direction::Left
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

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
        assert_eq!(vertex.orthogonal_edges.len(), 1);
        assert_eq!(vertex.orthogonal_edges[0].target_id, v2);
        assert_eq!(vertex.orthogonal_edges[0].distance, 10.0);
        assert!(vertex.orthogonal_edges[0].orthogonal);
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
    fn test_vertex_types() {
        let v1 = VertInf::new(1, Point::new(0.0, 0.0));
        assert_eq!(v1.vertex_type, VertexType::Normal);

        let v2 = VertInf::shape_corner(2, Point::new(10.0, 0.0), 1, 0, 1);
        assert_eq!(v2.vertex_type, VertexType::ShapeCorner);
        assert_eq!(v2.obstacle_id, Some(1));

        let v3 = VertInf::connector_end(3, Point::new(20.0, 0.0), Some(5));
        assert_eq!(v3.vertex_type, VertexType::ConnectorEnd);
        assert_eq!(v3.connection_pin, Some(5));
    }

    #[test]
    fn test_search_state() {
        let mut state = SearchState::new();
        assert_eq!(state.g_score, f64::INFINITY);
        assert_eq!(state.came_from, None);

        state.g_score = 10.0;
        state.came_from = Some(5);
        state.generation = 1;

        assert!(state.is_current(1));
        assert!(!state.is_current(2));

        state.reset();
        assert_eq!(state.g_score, f64::INFINITY);
        assert_eq!(state.came_from, None);
    }

    #[test]
    fn test_search_generation() {
        let mut graph = VisibilityGraph::new();
        assert_eq!(graph.search_generation(), 0);

        graph.prepare_for_search();
        assert_eq!(graph.search_generation(), 1);

        graph.prepare_for_search();
        assert_eq!(graph.search_generation(), 2);
    }

    #[test]
    fn test_edge_connectors() {
        let mut edge = EdgeInf::new(1, 2, 10.0);
        assert_eq!(edge.connector_count(), 0);

        edge.add_connector(100);
        edge.add_connector(101);
        assert_eq!(edge.connector_count(), 2);

        edge.remove_connector(100);
        assert_eq!(edge.connector_count(), 1);
    }
}
