//! Orthogonal visibility graph generation via sweep-line algorithm.
//!
//! This module implements the C++ libavoid algorithm from orthogonal.cpp
//! for generating visibility graphs suitable for orthogonal (rectilinear) routing.
//!
//! The algorithm uses two perpendicular sweep-line passes:
//! 1. Vertical sweep (sorted by Y, scanline in X) for horizontal visibility segments
//! 2. Horizontal sweep (sorted by X, scanline in Y) for vertical visibility segments
//!
//! Reference: libavoid/orthogonal.cpp - generateStaticOrthogonalVisGraph()
//! Reference: libavoid/scanline.cpp, scanline.h

use crate::connector::{ConnDirFlags, CONN_DIR_DOWN, CONN_DIR_LEFT, CONN_DIR_RIGHT, CONN_DIR_UP};
use crate::geometry::{Point, Polygon, PolygonInterface};
use crate::visibility::VisibilityGraph;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};

// ============================================================================
// Constants
// ============================================================================

/// Dimension index for X axis
const XDIM: usize = 0;
/// Dimension index for Y axis
const YDIM: usize = 1;

/// Very large positive value for "infinity"
const DBL_MAX: f64 = f64::MAX / 2.0;
/// Very large negative value for "negative infinity"
const NEG_DBL_MAX: f64 = f64::MIN / 2.0;

// ============================================================================
// Phase 1: Data Structures (Tasks #1-#8)
// ============================================================================

// ----------------------------------------------------------------------------
// Task #6: EventType enum
// C++ ref: scanline.h:107-114
// Note: Order matters for sorting! Open < SegOpen < ConnPoint < SegClose < Close
// ----------------------------------------------------------------------------

/// Event types for sweep-line algorithm.
/// C++ ref: scanline.h:107-114
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum EventType {
    /// Shape edge opens (entering shape boundary) - value 1
    Open = 1,
    /// Segment opens (for nudging) - value 2
    SegOpen = 2,
    /// Connector endpoint - value 3
    ConnPoint = 3,
    /// Segment closes - value 4
    SegClose = 4,
    /// Shape edge closes (leaving shape boundary) - value 5
    Close = 5,
}

// ----------------------------------------------------------------------------
// Task #1: ScanlineNode struct
// C++ ref: scanline.h:78-104, scanline.cpp:53-91
// ----------------------------------------------------------------------------

/// Unique identifier for a scanline node
pub type NodeIdx = usize;

/// Node in the scanline representing an active shape boundary or connector point.
/// C++ ref: class Node in scanline.h:78-104
#[derive(Clone, Debug)]
pub struct ScanlineNode {
    /// Obstacle ID (for shape nodes)
    pub obstacle_id: Option<u32>,
    /// Connector vertex ID (for connector point nodes)
    pub conn_vertex_id: Option<u32>,
    /// Visibility directions for connector points
    pub vis_directions: ConnDirFlags,
    /// Position along scanline axis (X for vertical sweep, Y for horizontal sweep)
    pub pos: f64,
    /// Min coordinate in each dimension [XDIM, YDIM]
    pub min: [f64; 2],
    /// Max coordinate in each dimension [XDIM, YDIM]
    pub max: [f64; 2],
    /// Index of first node above in scanline (lower position)
    pub first_above: Option<NodeIdx>,
    /// Index of first node below in scanline (higher position)
    pub first_below: Option<NodeIdx>,
}

impl ScanlineNode {
    /// Create a node for an obstacle.
    /// C++ ref: Node::Node(Obstacle *v, const double p) - scanline.cpp:53-67
    pub fn for_obstacle(
        obstacle_id: u32,
        pos: f64,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    ) -> Self {
        Self {
            obstacle_id: Some(obstacle_id),
            conn_vertex_id: None,
            vis_directions: 0,
            pos,
            min: [min_x, min_y],
            max: [max_x, max_y],
            first_above: None,
            first_below: None,
        }
    }

    /// Create a node for a connector point.
    /// C++ ref: Node::Node(VertInf *c, const double p) - scanline.cpp:69-79
    pub fn for_conn_point(vertex_id: u32, pos: f64, point_x: f64, point_y: f64, vis_dirs: ConnDirFlags) -> Self {
        Self {
            obstacle_id: None,
            conn_vertex_id: Some(vertex_id),
            vis_directions: vis_dirs,
            pos,
            min: [point_x, point_y],
            max: [point_x, point_y],
            first_above: None,
            first_below: None,
        }
    }

    /// Check if this is a connector point node
    pub fn is_conn_point(&self) -> bool {
        self.conn_vertex_id.is_some()
    }

    /// Check if this is an obstacle node
    pub fn is_obstacle(&self) -> bool {
        self.obstacle_id.is_some()
    }
}

// ----------------------------------------------------------------------------
// Task #7: Event struct
// C++ ref: scanline.h:117-124, scanline.cpp:285-290
// ----------------------------------------------------------------------------

/// Sweep-line event.
/// C++ ref: struct Event in scanline.h:117-124
#[derive(Clone, Debug)]
pub struct Event {
    /// Event type
    pub event_type: EventType,
    /// Index into nodes array
    pub node_idx: NodeIdx,
    /// Position along sweep axis (Y for vertical sweep, X for horizontal sweep)
    pub pos: f64,
}

impl Event {
    pub fn new(event_type: EventType, node_idx: NodeIdx, pos: f64) -> Self {
        Self {
            event_type,
            node_idx,
            pos,
        }
    }
}

// ----------------------------------------------------------------------------
// Task #11: Event sorting (compare_events)
// C++ ref: scanline.cpp:294-308
// Sort by: position, then event type, then node index (for stability)
// ----------------------------------------------------------------------------

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        (self.pos - other.pos).abs() < 1e-10
            && self.event_type == other.event_type
            && self.node_idx == other.node_idx
    }
}

impl Eq for Event {}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> Ordering {
        // C++ ref: compare_events in scanline.cpp:294-308
        // First compare by position
        match self.pos.partial_cmp(&other.pos) {
            Some(Ordering::Equal) | None => {}
            Some(ord) => return ord,
        }
        // Then by event type (using the enum ordering: Open < ConnPoint < Close)
        match self.event_type.cmp(&other.event_type) {
            Ordering::Equal => {}
            ord => return ord,
        }
        // Finally by node index for stability
        self.node_idx.cmp(&other.node_idx)
    }
}

// ----------------------------------------------------------------------------
// Task #8: LineSegment struct
// C++ ref: orthogonal.cpp class LineSegment
// ----------------------------------------------------------------------------

/// Position-indexed vertex information for breakpoint sets.
/// C++ ref: struct PosVertInf in orthogonal.cpp
#[derive(Clone, Debug)]
pub struct PosVertInf {
    /// Position along the segment
    pub pos: f64,
    /// Vertex ID in the visibility graph (None for pure breakpoints)
    pub vertex_id: Option<u32>,
    /// Visibility directions from this point (used for connector endpoints)
    pub vis_dirs: ConnDirFlags,
}

