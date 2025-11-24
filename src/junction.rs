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
    /// Whether the junction position is fixed (cannot be optimized)
    position_fixed: bool,
    /// Recommended position after optimization (may differ from current position)
    recommended_position: Option<Point>,
}

impl JunctionRef {
    /// Creates a new junction at the given position
    pub fn new(id: u32, position: Point) -> Self {
        JunctionRef {
            id,
            position,
            attached_connectors: HashSet::new(),
            active: true,
            position_fixed: false,
            recommended_position: None,
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

    /// Returns whether the junction position is fixed
    ///
    /// When fixed, the junction position cannot be changed by optimization algorithms.
    pub fn position_fixed(&self) -> bool {
        self.position_fixed
    }

    /// Sets whether the junction position is fixed
    ///
    /// When fixed, the junction position cannot be changed by optimization algorithms.
    pub fn set_position_fixed(&mut self, fixed: bool) {
        self.position_fixed = fixed;
    }

    /// Returns the recommended position for this junction
    ///
    /// After routing and optimization, this may contain a suggested position
    /// that would improve the overall layout. Returns None if no recommendation
    /// has been computed.
    pub fn recommended_position(&self) -> Option<Point> {
        self.recommended_position
    }

    /// Sets the recommended position for this junction
    ///
    /// Called by the router during optimization to suggest an improved position.
    pub fn set_recommended_position(&mut self, position: Option<Point>) {
        self.recommended_position = position;
    }

    /// Checks if this junction can have its connectors merged
    ///
    /// A junction can be removed and its connectors merged when exactly two
    /// connectors are attached.
    pub fn can_merge_connectors(&self) -> bool {
        self.attached_connectors.len() == 2
    }

    /// Returns the IDs of attached connectors for merging
    ///
    /// Returns Some((id1, id2)) if exactly two connectors attached, None otherwise.
    pub fn get_connectors_for_merge(&self) -> Option<(u32, u32)> {
        if self.attached_connectors.len() != 2 {
            return None;
        }

        let ids: Vec<_> = self.attached_connectors.iter().copied().collect();
        Some((ids[0], ids[1]))
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
