//! # libavoid
//!
//! A Rust port of libavoid - Fast, object-avoiding connector routing for interactive diagram editors.
//!
//! libavoid is a cross-platform library providing fast, object-avoiding connector routing for use
//! in interactive diagram editors. It implements incremental connector routing with orthogonal
//! routing support.
//!
//! ## Overview
//!
//! This library enables automatic routing of connectors (lines between objects) while avoiding
//! obstacles in diagram layouts. Key features include:
//!
//! - Fast incremental routing updates
//! - Orthogonal (rectilinear) and polyline connector routing
//! - Object-avoiding path planning
//! - Support for connection pins, junctions, and hyperedges
//! - Transaction-based batched updates
//!
//! ## Example
//!
//! ```rust
//! use libavoid::{Router, ShapeRef, ConnRef, Point, Polygon, Rectangle};
//!
//! // Create a router instance
//! let mut router = Router::new(0);
//!
//! // Add shapes (obstacles)
//! let rect1 = Rectangle::new(Point::new(0.0, 0.0), 50.0, 50.0);
//! let shape1 = router.add_shape(rect1.into(), 1);
//!
//! // Add connectors
//! // (connector creation example)
//! ```
//!
//! ## Original C++ Implementation
//!
//! This is a Rust port of the original C++ libavoid library by Michael Wybrow
//! from Monash University's Adaptive Diagrams and Documents lab.
//!
//! Original repository: <https://github.com/mjwybrow/adaptagrams>

pub mod action;
pub mod geometry;
pub mod router;
pub mod connector;
pub mod obstacle;
pub mod shape;
pub mod visibility;
pub mod graph;
pub mod orthogonal;
pub mod junction;
pub mod hyperedge;
pub mod hyperedge_improver;  // Tasks #15-16 stub
pub mod cluster;
pub mod cluster_features;  // Task #17 stub
pub mod vpsc;
pub mod channel;
pub mod orthogonal_visgraph;
pub mod pin_visibility;  // Task #14 stub

#[cfg(feature = "wasm")]
pub mod wasm;

// Re-export commonly used types
pub use geometry::{Point, Box as BBox, Polygon, Rectangle, Edge, PolygonInterface};
pub use router::{Router, RouterFlags, RoutingParameter, RoutingOption, RouterDebugState, ROUTER_FLAG_NONE, ROUTER_FLAG_USE_TRANSACTIONS};
pub use connector::{ConnRef, ConnEnd, ConnType};
pub use obstacle::Obstacle;
pub use shape::ShapeRef;
pub use junction::JunctionRef;
pub use hyperedge::{HyperedgeRef, HyperedgeRerouter};
pub use cluster::ClusterRef;
pub use action::{ActionInfo, ActionType};
pub use connector::{ConnDirFlags, Checkpoint, CONN_DIR_ALL, CONN_DIR_NONE, CONN_DIR_UP, CONN_DIR_DOWN, CONN_DIR_LEFT, CONN_DIR_RIGHT};

/// Dimension constants
pub const XDIM: usize = 0;
pub const YDIM: usize = 1;

/// Vertex number constants
pub const UNASSIGNED_VERTEX_NUMBER: u32 = 8;
pub const SHAPE_CONNECTION_PIN: u32 = 9;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_point() {
        let p1 = Point::new(1.0, 2.0);
        let p2 = Point::new(3.0, 4.0);
        let p3 = p1 + p2;
        assert_eq!(p3.x, 4.0);
        assert_eq!(p3.y, 6.0);
    }
}