impl PosVertInf {
    pub fn new(pos: f64, vertex_id: Option<u32>, vis_dirs: ConnDirFlags) -> Self {
        Self { pos, vertex_id, vis_dirs }
    }
}

impl PartialEq for PosVertInf {
    fn eq(&self, other: &Self) -> bool {
        (self.pos - other.pos).abs() < 1e-10
    }
}

impl Eq for PosVertInf {}

impl PartialOrd for PosVertInf {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PosVertInf {
    fn cmp(&self, other: &Self) -> Ordering {
        self.pos.partial_cmp(&other.pos).unwrap_or(Ordering::Equal)
    }
}

/// A visibility line segment during sweep.
/// C++ ref: class LineSegment in orthogonal.cpp
#[derive(Clone, Debug)]
pub struct LineSegment {
    /// Start position on parallel axis
    pub begin: f64,
    /// End position on parallel axis
    pub finish: f64,
    /// Position on perpendicular axis
    pub pos: f64,
    /// Whether this is a shape side (fixed position)
    pub shape_side: bool,
    /// Vertices on this segment (will become breakpoints)
    pub vert_infs: BTreeSet<PosVertInf>,
}

impl LineSegment {
    /// Create a new segment with two endpoints.
    pub fn new(begin: f64, finish: f64, pos: f64, shape_side: bool) -> Self {
        Self {
            begin,
            finish,
            pos,
            shape_side,
            vert_infs: BTreeSet::new(),
        }
    }

    /// Create a point segment (for connector endpoints without visibility).
    pub fn point(pos_along: f64, pos_perp: f64, vertex_id: Option<u32>) -> Self {
        let mut seg = Self::new(pos_along, pos_along, pos_perp, false);
        if let Some(vid) = vertex_id {
            seg.vert_infs.insert(PosVertInf::new(pos_along, Some(vid), 0));
        }
        seg
    }

    /// Check if this segment overlaps with another at the same perpendicular position.
    /// C++ ref: LineSegment::overlaps()
    pub fn overlaps(&self, other: &LineSegment) -> bool {
        (self.pos - other.pos).abs() < 1e-10
            && self.begin - 1e-10 <= other.finish
            && other.begin - 1e-10 <= self.finish
    }

    /// Merge another segment's vertices into this one.
    /// C++ ref: LineSegment::mergeVertInfs()
    pub fn merge_vert_infs(&mut self, other: &LineSegment) {
        for vi in &other.vert_infs {
            self.vert_infs.insert(vi.clone());
        }
        self.begin = self.begin.min(other.begin);
        self.finish = self.finish.max(other.finish);
    }

    /// Insert a vertex at a position along the segment.
    pub fn insert_vertex(&mut self, pos: f64, vertex_id: Option<u32>, vis_dirs: ConnDirFlags) {
        self.vert_infs.insert(PosVertInf::new(pos, vertex_id, vis_dirs));
    }
}

impl PartialEq for LineSegment {
    fn eq(&self, other: &Self) -> bool {
        (self.pos - other.pos).abs() < 1e-10 && (self.begin - other.begin).abs() < 1e-10
    }
}

impl Eq for LineSegment {}

impl PartialOrd for LineSegment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LineSegment {
    fn cmp(&self, other: &Self) -> Ordering {
        // Sort by pos first, then by begin
        match self.pos.partial_cmp(&other.pos) {
            Some(Ordering::Equal) | None => {}
            Some(ord) => return ord,
        }
        self.begin.partial_cmp(&other.begin).unwrap_or(Ordering::Equal)
    }
}

// ----------------------------------------------------------------------------
// Task #13: SegmentListWrapper
// C++ ref: orthogonal.cpp:1225-1270
// ----------------------------------------------------------------------------

/// Container that manages LineSegments and merges overlapping ones.
/// C++ ref: class SegmentListWrapper in orthogonal.cpp
#[derive(Debug, Default)]
pub struct SegmentListWrapper {
    segments: Vec<LineSegment>,
}

impl SegmentListWrapper {
    pub fn new() -> Self {
        Self { segments: Vec::new() }
    }

    /// Insert a segment, merging with existing overlapping segments.
    /// C++ ref: SegmentListWrapper::insert()
    pub fn insert(&mut self, segment: LineSegment) -> &mut LineSegment {
        let mut found_idx: Option<usize> = None;

        // Find all overlapping segments
        let mut i = 0;
        while i < self.segments.len() {
            if self.segments[i].overlaps(&segment) {
                if let Some(prev_idx) = found_idx {
                    // Merge current into previous found segment
                    let current = self.segments.remove(i);
                    self.segments[prev_idx].merge_vert_infs(&current);
                    // Adjust index since we removed an element
                    continue;
                } else {
                    // First overlapping segment - merge new segment into it
                    self.segments[i].merge_vert_infs(&segment);
                    found_idx = Some(i);
                }
            }
            i += 1;
        }

        if let Some(idx) = found_idx {
            &mut self.segments[idx]
        } else {
            // No overlapping segment found, add new one
            self.segments.push(segment);
            self.segments.last_mut().unwrap()
        }
    }

    /// Get reference to the segments list
    pub fn list(&self) -> &[LineSegment] {
        &self.segments
    }

    /// Get mutable reference to the segments list
    pub fn list_mut(&mut self) -> &mut Vec<LineSegment> {
        &mut self.segments
    }

    /// Sort the segments
    pub fn sort(&mut self) {
        self.segments.sort();
    }

    /// Clear all segments
    pub fn clear(&mut self) {
        self.segments.clear();
    }
}

// ============================================================================
// Phase 2 & 3: Scanline and Node Methods (Tasks #2-#5, #19-#21)
// ============================================================================

/// Scanline state during sweep.
/// C++ ref: NodeSet in scanline.h:76
#[derive(Debug)]
pub struct Scanline {
    /// Nodes in scanline, sorted by position
    node_indices: Vec<NodeIdx>,
}

impl Scanline {
    pub fn new() -> Self {
        Self { node_indices: Vec::new() }
    }

    /// Insert a node into the scanline, maintaining sort order.
    /// Returns the indices of the node above and below.
    /// C++ ref: processEventVert pass 1 - orthogonal.cpp:1376-1394
    pub fn insert(&mut self, nodes: &mut [ScanlineNode], node_idx: NodeIdx) -> (Option<NodeIdx>, Option<NodeIdx>) {
        let node_pos = nodes[node_idx].pos;

        // Find insertion point using binary search
        let insert_pos = self.node_indices.partition_point(|&idx| {
            let n = &nodes[idx];
            if (n.pos - node_pos).abs() < 1e-10 {
                // Same position, use index as tie-breaker
                idx < node_idx
            } else {
                n.pos < node_pos
            }
        });

        // Get neighbors before insertion
        let above = if insert_pos > 0 {
            Some(self.node_indices[insert_pos - 1])
        } else {
            None
        };

        let below = if insert_pos < self.node_indices.len() {
            Some(self.node_indices[insert_pos])
        } else {
            None
        };

        // Set up neighbor pointers
        nodes[node_idx].first_above = above;
        nodes[node_idx].first_below = below;

        // Update neighbors' pointers
        if let Some(above_idx) = above {
            nodes[above_idx].first_below = Some(node_idx);
        }
        if let Some(below_idx) = below {
            nodes[below_idx].first_above = Some(node_idx);
        }

        // Insert into scanline
        self.node_indices.insert(insert_pos, node_idx);

        (above, below)
    }

