//! Connector routing functionality
//!
//! This module provides the connector types that represent the lines to be routed
//! between endpoints in a diagram.

use crate::geometry::{Point, Polygon, PolygonInterface};
use std::sync::Arc;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ============================================================================
// Connection Direction Flags
// ============================================================================

/// Direction flags for connection pins and endpoints
pub type ConnDirFlags = u32;

/// No direction allowed
pub const CONN_DIR_NONE: ConnDirFlags = 0;
/// Up direction (negative Y)
pub const CONN_DIR_UP: ConnDirFlags = 1;
/// Down direction (positive Y)
pub const CONN_DIR_DOWN: ConnDirFlags = 2;
/// Left direction (negative X)
pub const CONN_DIR_LEFT: ConnDirFlags = 4;
/// Right direction (positive X)
pub const CONN_DIR_RIGHT: ConnDirFlags = 8;
/// All directions allowed
pub const CONN_DIR_ALL: ConnDirFlags = 15;

/// Check if direction flags include up
pub fn has_dir_up(flags: ConnDirFlags) -> bool {
    flags & CONN_DIR_UP != 0
}

/// Check if direction flags include down
pub fn has_dir_down(flags: ConnDirFlags) -> bool {
    flags & CONN_DIR_DOWN != 0
}

/// Check if direction flags include left
pub fn has_dir_left(flags: ConnDirFlags) -> bool {
    flags & CONN_DIR_LEFT != 0
}

/// Check if direction flags include right
pub fn has_dir_right(flags: ConnDirFlags) -> bool {
    flags & CONN_DIR_RIGHT != 0
}

// ============================================================================
// Connector Type
// ============================================================================

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

// ============================================================================
// Connection Endpoint Kind
// ============================================================================

/// Type of connection endpoint
#[derive(Debug, Clone, PartialEq)]
pub enum ConnEndKind {
    /// Free point in space (not attached to anything)
    FreePoint(Point),
    /// Attached to a shape via a pin class
    ShapePin {
        /// The shape obstacle ID
        shape_id: u32,
        /// The pin class ID to connect to
        pin_class_id: u32,
    },
    /// Attached to a junction
    Junction {
        /// The junction obstacle ID
        junction_id: u32,
    },
}

impl Default for ConnEndKind {
    fn default() -> Self {
        ConnEndKind::FreePoint(Point::new(0.0, 0.0))
    }
}

// ============================================================================
// Connection Endpoint
// ============================================================================

/// Represents one end of a connector
#[derive(Debug, Clone)]
pub struct ConnEnd {
    /// The type of endpoint
    pub kind: ConnEndKind,
    /// Allowed connection directions (bitfield of ConnDirFlags)
    pub directions: ConnDirFlags,
    /// Cached position (resolved from kind)
    pub position: Point,
    /// Optional connection to a shape (for backwards compatibility)
    pub shape_id: Option<u32>,
    /// Optional connection pin ID
    pub pin_id: Option<u32>,
}

impl ConnEnd {
    /// Creates a new connector end at the given position
    pub fn new(position: Point) -> Self {
        ConnEnd {
            kind: ConnEndKind::FreePoint(position),
            directions: CONN_DIR_ALL,
            position,
            shape_id: None,
            pin_id: None,
        }
    }

    /// Creates a free point endpoint with specific directions
    pub fn free_point(position: Point, directions: ConnDirFlags) -> Self {
        ConnEnd {
            kind: ConnEndKind::FreePoint(position),
            directions,
            position,
            shape_id: None,
            pin_id: None,
        }
    }

    /// Creates a connector end attached to a shape pin class
    pub fn shape_pin(shape_id: u32, pin_class_id: u32, position: Point) -> Self {
        ConnEnd {
            kind: ConnEndKind::ShapePin {
                shape_id,
                pin_class_id,
            },
            directions: CONN_DIR_ALL,
            position,
            shape_id: Some(shape_id),
            pin_id: Some(pin_class_id),
        }
    }

    /// Creates a connector end attached to a junction
    pub fn junction(junction_id: u32, position: Point) -> Self {
        ConnEnd {
            kind: ConnEndKind::Junction { junction_id },
            directions: CONN_DIR_ALL,
            position,
            shape_id: None,
            pin_id: None,
        }
    }

