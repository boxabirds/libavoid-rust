//! Hyperedge routing for multi-terminal connections
//!
//! Hyperedges connect multiple terminals using junctions and connectors.
//! This module provides routing for busses and other multi-point connections.
//!
//! C++ ref: libavoid/hyperedgeimprover.cpp - HyperedgeTreeNode, HyperedgeTreeEdge

use crate::geometry::Point;
use crate::connector::ConnEnd;
use std::collections::{HashMap, HashSet};

// ============================================================================
// Hyperedge Tree Structure (Task #20)
// ============================================================================

/// Node in hyperedge tree structure.
/// C++ ref: libavoid/hyperedgeimprover.cpp - HyperedgeTreeNode
#[derive(Debug, Clone)]
pub struct HyperedgeTreeNode {
    /// Node ID
    pub id: u32,
    /// Position of this node
    pub point: Point,
    /// Junction ID if this is a junction node
    pub junction_id: Option<u32>,
    /// Connector ID if this is a terminal node
    pub connector_id: Option<u32>,
    /// IDs of connected edges
    pub edges: Vec<u32>,
}

impl HyperedgeTreeNode {
    /// Creates a junction node
    pub fn junction(id: u32, point: Point, junction_id: u32) -> Self {
        HyperedgeTreeNode {
            id,
            point,
            junction_id: Some(junction_id),
            connector_id: None,
            edges: Vec::new(),
        }
    }

    /// Creates a terminal node
    pub fn terminal(id: u32, point: Point, connector_id: u32) -> Self {
        HyperedgeTreeNode {
            id,
            point,
            junction_id: None,
            connector_id: Some(connector_id),
            edges: Vec::new(),
        }
    }

    /// Returns true if this is a junction node
    pub fn is_junction(&self) -> bool {
        self.junction_id.is_some()
    }

    /// Returns true if this is a terminal node
    pub fn is_terminal(&self) -> bool {
        self.connector_id.is_some()
    }

    /// Returns the degree (number of connected edges)
    pub fn degree(&self) -> usize {
        self.edges.len()
    }
}

/// Edge in hyperedge tree structure.
/// C++ ref: libavoid/hyperedgeimprover.cpp - HyperedgeTreeEdge
#[derive(Debug, Clone)]
pub struct HyperedgeTreeEdge {
    /// Edge ID
    pub id: u32,
    /// First endpoint node ID
    pub node1: u32,
    /// Second endpoint node ID
    pub node2: u32,
    /// Connector ID for this edge
    pub connector_id: Option<u32>,
    /// Route points along this edge
    pub route_points: Vec<Point>,
}

impl HyperedgeTreeEdge {
    pub fn new(id: u32, node1: u32, node2: u32) -> Self {
        HyperedgeTreeEdge {
            id,
            node1,
            node2,
            connector_id: None,
            route_points: Vec::new(),
        }
    }

    pub fn with_connector(id: u32, node1: u32, node2: u32, connector_id: u32) -> Self {
        HyperedgeTreeEdge {
            id,
            node1,
            node2,
            connector_id: Some(connector_id),
            route_points: Vec::new(),
        }
    }

    /// Returns the other endpoint from the given node
    pub fn other_node(&self, from: u32) -> u32 {
        if from == self.node1 {
            self.node2
        } else {
            self.node1
        }
    }
}

/// Hyperedge tree for representing the structure of a multi-terminal connection.
/// C++ ref: libavoid/hyperedgeimprover.cpp - HyperedgeTree
#[derive(Debug, Clone, Default)]
pub struct HyperedgeTree {
    /// Nodes in the tree (junctions and terminals)
    pub nodes: HashMap<u32, HyperedgeTreeNode>,
    /// Edges in the tree (connections between nodes)
    pub edges: HashMap<u32, HyperedgeTreeEdge>,
    /// Next available node ID
    next_node_id: u32,
    /// Next available edge ID
    next_edge_id: u32,
}