    /// Remove a node from the scanline.
    /// C++ ref: processEventVert pass 3 - orthogonal.cpp:1518-1541
    pub fn remove(&mut self, nodes: &mut [ScanlineNode], node_idx: NodeIdx) {
        // Update neighbors
        let above = nodes[node_idx].first_above;
        let below = nodes[node_idx].first_below;

        if let Some(above_idx) = above {
            nodes[above_idx].first_below = below;
        }
        if let Some(below_idx) = below {
            nodes[below_idx].first_above = above;
        }

        // Remove from scanline
        if let Some(pos) = self.node_indices.iter().position(|&idx| idx == node_idx) {
            self.node_indices.remove(pos);
        }
    }

    pub fn len(&self) -> usize {
        self.node_indices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.node_indices.is_empty()
    }
}

impl Default for Scanline {
    fn default() -> Self {
        Self::new()
    }
}

// ----------------------------------------------------------------------------
// Tasks #2-#5: Node methods for finding visibility limits
// C++ ref: scanline.cpp:100-282
// ----------------------------------------------------------------------------

/// Find the first obstacle edge above in the scanline.
/// C++ ref: Node::firstObstacleAbove - scanline.cpp:100-113
pub fn first_obstacle_above(nodes: &[ScanlineNode], node_idx: NodeIdx, dim: usize) -> f64 {
    let node = &nodes[node_idx];
    let mut curr_idx = node.first_above;

    while let Some(idx) = curr_idx {
        let curr = &nodes[idx];
        // Skip connector points (they have max[dim] > pos meaning they're opening/closing)
        if !curr.is_conn_point() && curr.max[dim] <= node.pos {
            return curr.max[dim];
        }
        curr_idx = curr.first_above;
    }

    NEG_DBL_MAX
}

/// Find the first obstacle edge below in the scanline.
/// C++ ref: Node::firstObstacleBelow - scanline.cpp:118-131
pub fn first_obstacle_below(nodes: &[ScanlineNode], node_idx: NodeIdx, dim: usize) -> f64 {
    let node = &nodes[node_idx];
    let mut curr_idx = node.first_below;

    while let Some(idx) = curr_idx {
        let curr = &nodes[idx];
        // Skip connector points
        if !curr.is_conn_point() && curr.min[dim] >= node.pos {
            return curr.min[dim];
        }
        curr_idx = curr.first_below;
    }

    DBL_MAX
}

/// Find the first point above, ignoring shapes that are inline with edges.
/// C++ ref: Node::firstPointAbove - scanline.cpp:219-238
pub fn first_point_above(nodes: &[ScanlineNode], node_idx: NodeIdx, dim: usize) -> f64 {
    let node = &nodes[node_idx];
    let alt_dim = 1 - dim;
    let mut result = NEG_DBL_MAX;
    let mut curr_idx = node.first_above;

    while let Some(idx) = curr_idx {
        let curr = &nodes[idx];
        // Check if inline with edge (shapes that share an edge boundary)
        let in_line_with_edge = (node.min[alt_dim] - curr.min[alt_dim]).abs() < 1e-10
            || (node.min[alt_dim] - curr.max[alt_dim]).abs() < 1e-10;

        if !in_line_with_edge && curr.max[dim] <= node.pos {
            result = result.max(curr.max[dim]);
        }
        curr_idx = curr.first_above;
    }

    result
}

/// Find the first point below, ignoring shapes that are inline with edges.
/// C++ ref: Node::firstPointBelow - scanline.cpp:241-261
pub fn first_point_below(nodes: &[ScanlineNode], node_idx: NodeIdx, dim: usize) -> f64 {
    let node = &nodes[node_idx];
    let alt_dim = 1 - dim;
    let mut result = DBL_MAX;
    let mut curr_idx = node.first_below;

    while let Some(idx) = curr_idx {
        let curr = &nodes[idx];
        // Check if inline with edge
        let in_line_with_edge = (node.min[alt_dim] - curr.min[alt_dim]).abs() < 1e-10
            || (node.min[alt_dim] - curr.max[alt_dim]).abs() < 1e-10;

        if !in_line_with_edge && curr.min[dim] >= node.pos {
            result = result.min(curr.min[dim]);
        }
        curr_idx = curr.first_below;
    }

    result
}

/// Find visibility limits in both directions, handling overlapping shapes.
/// C++ ref: Node::findFirstPointAboveAndBelow - scanline.cpp:165-217
pub fn find_first_point_above_and_below(
    nodes: &[ScanlineNode],
    node_idx: NodeIdx,
    dim: usize,
    line_pos: f64,
) -> (f64, f64, f64, f64) {
    let node = &nodes[node_idx];
    let alt_dim = 1 - dim;

    let mut first_above_pos = NEG_DBL_MAX;
    let mut first_below_pos = DBL_MAX;
    // We start looking left from the right side of the shape, and vice versa
    let mut last_above_pos = node.max[dim];
    let mut last_below_pos = node.min[dim];

    // Look in both directions (above then below)
    for direction in 0..2 {
        let mut curr_idx = if direction == 0 {
            node.first_above
        } else {
            node.first_below
        };

        while let Some(idx) = curr_idx {
            let curr = &nodes[idx];

            // Check if events are at shared beginning or end of a shape
            let events_at_same_pos = ((line_pos - node.max[alt_dim]).abs() < 1e-10
                && (line_pos - curr.max[alt_dim]).abs() < 1e-10)
                || ((line_pos - node.min[alt_dim]).abs() < 1e-10
                    && (line_pos - curr.min[alt_dim]).abs() < 1e-10);

            if curr.max[dim] <= node.min[dim] {
                // Curr shape is completely to the left, add its right side as limit
                first_above_pos = first_above_pos.max(curr.max[dim]);
            } else if curr.min[dim] >= node.max[dim] {
                // Curr shape is completely to the right, add its left side as limit
                first_below_pos = first_below_pos.min(curr.min[dim]);
            } else if !events_at_same_pos {
                // Shapes overlap - determine where
                last_above_pos = last_above_pos.min(curr.min[dim]);
                last_below_pos = last_below_pos.max(curr.max[dim]);
            }

            curr_idx = if direction == 0 {
                curr.first_above
            } else {
                curr.first_below
            };
        }
    }

    (first_above_pos, first_below_pos, last_above_pos, last_below_pos)
}

/// Check if node position is inside any shape.
/// C++ ref: Node::isInsideShape - scanline.cpp:265-282
pub fn is_inside_shape(nodes: &[ScanlineNode], node_idx: NodeIdx, dim: usize) -> bool {
    let node = &nodes[node_idx];

    // Check below
    let mut curr_idx = node.first_below;
    while let Some(idx) = curr_idx {
        let curr = &nodes[idx];
        if curr.min[dim] < node.pos && node.pos < curr.max[dim] {
            return true;
        }
        curr_idx = curr.first_below;
    }

    // Check above
    curr_idx = node.first_above;
    while let Some(idx) = curr_idx {
        let curr = &nodes[idx];
        if curr.min[dim] < node.pos && node.pos < curr.max[dim] {
            return true;
        }
        curr_idx = curr.first_above;
    }

    false
}

// ============================================================================
// Input Structures
// ============================================================================

/// Input obstacle for visibility graph generation.
pub struct ObstacleInput {
    pub id: u32,
    pub polygon: Polygon,
}

/// Input connector endpoint for visibility graph generation.
pub struct ConnectorInput {
    pub id: u32,
    pub start: Point,
    pub end: Point,
    pub start_dirs: ConnDirFlags,
    pub end_dirs: ConnDirFlags,
}

impl ConnectorInput {
    pub fn new(id: u32, start: Point, end: Point) -> Self {
        // Default: all directions
        Self {
            id,
            start,
            end,
            start_dirs: CONN_DIR_LEFT | CONN_DIR_RIGHT | CONN_DIR_UP | CONN_DIR_DOWN,
            end_dirs: CONN_DIR_LEFT | CONN_DIR_RIGHT | CONN_DIR_UP | CONN_DIR_DOWN,
        }
    }

