//! Hyperedge routing for multi-terminal connections
//!
//! Hyperedges connect multiple terminals using junctions and connectors.
//! This module provides routing for busses and other multi-point connections.

use crate::geometry::Point;
use crate::connector::ConnEnd;
use std::collections::HashSet;

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
    pub(crate) fn add_connector(&mut self, conn_id: u32) {
        self.connectors.insert(conn_id);
    }

    /// Returns the junctions in this hyperedge
    pub fn junctions(&self) -> &HashSet<u32> {
        &self.junctions
    }

    /// Adds a junction to the hyperedge
    pub(crate) fn add_junction(&mut self, junction_id: u32) {
        self.junctions.insert(junction_id);
    }

    /// Returns whether this hyperedge needs rerouting
    pub fn needs_reroute(&self) -> bool {
        self.needs_reroute
    }

    /// Marks the hyperedge as needing reroute
    pub(crate) fn mark_needs_reroute(&mut self) {
        self.needs_reroute = true;
    }

    /// Clears the needs reroute flag
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
}

// ============================================================================
// Hyperedge Tree Building
// ============================================================================

/// Represents an edge in the hyperedge tree
#[derive(Debug, Clone)]
pub struct HyperedgeTreeEdge {
    pub from: Point,
    pub to: Point,
    pub is_terminal: bool,
}

/// Builds a minimum spanning tree connecting all terminals through junctions
pub fn build_hyperedge_tree(terminals: &[ConnEnd], junctions: &[Point]) -> Vec<HyperedgeTreeEdge> {
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
            edges.push(HyperedgeTreeEdge {
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

            edges.push(HyperedgeTreeEdge {
                from: *t,
                to: *nearest,
                is_terminal: true,
            });
        }

        // Connect junctions together (simple chain for now)
        for i in 0..junctions.len().saturating_sub(1) {
            edges.push(HyperedgeTreeEdge {
                from: junctions[i],
                to: junctions[i + 1],
                is_terminal: false,
            });
        }
    }

    edges
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
}
