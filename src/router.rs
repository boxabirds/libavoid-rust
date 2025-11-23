//! Core routing functionality
//!
//! This module provides the Router, which is the main entry point for using libavoid.
//! The Router manages shapes, connectors, and performs the routing calculations.

use crate::geometry::{Polygon, Point, PolygonInterface};
use crate::connector::{ConnRef, ConnEnd, ConnType};
use crate::shape::ShapeRef;
use crate::obstacle::Obstacle;
use crate::visibility::VisibilityGraph;
use crate::graph::PathFinder;
use crate::orthogonal::OrthogonalRouter;
use std::collections::HashMap;

/// Router flags for initialization
pub type RouterFlags = u32;

/// No special flags
pub const ROUTER_FLAG_NONE: RouterFlags = 0;

/// Use transactions for batched updates
pub const ROUTER_FLAG_USE_TRANSACTIONS: RouterFlags = 1;

/// Routing parameters
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoutingParameter {
    /// Penalty for segments
    SegmentPenalty,
    /// Penalty for bends/corners
    BendPenalty,
    /// Penalty for crossing another connector
    CrossingPenalty,
    /// Penalty for being close to obstacles
    ClusterCrossingPenalty,
    /// Ideal nudging distance for orthogonal routes
    IdealNudgingDistance,
    /// Shape buffer distance
    ShapeBufferDistance,
    /// Penalty for fixed shared path segments
    FixedSharedPathPenalty,
    /// Penalty for wrong port directions
    PortDirectionPenalty,
    /// Penalty for reverse direction routing
    ReverseDirectionPenalty,
}

/// Routing options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoutingOption {
    /// Nudge orthogonal routes to avoid overlap
    NudgeOrthogonalRoutes,
    /// Improve hyperedge routes
    ImproveHyperedgeRoutes,
    /// Penalise ports with incorrect direction
    PenalisePortDirections,
    /// Nudge shared paths to common routes
    NudgeSharedPathsWithCommonEndPoint,
}

/// The main router instance for managing connector routing
pub struct Router {
    /// Router configuration flags
    flags: RouterFlags,
    /// All shapes in the scene
    shapes: HashMap<u32, ShapeRef>,
    /// All connectors in the scene
    connectors: HashMap<u32, ConnRef>,
    /// Visibility graph for polyline routing
    vis_graph: VisibilityGraph,
    /// Visibility graph for orthogonal routing
    vis_orth_graph: VisibilityGraph,
    /// Pathfinder for route calculation
    path_finder: PathFinder,
    /// Orthogonal router
    orthogonal_router: OrthogonalRouter,
    /// Routing parameters
    parameters: HashMap<RoutingParameter, f64>,
    /// Routing options
    options: HashMap<RoutingOption, bool>,
    /// Transaction mode enabled
    transaction_mode: bool,
    /// Pending operations in transaction
    transaction_pending: Vec<TransactionOp>,
    /// Next shape ID
    next_shape_id: u32,
    /// Next connector ID
    next_connector_id: u32,
}

/// Transaction operation types
#[derive(Debug, Clone)]
enum TransactionOp {
    AddShape(u32),
    DeleteShape(u32),
    MoveShape(u32, Point),
    AddConnector(u32),
    DeleteConnector(u32),
}

impl Router {
    /// Creates a new router with the given flags
    pub fn new(flags: RouterFlags) -> Self {
        let mut router = Router {
            flags,
            shapes: HashMap::new(),
            connectors: HashMap::new(),
            vis_graph: VisibilityGraph::new(),
            vis_orth_graph: VisibilityGraph::new(),
            path_finder: PathFinder::new(),
            orthogonal_router: OrthogonalRouter::new(),
            parameters: HashMap::new(),
            options: HashMap::new(),
            transaction_mode: (flags & ROUTER_FLAG_USE_TRANSACTIONS) != 0,
            transaction_pending: Vec::new(),
            next_shape_id: 1,
            next_connector_id: 1,
        };

        // Set default parameters
        router.parameters.insert(RoutingParameter::SegmentPenalty, 1.0);
        router.parameters.insert(RoutingParameter::BendPenalty, 50.0);
        router.parameters.insert(RoutingParameter::CrossingPenalty, 0.0);
        router.parameters.insert(RoutingParameter::ClusterCrossingPenalty, 4000.0);
        router.parameters.insert(RoutingParameter::IdealNudgingDistance, 4.0);
        router.parameters.insert(RoutingParameter::ShapeBufferDistance, 8.0);

        // Set default options
        router.options.insert(RoutingOption::NudgeOrthogonalRoutes, false);
        router.options.insert(RoutingOption::ImproveHyperedgeRoutes, true);
        router.options.insert(RoutingOption::PenalisePortDirections, false);
        router.options.insert(RoutingOption::NudgeSharedPathsWithCommonEndPoint, false);

        router
    }