    pub fn with_directions(id: u32, start: Point, end: Point, start_dirs: ConnDirFlags, end_dirs: ConnDirFlags) -> Self {
        Self { id, start, end, start_dirs, end_dirs }
    }
}

// ============================================================================
// Main Algorithm (Phases 2-5)
// ============================================================================

/// Orthogonal visibility graph generator.
/// Implements the sweep-line algorithm from C++ libavoid.
pub struct OrthogonalVisGraphGenerator {
    // No persistent state needed - nodes are created per-sweep
}

impl OrthogonalVisGraphGenerator {
    pub fn new() -> Self {
        Self {}
    }

    /// Generate the static orthogonal visibility graph.
    /// C++ ref: generateStaticOrthogonalVisGraph() - orthogonal.cpp:1730-1981
    ///
    /// `shape_buffer_distance` expands obstacle bounds by this amount, creating
    /// a buffer zone that routes cannot enter.
    pub fn generate(
        &mut self,
        obstacles: &[ObstacleInput],
        connectors: &[ConnectorInput],
    ) -> VisibilityGraph {
        self.generate_with_buffer(obstacles, connectors, 0.0)
    }

    /// Generate visibility graph with a buffer distance around obstacles.
    pub fn generate_with_buffer(
        &mut self,
        obstacles: &[ObstacleInput],
        connectors: &[ConnectorInput],
        shape_buffer_distance: f64,
    ) -> VisibilityGraph {
        let mut graph = VisibilityGraph::new();

        // Maps to track vertices
        let mut point_to_vertex: HashMap<(i64, i64), u32> = HashMap::new();
        let point_key = |p: &Point| -> (i64, i64) {
            ((p.x * 10000.0).round() as i64, (p.y * 10000.0).round() as i64)
        };

        // Add connector endpoints to graph first
        for conn in connectors {
            let start_key = point_key(&conn.start);
            let start_id = graph.add_vertex(conn.start);
            point_to_vertex.insert(start_key, start_id);

            let end_key = point_key(&conn.end);
            let end_id = graph.add_vertex(conn.end);
            point_to_vertex.insert(end_key, end_id);
        }

        // ====================================================================
        // VERTICAL SWEEP - creates horizontal segments
        // C++ ref: orthogonal.cpp:1734-1861
        // ====================================================================

        // Create events for vertical sweep
        let (mut events, mut nodes) = self.create_vertical_events(obstacles, connectors, shape_buffer_distance);
        events.sort();

        // Fix visibility on graph boundary
        self.fix_connection_visibility_on_boundary(&mut nodes, &events, CONN_DIR_LEFT | CONN_DIR_RIGHT);

        // Process vertical sweep
        let mut h_segments = SegmentListWrapper::new();
        self.process_vertical_sweep(&mut events, &mut nodes, &mut h_segments);

        h_segments.sort();

        // ====================================================================
        // HORIZONTAL SWEEP - creates vertical segments
        // C++ ref: orthogonal.cpp:1863-1980
        // ====================================================================

        // Create events for horizontal sweep
        let (mut h_events, mut h_nodes) = self.create_horizontal_events(obstacles, connectors, shape_buffer_distance);
        h_events.sort();

        // Fix visibility on graph boundary
        self.fix_connection_visibility_on_boundary(&mut h_nodes, &h_events, CONN_DIR_UP | CONN_DIR_DOWN);

        // Process horizontal sweep with intersection
        self.process_horizontal_sweep_with_intersection(
            &mut h_events,
            &mut h_nodes,
            &mut h_segments,
            &mut graph,
            &mut point_to_vertex,
            &point_key,
        );

        // Generate remaining horizontal edges
        for seg in h_segments.list() {
            self.generate_edges_from_segment(seg, true, &mut graph, &mut point_to_vertex, &point_key);
        }

        graph
    }

    /// Create events and nodes for vertical sweep.
    /// C++ ref: orthogonal.cpp:1737-1801
    fn create_vertical_events(
        &mut self,
        obstacles: &[ObstacleInput],
        connectors: &[ConnectorInput],
        buffer: f64,
    ) -> (Vec<Event>, Vec<ScanlineNode>) {
        let mut events = Vec::new();
        let mut nodes = Vec::new();

        // Create obstacle events
        for obs in obstacles {
            let (min_x, min_y, max_x, max_y) = polygon_bounds(&obs.polygon);
            // Apply buffer - expand obstacle bounds
            let min_x = min_x - buffer;
            let min_y = min_y - buffer;
            let max_x = max_x + buffer;
            let max_y = max_y + buffer;
            let mid_x = min_x + (max_x - min_x) / 2.0;

            let node_idx = nodes.len();
            nodes.push(ScanlineNode::for_obstacle(obs.id, mid_x, min_x, min_y, max_x, max_y));

            // Open at min_y, Close at max_y
            events.push(Event::new(EventType::Open, node_idx, min_y));
            events.push(Event::new(EventType::Close, node_idx, max_y));
        }

        // Create connector endpoint events
        for conn in connectors {
            // Start point
            let start_idx = nodes.len();
            nodes.push(ScanlineNode::for_conn_point(
                conn.id * 2,
                conn.start.x,
                conn.start.x,
                conn.start.y,
                conn.start_dirs,
            ));
            events.push(Event::new(EventType::ConnPoint, start_idx, conn.start.y));

            // End point
            let end_idx = nodes.len();
            nodes.push(ScanlineNode::for_conn_point(
                conn.id * 2 + 1,
                conn.end.x,
                conn.end.x,
                conn.end.y,
                conn.end_dirs,
            ));
            events.push(Event::new(EventType::ConnPoint, end_idx, conn.end.y));
        }

        (events, nodes)
    }

