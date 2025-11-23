//! Transaction action types for batched router operations
//!
//! This module provides the action types used in the router's transaction
//! system, allowing operations to be queued and processed in batches.

use crate::geometry::{Point, Polygon};

// ============================================================================
// Type Aliases
// ============================================================================

/// Obstacle (shape/junction) ID
pub type ObstacleId = u32;

/// Connector ID
pub type ConnectorId = u32;

// ============================================================================
// Action Types
// ============================================================================

/// Type of transaction action
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionType {
    /// Add a shape obstacle
    ShapeAdd,
    /// Remove a shape obstacle
    ShapeRemove,
    /// Move a shape obstacle
    ShapeMove,
    /// Add a junction
    JunctionAdd,
    /// Remove a junction
    JunctionRemove,
    /// Move a junction
    JunctionMove,
    /// Add a connector
    ConnectorAdd,
    /// Remove a connector
    ConnectorRemove,
    /// Modify a connector (endpoints, type, etc.)
    ConnectorChange,
}

// ============================================================================
// Action Info
// ============================================================================

/// Queued action for transaction processing
#[derive(Clone, Debug)]
pub struct ActionInfo {
    /// Type of action
    pub action_type: ActionType,
    /// Obstacle ID (for shape/junction actions)
    pub obstacle_id: Option<ObstacleId>,
    /// Connector ID (for connector actions)
    pub connector_id: Option<ConnectorId>,
    /// New polygon (for add/move with new shape)
    pub new_polygon: Option<Polygon>,
    /// New position (for move actions)
    pub new_position: Option<Point>,
    /// Whether this is the first move in a series (for optimization)
    pub first_move: bool,
}

impl ActionInfo {
    /// Creates a shape add action
    pub fn shape_add(obstacle_id: ObstacleId) -> Self {
        ActionInfo {
            action_type: ActionType::ShapeAdd,
            obstacle_id: Some(obstacle_id),
            connector_id: None,
            new_polygon: None,
            new_position: None,
            first_move: false,
        }
    }

    /// Creates a shape remove action
    pub fn shape_remove(obstacle_id: ObstacleId) -> Self {
        ActionInfo {
            action_type: ActionType::ShapeRemove,
            obstacle_id: Some(obstacle_id),
            connector_id: None,
            new_polygon: None,
            new_position: None,
            first_move: false,
        }
    }

    /// Creates a shape move action
    pub fn shape_move(obstacle_id: ObstacleId, new_position: Point, first_move: bool) -> Self {
        ActionInfo {
            action_type: ActionType::ShapeMove,
            obstacle_id: Some(obstacle_id),
            connector_id: None,
            new_polygon: None,
            new_position: Some(new_position),
            first_move,
        }
    }

    /// Creates a junction add action
    pub fn junction_add(obstacle_id: ObstacleId) -> Self {
        ActionInfo {
            action_type: ActionType::JunctionAdd,
            obstacle_id: Some(obstacle_id),
            connector_id: None,
            new_polygon: None,
            new_position: None,
            first_move: false,
        }
    }

    /// Creates a junction remove action
    pub fn junction_remove(obstacle_id: ObstacleId) -> Self {
        ActionInfo {
            action_type: ActionType::JunctionRemove,
            obstacle_id: Some(obstacle_id),
            connector_id: None,
            new_polygon: None,
            new_position: None,
            first_move: false,
        }
    }

    /// Creates a junction move action
    pub fn junction_move(obstacle_id: ObstacleId, new_position: Point) -> Self {
        ActionInfo {
            action_type: ActionType::JunctionMove,
            obstacle_id: Some(obstacle_id),
            connector_id: None,
            new_polygon: None,
            new_position: Some(new_position),
            first_move: false,
        }
    }

    /// Creates a connector add action
    pub fn connector_add(connector_id: ConnectorId) -> Self {
        ActionInfo {
            action_type: ActionType::ConnectorAdd,
            obstacle_id: None,
            connector_id: Some(connector_id),
            new_polygon: None,
            new_position: None,
            first_move: false,
        }
    }

    /// Creates a connector remove action
    pub fn connector_remove(connector_id: ConnectorId) -> Self {
        ActionInfo {
            action_type: ActionType::ConnectorRemove,
            obstacle_id: None,
            connector_id: Some(connector_id),
            new_polygon: None,
            new_position: None,
            first_move: false,
        }
    }

    /// Creates a connector change action
    pub fn connector_change(connector_id: ConnectorId) -> Self {
        ActionInfo {
            action_type: ActionType::ConnectorChange,
            obstacle_id: None,
            connector_id: Some(connector_id),
            new_polygon: None,
            new_position: None,
            first_move: false,
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
    fn test_action_types() {
        let add = ActionInfo::shape_add(1);
        assert_eq!(add.action_type, ActionType::ShapeAdd);
        assert_eq!(add.obstacle_id, Some(1));
        assert!(add.connector_id.is_none());

        let remove = ActionInfo::connector_remove(5);
        assert_eq!(remove.action_type, ActionType::ConnectorRemove);
        assert_eq!(remove.connector_id, Some(5));

        let mv = ActionInfo::shape_move(2, Point::new(10.0, 20.0), true);
        assert_eq!(mv.action_type, ActionType::ShapeMove);
        assert!(mv.first_move);
        assert_eq!(mv.new_position, Some(Point::new(10.0, 20.0)));
    }
}