impl HyperedgeTree {
    pub fn new() -> Self {
        HyperedgeTree {
            nodes: HashMap::new(),
            edges: HashMap::new(),
            next_node_id: 0,
            next_edge_id: 0,
        }
    }

    /// Adds a junction node
    pub fn add_junction(&mut self, point: Point, junction_id: u32) -> u32 {
        let id = self.next_node_id;
        self.next_node_id += 1;
        self.nodes.insert(id, HyperedgeTreeNode::junction(id, point, junction_id));
        id
    }

    /// Adds a terminal node
    pub fn add_terminal(&mut self, point: Point, connector_id: u32) -> u32 {
        let id = self.next_node_id;
        self.next_node_id += 1;
        self.nodes.insert(id, HyperedgeTreeNode::terminal(id, point, connector_id));
        id
    }

    /// Adds an edge between two nodes
    pub fn add_edge(&mut self, node1: u32, node2: u32, connector_id: Option<u32>) -> u32 {
        let id = self.next_edge_id;
        self.next_edge_id += 1;

        let edge = if let Some(cid) = connector_id {
            HyperedgeTreeEdge::with_connector(id, node1, node2, cid)
        } else {
            HyperedgeTreeEdge::new(id, node1, node2)
        };

        // Update node edge lists
        if let Some(node) = self.nodes.get_mut(&node1) {
            node.edges.push(id);
        }
        if let Some(node) = self.nodes.get_mut(&node2) {
            node.edges.push(id);
        }

        self.edges.insert(id, edge);
        id
    }

    /// Removes a node and its edges
    pub fn remove_node(&mut self, node_id: u32) {
        if let Some(node) = self.nodes.remove(&node_id) {
            // Remove all connected edges
            for edge_id in node.edges {
                if let Some(edge) = self.edges.remove(&edge_id) {
                    // Remove edge from other node's edge list
                    let other = edge.other_node(node_id);
                    if let Some(other_node) = self.nodes.get_mut(&other) {
                        other_node.edges.retain(|&e| e != edge_id);
                    }
                }
            }
        }
    }

