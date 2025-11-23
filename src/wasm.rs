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

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct AvoidLib;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl AvoidLib {
    /// Load the library (compatibility with libavoid-js)
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
}

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
}

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

    #[wasm_bindgen(js_name = setDestEndpoint)]
    pub fn set_dest_endpoint(&mut self, conn_end: &ConnEnd) {
        self.inner.set_dest_endpoint(conn_end.inner.clone());
    }
}

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

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct Router {
    inner: RustRouter,
    next_id: u32,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub enum RoutingType {
    PolyLineRouting = 0,
    OrthogonalRouting = 1,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl Router {
    #[wasm_bindgen(constructor)]
    pub fn new(routing_type: RoutingType) -> Router {
        let flags = routing_type as u32;
        Router {
            inner: RustRouter::new(flags),
            next_id: 1,
        }
    }

    #[wasm_bindgen(js_name = processTransaction)]
    pub fn process_transaction(&mut self) {
        self.inner.process_transaction();
    }

    #[wasm_bindgen(js_name = moveShape)]
    pub fn move_shape(&mut self, shape: &ShapeRef, x: f64, y: f64) {
        self.inner.move_shape(shape.id(), RustPoint::new(x, y));
    }

    pub(crate) fn next_id(&self) -> u32 {
        self.next_id
    }
}

// Initialize WASM module
#[cfg(feature = "wasm")]
#[wasm_bindgen(start)]
pub fn main() {
    // WASM initialization complete
}
