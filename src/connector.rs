//! Connector routing functionality
//!
//! This module provides the connector types that represent the lines to be routed
//! between endpoints in a diagram.

use crate::geometry::{Point, Polygon, PolygonInterface};
use std::sync::Arc;

/// Type of connector routing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnType {
    /// Polyline routing (direct line segments)
    PolyLine,
    /// Orthogonal routing (only horizontal and vertical segments)
    Orthogonal,
}

impl Default for ConnType {
    fn default() -> Self {
        ConnType::PolyLine
    }
}

/// Represents one end of a connector
#[derive(Debug, Clone)]
pub struct ConnEnd {
    /// The position of the endpoint
    pub position: Point,
    /// Optional connection to a shape
    pub shape_id: Option<u32>,
    /// Optional connection pin ID
    pub pin_id: Option<u32>,
}

impl ConnEnd {
    /// Creates a new connector end at the given position
    pub fn new(position: Point) -> Self {
        ConnEnd {
            position,
            shape_id: None,
            pin_id: None,
        }
    }

    /// Creates a connector end attached to a shape
    pub fn with_shape(position: Point, shape_id: u32) -> Self {
        ConnEnd {
            position,
            shape_id: Some(shape_id),
            pin_id: None,
        }
    }

    /// Creates a connector end attached to a specific pin on a shape
    pub fn with_pin(position: Point, shape_id: u32, pin_id: u32) -> Self {
        ConnEnd {
            position,
            shape_id: Some(shape_id),
            pin_id: Some(pin_id),
        }
    }
}

/// Callback function type for connector updates
pub type ConnectorCallback = Arc<dyn Fn(&ConnRef) + Send + Sync>;

/// A connector reference representing a routed connection between two endpoints
#[derive(Clone)]
pub struct ConnRef {
    /// Unique identifier for this connector
    id: u32,
    /// Source endpoint
    src: ConnEnd,
    /// Destination endpoint
    dst: ConnEnd,
    /// Type of routing (polyline or orthogonal)
    routing_type: ConnType,
    /// The current route as a polygon
    route: Option<Polygon>,
    /// The display route (simplified/post-processed)
    display_route: Option<Polygon>,
    /// Routing checkpoints (waypoints the connector must visit)
    checkpoints: Vec<Point>,
    /// Whether the connector needs to be repainted
    needs_repaint: bool,
    /// Whether the route is fixed (not automatically routed)
    has_fixed_route: bool,
    /// Whether the connector is active
    active: bool,
    /// Callback function called when route changes
    callback: Option<ConnectorCallback>,
    /// Whether this connector hates crossing other connectors
    hate_crossings: bool,
}

impl ConnRef {
    /// Creates a new connector with a unique ID
    pub fn new(id: u32) -> Self {
        ConnRef {
            id,
            src: ConnEnd::new(Point::new(0.0, 0.0)),
            dst: ConnEnd::new(Point::new(0.0, 0.0)),
            routing_type: ConnType::PolyLine,
            route: None,
            display_route: None,
            checkpoints: Vec::new(),
            needs_repaint: false,
            has_fixed_route: false,
            active: true,
            callback: None,
            hate_crossings: false,
        }
    }

    /// Creates a new connector with specified endpoints
    pub fn with_endpoints(id: u32, src: ConnEnd, dst: ConnEnd) -> Self {
        ConnRef {
            id,
            src,
            dst,
            routing_type: ConnType::PolyLine,
            route: None,
            display_route: None,
            checkpoints: Vec::new(),
            needs_repaint: false,
            has_fixed_route: false,
            active: true,
            callback: None,
            hate_crossings: false,
        }
    }

    /// Sets whether this connector hates crossing other connectors
    pub fn set_hate_crossings(&mut self, value: bool) {
        self.hate_crossings = value;
    }

    /// Returns whether this connector hates crossing other connectors
    pub fn does_hate_crossings(&self) -> bool {
        self.hate_crossings
    }

    /// Returns the connector's unique ID
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Sets the source endpoint
    pub fn set_source_endpoint(&mut self, src: ConnEnd) {
        self.src = src;
        self.needs_repaint = true;
    }

    /// Sets the destination endpoint
    pub fn set_dest_endpoint(&mut self, dst: ConnEnd) {
        self.dst = dst;
        self.needs_repaint = true;
    }

    /// Sets both endpoints
    pub fn set_endpoints(&mut self, src: ConnEnd, dst: ConnEnd) {
        self.src = src;
        self.dst = dst;
        self.needs_repaint = true;
    }

    /// Returns the current endpoints
    pub fn endpoint_conn_ends(&self) -> (&ConnEnd, &ConnEnd) {
        (&self.src, &self.dst)
    }