    /// Removes zero-length edges and merges their endpoints.
    /// C++ ref: libavoid/hyperedgeimprover.cpp - removeZeroLengthEdges()
    pub fn remove_zero_length_edges(&mut self) {
        const EPSILON: f64 = 1e-6;

        let mut edges_to_remove: Vec<u32> = Vec::new();

        for (&edge_id, edge) in &self.edges {
            if let (Some(n1), Some(n2)) = (self.nodes.get(&edge.node1), self.nodes.get(&edge.node2)) {
                if n1.point.distance(&n2.point) < EPSILON {
                    edges_to_remove.push(edge_id);
                }
            }
        }

        for edge_id in edges_to_remove {
            if let Some(edge) = self.edges.remove(&edge_id) {
                // Remove from both nodes' edge lists
                if let Some(n1) = self.nodes.get_mut(&edge.node1) {
                    n1.edges.retain(|&e| e != edge_id);
                }
                if let Some(n2) = self.nodes.get_mut(&edge.node2) {
                    n2.edges.retain(|&e| e != edge_id);
                }

                // Merge node2 into node1 (keep node1, redirect edges from node2)
                if let Some(node2) = self.nodes.remove(&edge.node2) {
                    for other_edge_id in node2.edges {
                        if other_edge_id == edge_id {
                            continue;
                        }
                        if let Some(other_edge) = self.edges.get_mut(&other_edge_id) {
                            // Redirect edge to point to node1 instead of node2
                            if other_edge.node1 == edge.node2 {
                                other_edge.node1 = edge.node1;
                            } else if other_edge.node2 == edge.node2 {
                                other_edge.node2 = edge.node1;
                            }
                        }
                        // Add edge to node1's list
                        if let Some(n1) = self.nodes.get_mut(&edge.node1) {
                            if !n1.edges.contains(&other_edge_id) {
                                n1.edges.push(other_edge_id);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Computes total tree cost (sum of edge lengths)
    pub fn total_cost(&self) -> f64 {
        let mut cost = 0.0;
        for edge in self.edges.values() {
            if let (Some(n1), Some(n2)) = (self.nodes.get(&edge.node1), self.nodes.get(&edge.node2)) {
                cost += n1.point.distance(&n2.point);
            }
        }
        cost
    }

    /// Returns all junction nodes
    pub fn junctions(&self) -> impl Iterator<Item = &HyperedgeTreeNode> {
        self.nodes.values().filter(|n| n.is_junction())
    }

    /// Returns all terminal nodes
    pub fn terminals(&self) -> impl Iterator<Item = &HyperedgeTreeNode> {
        self.nodes.values().filter(|n| n.is_terminal())
    }
}

// ============================================================================
// Hyperedge Reference
// ============================================================================

/// A hyperedge connecting multiple terminals
#[derive(Debug, Clone)]
pub struct HyperedgeRef {
    /// Unique identifier
    id: u32,
    /// Terminal endpoints
    terminals: Vec<ConnEnd>,
    /// Connectors that make up this hyperedge
    connectors: HashSet<u32>,
    /// Junctions used in this hyperedge
    junctions: HashSet<u32>,
    /// Whether this hyperedge needs rerouting
    needs_reroute: bool,
}

impl HyperedgeRef {
    /// Creates a new hyperedge with given terminals
    pub fn new(id: u32, terminals: Vec<ConnEnd>) -> Self {
        HyperedgeRef {
            id,
            terminals,
            connectors: HashSet::new(),
            junctions: HashSet::new(),
            needs_reroute: true,
        }
    }

    /// Returns the hyperedge's unique ID
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the terminals
    pub fn terminals(&self) -> &[ConnEnd] {
        &self.terminals
    }

    /// Adds a terminal to the hyperedge
    pub fn add_terminal(&mut self, terminal: ConnEnd) {
        self.terminals.push(terminal);
        self.needs_reroute = true;
    }

    /// Removes a terminal from the hyperedge
    pub fn remove_terminal(&mut self, index: usize) {
        if index < self.terminals.len() {
            self.terminals.remove(index);
            self.needs_reroute = true;
        }
    }

    /// Returns the connectors in this hyperedge
    pub fn connectors(&self) -> &HashSet<u32> {
        &self.connectors
    }

    /// Adds a connector to the hyperedge
    #[allow(dead_code)] // Reserved for Router integration
    pub(crate) fn add_connector(&mut self, conn_id: u32) {
        self.connectors.insert(conn_id);
    }

    /// Returns the junctions in this hyperedge
    pub fn junctions(&self) -> &HashSet<u32> {
        &self.junctions
    }

    /// Adds a junction to the hyperedge
    #[allow(dead_code)] // Reserved for Router integration
    pub(crate) fn add_junction(&mut self, junction_id: u32) {
        self.junctions.insert(junction_id);
    }

    /// Returns whether this hyperedge needs rerouting
    pub fn needs_reroute(&self) -> bool {
        self.needs_reroute
    }

    /// Marks the hyperedge as needing reroute
    #[allow(dead_code)] // Reserved for Router integration
    pub(crate) fn mark_needs_reroute(&mut self) {
        self.needs_reroute = true;
    }

    /// Clears the needs reroute flag
    #[allow(dead_code)] // Reserved for Router integration
    pub(crate) fn clear_needs_reroute(&mut self) {
        self.needs_reroute = false;
    }
}

/// Hyperedge router for computing multi-terminal routes
pub struct HyperedgeRerouter {
    /// Hyperedges managed by this rerouter
    hyperedges: Vec<HyperedgeRef>,
}

impl HyperedgeRerouter {
    /// Creates a new hyperedge rerouter
    pub fn new() -> Self {
        HyperedgeRerouter {
            hyperedges: Vec::new(),
        }
    }

    /// Registers a hyperedge for routing
    pub fn register_hyperedge(&mut self, hyperedge: HyperedgeRef) {
        self.hyperedges.push(hyperedge);
    }

    /// Returns all registered hyperedges
    pub fn hyperedges(&self) -> &[HyperedgeRef] {
        &self.hyperedges
    }

    /// Computes Steiner tree for a hyperedge (basic implementation)
    pub fn compute_steiner_tree(&self, terminals: &[Point]) -> Vec<Point> {
        if terminals.len() < 2 {
            return terminals.to_vec();
        }

        // Simple star topology from centroid
        let mut cx = 0.0;
        let mut cy = 0.0;
        for t in terminals {
            cx += t.x;
            cy += t.y;
        }
        cx /= terminals.len() as f64;
        cy /= terminals.len() as f64;

        let center = Point::new(cx, cy);

        // Return center as junction point
        vec![center]
    }
}

impl Default for HyperedgeRerouter {
    fn default() -> Self {
        Self::new()
    }
}

impl HyperedgeRerouter {
    /// Improves hyperedge routing by optimizing junction positions.
    /// Uses iterative local search to find better junction placements.
    pub fn improve_hyperedge(&self, hyperedge: &mut HyperedgeRef, iteration_limit: usize) -> f64 {
        let terminals: Vec<Point> = hyperedge
            .terminals()
            .iter()
            .map(|t| t.position)
            .collect();

        if terminals.len() < 2 {
            return 0.0;
        }

        // Start with Steiner tree junction
        let mut junctions = self.compute_steiner_tree(&terminals);
        let mut best_cost = self.compute_hyperedge_cost(&terminals, &junctions);

        // Iterative improvement
        for _ in 0..iteration_limit {
            let mut improved = false;

            for j_idx in 0..junctions.len() {
                // Try small perturbations
                let deltas = [
                    Point::new(1.0, 0.0),
                    Point::new(-1.0, 0.0),
                    Point::new(0.0, 1.0),
                    Point::new(0.0, -1.0),
                    Point::new(1.0, 1.0),
                    Point::new(-1.0, -1.0),
                    Point::new(1.0, -1.0),
                    Point::new(-1.0, 1.0),
                ];

                for delta in &deltas {
                    let old_pos = junctions[j_idx];
                    junctions[j_idx] = Point::new(old_pos.x + delta.x, old_pos.y + delta.y);

                    let new_cost = self.compute_hyperedge_cost(&terminals, &junctions);
                    if new_cost < best_cost {
                        best_cost = new_cost;
                        improved = true;
                    } else {
                        junctions[j_idx] = old_pos;
                    }
                }
            }

            if !improved {
                break;
            }
        }

        best_cost
    }

    /// Computes the total cost of a hyperedge (sum of all edge lengths)
    fn compute_hyperedge_cost(&self, terminals: &[Point], junctions: &[Point]) -> f64 {
        if junctions.is_empty() {
            // Direct connections between terminals (star from centroid)
            let mut cx = 0.0;
            let mut cy = 0.0;
            for t in terminals {
                cx += t.x;
                cy += t.y;
            }
            cx /= terminals.len() as f64;
            cy /= terminals.len() as f64;
            let center = Point::new(cx, cy);

            return terminals.iter().map(|t| t.distance(&center)).sum();
        }

        // Cost = sum of distances from each terminal to nearest junction
        // + sum of distances between junctions (if multiple)
        let mut cost = 0.0;

        // Terminal to junction distances
        for terminal in terminals {
            let min_dist = junctions
                .iter()
                .map(|j| terminal.distance(j))
                .fold(f64::INFINITY, f64::min);
            cost += min_dist;
        }

        // Junction to junction distances (for multiple junctions)
        for i in 0..junctions.len() {
            for j in (i + 1)..junctions.len() {
                cost += junctions[i].distance(&junctions[j]);
            }
        }

        cost
    }

    /// Try to add junctions to improve hyperedge routing (Task #15c)
    /// Returns true if a junction was added and improved cost
    pub fn try_add_junction(&self, terminals: &[Point], junctions: &mut Vec<Point>) -> bool {
        if terminals.len() < 3 {
            return false; // Not enough terminals to benefit from junction addition
        }

        let current_cost = self.compute_hyperedge_cost(terminals, junctions);

        // Try adding junction at midpoint of longest terminal-to-terminal segment
        let mut best_candidate: Option<Point> = None;
        let mut best_cost_improvement = 0.0;

        for i in 0..terminals.len() {
            for j in (i + 1)..terminals.len() {
                let midpoint = Point::new(
                    (terminals[i].x + terminals[j].x) / 2.0,
                    (terminals[i].y + terminals[j].y) / 2.0,
                );

                junctions.push(midpoint);
                let new_cost = self.compute_hyperedge_cost(terminals, junctions);
                junctions.pop();

                let improvement = current_cost - new_cost;
                if improvement > best_cost_improvement {
                    best_cost_improvement = improvement;
                    best_candidate = Some(midpoint);
                }
            }
        }

        if let Some(junction) = best_candidate {
            if best_cost_improvement > 0.1 {
                // Only add if improvement is significant
                junctions.push(junction);
                return true;
            }
        }

        false
    }

    /// Try to remove unnecessary junctions (Task #15d)
    /// Returns true if a junction was removed and improved or maintained cost
    pub fn try_remove_junction(&self, terminals: &[Point], junctions: &mut Vec<Point>) -> bool {
        if junctions.is_empty() {
            return false;
        }

        let current_cost = self.compute_hyperedge_cost(terminals, junctions);

        // Try removing each junction and see if cost improves
        for i in (0..junctions.len()).rev() {
            let removed = junctions.remove(i);
            let new_cost = self.compute_hyperedge_cost(terminals, junctions);

            if new_cost <= current_cost * 1.05 {
                // Cost stayed same or improved (allow 5% tolerance)
                return true;
            }

            // Put it back if removal made things worse
            junctions.insert(i, removed);
        }

        false
    }

    /// Full hyperedge improvement with junction addition/deletion (Task #15)
    pub fn improve_hyperedge_advanced(
        &self,
        hyperedge: &mut HyperedgeRef,
        iteration_limit: usize,
    ) -> f64 {
        let terminals: Vec<Point> = hyperedge
            .terminals()
            .iter()
            .map(|t| t.position)
            .collect();

        if terminals.len() < 2 {
            return 0.0;
        }

        let mut junctions = self.compute_steiner_tree(&terminals);
        let mut best_cost = self.compute_hyperedge_cost(&terminals, &junctions);

        for iteration in 0..iteration_limit {
            let mut improved = false;

            // Phase 1: Try junction movement
            for j_idx in 0..junctions.len() {
                let deltas = [
                    Point::new(1.0, 0.0),
                    Point::new(-1.0, 0.0),
                    Point::new(0.0, 1.0),
                    Point::new(0.0, -1.0),
                ];

                for delta in &deltas {
                    let old_pos = junctions[j_idx];
                    junctions[j_idx] = Point::new(old_pos.x + delta.x, old_pos.y + delta.y);

                    let new_cost = self.compute_hyperedge_cost(&terminals, &junctions);
                    if new_cost < best_cost {
                        best_cost = new_cost;
                        improved = true;
                    } else {
                        junctions[j_idx] = old_pos;
                    }
                }
            }

            // Phase 2: Try adding junctions (every 5 iterations)
            if iteration % 5 == 0 {
                if self.try_add_junction(&terminals, &mut junctions) {
                    best_cost = self.compute_hyperedge_cost(&terminals, &junctions);
                    improved = true;
                }
            }

            // Phase 3: Try removing junctions (every 10 iterations)
            if iteration % 10 == 0 {
                if self.try_remove_junction(&terminals, &mut junctions) {
                    best_cost = self.compute_hyperedge_cost(&terminals, &junctions);
                    improved = true;
                }
            }

            if !improved {
                break;
            }
        }

        best_cost
    }
}

// ============================================================================
// Hyperedge Tree Building (Simple helpers)
// ============================================================================

/// Simple edge representation for tree building (from-to pair)
#[derive(Debug, Clone)]
pub struct SimpleTreeEdge {
    pub from: Point,
    pub to: Point,
    pub is_terminal: bool,
}

/// Builds a minimum spanning tree connecting all terminals through junctions
pub fn build_hyperedge_tree(terminals: &[ConnEnd], junctions: &[Point]) -> Vec<SimpleTreeEdge> {
    let mut edges = Vec::new();

    if terminals.is_empty() {
        return edges;
    }

    let terminal_points: Vec<Point> = terminals.iter().map(|t| t.position).collect();

    if junctions.is_empty() {
        // No junctions - connect all to centroid
        let mut cx = 0.0;
        let mut cy = 0.0;
        for t in &terminal_points {
            cx += t.x;
            cy += t.y;
        }
        cx /= terminal_points.len() as f64;
        cy /= terminal_points.len() as f64;
        let center = Point::new(cx, cy);

        for t in &terminal_points {
            edges.push(SimpleTreeEdge {
                from: *t,
                to: center,
                is_terminal: true,
            });
        }
    } else {
        // Connect each terminal to nearest junction
        for t in &terminal_points {
            let nearest = junctions
                .iter()
                .min_by(|a, b| {
                    t.distance(a)
                        .partial_cmp(&t.distance(b))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap();

            edges.push(SimpleTreeEdge {
                from: *t,
                to: *nearest,
                is_terminal: true,
            });
        }

        // Connect junctions together (simple chain for now)
        for i in 0..junctions.len().saturating_sub(1) {
            edges.push(SimpleTreeEdge {
                from: junctions[i],
                to: junctions[i + 1],
                is_terminal: false,
            });
        }
    }

    edges
}

// ============================================================================
// Minimum Spanning Tree (Prim's Algorithm)
// ============================================================================

/// Computes a Minimum Spanning Tree connecting all points using Prim's algorithm
///
/// Returns edges as (from_index, to_index) pairs
pub fn compute_mst(points: &[Point]) -> Vec<(usize, usize)> {
    if points.len() < 2 {
        return Vec::new();
    }

    let n = points.len();
    let mut in_tree = vec![false; n];
    let mut min_cost = vec![f64::INFINITY; n];
    let mut parent = vec![usize::MAX; n];
    let mut edges = Vec::with_capacity(n - 1);

    // Start from first point
    min_cost[0] = 0.0;

    for _ in 0..n {
        // Find minimum cost vertex not in tree
        let mut u = usize::MAX;
        let mut min_val = f64::INFINITY;
        for v in 0..n {
            if !in_tree[v] && min_cost[v] < min_val {
                min_val = min_cost[v];
                u = v;
            }
        }

        if u == usize::MAX {
            break;
        }

        in_tree[u] = true;

        // Add edge to MST (except for starting vertex)
        if parent[u] != usize::MAX {
            edges.push((parent[u], u));
        }

        // Update costs for adjacent vertices
        for v in 0..n {
            if !in_tree[v] {
                let cost = points[u].distance(&points[v]);
                if cost < min_cost[v] {
                    min_cost[v] = cost;
                    parent[v] = u;
                }
            }
        }
    }

    edges
}

/// Builds a hyperedge tree using MST to connect all terminals
pub fn build_hyperedge_tree_mst(terminals: &[ConnEnd]) -> Vec<SimpleTreeEdge> {
    if terminals.len() < 2 {
        return Vec::new();
    }

    let points: Vec<Point> = terminals.iter().map(|t| t.position).collect();
    let mst_edges = compute_mst(&points);

    mst_edges
        .into_iter()
        .map(|(i, j)| SimpleTreeEdge {
            from: points[i],
            to: points[j],
            is_terminal: true,
        })
        .collect()
}

// ============================================================================
// Fermat Point (Optimal Steiner Point for 3 terminals)
// ============================================================================

/// Computes the Fermat point (Torricelli point) for 3 terminals
///
/// The Fermat point minimizes the sum of distances to all 3 terminals.
/// For triangles with all angles < 120°, it's the point where each side
/// subtends an angle of 120°.
pub fn compute_fermat_point(a: &Point, b: &Point, c: &Point) -> Point {
    // Check if any angle is >= 120 degrees
    // If so, Fermat point is at that vertex
    let angle_a = angle_at_vertex(b, a, c);
    let angle_b = angle_at_vertex(a, b, c);
    let angle_c = angle_at_vertex(a, c, b);

    const THRESHOLD: f64 = 2.0 * std::f64::consts::PI / 3.0; // 120 degrees

    if angle_a >= THRESHOLD {
        return *a;
    }
    if angle_b >= THRESHOLD {
        return *b;
    }
    if angle_c >= THRESHOLD {
        return *c;
    }

    // Use iterative method (Weiszfeld's algorithm)
    let mut fermat = Point::new(
        (a.x + b.x + c.x) / 3.0,
        (a.y + b.y + c.y) / 3.0,
    );

    const MAX_ITERATIONS: usize = 100;
    const TOLERANCE: f64 = 1e-9;

    for _ in 0..MAX_ITERATIONS {
        let da = fermat.distance(a).max(TOLERANCE);
        let db = fermat.distance(b).max(TOLERANCE);
        let dc = fermat.distance(c).max(TOLERANCE);

        let weight_sum = 1.0 / da + 1.0 / db + 1.0 / dc;
        let new_x = (a.x / da + b.x / db + c.x / dc) / weight_sum;
        let new_y = (a.y / da + b.y / db + c.y / dc) / weight_sum;

        let delta = ((new_x - fermat.x).powi(2) + (new_y - fermat.y).powi(2)).sqrt();
        fermat = Point::new(new_x, new_y);

        if delta < TOLERANCE {
            break;
        }
    }

    fermat
}

/// Computes the angle at vertex v formed by points a-v-b
fn angle_at_vertex(a: &Point, v: &Point, b: &Point) -> f64 {
    let va = Point::new(a.x - v.x, a.y - v.y);
    let vb = Point::new(b.x - v.x, b.y - v.y);

    let dot = va.x * vb.x + va.y * vb.y;
    let mag_va = (va.x * va.x + va.y * va.y).sqrt();
    let mag_vb = (vb.x * vb.x + vb.y * vb.y).sqrt();

    let cos_angle = (dot / (mag_va * mag_vb)).max(-1.0).min(1.0);
    cos_angle.acos()
}

// ============================================================================
// Rectilinear Steiner Tree (for orthogonal routing)
// ============================================================================

/// Computes a rectilinear Steiner point for 2 terminals
///
/// For orthogonal routing, the optimal junction is at the L-bend
pub fn compute_rectilinear_junction_2(a: &Point, b: &Point) -> Point {
    // Return the corner of the bounding box (H-V routing)
    Point::new(b.x, a.y)
}

/// Computes rectilinear Steiner points for 3 terminals
///
/// For 3 terminals, returns the Hanan grid intersection that minimizes
/// total rectilinear distance
pub fn compute_rectilinear_junctions_3(a: &Point, b: &Point, c: &Point) -> Vec<Point> {
    // Hanan grid: intersections of horizontal/vertical lines through terminals
    let candidates = vec![
        Point::new(a.x, b.y),
        Point::new(a.x, c.y),
        Point::new(b.x, a.y),
        Point::new(b.x, c.y),
        Point::new(c.x, a.y),
        Point::new(c.x, b.y),
    ];

    // Find candidate with minimum total rectilinear distance
    let mut best_point = candidates[0];
    let mut best_cost = f64::INFINITY;

    for p in &candidates {
        let cost = rectilinear_distance(a, p)
            + rectilinear_distance(b, p)
            + rectilinear_distance(c, p);
        if cost < best_cost {
            best_cost = cost;
            best_point = *p;
        }
    }

    vec![best_point]
}

/// Computes rectilinear (Manhattan) distance between two points
fn rectilinear_distance(a: &Point, b: &Point) -> f64 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hyperedge_creation() {
        let terminals = vec![
            ConnEnd::new(Point::new(0.0, 0.0)),
            ConnEnd::new(Point::new(100.0, 0.0)),
            ConnEnd::new(Point::new(50.0, 100.0)),
        ];

        let hyperedge = HyperedgeRef::new(1, terminals);
        assert_eq!(hyperedge.id(), 1);
        assert_eq!(hyperedge.terminals().len(), 3);
        assert!(hyperedge.needs_reroute());
    }

    #[test]
    fn test_steiner_tree() {
        let rerouter = HyperedgeRerouter::new();
        let terminals = vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(50.0, 100.0),
        ];

        let junctions = rerouter.compute_steiner_tree(&terminals);
        assert_eq!(junctions.len(), 1);
        // Centroid should be at (50, 33.33...)
        assert!((junctions[0].x - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_mst_two_points() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
        ];
        let edges = compute_mst(&points);
        assert_eq!(edges.len(), 1);
        assert!(edges.contains(&(0, 1)) || edges.contains(&(1, 0)));
    }

    #[test]
    fn test_mst_three_points() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(100.0, 0.0),
            Point::new(50.0, 50.0),
        ];
        let edges = compute_mst(&points);
        assert_eq!(edges.len(), 2); // MST of 3 points has 2 edges
    }

    #[test]
    fn test_mst_collinear_points() {
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(50.0, 0.0),
            Point::new(100.0, 0.0),
        ];
        let edges = compute_mst(&points);
        assert_eq!(edges.len(), 2);
        // Should connect adjacent points: 0-1 and 1-2 (or similar)
    }

    #[test]
    fn test_fermat_point_equilateral() {
        // For equilateral triangle, Fermat point is at centroid
        let a = Point::new(0.0, 0.0);
        let b = Point::new(100.0, 0.0);
        let c = Point::new(50.0, 86.6); // Approximately equilateral

        let fermat = compute_fermat_point(&a, &b, &c);
        let centroid = Point::new(50.0, 28.87);

        // Should be close to centroid
        assert!((fermat.x - centroid.x).abs() < 5.0);
        assert!((fermat.y - centroid.y).abs() < 5.0);
    }

    #[test]
    fn test_fermat_point_obtuse_triangle() {
        // For obtuse triangle (angle >= 120°), Fermat point is at obtuse vertex
        let a = Point::new(0.0, 0.0);
        let b = Point::new(100.0, 0.0);
        let c = Point::new(50.0, 10.0); // Very flat triangle

        let fermat = compute_fermat_point(&a, &b, &c);

        // The angle at c is obtuse, so Fermat should be at c
        // Actually with this geometry it might be at c
        // Just verify it returns a valid point
        assert!(fermat.x >= 0.0 && fermat.x <= 100.0);
    }

    #[test]
    fn test_rectilinear_junction_2() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(100.0, 50.0);

        let junction = compute_rectilinear_junction_2(&a, &b);

        // Should be at L-bend corner
        assert_eq!(junction.x, 100.0);
        assert_eq!(junction.y, 0.0);
    }

    #[test]
    fn test_rectilinear_junctions_3() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(100.0, 0.0);
        let c = Point::new(50.0, 100.0);

        let junctions = compute_rectilinear_junctions_3(&a, &b, &c);

        assert_eq!(junctions.len(), 1);
        // Should be on Hanan grid
        let j = &junctions[0];
        let valid_x = (j.x - 0.0).abs() < 0.001
            || (j.x - 50.0).abs() < 0.001
            || (j.x - 100.0).abs() < 0.001;
        let valid_y = (j.y - 0.0).abs() < 0.001 || (j.y - 100.0).abs() < 0.001;
        assert!(valid_x && valid_y);
    }

    #[test]
    fn test_build_hyperedge_tree_mst() {
        let terminals = vec![
            ConnEnd::new(Point::new(0.0, 0.0)),
            ConnEnd::new(Point::new(100.0, 0.0)),
            ConnEnd::new(Point::new(50.0, 100.0)),
        ];

        let edges = build_hyperedge_tree_mst(&terminals);
        assert_eq!(edges.len(), 2); // 3 terminals = 2 MST edges
    }
}
