//! WASM bindings for libavoid
//!
//! This module provides JavaScript-compatible bindings matching the libavoid-js API.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use crate::{
    Router as RustRouter, Point as RustPoint, ConnRef as RustConnRef,
    ConnEnd as RustConnEnd, Polygon as RustPolygon,
    ShapeRef as RustShapeRef, PolygonInterface, Obstacle,
    BBox as RustBox, Rectangle as RustRectangle,
    JunctionRef as RustJunctionRef,
    shape::ConnectionPin as RustConnectionPin,
    hyperedge::HyperedgeRerouter as RustHyperedgeRerouter,
};

// =============================================================================
// Constants - matching libavoid-js values exactly
// These are Rust constants, exported to JS via setup.js
// =============================================================================

#[cfg(feature = "wasm")]
pub mod constants {
    /// Router flags (matching libavoid-js)
    pub const POLY_LINE_ROUTING: u32 = 1;
    pub const ORTHOGONAL_ROUTING: u32 = 2;

    /// Connection direction flags
    pub const CONN_DIR_NONE: u32 = 0;
    pub const CONN_DIR_UP: u32 = 1;
    pub const CONN_DIR_DOWN: u32 = 2;
    pub const CONN_DIR_LEFT: u32 = 4;
    pub const CONN_DIR_RIGHT: u32 = 8;
    pub const CONN_DIR_ALL: u32 = 15;

    /// Connection types
    pub const CONN_TYPE_NONE: u32 = 0;
    pub const CONN_TYPE_POLY_LINE: u32 = 1;
    pub const CONN_TYPE_ORTHOGONAL: u32 = 2;

    /// Routing parameters (for setRoutingParameter)
    pub const SEGMENT_PENALTY: u32 = 0;
    pub const ANGLE_PENALTY: u32 = 1;
    pub const CROSSING_PENALTY: u32 = 2;
    pub const CLUSTER_CROSSING_PENALTY: u32 = 3;
    pub const FIXED_SHARED_PATH_PENALTY: u32 = 4;
    pub const PORT_DIRECTION_PENALTY: u32 = 5;
    pub const SHAPE_BUFFER_DISTANCE: u32 = 6;
    pub const IDEAL_NUDGING_DISTANCE: u32 = 7;
    pub const REVERSE_DIRECTION_PENALTY: u32 = 8;

    /// Routing options (for setRoutingOption)
    pub const NUDGE_ORTHOGONAL_SEGMENTS: u32 = 0;
    pub const IMPROVE_HYPEREDGE_ROUTES: u32 = 1;
    pub const PENALISE_SHARED_PATHS: u32 = 2;
    pub const NUDGE_COLINEAR_SEGMENTS: u32 = 3;
    pub const UNIFYING_NUDGING_STEP: u32 = 4;
    pub const IMPROVE_HYPEREDGE_ADD_DELETE: u32 = 5;
    pub const NUDGE_SHARED_PATHS_COMMON_END: u32 = 6;
}

#[cfg(feature = "wasm")]
use constants::*;

// =============================================================================
// AvoidLib - Library loader (compatibility with libavoid-js)
// =============================================================================

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct AvoidLib;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl AvoidLib {
    /// Load the library (compatibility with libavoid-js)
    /// In wasm-pack, actual loading is handled by the generated init function
    #[wasm_bindgen]
    pub fn load() -> AvoidLib {
        AvoidLib
    }

    /// Get instance (compatibility with libavoid-js)
    #[wasm_bindgen(js_name = getInstance)]
    pub fn get_instance() -> AvoidLib {
        AvoidLib
    }
}