    /// Create events and nodes for horizontal sweep.
    /// C++ ref: orthogonal.cpp:1863-1902
    fn create_horizontal_events(
        &mut self,
        obstacles: &[ObstacleInput],
        connectors: &[ConnectorInput],
        buffer: f64,
    ) -> (Vec<Event>, Vec<ScanlineNode>) {
        let mut events = Vec::new();
        let mut nodes = Vec::new();

        // Create obstacle events
        for obs in obstacles {
            let (min_x, min_y, max_x, max_y) = polygon_bounds(&obs.polygon);
            // Apply buffer - expand obstacle bounds
            let min_x = min_x - buffer;
            let min_y = min_y - buffer;
            let max_x = max_x + buffer;
            let max_y = max_y + buffer;
            let mid_y = min_y + (max_y - min_y) / 2.0;

            let node_idx = nodes.len();
            nodes.push(ScanlineNode::for_obstacle(obs.id, mid_y, min_x, min_y, max_x, max_y));

            // Open at min_x, Close at max_x
            events.push(Event::new(EventType::Open, node_idx, min_x));
            events.push(Event::new(EventType::Close, node_idx, max_x));
        }

        // Create connector endpoint events
        for conn in connectors {
            // Start point
            let start_idx = nodes.len();
            nodes.push(ScanlineNode::for_conn_point(
                conn.id * 2,
                conn.start.y,
                conn.start.x,
                conn.start.y,
                conn.start_dirs,
            ));
            events.push(Event::new(EventType::ConnPoint, start_idx, conn.start.x));

            // End point
            let end_idx = nodes.len();
            nodes.push(ScanlineNode::for_conn_point(
                conn.id * 2 + 1,
                conn.end.y,
                conn.end.x,
                conn.end.y,
                conn.end_dirs,
            ));
            events.push(Event::new(EventType::ConnPoint, end_idx, conn.end.x));
        }

        (events, nodes)
    }

    /// Fix visibility for connector endpoints on the boundary of the visibility graph.
    /// C++ ref: fixConnectionPointVisibilityOnOutsideOfVisibilityGraph - orthogonal.cpp:1691-1728
    fn fix_connection_visibility_on_boundary(
        &self,
        nodes: &mut [ScanlineNode],
        events: &[Event],
        added_visibility: ConnDirFlags,
    ) {
        if events.is_empty() {
            return;
        }

        let first_pos = events[0].pos;
        let last_pos = events.last().unwrap().pos;

        // Fix leading edge
        for event in events.iter() {
            if event.pos > first_pos + 1e-10 {
                break;
            }
            if nodes[event.node_idx].is_conn_point() {
                nodes[event.node_idx].vis_directions |= added_visibility;
            }
        }

        // Fix trailing edge
        for event in events.iter().rev() {
            if event.pos < last_pos - 1e-10 {
                break;
            }
            if nodes[event.node_idx].is_conn_point() {
                nodes[event.node_idx].vis_directions |= added_visibility;
            }
        }
    }

    /// Process vertical sweep to create horizontal segments.
    /// C++ ref: orthogonal.cpp:1811-1860
    fn process_vertical_sweep(
        &mut self,
        events: &mut [Event],
        nodes: &mut [ScanlineNode],
        segments: &mut SegmentListWrapper,
    ) {
        let mut scanline = Scanline::new();
        let total_events = events.len();
        if total_events == 0 {
            return;
        }

        let mut this_pos = events[0].pos;
        let mut pos_start_index = 0;

        for i in 0..=total_events {
            // Process events at the same position
            if i == total_events || (events[i].pos - this_pos).abs() > 1e-10 {
                let pos_finish_index = i;

                // Passes 2 and 3: process and remove
                for pass in 2..=3 {
                    for j in pos_start_index..pos_finish_index {
                        self.process_event_vert(
                            &mut scanline,
                            nodes,
                            segments,
                            &events[j],
                            pass,
                        );
                    }
                }

                if i == total_events {
                    break;
                }

                this_pos = events[i].pos;
                pos_start_index = i;
            }

            // Pass 1: add to scanline
            self.process_event_vert(&mut scanline, nodes, segments, &events[i], 1);
        }
    }