    /// Creates a connector end attached to a shape (backwards compatible)
    pub fn with_shape(position: Point, shape_id: u32) -> Self {
        ConnEnd {
            kind: ConnEndKind::ShapePin {
                shape_id,
                pin_class_id: 0,
            },
            directions: CONN_DIR_ALL,
            position,
            shape_id: Some(shape_id),
            pin_id: None,
        }
    }

    /// Creates a connector end attached to a specific pin on a shape
    pub fn with_pin(position: Point, shape_id: u32, pin_id: u32) -> Self {
        ConnEnd {
            kind: ConnEndKind::ShapePin {
                shape_id,
                pin_class_id: pin_id,
            },
            directions: CONN_DIR_ALL,
            position,
            shape_id: Some(shape_id),
            pin_id: Some(pin_id),
        }
    }

    /// Sets the allowed connection directions
    pub fn set_directions(&mut self, directions: ConnDirFlags) {
        self.directions = directions;
    }

    /// Returns the allowed connection directions
    pub fn directions(&self) -> ConnDirFlags {
        self.directions
    }

    /// Updates the cached position
    pub fn set_position(&mut self, position: Point) {
        self.position = position;
        if let ConnEndKind::FreePoint(ref mut p) = self.kind {
            *p = position;
        }
    }

    /// Returns whether this endpoint is attached to a shape
    pub fn is_shape_attached(&self) -> bool {
        matches!(self.kind, ConnEndKind::ShapePin { .. })
    }

    /// Returns whether this endpoint is attached to a junction
    pub fn is_junction_attached(&self) -> bool {
        matches!(self.kind, ConnEndKind::Junction { .. })
    }

    /// Returns whether this endpoint is a free point
    pub fn is_free_point(&self) -> bool {
        matches!(self.kind, ConnEndKind::FreePoint(_))
    }

    /// Returns the shape ID if attached to a shape
    pub fn attached_shape_id(&self) -> Option<u32> {
        match &self.kind {
            ConnEndKind::ShapePin { shape_id, .. } => Some(*shape_id),
            _ => None,
        }
    }

    /// Returns the junction ID if attached to a junction
    pub fn attached_junction_id(&self) -> Option<u32> {
        match &self.kind {
            ConnEndKind::Junction { junction_id } => Some(*junction_id),
            _ => None,
        }
    }
}

// ============================================================================
// Checkpoint (Waypoint)
// ============================================================================

/// A routing checkpoint (waypoint) that a connector must pass through
#[derive(Debug, Clone)]
pub struct Checkpoint {
    /// Position of the checkpoint
    pub point: Point,
    /// Required arrival directions (connector must arrive from these directions)
    pub arrival_directions: ConnDirFlags,
    /// Required departure directions (connector must leave in these directions)
    pub departure_directions: ConnDirFlags,
}

impl Checkpoint {
    /// Creates a new checkpoint at the given position with all directions allowed
    pub fn new(point: Point) -> Self {
        Checkpoint {
            point,
            arrival_directions: CONN_DIR_ALL,
            departure_directions: CONN_DIR_ALL,
        }
    }

    /// Creates a checkpoint with specific arrival and departure directions
    pub fn with_directions(
        point: Point,
        arrival_directions: ConnDirFlags,
        departure_directions: ConnDirFlags,
    ) -> Self {
        Checkpoint {
            point,
            arrival_directions,
            departure_directions,
        }
    }

    /// Creates a checkpoint that must be traversed horizontally (left-right)
    pub fn horizontal(point: Point) -> Self {
        Checkpoint {
            point,
            arrival_directions: CONN_DIR_LEFT | CONN_DIR_RIGHT,
            departure_directions: CONN_DIR_LEFT | CONN_DIR_RIGHT,
        }
    }

    /// Creates a checkpoint that must be traversed vertically (up-down)
    pub fn vertical(point: Point) -> Self {
        Checkpoint {
            point,
            arrival_directions: CONN_DIR_UP | CONN_DIR_DOWN,
            departure_directions: CONN_DIR_UP | CONN_DIR_DOWN,
        }
    }
}

// ============================================================================
// Connector Callback
// ============================================================================

/// Callback function type for connector updates
pub type ConnectorCallback = Arc<dyn Fn(&ConnRef) + Send + Sync>;

// ============================================================================
// Route Cache (Task #20)
// ============================================================================