// =============================================================================
// Point
// =============================================================================

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct Point {
    inner: RustPoint,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Point {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64) -> Point {
        Point {
            inner: RustPoint::new(x, y),
        }
    }

    /// Create a Point at origin (0, 0)
    #[wasm_bindgen(js_name = origin)]
    pub fn origin() -> Point {
        Point {
            inner: RustPoint::default(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn x(&self) -> f64 {
        self.inner.x
    }

    #[wasm_bindgen(setter)]
    pub fn set_x(&mut self, x: f64) {
        self.inner.x = x;
    }

    #[wasm_bindgen(getter)]
    pub fn y(&self) -> f64 {
        self.inner.y
    }

    #[wasm_bindgen(setter)]
    pub fn set_y(&mut self, y: f64) {
        self.inner.y = y;
    }

    #[wasm_bindgen(getter)]
    pub fn id(&self) -> u32 {
        self.inner.id
    }

    #[wasm_bindgen(setter)]
    pub fn set_id(&mut self, id: u32) {
        self.inner.id = id;
    }

    #[wasm_bindgen(getter)]
    pub fn vn(&self) -> u32 {
        self.inner.vn
    }

    #[wasm_bindgen(setter)]
    pub fn set_vn(&mut self, vn: u32) {
        self.inner.vn = vn;
    }

    /// Check equality with another point
    pub fn equal(&self, other: &Point) -> bool {
        self.inner.equals(&other.inner)
    }
}

// =============================================================================
// Polygon
// =============================================================================

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct Polygon {
    inner: RustPolygon,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Polygon {
    #[wasm_bindgen(constructor)]
    pub fn new(vertex_count: usize) -> Polygon {
        Polygon {
            inner: RustPolygon::with_capacity(vertex_count),
        }
    }

    #[wasm_bindgen(js_name = set_ps)]
    pub fn set_ps(&mut self, index: usize, point: &Point) {
        if index >= self.inner.size() {
            // Resize if needed
            while self.inner.size() <= index {
                self.inner.push(RustPoint::new(0.0, 0.0));
            }
        }
        self.inner.set_point(index, point.inner);
    }

    #[wasm_bindgen(js_name = get_ps)]
    pub fn get_ps(&self, index: usize) -> Option<Point> {
        if index < self.inner.size() {
            Some(Point {
                inner: *self.inner.at(index),
            })
        } else {
            None
        }
    }

    #[wasm_bindgen]
    pub fn size(&self) -> usize {
        self.inner.size()
    }

    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[wasm_bindgen]
    pub fn empty(&self) -> bool {
        self.inner.empty()
    }

    #[wasm_bindgen(js_name = setPoint)]
    pub fn set_point(&mut self, index: usize, point: &Point) {
        self.set_ps(index, point);
    }

    #[wasm_bindgen]
    pub fn id(&self) -> u32 {
        self.inner.id()
    }

    #[wasm_bindgen]
    pub fn at(&self, index: usize) -> Option<Point> {
        if index < self.inner.size() {
            Some(Point {
                inner: *self.inner.at(index),
            })
        } else {
            None
        }
    }

    /// Returns the bounding rectangle as a polygon
    #[wasm_bindgen(js_name = boundingRectPolygon)]
    pub fn bounding_rect_polygon(&self) -> Polygon {
        let bbox = self.inner.bounding_rect();
        let mut poly = RustPolygon::with_capacity(4);
        poly.push(bbox.min);
        poly.push(RustPoint::new(bbox.max.x, bbox.min.y));
        poly.push(bbox.max);
        poly.push(RustPoint::new(bbox.min.x, bbox.max.y));
        Polygon { inner: poly }
    }

    /// Returns the bounding box offset by the given amount
    #[wasm_bindgen(js_name = offsetBoundingBox)]
    pub fn offset_bounding_box(&self, offset: f64) -> Box {
        let bbox = self.inner.bounding_rect();
        Box {
            inner: RustBox::new(
                RustPoint::new(bbox.min.x - offset, bbox.min.y - offset),
                RustPoint::new(bbox.max.x + offset, bbox.max.y + offset),
            ),
        }
    }

    /// Returns an offset polygon
    #[wasm_bindgen(js_name = offsetPolygon)]
    pub fn offset_polygon(&self, offset: f64) -> Polygon {
        Polygon {
            inner: self.inner.offset_polygon(offset),
        }
    }
}

// =============================================================================
// ConnEnd
// =============================================================================

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct ConnEnd {
    inner: RustConnEnd,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl ConnEnd {
    #[wasm_bindgen(constructor)]
    pub fn new(point: &Point) -> ConnEnd {
        ConnEnd {
            inner: RustConnEnd::new(point.inner),
        }
    }

    /// Create a ConnEnd attached to a shape's connection pin class
    #[wasm_bindgen(js_name = fromShapePin)]
    pub fn from_shape_pin(shape: &ShapeRef, pin_class_id: u32) -> ConnEnd {
        // Use default position (0,0) - will be resolved to actual pin position during routing
        ConnEnd {
            inner: RustConnEnd::with_pin(RustPoint::default(), shape.id(), pin_class_id),
        }
    }
}

// =============================================================================
// ConnRef
// =============================================================================

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct ConnRef {
    inner: RustConnRef,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl ConnRef {
    #[wasm_bindgen(constructor)]
    pub fn new(router: &Router) -> ConnRef {
        let id = router.next_id();
        ConnRef {
            inner: RustConnRef::new(id),
        }
    }

    /// Create a ConnRef with source and destination endpoints
    #[wasm_bindgen(js_name = createWithEndpoints)]
    pub fn create_with_endpoints(router: &Router, src: &ConnEnd, dst: &ConnEnd) -> ConnRef {
        let id = router.next_id();
        ConnRef {
            inner: RustConnRef::with_endpoints(id, src.inner.clone(), dst.inner.clone()),
        }
    }

    /// Create a ConnRef with endpoints and a specific ID
    #[wasm_bindgen(js_name = createWithId)]
    pub fn create_with_id(router: &Router, src: &ConnEnd, dst: &ConnEnd, id: u32) -> ConnRef {
        // Note: Using provided ID instead of router.next_id()
        let _ = router; // Router reference unused but kept for API consistency
        ConnRef {
            inner: RustConnRef::with_endpoints(id, src.inner.clone(), dst.inner.clone()),
        }
    }

    #[wasm_bindgen(js_name = setCallback)]
    pub fn set_callback(&mut self, _callback: JsValue, _context: JsValue) {
        // TODO: Implement callback support - would require storing js_sys::Function
        // and invoking it when route changes
    }

    #[wasm_bindgen(js_name = displayRoute)]
    pub fn display_route(&self) -> Option<Polygon> {
        self.inner.display_route().map(|r| Polygon {
            inner: r.clone(),
        })
    }

    #[wasm_bindgen]
    pub fn id(&self) -> u32 {
        self.inner.id()
    }

    #[wasm_bindgen(js_name = setSourceEndpoint)]
    pub fn set_source_endpoint(&mut self, conn_end: &ConnEnd) {
        self.inner.set_source_endpoint(conn_end.inner.clone());
    }

    #[wasm_bindgen(js_name = setDestEndpoint)]
    pub fn set_dest_endpoint(&mut self, conn_end: &ConnEnd) {
        self.inner.set_dest_endpoint(conn_end.inner.clone());
    }

    #[wasm_bindgen(js_name = routingType)]
    pub fn routing_type(&self) -> u32 {
        match self.inner.routing_type() {
            crate::ConnType::PolyLine => CONN_TYPE_POLY_LINE,
            crate::ConnType::Orthogonal => CONN_TYPE_ORTHOGONAL,
        }
    }

    #[wasm_bindgen(js_name = setRoutingType)]
    pub fn set_routing_type(&mut self, routing_type: u32) {
        let conn_type = if routing_type == CONN_TYPE_ORTHOGONAL {
            crate::ConnType::Orthogonal
        } else {
            crate::ConnType::PolyLine
        };
        self.inner.set_routing_type(conn_type);
    }

    /// Set whether this connector hates crossings
    #[wasm_bindgen(js_name = setHateCrossings)]
    pub fn set_hate_crossings(&mut self, value: bool) {
        self.inner.set_hate_crossings(value);
    }

    /// Check if this connector hates crossings
    #[wasm_bindgen(js_name = doesHateCrossings)]
    pub fn does_hate_crossings(&self) -> bool {
        self.inner.does_hate_crossings()
    }
}

// =============================================================================
// ShapeRef
// =============================================================================

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct ShapeRef {
    inner: RustShapeRef,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl ShapeRef {
    #[wasm_bindgen(constructor)]
    pub fn new(router: &Router, polygon: &Polygon) -> ShapeRef {
        let id = router.next_id();
        ShapeRef {
            inner: RustShapeRef::new(id, polygon.inner.clone()),
        }
    }

    /// Create a ShapeRef with a specific ID
    #[wasm_bindgen(js_name = createWithId)]
    pub fn create_with_id(_router: &Router, polygon: &Polygon, id: u32) -> ShapeRef {
        ShapeRef {
            inner: RustShapeRef::new(id, polygon.inner.clone()),
        }
    }

    #[wasm_bindgen]
    pub fn id(&self) -> u32 {
        self.inner.id()
    }

    /// Returns the shape's polygon
    #[wasm_bindgen]
    pub fn polygon(&self) -> Polygon {
        Polygon {
            inner: self.inner.polygon().clone(),
        }
    }

    /// Returns the shape's position (center of bounding box)
    #[wasm_bindgen]
    pub fn position(&self) -> Point {
        Point {
            inner: self.inner.position(),
        }
    }

    /// Updates the shape's polygon
    #[wasm_bindgen(js_name = setNewPoly)]
    pub fn set_new_poly(&mut self, polygon: &Polygon) {
        self.inner.set_polygon(polygon.inner.clone());
    }
}

// =============================================================================
// ShapeConnectionPin
// =============================================================================

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct ShapeConnectionPin {
    inner: RustConnectionPin,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl ShapeConnectionPin {
    /// Create a connection pin on a shape
    /// shape: The shape to attach the pin to
    /// class_id: Class ID for grouping pins
    /// x_offset: X offset from shape center (or proportion if proportional)
    /// y_offset: Y offset from shape center (or proportion if proportional)
    /// inside_offset: Offset inside the shape boundary
    /// vis_dirs: Visibility directions (ConnDir flags)
    #[wasm_bindgen(constructor)]
    pub fn new(
        shape: &mut ShapeRef,
        class_id: u32,
        x_offset: f64,
        y_offset: f64,
        inside_offset: f64,
        vis_dirs: u32,
    ) -> ShapeConnectionPin {
        let position = RustPoint::new(x_offset, y_offset);
        let pin = RustConnectionPin::with_all(class_id, class_id, position, vis_dirs, inside_offset);
        shape.inner.add_connection_pin(pin.clone());
        ShapeConnectionPin { inner: pin }
    }

    /// Create a connection pin on a junction
    #[wasm_bindgen(js_name = createOnJunction)]
    pub fn create_on_junction(
        _junction: &JunctionRef,
        class_id: u32,
        vis_dirs: Option<u32>,
    ) -> ShapeConnectionPin {
        let dirs = vis_dirs.unwrap_or(CONN_DIR_ALL);
        let pin = RustConnectionPin::with_directions(class_id, RustPoint::default(), dirs);
        ShapeConnectionPin { inner: pin }
    }

    /// Set the connection cost for this pin
    #[wasm_bindgen(js_name = setConnectionCost)]
    pub fn set_connection_cost(&mut self, cost: f64) {
        self.inner.set_connection_cost(cost);
    }

    /// Get the pin's position
    #[wasm_bindgen]
    pub fn position(&self) -> Point {
        Point { inner: self.inner.position }
    }

    /// Get the visibility directions
    #[wasm_bindgen]
    pub fn directions(&self) -> u32 {
        self.inner.directions
    }

    /// Set whether this pin is exclusive
    #[wasm_bindgen(js_name = setExclusive)]
    pub fn set_exclusive(&mut self, exclusive: bool) {
        self.inner.set_exclusive(exclusive);
    }

    /// Check if this pin is exclusive
    #[wasm_bindgen(js_name = isExclusive)]
    pub fn is_exclusive(&self) -> bool {
        self.inner.is_exclusive()
    }
}

// =============================================================================
// JunctionRef
// =============================================================================

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct JunctionRef {
    inner: RustJunctionRef,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl JunctionRef {
    #[wasm_bindgen(constructor)]
    pub fn new(router: &Router, position: &Point) -> JunctionRef {
        let id = router.next_id();
        JunctionRef {
            inner: RustJunctionRef::new(id, position.inner),
        }
    }

    /// Create a JunctionRef with a specific ID
    #[wasm_bindgen(js_name = createWithId)]
    pub fn create_with_id(_router: &Router, position: &Point, id: u32) -> JunctionRef {
        JunctionRef {
            inner: RustJunctionRef::new(id, position.inner),
        }
    }

    #[wasm_bindgen]
    pub fn id(&self) -> u32 {
        self.inner.id()
    }

    /// Returns the junction's position
    #[wasm_bindgen]
    pub fn position(&self) -> Point {
        Point {
            inner: self.inner.position(),
        }
    }

    /// Sets the junction's position
    #[wasm_bindgen(js_name = setPosition)]
    pub fn set_position(&mut self, position: &Point) {
        self.inner.set_position(position.inner);
    }
}

// =============================================================================
// HyperedgeRerouter
// =============================================================================

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct HyperedgeRerouter {
    inner: RustHyperedgeRerouter,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl HyperedgeRerouter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> HyperedgeRerouter {
        HyperedgeRerouter {
            inner: RustHyperedgeRerouter::new(),
        }
    }

    /// Register a hyperedge for rerouting based on a junction
    /// Returns the index/ID of the registered hyperedge
    #[wasm_bindgen(js_name = registerHyperedgeForRerouting)]
    pub fn register_hyperedge_for_rerouting(&mut self, junction: &JunctionRef) -> u32 {
        // Create a new hyperedge with the junction as the starting point
        use crate::hyperedge::HyperedgeRef;
        use crate::ConnEnd as RustConnEndType;

        let terminal = RustConnEndType::new(junction.inner.position());
        let hyperedge = HyperedgeRef::new(self.inner.hyperedges().len() as u32, vec![terminal]);
        self.inner.register_hyperedge(hyperedge);
        (self.inner.hyperedges().len() - 1) as u32
    }
}

// =============================================================================
// Router
// =============================================================================

#[cfg(feature = "wasm")]
use std::cell::Cell;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct Router {
    inner: RustRouter,
    next_id: Cell<u32>,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Router {
    /// Create a new router with the specified routing flags
    /// flags should be PolyLineRouting (1) or OrthogonalRouting (2)
    #[wasm_bindgen(constructor)]
    pub fn new(flags: u32) -> Router {
        Router {
            inner: RustRouter::new(flags),
            next_id: Cell::new(1),
        }
    }

    #[wasm_bindgen(js_name = processTransaction)]
    pub fn process_transaction(&mut self) -> bool {
        self.inner.process_transaction();
        true
    }

    /// Enable or disable transaction mode
    /// When enabled, changes are batched until processTransaction is called
    /// This is required for nudging to work correctly
    #[wasm_bindgen(js_name = setTransactionUse)]
    pub fn set_transaction_use(&mut self, use_transactions: bool) {
        self.inner.set_transaction_use(use_transactions);
    }

    /// Move shape by offset from current position
    /// x_diff, y_diff are offsets (deltas) from current position, matching libavoid-js semantics
    #[wasm_bindgen(js_name = moveShape)]
    pub fn move_shape(&mut self, shape: &ShapeRef, x_diff: f64, y_diff: f64) {
        // libavoid-js semantics: x_diff/y_diff are offsets from current position
        if let Some(router_shape) = self.inner.get_shape(shape.id()) {
            let current = router_shape.position();
            let new_pos = RustPoint::new(current.x + x_diff, current.y + y_diff);
            self.inner.move_shape(shape.id(), new_pos);
        }
    }

    /// Move shape to a new polygon position
    #[wasm_bindgen(js_name = moveShapeTo)]
    pub fn move_shape_to(&mut self, shape: &ShapeRef, new_polygon: &Polygon) {
        // Calculate the center offset needed to move from current to new polygon center
        let new_bbox = new_polygon.inner.bounding_rect();
        let new_center = RustPoint::new(
            (new_bbox.min.x + new_bbox.max.x) / 2.0,
            (new_bbox.min.y + new_bbox.max.y) / 2.0,
        );
        self.inner.move_shape(shape.id(), new_center);
    }

    #[wasm_bindgen(js_name = setRoutingParameter)]
    pub fn set_routing_parameter(&mut self, param: u32, value: f64) {
        use crate::RoutingParameter;
        let param = match param {
            0 => RoutingParameter::SegmentPenalty,
            1 => RoutingParameter::BendPenalty,
            2 => RoutingParameter::CrossingPenalty,
            3 => RoutingParameter::ClusterCrossingPenalty,
            4 => RoutingParameter::FixedSharedPathPenalty,
            5 => RoutingParameter::PortDirectionPenalty,
            6 => RoutingParameter::ShapeBufferDistance,
            7 => RoutingParameter::IdealNudgingDistance,
            8 => RoutingParameter::ReverseDirectionPenalty,
            _ => return,
        };
        self.inner.set_routing_parameter(param, value);
    }

    #[wasm_bindgen(js_name = routingParameter)]
    pub fn routing_parameter(&self, param: u32) -> f64 {
        use crate::RoutingParameter;
        let param = match param {
            0 => RoutingParameter::SegmentPenalty,
            1 => RoutingParameter::BendPenalty,
            2 => RoutingParameter::CrossingPenalty,
            3 => RoutingParameter::ClusterCrossingPenalty,
            4 => RoutingParameter::FixedSharedPathPenalty,
            5 => RoutingParameter::PortDirectionPenalty,
            6 => RoutingParameter::ShapeBufferDistance,
            7 => RoutingParameter::IdealNudgingDistance,
            8 => RoutingParameter::ReverseDirectionPenalty,
            _ => return 0.0,
        };
        self.inner.routing_parameter(param)
    }

    #[wasm_bindgen(js_name = setRoutingOption)]
    pub fn set_routing_option(&mut self, option: u32, value: bool) {
        use crate::RoutingOption;
        let opt = match option {
            0 => RoutingOption::NudgeOrthogonalRoutes,
            1 => RoutingOption::ImproveHyperedgeRoutes,
            2 => RoutingOption::PenalisePortDirections,
            6 => RoutingOption::NudgeSharedPathsWithCommonEndPoint,
            _ => return,
        };
        self.inner.set_routing_option(opt, value);
    }

    #[wasm_bindgen(js_name = routingOption)]
    pub fn routing_option(&self, option: u32) -> bool {
        use crate::RoutingOption;
        let opt = match option {
            0 => RoutingOption::NudgeOrthogonalRoutes,
            1 => RoutingOption::ImproveHyperedgeRoutes,
            2 => RoutingOption::PenalisePortDirections,
            6 => RoutingOption::NudgeSharedPathsWithCommonEndPoint,
            _ => return false,
        };
        self.inner.routing_option(opt)
    }

    /// Add a shape to the router for routing consideration
    #[wasm_bindgen(js_name = addShape)]
    pub fn add_shape(&mut self, shape: &ShapeRef) {
        self.inner.add_shape(shape.inner.polygon().clone(), shape.id());
    }

    /// Add a connector to the router for routing
    #[wasm_bindgen(js_name = addConnector)]
    pub fn add_connector(&mut self, conn: &ConnRef) {
        self.inner.add_connector(conn.inner.clone());
    }

    /// Get the display route for a connector by ID
    /// Use this after processTransaction to get the computed route
    #[wasm_bindgen(js_name = getConnectorRoute)]
    pub fn get_connector_route(&self, conn_id: u32) -> Option<Polygon> {
        self.inner.get_connector(conn_id)
            .and_then(|c| c.display_route())
            .map(|r| Polygon { inner: r.clone() })
    }

    /// Update a connector's endpoints in the router
    /// Call this after modifying connector endpoints to sync with router
    #[wasm_bindgen(js_name = updateConnector)]
    pub fn update_connector(&mut self, conn: &ConnRef) {
        if let Some(internal_conn) = self.inner.get_connector_mut(conn.id()) {
            let (src, dst) = conn.inner.endpoint_conn_ends();
            internal_conn.set_source_endpoint(src.clone());
            internal_conn.set_dest_endpoint(dst.clone());
        }
    }

    #[wasm_bindgen(js_name = deleteShape)]
    pub fn delete_shape(&mut self, shape: &ShapeRef) {
        self.inner.delete_shape(shape.id());
    }

    #[wasm_bindgen(js_name = deleteConnector)]
    pub fn delete_connector(&mut self, conn: &ConnRef) {
        self.inner.delete_connector(conn.id());
    }

    /// Output info about the router (for debugging)
    #[wasm_bindgen(js_name = outputInstanceToSVG)]
    pub fn output_instance_to_svg(&self) -> String {
        // Return a simple SVG representation of the router state
        format!("<!-- Router SVG output not yet implemented -->")
    }

    pub(crate) fn next_id(&self) -> u32 {
        let id = self.next_id.get();
        self.next_id.set(id + 1);
        id
    }
}

// =============================================================================
// Box
// =============================================================================

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct Box {
    inner: RustBox,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Box {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Box {
        Box {
            inner: RustBox::new(RustPoint::default(), RustPoint::default()),
        }
    }

    /// Create a Box from coordinates
    #[wasm_bindgen(js_name = fromCoords)]
    pub fn from_coords(x1: f64, y1: f64, x2: f64, y2: f64) -> Box {
        Box {
            inner: RustBox::from_coords(x1, y1, x2, y2),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn min(&self) -> Point {
        Point { inner: self.inner.min }
    }

    #[wasm_bindgen(setter)]
    pub fn set_min(&mut self, point: &Point) {
        self.inner.min = point.inner;
    }

    #[wasm_bindgen(getter)]
    pub fn max(&self) -> Point {
        Point { inner: self.inner.max }
    }

    #[wasm_bindgen(setter)]
    pub fn set_max(&mut self, point: &Point) {
        self.inner.max = point.inner;
    }

    #[wasm_bindgen]
    pub fn width(&self) -> f64 {
        self.inner.width()
    }

    #[wasm_bindgen]
    pub fn height(&self) -> f64 {
        self.inner.height()
    }

    /// Returns length along the specified dimension (0 = width, 1 = height)
    #[wasm_bindgen]
    pub fn length(&self, dimension: usize) -> f64 {
        match dimension {
            0 => self.inner.width(),
            1 => self.inner.height(),
            _ => self.inner.length(),
        }
    }

    /// Checks if the box contains a point
    #[wasm_bindgen]
    pub fn contains(&self, point: &Point) -> bool {
        self.inner.contains(&point.inner)
    }
}

// =============================================================================
// Rectangle
// =============================================================================

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct Rectangle {
    inner: RustRectangle,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Rectangle {
    /// Create a rectangle from center point, width, and height
    #[wasm_bindgen(constructor)]
    pub fn new(center: &Point, width: f64, height: f64) -> Rectangle {
        Rectangle {
            inner: RustRectangle::new(center.inner, width, height),
        }
    }

    /// Create a rectangle from two corner points
    #[wasm_bindgen(js_name = fromCorners)]
    pub fn from_corners(p1: &Point, p2: &Point) -> Rectangle {
        Rectangle {
            inner: RustRectangle::new_from_points(p1.inner, p2.inner),
        }
    }

    #[wasm_bindgen]
    pub fn width(&self) -> f64 {
        self.inner.width()
    }

    #[wasm_bindgen]
    pub fn height(&self) -> f64 {
        self.inner.height()
    }

    #[wasm_bindgen]
    pub fn center(&self) -> Point {
        Point { inner: self.inner.center() }
    }

    /// Convert rectangle to a polygon (for use with ShapeRef)
    #[wasm_bindgen(js_name = toPolygon)]
    pub fn to_polygon(&self) -> Polygon {
        Polygon {
            inner: self.inner.clone().into(),
        }
    }
}

// =============================================================================
// WASM Initialization
// =============================================================================

#[cfg(feature = "wasm")]
#[wasm_bindgen(start)]
pub fn main() {
    // WASM initialization complete
}