    /// Process a single event in the vertical sweep.
    /// C++ ref: processEventVert - orthogonal.cpp:1368-1543
    fn process_event_vert(
        &mut self,
        scanline: &mut Scanline,
        nodes: &mut [ScanlineNode],
        segments: &mut SegmentListWrapper,
        event: &Event,
        pass: u32,
    ) {
        let node_idx = event.node_idx;

        // Pass 1: Insert Open events into scanline
        // Pass 2 for ConnPoint: Insert into scanline
        if (pass == 1 && event.event_type == EventType::Open)
            || (pass == 2 && event.event_type == EventType::ConnPoint)
        {
            scanline.insert(nodes, node_idx);
        }

        // Pass 2: Process events to create segments
        if pass == 2 {
            let node = &nodes[node_idx];

            if event.event_type == EventType::Open || event.event_type == EventType::Close {
                // Shape edge event
                let line_y = if event.event_type == EventType::Open {
                    node.min[YDIM]
                } else {
                    node.max[YDIM]
                };

                let min_shape = node.min[XDIM];
                let max_shape = node.max[XDIM];

                // Find visibility limits
                let (min_limit, max_limit, min_limit_max, max_limit_min) =
                    find_first_point_above_and_below(nodes, node_idx, XDIM, line_y);

                // Create segments based on overlapping shapes
                if min_limit_max >= max_limit_min {
                    // No overlapping shapes - full visibility
                    // Segment from minLimit to minShape (if visible left)
                    if min_limit < min_shape {
                        let mut seg = LineSegment::new(min_limit, min_shape, line_y, true);
                        seg.insert_vertex(min_shape, None, 0); // Shape corner
                        segments.insert(seg);
                    }

                    // Segment along shape edge
                    let mut edge_seg = LineSegment::new(min_shape, max_shape, line_y, true);
                    edge_seg.insert_vertex(min_shape, None, 0);
                    edge_seg.insert_vertex(max_shape, None, 0);
                    segments.insert(edge_seg);

                    // Segment from maxShape to maxLimit (if visible right)
                    if max_shape < max_limit {
                        let mut seg = LineSegment::new(max_shape, max_limit, line_y, true);
                        seg.insert_vertex(max_shape, None, 0); // Shape corner
                        segments.insert(seg);
                    }
                } else {
                    // Overlapping shapes
                    if min_limit_max > min_limit && min_limit_max >= min_shape {
                        let mut seg = LineSegment::new(min_limit, min_limit_max, line_y, true);
                        seg.insert_vertex(min_shape, None, 0);
                        segments.insert(seg);
                    }
                    if max_limit_min < max_limit && max_limit_min <= max_shape {
                        let mut seg = LineSegment::new(max_limit_min, max_limit, line_y, true);
                        seg.insert_vertex(max_shape, None, 0);
                        segments.insert(seg);
                    }
                }
            } else if event.event_type == EventType::ConnPoint {
                // Connector endpoint event
                let cp_x = node.min[XDIM];
                let cp_y = event.pos;
                let vis_dirs = node.vis_directions;

                // Find visibility limits
                let min_limit = first_point_above(nodes, node_idx, XDIM);
                let max_limit = first_point_below(nodes, node_idx, XDIM);
                let in_shape = is_inside_shape(nodes, node_idx, XDIM);

                let mut line1_created = false;
                let mut line2_created = false;

                // Create segment to the left
                if (vis_dirs & CONN_DIR_LEFT) != 0 && min_limit < cp_x {
                    let mut seg = LineSegment::new(min_limit, cp_x, cp_y, true);
                    seg.insert_vertex(cp_x, node.conn_vertex_id, vis_dirs);
                    segments.insert(seg);
                    line1_created = true;
                }

                // Create segment to the right
                if (vis_dirs & CONN_DIR_RIGHT) != 0 && cp_x < max_limit {
                    let mut seg = LineSegment::new(cp_x, max_limit, cp_y, true);
                    seg.insert_vertex(cp_x, node.conn_vertex_id, vis_dirs);
                    segments.insert(seg);
                    line2_created = true;
                }

                // Add point segment if no lines created
                if !line1_created && !line2_created {
                    let seg = LineSegment::point(cp_x, cp_y, node.conn_vertex_id);
                    segments.insert(seg);
                }

                // Add dummy vertex if not inside shape (for routing around)
                if !in_shape && (line1_created || line2_created) {
                    // The connector endpoint vertex is already added; we might add
                    // a general routing vertex at the same position
                }
            }
        }

        // Pass 3: Remove Close events from scanline
        // Pass 2 for ConnPoint: Remove from scanline
        if (pass == 3 && event.event_type == EventType::Close)
            || (pass == 2 && event.event_type == EventType::ConnPoint)
        {
            scanline.remove(nodes, node_idx);
        }
    }

    /// Process horizontal sweep and intersect with horizontal segments.
    /// C++ ref: orthogonal.cpp:1912-1960
    fn process_horizontal_sweep_with_intersection<F>(
        &mut self,
        events: &mut [Event],
        nodes: &mut [ScanlineNode],
        h_segments: &mut SegmentListWrapper,
        graph: &mut VisibilityGraph,
        point_to_vertex: &mut HashMap<(i64, i64), u32>,
        point_key: &F,
    ) where
        F: Fn(&Point) -> (i64, i64),
    {
        let mut scanline = Scanline::new();
        let mut v_segments = SegmentListWrapper::new();
        let total_events = events.len();
        if total_events == 0 {
            return;
        }

        let mut this_pos = events[0].pos;
        let mut pos_start_index = 0;

        for i in 0..=total_events {
            if i == total_events || (events[i].pos - this_pos).abs() > 1e-10 {
                let pos_finish_index = i;

                // Passes 2 and 3
                for pass in 2..=3 {
                    for j in pos_start_index..pos_finish_index {
                        self.process_event_hori(&mut scanline, nodes, &mut v_segments, &events[j], pass);
                    }
                }

                // Process vertical segments and intersect with horizontal
                v_segments.sort();
                for v_seg in v_segments.list_mut() {
                    self.intersect_segments(h_segments, v_seg, graph, point_to_vertex, point_key);
                }
                v_segments.clear();

                if i == total_events {
                    break;
                }

                this_pos = events[i].pos;
                pos_start_index = i;
            }

            // Pass 1
            self.process_event_hori(&mut scanline, nodes, &mut v_segments, &events[i], 1);
        }
    }

    /// Process a single event in the horizontal sweep.
    /// C++ ref: processEventHori - orthogonal.cpp:1551-1686
    fn process_event_hori(
        &mut self,
        scanline: &mut Scanline,
        nodes: &mut [ScanlineNode],
        segments: &mut SegmentListWrapper,
        event: &Event,
        pass: u32,
    ) {
        let node_idx = event.node_idx;

        // Pass 1: Insert Open events
        // Pass 2 for ConnPoint: Insert
        if (pass == 1 && event.event_type == EventType::Open)
            || (pass == 2 && event.event_type == EventType::ConnPoint)
        {
            scanline.insert(nodes, node_idx);
        }

        // Pass 2: Process events
        if pass == 2 {
            let node = &nodes[node_idx];

            if event.event_type == EventType::Open || event.event_type == EventType::Close {
                // Shape edge event
                let line_x = if event.event_type == EventType::Open {
                    node.min[XDIM]
                } else {
                    node.max[XDIM]
                };

                let min_shape = node.min[YDIM];
                let max_shape = node.max[YDIM];

                // Find visibility limits
                let (min_limit, max_limit, min_limit_max, max_limit_min) =
                    find_first_point_above_and_below(nodes, node_idx, YDIM, line_x);

                if min_limit_max >= max_limit_min {
                    // No overlapping shapes
                    let mut seg = LineSegment::new(min_limit, max_limit, line_x, false);
                    seg.insert_vertex(min_shape, None, 0);
                    seg.insert_vertex(max_shape, None, 0);
                    segments.insert(seg);
                } else {
                    // Overlapping shapes
                    if min_limit_max > min_limit && min_limit_max >= min_shape {
                        let mut seg = LineSegment::new(min_limit, min_limit_max, line_x, false);
                        seg.insert_vertex(min_shape, None, 0);
                        segments.insert(seg);
                    }
                    if max_limit_min < max_limit && max_limit_min <= max_shape {
                        let mut seg = LineSegment::new(max_limit_min, max_limit, line_x, false);
                        seg.insert_vertex(max_shape, None, 0);
                        segments.insert(seg);
                    }
                }
            } else if event.event_type == EventType::ConnPoint {
                // Connector endpoint event
                let cp_x = event.pos;
                let cp_y = node.min[YDIM];
                let vis_dirs = node.vis_directions;

                let min_limit = first_point_above(nodes, node_idx, YDIM);
                let max_limit = first_point_below(nodes, node_idx, YDIM);

                // Create segment upward (includes connector point as vertex)
                if (vis_dirs & CONN_DIR_UP) != 0 && min_limit < cp_y {
                    let mut seg = LineSegment::new(min_limit, cp_y, cp_x, false);
                    seg.insert_vertex(cp_y, node.conn_vertex_id, vis_dirs);
                    segments.insert(seg);
                }

                // Create segment downward (includes connector point as vertex)
                if (vis_dirs & CONN_DIR_DOWN) != 0 && cp_y < max_limit {
                    let mut seg = LineSegment::new(cp_y, max_limit, cp_x, false);
                    seg.insert_vertex(cp_y, node.conn_vertex_id, vis_dirs);
                    segments.insert(seg);
                }
            }
        }

        // Pass 3: Remove Close events
        // Pass 2 for ConnPoint: Remove
        if (pass == 3 && event.event_type == EventType::Close)
            || (pass == 2 && event.event_type == EventType::ConnPoint)
        {
            scanline.remove(nodes, node_idx);
        }
    }