/// Cache entry for computed routes
#[derive(Debug, Clone)]
pub struct RouteCache {
    /// Hash of the routing configuration (endpoints + obstacles + params)
    config_hash: u64,
    /// Cached route
    cached_route: Polygon,
    /// When the cache was created (for age-based invalidation)
    timestamp: std::time::Instant,
}

impl RouteCache {
    /// Create new route cache entry
    fn new(config_hash: u64, route: Polygon) -> Self {
        RouteCache {
            config_hash,
            cached_route: route,
            timestamp: std::time::Instant::now(),
        }
    }

    /// Check if cache is valid for given configuration hash
    fn is_valid(&self, config_hash: u64, max_age_ms: u64) -> bool {
        if self.config_hash != config_hash {
            return false;
        }

        let age = self.timestamp.elapsed().as_millis() as u64;
        age < max_age_ms
    }

    /// Get cached route
    fn get_route(&self) -> &Polygon {
        &self.cached_route
    }
}

/// Compute configuration hash for route caching
pub fn compute_route_config_hash(
    src: &Point,
    dst: &Point,
    checkpoints: &[Point],
    obstacle_count: usize,
    routing_type: ConnType,
) -> u64 {
    let mut hasher = DefaultHasher::new();

    // Hash source and destination (quantized to avoid floating point issues)
    ((src.x * 1000.0) as i64).hash(&mut hasher);
    ((src.y * 1000.0) as i64).hash(&mut hasher);
    ((dst.x * 1000.0) as i64).hash(&mut hasher);
    ((dst.y * 1000.0) as i64).hash(&mut hasher);

    // Hash checkpoints
    for cp in checkpoints {
        ((cp.x * 1000.0) as i64).hash(&mut hasher);
        ((cp.y * 1000.0) as i64).hash(&mut hasher);
    }

    // Hash obstacle count (as proxy for obstacle configuration)
    obstacle_count.hash(&mut hasher);

    // Hash routing type
    (routing_type as u32).hash(&mut hasher);

    hasher.finish()
}

