//! Core routing functionality
//!
//! This module provides the Router, which is the main entry point for using libavoid.
//! The Router manages shapes, connectors, and performs the routing calculations.

use crate::geometry::{Polygon, Point, PolygonInterface};
use crate::connector::{ConnRef, ConnEnd, ConnType};
use crate::shape::ShapeRef;
use crate::junction::JunctionRef;
use crate::obstacle::Obstacle;
use crate::visibility::VisibilityGraph;
use crate::graph::PathFinder;
use crate::orthogonal::OrthogonalRouter;
use crate::channel::ChannelRouter;
use crate::action::{ActionInfo, ActionType};
use crate::orthogonal_visgraph::{OrthogonalVisGraphGenerator, ObstacleInput, ConnectorInput};
use std::collections::{HashMap, HashSet, VecDeque};

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
    /// All junctions in the scene
    junctions: HashMap<u32, JunctionRef>,
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
    /// Channel router for nudging overlapping segments
    channel_router: ChannelRouter,
    /// Routing parameters
    parameters: HashMap<RoutingParameter, f64>,
    /// Routing options
    options: HashMap<RoutingOption, bool>,
    /// Transaction mode enabled
    transaction_mode: bool,
    /// Pending operations in transaction (legacy)
    transaction_pending: Vec<TransactionOp>,
    /// Action queue for transaction processing
    action_queue: VecDeque<ActionInfo>,
    /// Connectors that need rerouting
    reroute_queue: HashSet<u32>,
    /// Whether visibility graph needs rebuilding
    needs_vis_rebuild: bool,
    /// Shapes that need their visibility updated (incremental updates)
    dirty_shapes: HashSet<u32>,
    /// Whether to use incremental visibility updates (vs full rebuild)
    use_incremental_updates: bool,
    /// Next shape ID
    next_shape_id: u32,
    /// Next connector ID
    next_connector_id: u32,
    /// Next junction ID
    next_junction_id: u32,
}

/// Transaction operation types (legacy, kept for backwards compatibility)
#[derive(Debug, Clone)]
enum TransactionOp {
    AddShape(u32),
    DeleteShape(u32),
    MoveShape(u32, Point),
    AddConnector(u32),
    DeleteConnector(u32),
    AddJunction(u32),
    DeleteJunction(u32),
    MoveJunction(u32, Point),
}

impl Router {
    /// Creates a new router with the given flags
    pub fn new(flags: RouterFlags) -> Self {
        let mut router = Router {
            flags,
            shapes: HashMap::new(),
            junctions: HashMap::new(),
            connectors: HashMap::new(),
            vis_graph: VisibilityGraph::new(),
            vis_orth_graph: VisibilityGraph::new(),
            path_finder: PathFinder::new(),
            orthogonal_router: OrthogonalRouter::new(),
            channel_router: ChannelRouter::new(),
            parameters: HashMap::new(),
            options: HashMap::new(),
            transaction_mode: (flags & ROUTER_FLAG_USE_TRANSACTIONS) != 0,
            transaction_pending: Vec::new(),
            action_queue: VecDeque::new(),
            reroute_queue: HashSet::new(),
            needs_vis_rebuild: false,
            dirty_shapes: HashSet::new(),
            use_incremental_updates: true, // Enable incremental by default
            next_shape_id: 1,
            next_connector_id: 1,
            next_junction_id: 1,
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
            self.mark_shape_dirty(shape_id);
        } else {
            // For new shapes, we need full rebuild since there are new vertices
            self.rebuild_visibility_graph_full();
        }

        shape_id
    }