    /// Adds a shape to the router
    pub fn add_shape(&mut self, polygon: Polygon, id: u32) -> u32 {
        let shape_id = if id == 0 {
            let id = self.next_shape_id;
            self.next_shape_id += 1;
            id
        } else {
            if id >= self.next_shape_id {
                self.next_shape_id = id + 1;
            }
            id
        };

        let shape = ShapeRef::new(shape_id, polygon);
        self.shapes.insert(shape_id, shape);

        if self.transaction_mode {
            self.transaction_pending.push(TransactionOp::AddShape(shape_id));
        } else {
            self.rebuild_visibility_graph();
        }

        shape_id
    }

    /// Deletes a shape from the router
    pub fn delete_shape(&mut self, shape_id: u32) {
        self.shapes.remove(&shape_id);

        if self.transaction_mode {
            self.transaction_pending.push(TransactionOp::DeleteShape(shape_id));
        } else {
            self.rebuild_visibility_graph();
        }
    }

    /// Moves a shape to a new position
    pub fn move_shape(&mut self, shape_id: u32, new_position: Point) {
        if let Some(shape) = self.shapes.get_mut(&shape_id) {
            let old_pos = shape.position();
            let offset = new_position - old_pos;

            let mut new_poly = shape.polygon().clone();
            new_poly.translate(offset);
            shape.set_polygon(new_poly);

            if self.transaction_mode {
                self.transaction_pending.push(TransactionOp::MoveShape(shape_id, new_position));
            } else {
                self.rebuild_visibility_graph();
                self.reroute_all_connectors();
            }
        }
    }

    /// Adds a connector to the router
    pub fn add_connector(&mut self, connector: ConnRef) -> u32 {
        let conn_id = connector.id();

        if conn_id >= self.next_connector_id {
            self.next_connector_id = conn_id + 1;
        }

        self.connectors.insert(conn_id, connector);

        if self.transaction_mode {
            self.transaction_pending.push(TransactionOp::AddConnector(conn_id));
        } else {
            self.route_connector(conn_id);
        }

        conn_id
    }

    /// Creates and adds a new connector
    pub fn new_connector(&mut self, src: ConnEnd, dst: ConnEnd) -> u32 {
        let conn_id = self.next_connector_id;
        self.next_connector_id += 1;

        let connector = ConnRef::with_endpoints(conn_id, src, dst);
        self.add_connector(connector)
    }

    /// Deletes a connector from the router
    pub fn delete_connector(&mut self, conn_id: u32) {
        self.connectors.remove(&conn_id);

        if self.transaction_mode {
            self.transaction_pending.push(TransactionOp::DeleteConnector(conn_id));
        }
    }

    /// Gets a connector by ID
    pub fn get_connector(&self, conn_id: u32) -> Option<&ConnRef> {
        self.connectors.get(&conn_id)
    }

    /// Gets a mutable connector by ID
    pub fn get_connector_mut(&mut self, conn_id: u32) -> Option<&mut ConnRef> {
        self.connectors.get_mut(&conn_id)
    }

    /// Gets a shape by ID
    pub fn get_shape(&self, shape_id: u32) -> Option<&ShapeRef> {
        self.shapes.get(&shape_id)
    }

    /// Sets a routing parameter
    pub fn set_routing_parameter(&mut self, param: RoutingParameter, value: f64) {
        self.parameters.insert(param, value);

        // Update orthogonal router if bend penalty changed
        if param == RoutingParameter::BendPenalty {
            let segment_penalty = *self.parameters.get(&RoutingParameter::SegmentPenalty)
                .unwrap_or(&1.0);
            self.orthogonal_router = OrthogonalRouter::with_penalties(value, segment_penalty);
        }
    }

    /// Gets a routing parameter
    pub fn routing_parameter(&self, param: RoutingParameter) -> f64 {
        *self.parameters.get(&param).unwrap_or(&0.0)
    }

    /// Sets a routing option
    pub fn set_routing_option(&mut self, option: RoutingOption, value: bool) {
        self.options.insert(option, value);
    }

    /// Gets a routing option
    pub fn routing_option(&self, option: RoutingOption) -> bool {
        *self.options.get(&option).unwrap_or(&false)
    }

    /// Enables or disables transaction mode
    pub fn set_transaction_use(&mut self, enabled: bool) {
        self.transaction_mode = enabled;
    }

    /// Returns whether transaction mode is enabled
    pub fn transaction_use(&self) -> bool {
        self.transaction_mode
    }

