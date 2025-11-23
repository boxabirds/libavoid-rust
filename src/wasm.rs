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

    /// Routing parameters
    pub const SEGMENT_PENALTY: u32 = 0;
    pub const ANGLE_PENALTY: u32 = 1;
    pub const CROSSING_PENALTY: u32 = 2;
    pub const CLUSTER_CROSSING_PENALTY: u32 = 3;
    pub const FIXED_SHARED_PATH_PENALTY: u32 = 4;
    pub const PORT_DIRECTION_PENALTY: u32 = 5;
    pub const SHAPE_BUFFER_DISTANCE: u32 = 6;
    pub const IDEAL_NUDGING_DISTANCE: u32 = 7;
    pub const REVERSE_DIRECTION_PENALTY: u32 = 8;

    /// Routing options
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

    #[wasm_bindgen(js_name = setCallback)]
    pub fn set_callback(&mut self, _callback: JsValue, _context: JsValue) {
        // TODO: Implement callback support
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

    #[wasm_bindgen]
    pub fn id(&self) -> u32 {
        self.inner.id()
    }
}

// =============================================================================
// Router
// =============================================================================

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct Router {
    inner: RustRouter,
    next_id: u32,
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
            next_id: 1,
        }
    }

    #[wasm_bindgen(js_name = processTransaction)]
    pub fn process_transaction(&mut self) -> bool {
        self.inner.process_transaction();
        true
    }

    #[wasm_bindgen(js_name = moveShape)]
    pub fn move_shape(&mut self, shape: &ShapeRef, x: f64, y: f64) {
        self.inner.move_shape(shape.id(), RustPoint::new(x, y));
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

    #[wasm_bindgen(js_name = deleteShape)]
    pub fn delete_shape(&mut self, shape: &ShapeRef) {
        self.inner.delete_shape(shape.id());
    }

    #[wasm_bindgen(js_name = deleteConnector)]
    pub fn delete_connector(&mut self, conn: &ConnRef) {
        self.inner.delete_connector(conn.id());
    }

    pub(crate) fn next_id(&self) -> u32 {
        self.next_id
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