    /// Sets the routing type (polyline or orthogonal)
    pub fn set_routing_type(&mut self, routing_type: ConnType) {
        if self.routing_type != routing_type {
            self.routing_type = routing_type;
            self.needs_repaint = true;
        }
    }

    /// Returns the current routing type
    pub fn routing_type(&self) -> ConnType {
        self.routing_type
    }

    /// Sets routing checkpoints (waypoints the connector must visit)
    pub fn set_routing_checkpoints(&mut self, checkpoints: Vec<Point>) {
        self.checkpoints = checkpoints;
        self.needs_repaint = true;
    }

    /// Returns the routing checkpoints
    pub fn routing_checkpoints(&self) -> &[Point] {
        &self.checkpoints
    }

    /// Returns the raw route (for debugging)
    pub fn route(&self) -> Option<&Polygon> {
        self.route.as_ref()
    }

    /// Returns the display route (simplified, post-processed)
    pub fn display_route(&self) -> Option<&Polygon> {
        self.display_route.as_ref()
    }

    /// Sets a fixed route for the connector
    pub fn set_fixed_route(&mut self, route: Polygon) {
        self.route = Some(route.clone());
        self.display_route = Some(route);
        self.has_fixed_route = true;
        self.needs_repaint = false;
    }

    /// Sets the existing route as fixed
    pub fn set_fixed_existing_route(&mut self) {
        self.has_fixed_route = true;
    }

    /// Clears the fixed route
    pub fn clear_fixed_route(&mut self) {
        self.has_fixed_route = false;
        self.needs_repaint = true;
    }

    /// Returns whether the route is fixed
    pub fn has_fixed_route(&self) -> bool {
        self.has_fixed_route
    }

    /// Returns whether the connector needs repainting
    pub fn needs_repaint(&self) -> bool {
        self.needs_repaint
    }

    /// Sets the callback function for route updates
    pub fn set_callback(&mut self, callback: ConnectorCallback) {
        self.callback = Some(callback);
    }

    /// Internal method to set the route
    pub(crate) fn set_route(&mut self, route: Polygon) {
        self.route = Some(route.clone());
        self.display_route = Some(route);
        self.needs_repaint = true;

        // Call callback if set
        if let Some(ref callback) = self.callback {
            callback(self);
        }
    }

    /// Internal method to mark as not needing repaint
    pub(crate) fn mark_painted(&mut self) {
        self.needs_repaint = false;
    }

    /// Returns whether the connector is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Sets whether the connector is active
    pub(crate) fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Splits the connector at the given segment index
    /// Returns the ID of the new connector that should be created
    pub fn split_at_segment(&self, segment_index: usize) -> Option<(ConnEnd, ConnEnd, ConnEnd)> {
        if let Some(route) = &self.route {
            if segment_index < route.size() - 1 {
                let split_point = route.at(segment_index);
                let new_end = ConnEnd::new(*split_point);

                // Return the endpoints for two new connectors
                return Some((self.src.clone(), new_end.clone(), self.dst.clone()));
            }
        }
        None
    }
}

impl std::fmt::Debug for ConnRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConnRef")
            .field("id", &self.id)
            .field("routing_type", &self.routing_type)
            .field("needs_repaint", &self.needs_repaint)
            .field("has_fixed_route", &self.has_fixed_route)
            .field("active", &self.active)
            .field("checkpoints", &self.checkpoints)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connector_creation() {
        let conn = ConnRef::new(1);
        assert_eq!(conn.id(), 1);
        assert_eq!(conn.routing_type(), ConnType::PolyLine);
        assert!(!conn.needs_repaint());
    }

    #[test]
    fn test_connector_endpoints() {
        let mut conn = ConnRef::new(1);
        let src = ConnEnd::new(Point::new(0.0, 0.0));
        let dst = ConnEnd::new(Point::new(10.0, 10.0));

        conn.set_endpoints(src, dst);
        assert!(conn.needs_repaint());

        let (src_end, dst_end) = conn.endpoint_conn_ends();
        assert_eq!(src_end.position.x, 0.0);
        assert_eq!(dst_end.position.x, 10.0);
    }

    #[test]
    fn test_routing_type() {
        let mut conn = ConnRef::new(1);
        assert_eq!(conn.routing_type(), ConnType::PolyLine);

        conn.set_routing_type(ConnType::Orthogonal);
        assert_eq!(conn.routing_type(), ConnType::Orthogonal);
    }

    #[test]
    fn test_fixed_route() {
        let mut conn = ConnRef::new(1);
        assert!(!conn.has_fixed_route());

        let mut route = Polygon::new();
        route.push(Point::new(0.0, 0.0));
        route.push(Point::new(10.0, 10.0));

        conn.set_fixed_route(route);
        assert!(conn.has_fixed_route());
        assert!(conn.route().is_some());
    }
}