    /// Processes all pending transaction operations
    pub fn process_transaction(&mut self) {
        if !self.transaction_mode {
            return;
        }

        // Process all pending operations
        let ops = std::mem::take(&mut self.transaction_pending);

        // Rebuild visibility graph once for all shape changes
        let has_shape_ops = ops.iter().any(|op| matches!(op,
            TransactionOp::AddShape(_) | TransactionOp::DeleteShape(_) | TransactionOp::MoveShape(_, _)
        ));

        if has_shape_ops {
            self.rebuild_visibility_graph();
        }

        // Reroute all connectors
        self.reroute_all_connectors();
    }

    /// Routes a single connector
    fn route_connector(&mut self, conn_id: u32) {
        let connector = match self.connectors.get(&conn_id) {
            Some(c) => c,
            None => return,
        };

        if connector.has_fixed_route() {
            return;
        }

        let (src, dst) = connector.endpoint_conn_ends();
        let src_point = src.position;
        let dst_point = dst.position;

        let routing_type = connector.routing_type();

        // Choose the appropriate graph and routing method
        let route = match routing_type {
            ConnType::PolyLine => self.route_polyline(src_point, dst_point),
            ConnType::Orthogonal => self.route_orthogonal(src_point, dst_point),
        };

        // Set the route
        if let Some(connector) = self.connectors.get_mut(&conn_id) {
            connector.set_route(route);
        }
    }

    /// Routes using polyline (direct path through visibility graph)
    fn route_polyline(&mut self, src: Point, dst: Point) -> Polygon {
        let obstacles: Vec<&dyn Obstacle> = self.shapes.values()
            .map(|s| s as &dyn Obstacle)
            .collect();

        // Check if direct path is clear first (optimization)
        if self.is_direct_path_clear(&src, &dst, &obstacles) {
            let mut route = Polygon::new();
            route.push(src);
            route.push(dst);
            return route;
        }

        // Add temporary vertices for source and destination
        let src_id = self.vis_graph.add_vertex(src);
        let dst_id = self.vis_graph.add_vertex(dst);

        // Compute visibility for the new vertices
        self.vis_graph.compute_vertex_visibility(src_id, &obstacles);
        self.vis_graph.compute_vertex_visibility(dst_id, &obstacles);

        // Find path using A*
        let path_result = self.path_finder.find_path(&self.vis_graph, src_id, dst_id);

        // Convert path to polygon (before removing temporary vertices)
        let route = if let Some(path) = path_result {
            // Reconstruct polygon from path vertex IDs
            let mut route = Polygon::new();
            route.push(src);

            // Add intermediate waypoints (skip first and last as they are src/dst)
            for i in 1..path.len().saturating_sub(1) {
                if let Some(vertex) = self.vis_graph.get_vertex(path[i]) {
                    route.push(vertex.point);
                }
            }

            route.push(dst);
            route.simplify();
            route
        } else {
            // No path found through visibility graph, use direct path as fallback
            let mut route = Polygon::new();
            route.push(src);
            route.push(dst);
            route
        };

        // Remove temporary vertices
        self.vis_graph.remove_vertex(src_id);
        self.vis_graph.remove_vertex(dst_id);

        route
    }

    /// Routes using orthogonal segments
    fn route_orthogonal(&mut self, src: Point, dst: Point) -> Polygon {
        let obstacles: Vec<&dyn Obstacle> = self.shapes.values()
            .map(|s| s as &dyn Obstacle)
            .collect();

        self.orthogonal_router.route_orthogonal(src, dst, &obstacles)
    }

    /// Checks if a direct path is clear of all obstacles.
    /// Uses proper polygon intersection test, not just bounding box.
    fn is_direct_path_clear(&self, from: &Point, to: &Point, obstacles: &[&dyn Obstacle]) -> bool {
        use crate::geometry::{segment_intersects_polygon_interior, point_in_polygon};

        for obstacle in obstacles {
            if !obstacle.is_active() {
                continue;
            }

            let polygon = obstacle.polygon();

            // First quick check: bounding box
            let bbox = polygon.bounding_rect();
            if !self.line_might_intersect_box(from, to, &bbox) {
                continue; // Can't possibly intersect
            }

            // Full polygon intersection test
            if segment_intersects_polygon_interior(from, to, polygon) {
                return false;
            }

            // Also check if either endpoint is inside the polygon
            if point_in_polygon(from, polygon) || point_in_polygon(to, polygon) {
                return false;
            }
        }

        true
    }

    /// Quick bounding box check to see if a line MIGHT intersect.
    /// Returns true if intersection is possible, false if definitely not.
    fn line_might_intersect_box(&self, from: &Point, to: &Point, bbox: &crate::geometry::Box) -> bool {
        // Expand bbox slightly for floating point tolerance
        const TOLERANCE: f64 = 1e-6;
        let min_x = bbox.min.x - TOLERANCE;
        let max_x = bbox.max.x + TOLERANCE;
        let min_y = bbox.min.y - TOLERANCE;
        let max_y = bbox.max.y + TOLERANCE;

        // Check if line segment's bounding box intersects obstacle's bounding box
        let line_min_x = from.x.min(to.x);
        let line_max_x = from.x.max(to.x);
        let line_min_y = from.y.min(to.y);
        let line_max_y = from.y.max(to.y);

        !(line_max_x < min_x || line_min_x > max_x || line_max_y < min_y || line_min_y > max_y)
    }

