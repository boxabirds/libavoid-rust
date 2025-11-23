//! Shape obstacles for connector routing
//!
//! This module provides the ShapeRef type representing shapes that
//! connectors must route around.

use crate::geometry::{Polygon, Point};
use crate::obstacle::{Obstacle, ObstacleData};
use std::collections::HashSet;

/// Transform type for shape transformations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShapeTransformationType {
    /// Rotate 90 degrees clockwise
    Rotate90,
    /// Rotate 180 degrees
    Rotate180,
    /// Rotate 270 degrees clockwise (90 counter-clockwise)
    Rotate270,
    /// Flip horizontally
    FlipHorizontal,
    /// Flip vertically
    FlipVertical,
}

/// A shape reference representing a shape obstacle in the routing scene
#[derive(Debug, Clone)]
pub struct ShapeRef {
    /// The obstacle data
    data: ObstacleData,
    /// Connection pins on this shape
    connection_pins: Vec<ConnectionPin>,
}

/// A connection pin on a shape
#[derive(Debug, Clone)]
pub struct ConnectionPin {
    /// Unique ID for the pin
    pub id: u32,
    /// Class ID for grouping pins
    pub class_id: u32,
    /// Position relative to shape
    pub position: Point,
    /// Directions this pin can connect (bitfield)
    pub directions: u32,
    /// Whether this pin is exclusive (only one connection allowed)
    pub exclusive: bool,
    /// Connection cost for this pin
    pub connection_cost: f64,
    /// Inside offset from shape boundary
    pub inside_offset: f64,
}

impl ConnectionPin {
    /// Creates a new connection pin
    pub fn new(id: u32, position: Point) -> Self {
        ConnectionPin {
            id,
            class_id: 0,
            position,
            directions: 0xF, // All directions by default
            exclusive: false,
            connection_cost: 0.0,
            inside_offset: 0.0,
        }
    }

    /// Creates a pin with specific allowed directions
    pub fn with_directions(id: u32, position: Point, directions: u32) -> Self {
        ConnectionPin {
            id,
            class_id: 0,
            position,
            directions,
            exclusive: false,
            connection_cost: 0.0,
            inside_offset: 0.0,
        }
    }

    /// Creates a pin with all parameters
    pub fn with_all(id: u32, class_id: u32, position: Point, directions: u32, inside_offset: f64) -> Self {
        ConnectionPin {
            id,
            class_id,
            position,
            directions,
            exclusive: false,
            connection_cost: 0.0,
            inside_offset,
        }
    }

    /// Sets the exclusive flag
    pub fn set_exclusive(&mut self, exclusive: bool) {
        self.exclusive = exclusive;
    }

    /// Returns whether this pin is exclusive
    pub fn is_exclusive(&self) -> bool {
        self.exclusive
    }

    /// Sets the connection cost
    pub fn set_connection_cost(&mut self, cost: f64) {
        self.connection_cost = cost;
    }

    /// Returns the connection cost
    pub fn connection_cost(&self) -> f64 {
        self.connection_cost
    }
}

impl ShapeRef {
    /// Creates a new shape with the given polygon
    pub fn new(id: u32, polygon: Polygon) -> Self {
        ShapeRef {
            data: ObstacleData::new(id, polygon),
            connection_pins: Vec::new(),
        }
    }

    /// Adds a connection pin to the shape
    pub fn add_connection_pin(&mut self, pin: ConnectionPin) {
        self.connection_pins.push(pin);
    }

    /// Returns the connection pins
    pub fn connection_pins(&self) -> &[ConnectionPin] {
        &self.connection_pins
    }

    /// Finds a connection pin by ID
    pub fn find_pin(&self, pin_id: u32) -> Option<&ConnectionPin> {
        self.connection_pins.iter().find(|p| p.id == pin_id)
    }

    /// Transforms connection pin positions
    pub fn transform_connection_pin_positions(&mut self, transform: ShapeTransformationType) {
        let center = self.position();

        for pin in &mut self.connection_pins {
            // Translate to origin
            let mut x = pin.position.x - center.x;
            let mut y = pin.position.y - center.y;

            // Apply transformation
            match transform {
                ShapeTransformationType::Rotate90 => {
                    let temp = x;
                    x = y;
                    y = -temp;
                }
                ShapeTransformationType::Rotate180 => {
                    x = -x;
                    y = -y;
                }
                ShapeTransformationType::Rotate270 => {
                    let temp = x;
                    x = -y;
                    y = temp;
                }
                ShapeTransformationType::FlipHorizontal => {
                    x = -x;
                }
                ShapeTransformationType::FlipVertical => {
                    y = -y;
                }
            }

            // Translate back
            pin.position.x = x + center.x;
            pin.position.y = y + center.y;
        }
    }

    /// Updates the shape's polygon
    pub fn set_polygon(&mut self, polygon: Polygon) {
        self.data.set_polygon(polygon);
    }

    /// Returns a mutable reference to the obstacle data
    pub(crate) fn data_mut(&mut self) -> &mut ObstacleData {
        &mut self.data
    }
}

impl Obstacle for ShapeRef {
    fn id(&self) -> u32 {
        self.data.id()
    }

    fn polygon(&self) -> &Polygon {
        self.data.polygon()
    }

    fn position(&self) -> Point {
        self.data.position()
    }

    fn is_active(&self) -> bool {
        self.data.is_active()
    }

    fn attached_connectors(&self) -> &HashSet<u32> {
        self.data.attached_connectors()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shape_creation() {
        let mut poly = Polygon::new();
        poly.push(Point::new(0.0, 0.0));
        poly.push(Point::new(10.0, 0.0));
        poly.push(Point::new(10.0, 10.0));
        poly.push(Point::new(0.0, 10.0));

        let shape = ShapeRef::new(1, poly);
        assert_eq!(shape.id(), 1);
        assert_eq!(shape.connection_pins().len(), 0);
    }

    #[test]
    fn test_connection_pins() {
        let mut poly = Polygon::new();
        poly.push(Point::new(0.0, 0.0));
        poly.push(Point::new(10.0, 0.0));
        poly.push(Point::new(10.0, 10.0));
        poly.push(Point::new(0.0, 10.0));

        let mut shape = ShapeRef::new(1, poly);

        let pin = ConnectionPin::new(1, Point::new(5.0, 0.0));
        shape.add_connection_pin(pin);

        assert_eq!(shape.connection_pins().len(), 1);
        let found_pin = shape.find_pin(1);
        assert!(found_pin.is_some());
    }

    #[test]
    fn test_pin_transformation() {
        let mut poly = Polygon::new();
        poly.push(Point::new(0.0, 0.0));
        poly.push(Point::new(10.0, 0.0));
        poly.push(Point::new(10.0, 10.0));
        poly.push(Point::new(0.0, 10.0));

        let mut shape = ShapeRef::new(1, poly);

        // Add a pin at the top center
        let pin = ConnectionPin::new(1, Point::new(5.0, 0.0));
        shape.add_connection_pin(pin);

        // Rotate 90 degrees - top should become right
        shape.transform_connection_pin_positions(ShapeTransformationType::Rotate90);

        // After rotation, the pin should be on the right side
        let pin = shape.find_pin(1).unwrap();
        // The center is at (5, 5), pin was at (5, 0) -> relative (0, -5)
        // After 90° rotation: (-5, 0) -> absolute (0, 5)
        assert!((pin.position.y - 5.0).abs() < 0.01);
    }
}