// ============================================================================
// Connector Reference
// ============================================================================

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
    checkpoints: Vec<Checkpoint>,
    /// Legacy checkpoints (simple points for backwards compatibility)
    legacy_checkpoints: Vec<Point>,
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
    /// Whether the route needs attention (fallback was used)
    needs_attention: bool,
    /// Route cache for performance optimization (Task #20)
    route_cache: Option<RouteCache>,
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
            legacy_checkpoints: Vec::new(),
            needs_repaint: false,
            has_fixed_route: false,
            active: true,
            callback: None,
            hate_crossings: false,
            needs_attention: false,
            route_cache: None,
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
            legacy_checkpoints: Vec::new(),
            needs_repaint: false,
            has_fixed_route: false,
            active: true,
            callback: None,
            hate_crossings: false,
            needs_attention: false,
            route_cache: None,
        }
    }

    /// Creates a connector with specific routing type
    pub fn with_type(id: u32, src: ConnEnd, dst: ConnEnd, routing_type: ConnType) -> Self {
        ConnRef {
            id,
            src,
            dst,
            routing_type,
            route: None,
            display_route: None,
            checkpoints: Vec::new(),
            legacy_checkpoints: Vec::new(),
            needs_repaint: false,
            has_fixed_route: false,
            active: true,
            callback: None,
            hate_crossings: false,
            needs_attention: false,
            route_cache: None,
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

    /// Returns mutable references to the endpoints
    pub fn endpoint_conn_ends_mut(&mut self) -> (&mut ConnEnd, &mut ConnEnd) {
        (&mut self.src, &mut self.dst)
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

    /// Sets routing checkpoints with direction constraints
    pub fn set_checkpoints(&mut self, checkpoints: Vec<Checkpoint>) {
        self.checkpoints = checkpoints;
        self.needs_repaint = true;
    }

    /// Returns the routing checkpoints
    pub fn checkpoints(&self) -> &[Checkpoint] {
        &self.checkpoints
    }

    /// Sets routing checkpoints (simple points, backwards compatible)
    pub fn set_routing_checkpoints(&mut self, checkpoints: Vec<Point>) {
        self.legacy_checkpoints = checkpoints;
        // Also update the new checkpoints format
        self.checkpoints = self
            .legacy_checkpoints
            .iter()
            .map(|p| Checkpoint::new(*p))
            .collect();
        self.needs_repaint = true;
    }

    /// Returns the routing checkpoints as simple points (backwards compatible)
    pub fn routing_checkpoints(&self) -> &[Point] {
        &self.legacy_checkpoints
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

    /// Returns whether the route needs attention (fallback was used)
    pub fn needs_attention(&self) -> bool {
        self.needs_attention
    }

    /// Sets whether the route needs attention
    pub fn set_needs_attention(&mut self, value: bool) {
        self.needs_attention = value;
    }

    /// Clear the route cache (Task #20)
    pub fn clear_route_cache(&mut self) {
        self.route_cache = None;
    }

    /// Check if route is cached and valid (Task #20)
    pub fn has_valid_cache(&self, config_hash: u64, max_age_ms: u64) -> bool {
        if let Some(ref cache) = self.route_cache {
            cache.is_valid(config_hash, max_age_ms)
        } else {
            false
        }
    }

    /// Get cached route if available (Task #20)
    pub fn get_cached_route(&self, config_hash: u64, max_age_ms: u64) -> Option<Polygon> {
        if let Some(ref cache) = self.route_cache {
            if cache.is_valid(config_hash, max_age_ms) {
                return Some(cache.get_route().clone());
            }
        }
        None
    }

    /// Store route in cache (Task #20)
    pub fn cache_route(&mut self, config_hash: u64, route: Polygon) {
        self.route_cache = Some(RouteCache::new(config_hash, route));
    }

    /// Sets the callback function for route updates
    pub fn set_callback(&mut self, callback: ConnectorCallback) {
        self.callback = Some(callback);
    }

    /// Clears the callback function
    pub fn clear_callback(&mut self) {
        self.callback = None;
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

    /// Internal method to set both route and display route
    pub(crate) fn set_routes(&mut self, route: Polygon, display_route: Polygon) {
        self.route = Some(route);
        self.display_route = Some(display_route);
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

    /// Returns the source endpoint's shape ID if attached
    pub fn source_shape_id(&self) -> Option<u32> {
        self.src.attached_shape_id()
    }

    /// Returns the destination endpoint's shape ID if attached
    pub fn dest_shape_id(&self) -> Option<u32> {
        self.dst.attached_shape_id()
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
            .field("checkpoints", &self.checkpoints.len())
            .field("hate_crossings", &self.hate_crossings)
            .finish()
    }
}

// ============================================================================
// Tests
// ============================================================================

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

    #[test]
    fn test_conn_end_types() {
        // Free point
        let free = ConnEnd::new(Point::new(5.0, 5.0));
        assert!(free.is_free_point());
        assert!(!free.is_shape_attached());

        // Shape attached
        let shape = ConnEnd::shape_pin(1, 0, Point::new(10.0, 10.0));
        assert!(shape.is_shape_attached());
        assert_eq!(shape.attached_shape_id(), Some(1));

        // Junction attached
        let junction = ConnEnd::junction(2, Point::new(15.0, 15.0));
        assert!(junction.is_junction_attached());
        assert_eq!(junction.attached_junction_id(), Some(2));
    }

    #[test]
    fn test_conn_directions() {
        let mut end = ConnEnd::new(Point::new(0.0, 0.0));
        assert_eq!(end.directions(), CONN_DIR_ALL);

        end.set_directions(CONN_DIR_UP | CONN_DIR_DOWN);
        assert!(has_dir_up(end.directions()));
        assert!(has_dir_down(end.directions()));
        assert!(!has_dir_left(end.directions()));
        assert!(!has_dir_right(end.directions()));
    }

    #[test]
    fn test_checkpoint() {
        let cp = Checkpoint::new(Point::new(50.0, 50.0));
        assert_eq!(cp.arrival_directions, CONN_DIR_ALL);

        let h_cp = Checkpoint::horizontal(Point::new(50.0, 50.0));
        assert!(has_dir_left(h_cp.arrival_directions));
        assert!(has_dir_right(h_cp.arrival_directions));
        assert!(!has_dir_up(h_cp.arrival_directions));

        let v_cp = Checkpoint::vertical(Point::new(50.0, 50.0));
        assert!(has_dir_up(v_cp.arrival_directions));
        assert!(has_dir_down(v_cp.arrival_directions));
        assert!(!has_dir_left(v_cp.arrival_directions));
    }
}
