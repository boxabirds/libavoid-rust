//! Junction management for connector routing
//!
//! Junctions are special vertices where multiple connectors meet.
//! They can be optimized and repositioned for better diagram layout.

use crate::geometry::Point;
use std::collections::HashSet;

/// A junction where multiple connectors meet
#[derive(Debug, Clone)]
pub struct JunctionRef {
    /// Unique identifier
    id: u32,
    /// Position of the junction
    position: Point,
    /// Connectors attached to this junction
    attached_connectors: HashSet<u32>,
    /// Whether this junction is active
    active: bool,
}

impl JunctionRef {
    /// Creates a new junction at the given position
    pub fn new(id: u32, position: Point) -> Self {
        JunctionRef {
            id,
            position,
            attached_connectors: HashSet::new(),
            active: true,
        }
    }

    /// Returns the junction's unique ID
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the junction's position
    pub fn position(&self) -> Point {
        self.position
    }

    /// Sets the junction's position
    pub fn set_position(&mut self, position: Point) {
        self.position = position;
    }

    /// Attaches a connector to this junction
    pub fn attach_connector(&mut self, conn_id: u32) {
        self.attached_connectors.insert(conn_id);
    }

    /// Detaches a connector from this junction
    pub fn detach_connector(&mut self, conn_id: u32) {
        self.attached_connectors.remove(&conn_id);
    }

    /// Returns the number of connectors attached
    pub fn connector_count(&self) -> usize {
        self.attached_connectors.len()
    }

    /// Returns whether this junction is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Sets whether this junction is active
    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    /// Returns the attached connector IDs
    pub fn attached_connectors(&self) -> &HashSet<u32> {
        &self.attached_connectors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_junction_creation() {
        let junction = JunctionRef::new(1, Point::new(10.0, 20.0));
        assert_eq!(junction.id(), 1);
        assert_eq!(junction.position().x, 10.0);
        assert_eq!(junction.connector_count(), 0);
    }

    #[test]
    fn test_junction_connectors() {
        let mut junction = JunctionRef::new(1, Point::new(10.0, 20.0));

        junction.attach_connector(100);
        junction.attach_connector(101);

        assert_eq!(junction.connector_count(), 2);
        assert!(junction.attached_connectors().contains(&100));

        junction.detach_connector(100);
        assert_eq!(junction.connector_count(), 1);
    }
}