    /// Reroutes all connectors
    fn reroute_all_connectors(&mut self) {
        let conn_ids: Vec<u32> = self.connectors.keys().copied().collect();
        for conn_id in conn_ids {
            self.route_connector(conn_id);
        }
    }

    /// Rebuilds the visibility graph
    fn rebuild_visibility_graph(&mut self) {
        self.vis_graph.clear();
        self.vis_orth_graph.clear();

        // Add vertices for all shape corners
        for shape in self.shapes.values() {
            let poly = shape.polygon();
            for point in poly.points() {
                self.vis_graph.add_vertex(*point);
                self.vis_orth_graph.add_vertex(*point);
            }
        }

        // Compute visibility between vertices
        let obstacles: Vec<&dyn Obstacle> = self.shapes.values()
            .map(|s| s as &dyn Obstacle)
            .collect();

        let vertex_ids: Vec<u32> = self.vis_graph.vertices()
            .map(|v| v.id)
            .collect();

        for vertex_id in vertex_ids {
            self.vis_graph.compute_vertex_visibility(vertex_id, &obstacles);
            self.vis_orth_graph.compute_vertex_visibility(vertex_id, &obstacles);
        }
    }

    /// Returns all connectors
    pub fn connectors(&self) -> impl Iterator<Item = &ConnRef> {
        self.connectors.values()
    }

    /// Returns all shapes
    pub fn shapes(&self) -> impl Iterator<Item = &ShapeRef> {
        self.shapes.values()
    }

    /// Outputs the current router state to SVG for debugging
    pub fn output_instance_to_svg(&self, filename: &str) -> std::io::Result<()> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(filename)?;

        // Calculate bounding box
        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for shape in self.shapes.values() {
            let bbox = shape.polygon().bounding_rect();
            min_x = min_x.min(bbox.min.x);
            min_y = min_y.min(bbox.min.y);
            max_x = max_x.max(bbox.max.x);
            max_y = max_y.max(bbox.max.y);
        }

        let width = max_x - min_x + 100.0;
        let height = max_y - min_y + 100.0;

        writeln!(file, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
        writeln!(file, r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#, width, height)?;

        // Draw shapes
        for shape in self.shapes.values() {
            let poly = shape.polygon();
            if poly.size() > 0 {
                write!(file, r#"  <polygon points=""#)?;
                for point in poly.points() {
                    write!(file, "{},{} ", point.x - min_x + 50.0, point.y - min_y + 50.0)?;
                }
                writeln!(file, r#"" fill="lightblue" stroke="black" stroke-width="1"/>"#)?;
            }
        }

        // Draw connectors
        for conn in self.connectors.values() {
            if let Some(route) = conn.display_route() {
                if route.size() > 1 {
                    write!(file, r#"  <polyline points=""#)?;
                    for point in route.points() {
                        write!(file, "{},{} ", point.x - min_x + 50.0, point.y - min_y + 50.0)?;
                    }
                    writeln!(file, r#"" fill="none" stroke="red" stroke-width="2"/>"#)?;
                }
            }
        }

        writeln!(file, "</svg>")?;

        Ok(())
    }
}

impl Default for Router {
    fn default() -> Self {
        Router::new(ROUTER_FLAG_NONE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let router = Router::new(ROUTER_FLAG_NONE);
        assert!(!router.transaction_use());
    }

    #[test]
    fn test_add_shape() {
        let mut router = Router::new(ROUTER_FLAG_NONE);

        let mut poly = Polygon::new();
        poly.push(Point::new(0.0, 0.0));
        poly.push(Point::new(10.0, 0.0));
        poly.push(Point::new(10.0, 10.0));
        poly.push(Point::new(0.0, 10.0));

        let shape_id = router.add_shape(poly, 0);
        assert!(router.get_shape(shape_id).is_some());
    }

    #[test]
    fn test_add_connector() {
        let mut router = Router::new(ROUTER_FLAG_NONE);

        let src = ConnEnd::new(Point::new(0.0, 0.0));
        let dst = ConnEnd::new(Point::new(100.0, 100.0));

        let conn_id = router.new_connector(src, dst);
        assert!(router.get_connector(conn_id).is_some());
    }

    #[test]
    fn test_routing_parameters() {
        let mut router = Router::new(ROUTER_FLAG_NONE);

        router.set_routing_parameter(RoutingParameter::BendPenalty, 100.0);
        assert_eq!(router.routing_parameter(RoutingParameter::BendPenalty), 100.0);
    }
}