    /// Deletes a shape from the router
    pub fn delete_shape(&mut self, shape_id: u32) {
        // Mark dirty before removal so we know which vertices to clean up
        self.mark_shape_dirty(shape_id);

        self.shapes.remove(&shape_id);

        if self.transaction_mode {
            self.transaction_pending.push(TransactionOp::DeleteShape(shape_id));
        } else {
            // For removed shapes, need full rebuild to remove stale vertices
            self.rebuild_visibility_graph_full();
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

            // Mark shape as dirty for incremental update
            self.mark_shape_dirty(shape_id);

            if self.transaction_mode {
                self.transaction_pending.push(TransactionOp::MoveShape(shape_id, new_position));
            } else {
                // Use incremental update for shape movement
                self.update_visibility_graph();
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

    // ========================================================================
    // Junction Management
    // ========================================================================

    /// Adds a junction at the given position
    pub fn add_junction(&mut self, position: Point, id: u32) -> u32 {
        let junction_id = if id == 0 {
            let id = self.next_junction_id;
            self.next_junction_id += 1;
            id
        } else {
            if id >= self.next_junction_id {
                self.next_junction_id = id + 1;
            }
            id
        };

        let junction = JunctionRef::new(junction_id, position);
        self.junctions.insert(junction_id, junction);

        if self.transaction_mode {
            self.transaction_pending.push(TransactionOp::AddJunction(junction_id));
            self.action_queue.push_back(ActionInfo::junction_add(junction_id));
            self.needs_vis_rebuild = true;
        } else {
            self.rebuild_visibility_graph();
        }

        junction_id
    }

    /// Deletes a junction from the router
    pub fn delete_junction(&mut self, junction_id: u32) {
        // Detach all connectors from this junction
        if let Some(junction) = self.junctions.get(&junction_id) {
            let attached: Vec<u32> = junction.attached_connectors().iter().copied().collect();
            for conn_id in attached {
                self.reroute_queue.insert(conn_id);
            }
        }

        self.junctions.remove(&junction_id);

        if self.transaction_mode {
            self.transaction_pending.push(TransactionOp::DeleteJunction(junction_id));
            self.action_queue.push_back(ActionInfo::junction_remove(junction_id));
            self.needs_vis_rebuild = true;
        } else {
            self.rebuild_visibility_graph();
        }
    }

    /// Moves a junction to a new position
    pub fn move_junction(&mut self, junction_id: u32, new_position: Point) {
        if let Some(junction) = self.junctions.get_mut(&junction_id) {
            junction.set_position(new_position);

            // Mark attached connectors for reroute
            for conn_id in junction.attached_connectors() {
                self.reroute_queue.insert(*conn_id);
            }

            if self.transaction_mode {
                self.transaction_pending.push(TransactionOp::MoveJunction(junction_id, new_position));
                self.action_queue.push_back(ActionInfo::junction_move(junction_id, new_position));
            } else {
                self.rebuild_visibility_graph();
                self.reroute_all_connectors();
            }
        }
    }

    /// Gets a junction by ID
    pub fn get_junction(&self, junction_id: u32) -> Option<&JunctionRef> {
        self.junctions.get(&junction_id)
    }

    /// Gets a mutable junction by ID
    pub fn get_junction_mut(&mut self, junction_id: u32) -> Option<&mut JunctionRef> {
        self.junctions.get_mut(&junction_id)
    }

    /// Returns all junctions
    pub fn junctions(&self) -> impl Iterator<Item = &JunctionRef> {
        self.junctions.values()
    }

    // ========================================================================
    // Connector Reroute Queue
    // ========================================================================

    /// Marks a connector as needing rerouting
    pub fn mark_connector_for_reroute(&mut self, conn_id: u32) {
        self.reroute_queue.insert(conn_id);
    }

    /// Processes the reroute queue
    fn process_reroute_queue(&mut self) {
        let queue: Vec<u32> = self.reroute_queue.drain().collect();
        for conn_id in queue {
            self.route_connector(conn_id);
        }
    }

    // ========================================================================
    // Action Queue Processing
    // ========================================================================

    /// Adds an action to the queue
    pub fn queue_action(&mut self, action: ActionInfo) {
        let action_type = action.action_type;
        let connector_id = action.connector_id;
        self.action_queue.push_back(action);

        match action_type {
            ActionType::ShapeAdd | ActionType::ShapeRemove | ActionType::ShapeMove |
            ActionType::JunctionAdd | ActionType::JunctionRemove | ActionType::JunctionMove => {
                self.needs_vis_rebuild = true;
            }
            ActionType::ConnectorChange => {
                if let Some(conn_id) = connector_id {
                    self.reroute_queue.insert(conn_id);
                }
            }
            _ => {}
        }
    }

    /// Processes the action queue
    fn process_action_queue(&mut self) {
        let actions: Vec<ActionInfo> = self.action_queue.drain(..).collect();

        for action in actions {
            match action.action_type {
                ActionType::ConnectorAdd => {
                    if let Some(conn_id) = action.connector_id {
                        self.reroute_queue.insert(conn_id);
                    }
                }
                ActionType::ConnectorChange => {
                    if let Some(conn_id) = action.connector_id {
                        self.reroute_queue.insert(conn_id);
                    }
                }
                _ => {}
            }
        }
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

        // Rebuild visibility graph once for all shape/junction changes
        let has_obstacle_ops = ops.iter().any(|op| matches!(op,
            TransactionOp::AddShape(_) | TransactionOp::DeleteShape(_) | TransactionOp::MoveShape(_, _) |
            TransactionOp::AddJunction(_) | TransactionOp::DeleteJunction(_) | TransactionOp::MoveJunction(_, _)
        ));

        // Process action queue
        self.process_action_queue();

        // Rebuild visibility graph if needed
        if has_obstacle_ops || self.needs_vis_rebuild {
            self.rebuild_visibility_graph();
            self.needs_vis_rebuild = false;
            // After rebuilding, all connectors need rerouting
            for conn_id in self.connectors.keys() {
                self.reroute_queue.insert(*conn_id);
            }
        }

        // Process the reroute queue
        if !self.reroute_queue.is_empty() {
            self.process_reroute_queue();
        } else {
            // Fallback: reroute all connectors if nothing in queue
            self.reroute_all_connectors();
        }
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
            #[cfg(test)]
            eprintln!("route_polyline: direct path is clear, using direct route");
            let mut route = Polygon::new();
            route.push(src);
            route.push(dst);
            return route;
        }

        #[cfg(test)]
        eprintln!("route_polyline: direct path blocked, using visibility graph");
        #[cfg(test)]
        eprintln!("  vis_graph has {} vertices before adding src/dst", self.vis_graph.vertex_count());

        // Add temporary vertices for source and destination
        let src_id = self.vis_graph.add_vertex(src);
        let dst_id = self.vis_graph.add_vertex(dst);

        #[cfg(test)]
        eprintln!("  added src vertex {} at ({}, {})", src_id, src.x, src.y);
        #[cfg(test)]
        eprintln!("  added dst vertex {} at ({}, {})", dst_id, dst.x, dst.y);

        // Compute visibility for the new vertices
        self.vis_graph.compute_vertex_visibility(src_id, &obstacles);
        self.vis_graph.compute_vertex_visibility(dst_id, &obstacles);

        #[cfg(test)]
        eprintln!("  vis_graph has {} vertices after visibility computation", self.vis_graph.vertex_count());

        // Find path using A*
        let path_result = self.path_finder.find_path(&self.vis_graph, src_id, dst_id);

        #[cfg(test)]
        eprintln!("  path_finder result: {:?}", path_result);

        // Convert path to polygon (before removing temporary vertices)
        let route = if let Some(path) = path_result {
            #[cfg(test)]
            eprintln!("  found path with {} vertices", path.len());
            // Reconstruct polygon from path vertex IDs
            let mut route = Polygon::new();
            route.push(src);

            // Add intermediate waypoints (skip first and last as they are src/dst)
            for i in 1..path.len().saturating_sub(1) {
                if let Some(vertex) = self.vis_graph.get_vertex(path[i]) {
                    #[cfg(test)]
                    eprintln!("    adding waypoint ({}, {})", vertex.point.x, vertex.point.y);
                    route.push(vertex.point);
                }
            }

            route.push(dst);
            route.simplify();
            route
        } else {
            // No path found through visibility graph.
            // This can legitimately happen if src/dst are completely blocked.
            // Return a direct path but mark it needs attention.
            // The connector's needs_attention flag should be set by the caller.
            #[cfg(test)]
            eprintln!("  WARNING: No path found through visibility graph, returning direct path");
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

    /// Routes using orthogonal segments via visibility graph
    fn route_orthogonal(&mut self, src: Point, dst: Point) -> Polygon {
        // Build obstacle inputs
        let obstacles: Vec<ObstacleInput> = self.shapes.iter()
            .filter(|(_, shape)| shape.is_active())
            .map(|(id, shape)| ObstacleInput {
                id: *id,
                polygon: shape.polygon().clone(),
            })
            .collect();

        // If no obstacles, return direct L-shaped path
        if obstacles.is_empty() {
            return self.simple_orthogonal_path(src, dst);
        }

        // Create connector input for this route
        let connectors = vec![ConnectorInput {
            id: 0,
            start: src,
            end: dst,
        }];

        // Generate orthogonal visibility graph
        let generator = OrthogonalVisGraphGenerator::new();
        let ortho_graph = generator.generate(&obstacles, &connectors);

        // Find start and end vertices in the graph
        let start_vertex = ortho_graph.vertices()
            .find(|v| (v.point.x - src.x).abs() < 1e-6 && (v.point.y - src.y).abs() < 1e-6);
        let end_vertex = ortho_graph.vertices()
            .find(|v| (v.point.x - dst.x).abs() < 1e-6 && (v.point.y - dst.y).abs() < 1e-6);

        match (start_vertex, end_vertex) {
            (Some(start_v), Some(end_v)) => {
                // Use pathfinder to find route through visibility graph
                if let Some(path_ids) = self.path_finder.find_path(&ortho_graph, start_v.id, end_v.id) {
                    // Convert path IDs to points
                    let mut route = Polygon::with_capacity(path_ids.len());
                    for id in path_ids {
                        if let Some(vertex) = ortho_graph.get_vertex(id) {
                            route.push(vertex.point);
                        }
                    }
                    if route.size() >= 2 {
                        return route;
                    }
                }
            }
            _ => {}
        }

        // Fallback to old orthogonal router if visgraph approach fails
        let obs_refs: Vec<&dyn Obstacle> = self.shapes.values()
            .map(|s| s as &dyn Obstacle)
            .collect();
        self.orthogonal_router.route_orthogonal(src, dst, &obs_refs)
    }

    /// Simple L-shaped orthogonal path (no obstacles)
    fn simple_orthogonal_path(&self, src: Point, dst: Point) -> Polygon {
        let mut route = Polygon::with_capacity(3);
        route.push(src);
        // Horizontal first, then vertical
        if (src.x - dst.x).abs() > 1e-6 {
            route.push(Point::new(dst.x, src.y));
        }
        route.push(dst);
        route
    }

    /// Checks if a direct path is clear of all obstacles.
    /// Uses proper polygon intersection test, not just bounding box.
    fn is_direct_path_clear(&self, from: &Point, to: &Point, obstacles: &[&dyn Obstacle]) -> bool {
        use crate::geometry::{segment_intersects_polygon_interior, point_in_polygon};

        #[cfg(test)]
        eprintln!("is_direct_path_clear: checking {} obstacles", obstacles.len());

        for obstacle in obstacles {
            if !obstacle.is_active() {
                #[cfg(test)]
                eprintln!("  obstacle inactive, skipping");
                continue;
            }

            let polygon = obstacle.polygon();

            #[cfg(test)]
            eprintln!("  checking obstacle with {} vertices", polygon.size());

            // First quick check: bounding box
            let bbox = polygon.bounding_rect();
            if !self.line_might_intersect_box(from, to, &bbox) {
                #[cfg(test)]
                eprintln!("  bbox check: no intersection possible");
                continue; // Can't possibly intersect
            }

            #[cfg(test)]
            eprintln!("  bbox check passed, doing full intersection test");

            // Full polygon intersection test
            if segment_intersects_polygon_interior(from, to, polygon) {
                #[cfg(test)]
                eprintln!("  INTERSECTION DETECTED - path blocked!");
                return false;
            }

            // Also check if either endpoint is inside the polygon
            if point_in_polygon(from, polygon) || point_in_polygon(to, polygon) {
                #[cfg(test)]
                eprintln!("  ENDPOINT INSIDE - path blocked!");
                return false;
            }

            #[cfg(test)]
            eprintln!("  no intersection with this obstacle");
        }

        #[cfg(test)]
        eprintln!("  path is CLEAR");
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

        // Apply nudging to orthogonal routes if enabled
        if self.options.get(&RoutingOption::NudgeOrthogonalRoutes).copied().unwrap_or(false) {
            self.nudge_orthogonal_routes();
        }
    }

    /// Nudges orthogonal routes to prevent overlap
    fn nudge_orthogonal_routes(&mut self) {
        // Collect orthogonal connectors with valid routes
        let orthogonal_conn_ids: Vec<u32> = self.connectors
            .iter()
            .filter(|(_, conn)| conn.routing_type() == ConnType::Orthogonal && conn.route().is_some())
            .map(|(id, _)| *id)
            .collect();

        if orthogonal_conn_ids.is_empty() {
            return;
        }

        // Extract routes as polygons
        let mut routes: Vec<Polygon> = orthogonal_conn_ids
            .iter()
            .filter_map(|id| self.connectors.get(id))
            .filter_map(|conn| conn.route().cloned())
            .collect();

        if routes.is_empty() {
            return;
        }

        // Collect obstacle polygons
        let obstacles: Vec<Polygon> = self.shapes
            .values()
            .map(|shape| shape.polygon().clone())
            .collect();

        // Apply channel-based nudging with obstacle awareness
        self.channel_router.nudge_routes_with_obstacles(&mut routes, &obstacles);

        // Update connector routes with nudged positions
        for (i, conn_id) in orthogonal_conn_ids.iter().enumerate() {
            if let Some(conn) = self.connectors.get_mut(conn_id) {
                conn.set_route(routes[i].clone());
            }
        }
    }

    /// Reroutes a specific connector (public API)
    ///
    /// This rebuilds the visibility graph if needed and recalculates
    /// the route for the specified connector.
    pub fn reroute_connector(&mut self, conn_id: u32) {
        // Rebuild visibility graph if needed
        if self.needs_vis_rebuild {
            self.rebuild_visibility_graph();
            self.needs_vis_rebuild = false;
        }
        self.route_connector(conn_id);
    }

    /// Updates the visibility graph, using incremental updates if possible
    fn update_visibility_graph(&mut self) {
        if !self.use_incremental_updates || self.dirty_shapes.len() > self.shapes.len() / 2 {
            // Fall back to full rebuild if too many dirty shapes
            self.rebuild_visibility_graph_full();
        } else if !self.dirty_shapes.is_empty() {
            self.update_visibility_incremental();
        }
        self.dirty_shapes.clear();
    }

    /// Rebuilds the visibility graph from scratch
    fn rebuild_visibility_graph_full(&mut self) {
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

    /// Performs incremental visibility update for dirty shapes only
    ///
    /// This is more efficient than a full rebuild when only a few shapes have changed.
    /// The algorithm:
    /// 1. Remove edges from vertices of dirty shapes
    /// 2. Update vertices for dirty shapes (add new, remove old)
    /// 3. Recompute visibility for affected vertices
    /// 4. Also recompute visibility for vertices that might now see dirty shapes
    fn update_visibility_incremental(&mut self) {
        let dirty_shapes: Vec<u32> = self.dirty_shapes.iter().copied().collect();

        // Collect all shape bounding boxes for proximity check
        let all_bboxes: HashMap<u32, crate::geometry::Box> = self.shapes.iter()
            .map(|(id, s)| (*id, s.polygon().bounding_rect()))
            .collect();

        // Find vertices that need their visibility recomputed
        // This includes vertices of dirty shapes AND vertices whose visibility
        // might be affected by the dirty shapes
        let mut affected_vertex_ids: HashSet<u32> = HashSet::new();

        // First pass: collect vertices from dirty shapes and find their IDs
        for shape_id in &dirty_shapes {
            if let Some(shape) = self.shapes.get(shape_id) {
                for point in shape.polygon().points() {
                    // Find vertex ID at this point
                    if let Some(vertex_id) = self.vis_graph.find_vertex_at(point) {
                        self.vis_graph.remove_edges_for_vertex(vertex_id);
                        affected_vertex_ids.insert(vertex_id);
                    }
                    if let Some(vertex_id) = self.vis_orth_graph.find_vertex_at(point) {
                        self.vis_orth_graph.remove_edges_for_vertex(vertex_id);
                    }
                }
            }
        }

        // Second pass: find other vertices that might be affected
        // A vertex is affected if its visibility to any dirty vertex might have changed
        // This is approximated by checking if the vertex's bounding box overlaps with
        // the dirty shapes' bounding boxes (with some margin)
        let visibility_margin = 1000.0; // Large margin to be safe

        for dirty_id in &dirty_shapes {
            if let Some(dirty_bbox) = all_bboxes.get(dirty_id) {
                // Expand dirty bbox by visibility margin
                let expanded_min = Point::new(
                    dirty_bbox.min.x - visibility_margin,
                    dirty_bbox.min.y - visibility_margin,
                );
                let expanded_max = Point::new(
                    dirty_bbox.max.x + visibility_margin,
                    dirty_bbox.max.y + visibility_margin,
                );

                // Find all vertices within this expanded region
                let vertex_ids: Vec<u32> = self.vis_graph.vertices()
                    .filter(|v| {
                        v.point.x >= expanded_min.x && v.point.x <= expanded_max.x
                            && v.point.y >= expanded_min.y && v.point.y <= expanded_max.y
                    })
                    .map(|v| v.id)
                    .collect();

                for vertex_id in vertex_ids {
                    if !affected_vertex_ids.contains(&vertex_id) {
                        self.vis_graph.remove_edges_for_vertex(vertex_id);
                        affected_vertex_ids.insert(vertex_id);
                    }
                }
            }
        }

        // Recompute visibility for affected vertices
        let obstacles: Vec<&dyn Obstacle> = self.shapes.values()
            .map(|s| s as &dyn Obstacle)
            .collect();

        for vertex_id in affected_vertex_ids {
            self.vis_graph.compute_vertex_visibility(vertex_id, &obstacles);
            self.vis_orth_graph.compute_vertex_visibility(vertex_id, &obstacles);
        }
    }

    /// Enables or disables incremental visibility updates
    pub fn set_use_incremental_updates(&mut self, enable: bool) {
        self.use_incremental_updates = enable;
    }

    /// Returns whether incremental visibility updates are enabled
    pub fn use_incremental_updates(&self) -> bool {
        self.use_incremental_updates
    }

    /// Marks a shape as dirty (needing visibility update)
    fn mark_shape_dirty(&mut self, shape_id: u32) {
        self.dirty_shapes.insert(shape_id);
        self.needs_vis_rebuild = true;
    }

    /// Legacy method - calls update_visibility_graph
    fn rebuild_visibility_graph(&mut self) {
        self.rebuild_visibility_graph_full();
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

    // =========================================================================
    // Connection Pin Management
    // =========================================================================

    /// Adds a connection pin to a shape
    ///
    /// # Arguments
    /// * `shape_id` - The ID of the shape to add the pin to
    /// * `class_id` - The class ID for grouping pins
    /// * `position` - The position of the pin relative to the shape
    /// * `directions` - Allowed connection directions (bitfield)
    pub fn add_connection_pin_to_shape(
        &mut self,
        shape_id: u32,
        class_id: u32,
        position: Point,
        directions: u32,
    ) {
        if let Some(shape) = self.shapes.get_mut(&shape_id) {
            let pin = crate::shape::ConnectionPin::with_all(
                class_id,  // id
                class_id,  // class_id
                position,
                directions,
                0.0,  // inside_offset
            );
            shape.add_connection_pin(pin);

            // Mark connectors attached to this shape for rerouting
            for conn in self.connectors.values() {
                let (src, dst) = conn.endpoint_conn_ends();
                if src.shape_id == Some(shape_id) || dst.shape_id == Some(shape_id) {
                    self.reroute_queue.insert(conn.id());
                }
            }
        }
    }

    /// Updates the position of a connection pin on a shape
    ///
    /// # Arguments
    /// * `shape_id` - The ID of the shape containing the pin
    /// * `pin_id` - The ID of the pin to update
    /// * `new_position` - The new position for the pin
    pub fn update_connection_pin_position(
        &mut self,
        shape_id: u32,
        pin_id: u32,
        new_position: Point,
    ) {
        if let Some(shape) = self.shapes.get_mut(&shape_id) {
            // Find and update the pin
            for pin in shape.connection_pins_mut() {
                if pin.id == pin_id || pin.class_id == pin_id {
                    pin.update_position(new_position);
                    break;
                }
            }

            // Mark connectors attached to this shape for rerouting
            for conn in self.connectors.values() {
                let (src, dst) = conn.endpoint_conn_ends();
                if src.shape_id == Some(shape_id) || dst.shape_id == Some(shape_id) {
                    self.reroute_queue.insert(conn.id());
                }
            }

            self.needs_vis_rebuild = true;
        }
    }

    // =========================================================================
    // Debug and Info Methods
    // =========================================================================

    /// Returns a string with information about the router state
    ///
    /// This is useful for debugging and understanding the current state
    /// of the router.
    pub fn print_info(&self) -> String {
        let mut info = String::new();

        info.push_str(&format!("Router Info:\n"));
        info.push_str(&format!("  Shapes: {}\n", self.shapes.len()));
        info.push_str(&format!("  Connectors: {}\n", self.connectors.len()));
        info.push_str(&format!("  Junctions: {}\n", self.junctions.len()));
        info.push_str(&format!("  Visibility vertices: {}\n", self.vis_graph.vertex_count()));
        info.push_str(&format!("  Transaction mode: {}\n", self.transaction_mode));
        info.push_str(&format!("  Pending reroutes: {}\n", self.reroute_queue.len()));

        info
    }

    /// Returns debug state information about the router
    ///
    /// This provides structured access to router statistics for testing
    /// and debugging purposes.
    pub fn debug_state(&self) -> RouterDebugState {
        RouterDebugState {
            shape_count: self.shapes.len(),
            connector_count: self.connectors.len(),
            junction_count: self.junctions.len(),
            vertex_count: self.vis_graph.vertex_count(),
            transaction_mode: self.transaction_mode,
            pending_reroutes: self.reroute_queue.len(),
        }
    }
}

/// Debug state information from the router
#[derive(Debug, Clone)]
pub struct RouterDebugState {
    /// Number of shapes in the router
    pub shape_count: usize,
    /// Number of connectors in the router
    pub connector_count: usize,
    /// Number of junctions in the router
    pub junction_count: usize,
    /// Number of vertices in the visibility graph
    pub vertex_count: usize,
    /// Whether transaction mode is enabled
    pub transaction_mode: bool,
    /// Number of connectors pending reroute
    pub pending_reroutes: usize,
}

impl Default for Router {
    fn default() -> Self {
        Router::new(ROUTER_FLAG_NONE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Rectangle;

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

    /// This test verifies the obstacle avoidance routing with debug output.
    /// The route from (50, 125) to (350, 125) should NOT go through an
    /// obstacle centered at (200, 125) with size 50x50.
    #[test]
    fn test_obstacle_avoidance_debug() {
        let mut router = Router::new(ROUTER_FLAG_NONE);

        // Create obstacle at center (200, 125) with size 50x50
        // This creates bounds: x: 175-225, y: 100-150
        let rect = Rectangle::new(Point::new(200.0, 125.0), 50.0, 50.0);
        let poly: Polygon = rect.into();

        eprintln!("Obstacle polygon vertices:");
        for i in 0..poly.size() {
            let p = poly.at(i);
            eprintln!("  ({}, {})", p.x, p.y);
        }

        let shape_id = router.add_shape(poly, 1);
        eprintln!("Shape added with id: {}", shape_id);
        eprintln!("Number of shapes: {}", router.shapes.len());

        // Route that should go THROUGH the obstacle if bug exists
        let src = Point::new(50.0, 125.0);
        let dst = Point::new(350.0, 125.0);

        // Test is_direct_path_clear directly
        let obstacles: Vec<&dyn Obstacle> = router.shapes.values()
            .map(|s| s as &dyn Obstacle)
            .collect();

        eprintln!("\nTesting is_direct_path_clear:");
        eprintln!("  from: ({}, {})", src.x, src.y);
        eprintln!("  to: ({}, {})", dst.x, dst.y);
        eprintln!("  obstacles count: {}", obstacles.len());

        let is_clear = router.is_direct_path_clear(&src, &dst, &obstacles);
        eprintln!("  is_direct_path_clear returned: {}", is_clear);

        // The path SHOULD NOT be clear - it passes through the obstacle
        assert!(!is_clear, "Path from (50,125) to (350,125) should NOT be clear - obstacle at x:175-225, y:100-150");

        // Now test actual routing
        let conn_id = router.new_connector(
            ConnEnd::new(src),
            ConnEnd::new(dst)
        );

        let conn = router.get_connector(conn_id).unwrap();
        let route = conn.display_route().expect("Route should exist");

        eprintln!("\nRoute points:");
        for i in 0..route.size() {
            let p = route.at(i);
            eprintln!("  ({}, {})", p.x, p.y);
        }

        assert!(route.size() > 2,
            "Route should avoid obstacle and have more than 2 points, got {}",
            route.size());
    }
}
