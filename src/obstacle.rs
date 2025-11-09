//! Obstacle representation for routing
//!
//! This module provides the Obstacle trait and related types for objects
//! that connectors must route around.

use crate::geometry::{Polygon, Point, Box as BBox, PolygonInterface};
use std::collections::HashSet;

/// Trait representing an obstacle that must be routed around
pub trait Obstacle {
    /// Returns the unique identifier of the obstacle
    fn id(&self) -> u32;

    /// Returns the polygon boundary of the obstacle
    fn polygon(&self) -> &Polygon;

    /// Returns the position of the obstacle
    fn position(&self) -> Point;

    /// Returns the routing box (bounding box with routing offset)
    fn routing_box(&self) -> BBox {
        self.polygon().bounding_rect()
    }

    /// Returns the routing polygon (polygon with routing offset)
    fn routing_polygon(&self) -> Polygon {
        self.polygon().clone()
    }

    /// Returns whether the obstacle is active
    fn is_active(&self) -> bool;

    /// Returns the set of connector IDs attached to this obstacle
    fn attached_connectors(&self) -> &HashSet<u32>;
}

/// Base implementation for obstacle data
#[derive(Debug, Clone)]
pub struct ObstacleData {
    /// Unique identifier
    pub id: u32,
    /// Polygon boundary
    pub polygon: Polygon,
    /// Whether the obstacle is active
    pub active: bool,
    /// Set of attached connector IDs
    pub attached_connectors: HashSet<u32>,
}

impl ObstacleData {
    /// Creates a new obstacle with the given polygon
    pub fn new(id: u32, polygon: Polygon) -> Self {
        ObstacleData {
            id,
            polygon,
            active: true,
            attached_connectors: HashSet::new(),
        }
    }

    /// Returns the position (center) of the obstacle
    pub fn position(&self) -> Point {
        let bbox = self.polygon.bounding_rect();
        let x = (bbox.min.x + bbox.max.x) / 2.0;
        let y = (bbox.min.y + bbox.max.y) / 2.0;
        Point::new(x, y)
    }

    /// Updates the polygon boundary
    pub fn set_polygon(&mut self, polygon: Polygon) {
        self.polygon = polygon;
    }

    /// Attaches a connector to this obstacle
    pub fn attach_connector(&mut self, connector_id: u32) {
        self.attached_connectors.insert(connector_id);
    }

    /// Detaches a connector from this obstacle
    pub fn detach_connector(&mut self, connector_id: u32) {
        self.attached_connectors.remove(&connector_id);
    }

    /// Sets whether the obstacle is active
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }
}

impl Obstacle for ObstacleData {
    fn id(&self) -> u32 {
        self.id
    }

    fn polygon(&self) -> &Polygon {
        &self.polygon
    }

    fn position(&self) -> Point {
        self.position()
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn attached_connectors(&self) -> &HashSet<u32> {
        &self.attached_connectors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obstacle_creation() {
        let mut poly = Polygon::new();
        poly.push(Point::new(0.0, 0.0));
        poly.push(Point::new(10.0, 0.0));
        poly.push(Point::new(10.0, 10.0));
        poly.push(Point::new(0.0, 10.0));

        let obs = ObstacleData::new(1, poly);
        assert_eq!(obs.id(), 1);
        assert!(obs.is_active());
    }

    #[test]
    fn test_obstacle_position() {
        let mut poly = Polygon::new();
        poly.push(Point::new(0.0, 0.0));
        poly.push(Point::new(20.0, 0.0));
        poly.push(Point::new(20.0, 20.0));
        poly.push(Point::new(0.0, 20.0));

        let obs = ObstacleData::new(1, poly);
        let pos = obs.position();
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 10.0);
    }

    #[test]
    fn test_connector_attachment() {
        let mut poly = Polygon::new();
        poly.push(Point::new(0.0, 0.0));
        poly.push(Point::new(10.0, 10.0));

        let mut obs = ObstacleData::new(1, poly);
        assert_eq!(obs.attached_connectors().len(), 0);

        obs.attach_connector(10);
        assert_eq!(obs.attached_connectors().len(), 1);
        assert!(obs.attached_connectors().contains(&10));

        obs.detach_connector(10);
        assert_eq!(obs.attached_connectors().len(), 0);
    }
}
