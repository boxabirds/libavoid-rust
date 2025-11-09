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