    /// Intersect vertical segments with horizontal segments.
    /// C++ ref: intersectSegments - orthogonal.cpp:1276-1360
    fn intersect_segments<F>(
        &mut self,
        h_segments: &mut SegmentListWrapper,
        v_seg: &mut LineSegment,
        graph: &mut VisibilityGraph,
        point_to_vertex: &mut HashMap<(i64, i64), u32>,
        point_key: &F,
    ) where
        F: Fn(&Point) -> (i64, i64),
    {
        let vert_x = v_seg.pos;

        let mut segments_to_remove = Vec::new();

        for (idx, h_seg) in h_segments.list_mut().iter_mut().enumerate() {
            let in_vert_seg_region =
                v_seg.begin <= h_seg.pos + 1e-10 && v_seg.finish >= h_seg.pos - 1e-10;

            if vert_x < h_seg.begin - 1e-10 {
                // Haven't reached this segment yet
                continue;
            } else if (vert_x - h_seg.begin).abs() < 1e-10 {
                // At beginning of horizontal segment
                if in_vert_seg_region {
                    // Add intersection vertex to BOTH segments
                    let point = Point::new(vert_x, h_seg.pos);
                    let key = point_key(&point);
                    let vertex_id = *point_to_vertex
                        .entry(key)
                        .or_insert_with(|| graph.add_vertex(point));
                    h_seg.insert_vertex(vert_x, Some(vertex_id), 0);
                    v_seg.insert_vertex(h_seg.pos, Some(vertex_id), 0);
                }
            } else if (vert_x - h_seg.finish).abs() < 1e-10 {
                // At end of horizontal segment
                if in_vert_seg_region {
                    let point = Point::new(vert_x, h_seg.pos);
                    let key = point_key(&point);
                    let vertex_id = *point_to_vertex
                        .entry(key)
                        .or_insert_with(|| graph.add_vertex(point));
                    h_seg.insert_vertex(vert_x, Some(vertex_id), 0);
                    v_seg.insert_vertex(h_seg.pos, Some(vertex_id), 0);

                    // Generate edges for this horizontal segment
                    self.generate_edges_from_segment(h_seg, true, graph, point_to_vertex, point_key);
                    segments_to_remove.push(idx);
                }
            } else if vert_x > h_seg.finish + 1e-10 {
                // Past this horizontal segment
                self.generate_edges_from_segment(h_seg, true, graph, point_to_vertex, point_key);
                segments_to_remove.push(idx);
            } else {
                // In the middle of horizontal segment
                if in_vert_seg_region {
                    let point = Point::new(vert_x, h_seg.pos);
                    let key = point_key(&point);
                    let vertex_id = *point_to_vertex
                        .entry(key)
                        .or_insert_with(|| graph.add_vertex(point));
                    h_seg.insert_vertex(vert_x, Some(vertex_id), 0);
                    v_seg.insert_vertex(h_seg.pos, Some(vertex_id), 0);
                }
            }
        }

        // Remove processed segments (in reverse order to maintain indices)
        for idx in segments_to_remove.into_iter().rev() {
            h_segments.list_mut().remove(idx);
        }

        // Generate edges from vertical segment
        self.generate_edges_from_segment(v_seg, false, graph, point_to_vertex, point_key);
    }

    /// Generate visibility edges from a segment's breakpoints.
    fn generate_edges_from_segment<F>(
        &mut self,
        segment: &LineSegment,
        is_horizontal: bool,
        graph: &mut VisibilityGraph,
        point_to_vertex: &mut HashMap<(i64, i64), u32>,
        point_key: &F,
    ) where
        F: Fn(&Point) -> (i64, i64),
    {
        let breakpoints: Vec<_> = segment.vert_infs.iter().collect();

        // Add begin and end points if not present
        let begin_pos = segment.begin;
        let finish_pos = segment.finish;

        // Collect all vertex positions and ensure vertices exist
        let mut vertex_ids: Vec<u32> = Vec::new();

        // Add begin point vertex if needed
        if (breakpoints.is_empty() || breakpoints[0].pos > begin_pos + 1e-10) && begin_pos > NEG_DBL_MAX + 1e10 {
            let point = if is_horizontal {
                Point::new(begin_pos, segment.pos)
            } else {
                Point::new(segment.pos, begin_pos)
            };
            let key = point_key(&point);
            let vertex_id = *point_to_vertex
                .entry(key)
                .or_insert_with(|| graph.add_vertex(point));
            vertex_ids.push(vertex_id);
        }

        // Add vertices from breakpoints
        for bp in &segment.vert_infs {
            let point = if is_horizontal {
                Point::new(bp.pos, segment.pos)
            } else {
                Point::new(segment.pos, bp.pos)
            };
            let key = point_key(&point);
            let vertex_id = *point_to_vertex
                .entry(key)
                .or_insert_with(|| graph.add_vertex(point));
            vertex_ids.push(vertex_id);
        }

        // Add end point vertex if needed
        if (breakpoints.is_empty() || breakpoints.last().map_or(true, |b| b.pos < finish_pos - 1e-10)) && finish_pos < DBL_MAX - 1e10 {
            let point = if is_horizontal {
                Point::new(finish_pos, segment.pos)
            } else {
                Point::new(segment.pos, finish_pos)
            };
            let key = point_key(&point);
            let vertex_id = *point_to_vertex
                .entry(key)
                .or_insert_with(|| graph.add_vertex(point));
            vertex_ids.push(vertex_id);
        }

        // Create bidirectional edges between consecutive vertices
        for window in vertex_ids.windows(2) {
            graph.add_edge(window[0], window[1], true);
            graph.add_edge(window[1], window[0], true);
        }
    }
}

impl Default for OrthogonalVisGraphGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Compute bounding box of a polygon.
fn polygon_bounds(poly: &Polygon) -> (f64, f64, f64, f64) {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for i in 0..poly.size() {
        let p = poly.at(i);
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }

    (min_x, min_y, max_x, max_y)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn rect_obstacle(id: u32, x: f64, y: f64, w: f64, h: f64) -> ObstacleInput {
        let mut poly = Polygon::with_capacity(4);
        poly.push(Point::new(x, y));
        poly.push(Point::new(x + w, y));
        poly.push(Point::new(x + w, y + h));
        poly.push(Point::new(x, y + h));
        ObstacleInput { id, polygon: poly }
    }

    #[test]
    fn test_event_type_ordering() {
        assert!(EventType::Open < EventType::SegOpen);
        assert!(EventType::SegOpen < EventType::ConnPoint);
        assert!(EventType::ConnPoint < EventType::SegClose);
        assert!(EventType::SegClose < EventType::Close);
    }

    #[test]
    fn test_event_sorting() {
        let mut events = vec![
            Event::new(EventType::Close, 0, 10.0),
            Event::new(EventType::Open, 0, 10.0),
            Event::new(EventType::ConnPoint, 1, 10.0),
            Event::new(EventType::Open, 2, 5.0),
        ];
        events.sort();

        assert_eq!(events[0].pos, 5.0);
        assert_eq!(events[1].event_type, EventType::Open);
        assert_eq!(events[2].event_type, EventType::ConnPoint);
        assert_eq!(events[3].event_type, EventType::Close);
    }

    #[test]
    fn test_line_segment_overlap() {
        let seg1 = LineSegment::new(0.0, 100.0, 50.0, false);
        let seg2 = LineSegment::new(50.0, 150.0, 50.0, false);
        let seg3 = LineSegment::new(0.0, 100.0, 60.0, false);

        assert!(seg1.overlaps(&seg2)); // Same pos, overlapping range
        assert!(!seg1.overlaps(&seg3)); // Different pos
    }

    #[test]
    fn test_scanline_insert_remove() {
        let mut nodes = vec![
            ScanlineNode::for_obstacle(0, 10.0, 0.0, 0.0, 20.0, 20.0),
            ScanlineNode::for_obstacle(1, 30.0, 25.0, 0.0, 45.0, 20.0),
            ScanlineNode::for_obstacle(2, 20.0, 15.0, 0.0, 25.0, 20.0),
        ];

        let mut scanline = Scanline::new();

        // Insert nodes
        scanline.insert(&mut nodes, 0);
        scanline.insert(&mut nodes, 1);
        scanline.insert(&mut nodes, 2);

        assert_eq!(scanline.len(), 3);

        // Check neighbor pointers
        assert_eq!(nodes[0].first_below, Some(2)); // 0 -> 2
        assert_eq!(nodes[2].first_above, Some(0)); // 2 <- 0
        assert_eq!(nodes[2].first_below, Some(1)); // 2 -> 1
        assert_eq!(nodes[1].first_above, Some(2)); // 1 <- 2

        // Remove middle node
        scanline.remove(&mut nodes, 2);
        assert_eq!(scanline.len(), 2);
        assert_eq!(nodes[0].first_below, Some(1));
        assert_eq!(nodes[1].first_above, Some(0));
    }

    #[test]
    fn test_simple_visibility_graph() {
        let mut generator = OrthogonalVisGraphGenerator::new();

        let obstacles = vec![rect_obstacle(1, 50.0, 50.0, 50.0, 50.0)];

        let connectors = vec![ConnectorInput::new(
            1,
            Point::new(25.0, 75.0),
            Point::new(125.0, 75.0),
        )];

        let graph = generator.generate(&obstacles, &connectors);

        // Should have vertices for connector endpoints plus obstacle corners
        assert!(graph.vertices().count() >= 2);
    }

    #[test]
    fn test_no_obstacles() {
        let mut generator = OrthogonalVisGraphGenerator::new();

        let connectors = vec![ConnectorInput::new(
            1,
            Point::new(0.0, 0.0),
            Point::new(100.0, 100.0),
        )];

        let graph = generator.generate(&[], &connectors);

        // Should have at least the connector endpoints
        assert!(graph.vertices().count() >= 2);
    }

    #[test]
    fn test_two_obstacles_route_around() {
        let mut generator = OrthogonalVisGraphGenerator::new();

        // Two obstacles side by side with a gap
        let obstacles = vec![
            rect_obstacle(1, 100.0, 50.0, 40.0, 40.0),
            rect_obstacle(2, 160.0, 50.0, 40.0, 40.0),
        ];

        // Connector from left of first to right of second
        let connectors = vec![ConnectorInput::new(
            1,
            Point::new(80.0, 70.0),
            Point::new(220.0, 70.0),
        )];

        let graph = generator.generate(&obstacles, &connectors);

        // Should have multiple vertices for the route
        let vertex_count = graph.vertices().count();
        assert!(vertex_count >= 4, "Expected at least 4 vertices, got {}", vertex_count);
    }

    #[test]
    fn test_route_through_obstacle_blocked() {
        // This test simulates the webdemo example 10 scenario
        // Route: (30, 125) -> (370, 125) with obstacle at (175, 100) size 50x50
        let mut generator = OrthogonalVisGraphGenerator::new();

        // Obstacle bounds: x: 175-225, y: 100-150
        // Route Y=125 passes through the obstacle
        let obstacles = vec![
            rect_obstacle(1, 175.0, 100.0, 50.0, 50.0),
        ];

        let connectors = vec![ConnectorInput::new(
            1,
            Point::new(30.0, 125.0),
            Point::new(370.0, 125.0),
        )];

        let graph = generator.generate(&obstacles, &connectors);

        // Debug: print all vertices and their edges
        eprintln!("\n=== Visibility Graph Debug ===");
        eprintln!("Vertices and edges:");
        for v in graph.vertices() {
            eprintln!("  v{}: ({}, {})", v.id, v.point.x, v.point.y);
            for e in v.all_edges() {
                if let Some(target) = graph.get_vertex(e.target_id) {
                    eprintln!("    -> v{}: ({}, {})", e.target_id, target.point.x, target.point.y);
                }
            }
        }
        eprintln!("=== End Debug ===\n");

        // Check that there's no direct horizontal edge at y=125 that spans through obstacle
        // An edge from x < 175 to x > 225 at y=125 would cross the obstacle
        for v1 in graph.vertices() {
            if (v1.point.y - 125.0).abs() >= 1.0 {
                continue;
            }
            for e in v1.all_edges() {
                if let Some(v2) = graph.get_vertex(e.target_id) {
                    // Check if horizontal edge at y=125
                    if (v2.point.y - 125.0).abs() < 1.0 {
                        let min_x = v1.point.x.min(v2.point.x);
                        let max_x = v1.point.x.max(v2.point.x);

                        // This edge should not span across the obstacle (175-225)
                        if min_x < 175.0 && max_x > 225.0 {
                            panic!("Found edge that crosses obstacle: ({}, {}) -> ({}, {})",
                                v1.point.x, v1.point.y, v2.point.x, v2.point.y);
                        }
                    }
                }
            }
        }
    }
}
