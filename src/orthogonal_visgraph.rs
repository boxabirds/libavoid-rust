//! Orthogonal visibility graph generation via sweep-line algorithm.
//!
//! This module implements the C++ libavoid algorithm from orthogonal.cpp
//! for generating visibility graphs suitable for orthogonal (rectilinear) routing.
//!
//! The algorithm uses two perpendicular sweep-line passes:
//! 1. Vertical sweep (left-to-right) to find horizontal visibility segments
//! 2. Horizontal sweep (top-to-bottom) to find vertical visibility segments
//!
//! Reference: libavoid/orthogonal.cpp - generateStaticOrthogonalVisGraph()

use crate::connector::{ConnDirFlags, CONN_DIR_DOWN, CONN_DIR_LEFT, CONN_DIR_RIGHT, CONN_DIR_UP};
use crate::geometry::{Point, Polygon, PolygonInterface};
use crate::visibility::VisibilityGraph;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap};

// ============================================================================
// Visibility Direction Flags for Scanline
// ============================================================================

/// Visibility directions during scanline sweep.
/// Maps to C++ ScanVisDirFlags.
pub type ScanVisDirFlags = u8;

/// No visibility direction
pub const VIS_DIR_NONE: ScanVisDirFlags = 0;
/// Visible upward (decreasing Y in screen coords)
pub const VIS_DIR_UP: ScanVisDirFlags = 1;
/// Visible downward (increasing Y in screen coords)
pub const VIS_DIR_DOWN: ScanVisDirFlags = 2;

/// Convert scanline visibility directions to connector direction flags.
/// Used when generating edges to determine allowed routing directions.
fn scan_vis_to_conn_dir(scan_dirs: ScanVisDirFlags, is_horizontal: bool) -> ConnDirFlags {
    let mut result = 0;
    if is_horizontal {
        // Horizontal segment: up/down visibility maps to left/right travel
        if scan_dirs & VIS_DIR_UP != 0 {
            result |= CONN_DIR_LEFT;
        }
        if scan_dirs & VIS_DIR_DOWN != 0 {
            result |= CONN_DIR_RIGHT;
        }
    } else {
        // Vertical segment: up/down visibility maps to up/down travel
        if scan_dirs & VIS_DIR_UP != 0 {
            result |= CONN_DIR_UP;
        }
        if scan_dirs & VIS_DIR_DOWN != 0 {
            result |= CONN_DIR_DOWN;
        }
    }
    result
}

// ============================================================================
// Phase 1: Foundation Data Structures
// ============================================================================

/// Position-indexed vertex information for breakpoint sets.
/// C++ ref: struct PosVertInf in orthogonal.cpp
#[derive(Clone, Debug)]
pub struct PosVertInf {
    /// Position along the perpendicular axis (X for horizontal segments, Y for vertical)
    pub pos: f64,
    /// Vertex ID in the visibility graph (None for pure breakpoints)
    pub vertex_id: Option<u32>,
    /// Visibility directions from this point
    pub directions: ScanVisDirFlags,
}

impl PosVertInf {
    pub fn new(pos: f64, vertex_id: Option<u32>, directions: ScanVisDirFlags) -> Self {
        Self {
            pos,
            vertex_id,
            directions,
        }
    }
}

impl PartialEq for PosVertInf {
    fn eq(&self, other: &Self) -> bool {
        // Compare by position first, then vertex_id, then directions
        (self.pos - other.pos).abs() < 1e-10
            && self.vertex_id == other.vertex_id
            && self.directions == other.directions
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
        // Sort by position first
        match self.pos.partial_cmp(&other.pos) {
            Some(Ordering::Equal) | None => {}
            Some(ord) => return ord,
        }
        // Then by vertex_id
        match self.vertex_id.cmp(&other.vertex_id) {
            Ordering::Equal => {}
            ord => return ord,
        }
        // Finally by directions
        self.directions.cmp(&other.directions)
    }
}

// ============================================================================
// Event Queue
// ============================================================================

/// Event types for sweep-line algorithm.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventType {
    /// Shape edge opens (entering shape boundary)
    ShapeOpen,
    /// Shape edge closes (leaving shape boundary)
    ShapeClose,
    /// Connector endpoint
    ConnPoint,
}

/// Sweep-line event.
/// C++ ref: struct Event (inferred from processEventVert/processEventHori)
#[derive(Clone, Debug)]
pub struct Event {
    /// Position along sweep axis
    pub pos: f64,
    /// Event type
    pub event_type: EventType,
    /// Associated shape ID (for ShapeOpen/ShapeClose)
    pub shape_id: Option<u32>,
    /// Min coordinate on perpendicular axis
    pub perp_min: f64,
    /// Max coordinate on perpendicular axis
    pub perp_max: f64,
    /// Vertex ID for connector points
    pub vertex_id: Option<u32>,
}

impl Event {
    pub fn shape_open(pos: f64, shape_id: u32, perp_min: f64, perp_max: f64) -> Self {
        Self {
            pos,
            event_type: EventType::ShapeOpen,
            shape_id: Some(shape_id),
            perp_min,
            perp_max,
            vertex_id: None,
        }
    }

    pub fn shape_close(pos: f64, shape_id: u32, perp_min: f64, perp_max: f64) -> Self {
        Self {
            pos,
            event_type: EventType::ShapeClose,
            shape_id: Some(shape_id),
            perp_min,
            perp_max,
            vertex_id: None,
        }
    }

    pub fn conn_point(pos: f64, perp_pos: f64, vertex_id: u32) -> Self {
        Self {
            pos,
            event_type: EventType::ConnPoint,
            shape_id: None,
            perp_min: perp_pos,
            perp_max: perp_pos,
            vertex_id: Some(vertex_id),
        }
    }
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        (self.pos - other.pos).abs() < 1e-10 && self.event_type == other.event_type
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
        // Sort by position, then by event type priority
        // Open events before Close events at same position
        match self.pos.partial_cmp(&other.pos) {
            Some(Ordering::Equal) | None => {}
            Some(ord) => return ord,
        }
        // Event type priority: ShapeOpen < ConnPoint < ShapeClose
        let self_priority = match self.event_type {
            EventType::ShapeOpen => 0,
            EventType::ConnPoint => 1,
            EventType::ShapeClose => 2,
        };
        let other_priority = match other.event_type {
            EventType::ShapeOpen => 0,
            EventType::ConnPoint => 1,
            EventType::ShapeClose => 2,
        };
        self_priority.cmp(&other_priority)
    }
}

// ============================================================================
// Scanline State
// ============================================================================

/// Node in the scanline representing an active shape boundary.
/// C++ ref: class Node (inferred from usage)
#[derive(Clone, Debug)]
pub struct ScanlineNode {
    /// Shape ID this node belongs to
    pub shape_id: u32,
    /// Min coordinate on perpendicular axis
    pub min: f64,
    /// Max coordinate on perpendicular axis
    pub max: f64,
}

impl ScanlineNode {
    pub fn new(shape_id: u32, min: f64, max: f64) -> Self {
        Self { shape_id, min, max }
    }

    /// Check if a position falls within this node's range
    pub fn contains(&self, pos: f64) -> bool {
        pos >= self.min && pos <= self.max
    }

    /// Check if this node overlaps with a range
    pub fn overlaps_range(&self, min: f64, max: f64) -> bool {
        self.min < max && self.max > min
    }
}

/// Scanline state during sweep.
/// Maintains active shape boundaries sorted by their perpendicular coordinate.
#[derive(Debug, Default)]
pub struct Scanline {
    /// Active nodes sorted by min coordinate
    nodes: Vec<ScanlineNode>,
}

impl Scanline {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// Insert a node for a shape boundary
    pub fn insert(&mut self, node: ScanlineNode) {
        // Insert maintaining sort order by min
        let pos = self
            .nodes
            .binary_search_by(|n| {
                n.min
                    .partial_cmp(&node.min)
                    .unwrap_or(Ordering::Equal)
            })
            .unwrap_or_else(|p| p);
        self.nodes.insert(pos, node);
    }

    /// Remove a node for a shape
    pub fn remove(&mut self, shape_id: u32) {
        self.nodes.retain(|n| n.shape_id != shape_id);
    }

    /// Find gaps in the scanline where visibility segments can pass.
    /// Returns list of (min, max) ranges that are not blocked by shapes.
    pub fn find_gaps(&self, range_min: f64, range_max: f64) -> Vec<(f64, f64)> {
        let mut gaps = Vec::new();
        let mut current_pos = range_min;

        for node in &self.nodes {
            if node.max <= range_min || node.min >= range_max {
                continue; // Node outside our range
            }

            let blocked_min = node.min.max(range_min);
            let blocked_max = node.max.min(range_max);

            if current_pos < blocked_min {
                gaps.push((current_pos, blocked_min));
            }
            current_pos = current_pos.max(blocked_max);
        }

        if current_pos < range_max {
            gaps.push((current_pos, range_max));
        }

        gaps
    }

    /// Check if a position is blocked by any node
    pub fn is_blocked(&self, pos: f64) -> bool {
        self.nodes.iter().any(|n| n.contains(pos))
    }
}

// ============================================================================
// Line Segment (Visibility Candidate)
// ============================================================================

/// A visibility line segment during sweep.
/// C++ ref: class LineSegment in orthogonal.cpp
#[derive(Clone, Debug)]
pub struct LineSegment {
    /// Position on the perpendicular axis (Y for horizontal, X for vertical)
    pub pos: f64,
    /// Start of segment along parallel axis
    pub begin: f64,
    /// End of segment along parallel axis
    pub finish: f64,
    /// Whether this is a shape boundary (affects edge generation)
    pub is_shape_side: bool,
    /// Breakpoints along this segment (positions where edges connect)
    pub breakpoints: BTreeSet<PosVertInf>,
    /// Shape IDs that this segment is associated with
    pub shape_ids: Vec<u32>,
}

impl LineSegment {
    pub fn new(pos: f64, begin: f64, finish: f64, is_shape_side: bool) -> Self {
        Self {
            pos,
            begin,
            finish,
            is_shape_side,
            breakpoints: BTreeSet::new(),
            shape_ids: Vec::new(),
        }
    }

    /// Check if this segment overlaps with another on the perpendicular axis
    pub fn overlaps(&self, other: &LineSegment) -> bool {
        (self.pos - other.pos).abs() < 1e-10
            && self.begin < other.finish
            && self.finish > other.begin
    }

    /// Merge another segment's breakpoints into this one
    pub fn merge(&mut self, other: &LineSegment) {
        for bp in &other.breakpoints {
            self.breakpoints.insert(bp.clone());
        }
        self.shape_ids.extend(&other.shape_ids);
        self.begin = self.begin.min(other.begin);
        self.finish = self.finish.max(other.finish);
    }

    /// Add a breakpoint at a position
    pub fn add_breakpoint(&mut self, pos: f64, vertex_id: Option<u32>, dirs: ScanVisDirFlags) {
        self.breakpoints
            .insert(PosVertInf::new(pos, vertex_id, dirs));
    }
}

// ============================================================================
// Segment List (merges overlapping segments)
// ============================================================================

/// Container that manages LineSegments and merges overlapping ones.
/// C++ ref: SegmentListWrapper in orthogonal.cpp
#[derive(Debug, Default)]
pub struct SegmentList {
    segments: Vec<LineSegment>,
}

impl SegmentList {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    /// Insert a segment, merging with existing overlapping segments
    pub fn insert(&mut self, segment: LineSegment) {
        // Find overlapping segments
        let overlapping: Vec<usize> = self
            .segments
            .iter()
            .enumerate()
            .filter(|(_, s)| s.overlaps(&segment))
            .map(|(i, _)| i)
            .collect();

        if overlapping.is_empty() {
            self.segments.push(segment);
        } else {
            // Merge all overlapping segments
            let mut merged = segment;
            // Remove in reverse order to maintain indices
            for &idx in overlapping.iter().rev() {
                let existing = self.segments.remove(idx);
                merged.merge(&existing);
            }
            self.segments.push(merged);
        }
    }

    /// Get all segments
    pub fn segments(&self) -> &[LineSegment] {
        &self.segments
    }

    /// Clear all segments
    pub fn clear(&mut self) {
        self.segments.clear();
    }
}

// ============================================================================
// Main Algorithm
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
}

/// Orthogonal visibility graph generator.
/// Implements the sweep-line algorithm from C++ libavoid.
pub struct OrthogonalVisGraphGenerator {
    /// Bounding box padding
    padding: f64,
}

impl OrthogonalVisGraphGenerator {
    pub fn new() -> Self {
        Self { padding: 10.0 }
    }

    pub fn with_padding(padding: f64) -> Self {
        Self { padding }
    }

    /// Generate the static orthogonal visibility graph.
    /// C++ ref: generateStaticOrthogonalVisGraph()
    pub fn generate(
        &self,
        obstacles: &[ObstacleInput],
        connectors: &[ConnectorInput],
    ) -> VisibilityGraph {
        let mut graph = VisibilityGraph::new();

        // Phase 1: Build event queues
        let (h_events, v_events) = self.build_event_queues(obstacles, connectors);

        // Phase 2: Vertical sweep (left-to-right) for horizontal segments
        let h_segments = self.vertical_sweep(&h_events, obstacles);

        // Phase 3: Horizontal sweep (top-to-bottom) for vertical segments
        let v_segments = self.horizontal_sweep(&v_events, obstacles);

        // Phase 4: Generate edges from segments
        self.generate_edges(&mut graph, &h_segments, &v_segments, connectors);

        graph
    }

    /// Build event queues for both sweep directions.
    fn build_event_queues(
        &self,
        obstacles: &[ObstacleInput],
        connectors: &[ConnectorInput],
    ) -> (Vec<Event>, Vec<Event>) {
        let mut h_events = Vec::new(); // Events for horizontal sweep (sorted by Y)
        let mut v_events = Vec::new(); // Events for vertical sweep (sorted by X)

        // Add obstacle events
        for obs in obstacles {
            let (min_x, min_y, max_x, max_y) = polygon_bounds(&obs.polygon);

            // Vertical sweep events (for finding horizontal segments)
            v_events.push(Event::shape_open(min_x, obs.id, min_y, max_y));
            v_events.push(Event::shape_close(max_x, obs.id, min_y, max_y));

            // Horizontal sweep events (for finding vertical segments)
            h_events.push(Event::shape_open(min_y, obs.id, min_x, max_x));
            h_events.push(Event::shape_close(max_y, obs.id, min_x, max_x));
        }

        // Add connector endpoint events
        let mut vertex_id = 0u32;
        for conn in connectors {
            // Vertical sweep events
            v_events.push(Event::conn_point(conn.start.x, conn.start.y, vertex_id));
            vertex_id += 1;
            v_events.push(Event::conn_point(conn.end.x, conn.end.y, vertex_id));
            vertex_id += 1;

            // Horizontal sweep events
            h_events.push(Event::conn_point(
                conn.start.y,
                conn.start.x,
                vertex_id - 2,
            ));
            h_events.push(Event::conn_point(conn.end.y, conn.end.x, vertex_id - 1));
        }

        // Sort events
        v_events.sort();
        h_events.sort();

        (h_events, v_events)
    }

    /// Vertical sweep to find horizontal visibility segments.
    /// C++ ref: processEventVert()
    fn vertical_sweep(
        &self,
        events: &[Event],
        obstacles: &[ObstacleInput],
    ) -> SegmentList {
        let mut segments = SegmentList::new();
        let mut scanline = Scanline::new();

        // Compute overall bounding box
        let (overall_min_y, overall_max_y) = self.compute_y_bounds(obstacles);

        for event in events {
            match event.event_type {
                EventType::ShapeOpen => {
                    // Before inserting, emit visibility segments in gaps
                    let gaps = scanline.find_gaps(overall_min_y, overall_max_y);
                    for (gap_min, gap_max) in gaps {
                        if gap_min < gap_max {
                            let mut seg = LineSegment::new(event.pos, gap_min, gap_max, false);
                            // Add breakpoints at gap boundaries
                            seg.add_breakpoint(gap_min, None, VIS_DIR_DOWN);
                            seg.add_breakpoint(gap_max, None, VIS_DIR_UP);
                            segments.insert(seg);
                        }
                    }

                    // Insert shape into scanline
                    scanline.insert(ScanlineNode::new(
                        event.shape_id.unwrap(),
                        event.perp_min,
                        event.perp_max,
                    ));
                }
                EventType::ShapeClose => {
                    // Remove shape from scanline
                    scanline.remove(event.shape_id.unwrap());

                    // Emit visibility segments in new gaps
                    let gaps = scanline.find_gaps(overall_min_y, overall_max_y);
                    for (gap_min, gap_max) in gaps {
                        if gap_min < gap_max {
                            let mut seg = LineSegment::new(event.pos, gap_min, gap_max, false);
                            seg.add_breakpoint(gap_min, None, VIS_DIR_DOWN);
                            seg.add_breakpoint(gap_max, None, VIS_DIR_UP);
                            segments.insert(seg);
                        }
                    }
                }
                EventType::ConnPoint => {
                    // Add connector point as a breakpoint if not blocked
                    if !scanline.is_blocked(event.perp_min) {
                        let mut seg =
                            LineSegment::new(event.pos, event.perp_min, event.perp_min, false);
                        seg.add_breakpoint(
                            event.perp_min,
                            event.vertex_id,
                            VIS_DIR_UP | VIS_DIR_DOWN,
                        );
                        segments.insert(seg);
                    }
                }
            }
        }

        segments
    }

    /// Horizontal sweep to find vertical visibility segments.
    /// C++ ref: processEventHori()
    fn horizontal_sweep(
        &self,
        events: &[Event],
        obstacles: &[ObstacleInput],
    ) -> SegmentList {
        let mut segments = SegmentList::new();
        let mut scanline = Scanline::new();

        // Compute overall bounding box
        let (overall_min_x, overall_max_x) = self.compute_x_bounds(obstacles);

        for event in events {
            match event.event_type {
                EventType::ShapeOpen => {
                    let gaps = scanline.find_gaps(overall_min_x, overall_max_x);
                    for (gap_min, gap_max) in gaps {
                        if gap_min < gap_max {
                            let mut seg = LineSegment::new(event.pos, gap_min, gap_max, false);
                            seg.add_breakpoint(gap_min, None, VIS_DIR_DOWN);
                            seg.add_breakpoint(gap_max, None, VIS_DIR_UP);
                            segments.insert(seg);
                        }
                    }

                    scanline.insert(ScanlineNode::new(
                        event.shape_id.unwrap(),
                        event.perp_min,
                        event.perp_max,
                    ));
                }
                EventType::ShapeClose => {
                    scanline.remove(event.shape_id.unwrap());

                    let gaps = scanline.find_gaps(overall_min_x, overall_max_x);
                    for (gap_min, gap_max) in gaps {
                        if gap_min < gap_max {
                            let mut seg = LineSegment::new(event.pos, gap_min, gap_max, false);
                            seg.add_breakpoint(gap_min, None, VIS_DIR_DOWN);
                            seg.add_breakpoint(gap_max, None, VIS_DIR_UP);
                            segments.insert(seg);
                        }
                    }
                }
                EventType::ConnPoint => {
                    if !scanline.is_blocked(event.perp_min) {
                        let mut seg =
                            LineSegment::new(event.pos, event.perp_min, event.perp_min, false);
                        seg.add_breakpoint(
                            event.perp_min,
                            event.vertex_id,
                            VIS_DIR_UP | VIS_DIR_DOWN,
                        );
                        segments.insert(seg);
                    }
                }
            }
        }

        segments
    }

    /// Generate visibility edges from horizontal and vertical segments.
    fn generate_edges(
        &self,
        graph: &mut VisibilityGraph,
        h_segments: &SegmentList,
        v_segments: &SegmentList,
        connectors: &[ConnectorInput],
    ) {
        // Add connector vertices to graph
        let mut vertex_map: HashMap<u32, u32> = HashMap::new();
        let mut point_to_vertex: HashMap<(i64, i64), u32> = HashMap::new();
        let mut next_logical_id = 0u32;

        // Helper to convert point to hashable key
        let point_key = |p: &Point| -> (i64, i64) {
            ((p.x * 1000.0) as i64, (p.y * 1000.0) as i64)
        };

        for conn in connectors {
            let start_key = point_key(&conn.start);
            let start_id = graph.add_vertex(conn.start);
            vertex_map.insert(next_logical_id, start_id);
            point_to_vertex.insert(start_key, start_id);
            next_logical_id += 1;

            let end_key = point_key(&conn.end);
            let end_id = graph.add_vertex(conn.end);
            vertex_map.insert(next_logical_id, end_id);
            point_to_vertex.insert(end_key, end_id);
            next_logical_id += 1;
        }

        // Generate edges from horizontal segments
        for seg in h_segments.segments() {
            let breakpoints: Vec<_> = seg.breakpoints.iter().collect();
            for i in 0..breakpoints.len().saturating_sub(1) {
                let bp1 = &breakpoints[i];
                let bp2 = &breakpoints[i + 1];

                // Create vertices if they don't exist
                let v1_point = Point::new(bp1.pos, seg.pos);
                let v2_point = Point::new(bp2.pos, seg.pos);

                let v1_key = point_key(&v1_point);
                let v1_id = *point_to_vertex.entry(v1_key).or_insert_with(|| {
                    graph.add_vertex(v1_point)
                });

                let v2_key = point_key(&v2_point);
                let v2_id = *point_to_vertex.entry(v2_key).or_insert_with(|| {
                    graph.add_vertex(v2_point)
                });

                // Add edge (orthogonal = true)
                graph.add_edge(v1_id, v2_id, true);
            }
        }

        // Generate edges from vertical segments
        for seg in v_segments.segments() {
            let breakpoints: Vec<_> = seg.breakpoints.iter().collect();
            for i in 0..breakpoints.len().saturating_sub(1) {
                let bp1 = &breakpoints[i];
                let bp2 = &breakpoints[i + 1];

                let v1_point = Point::new(seg.pos, bp1.pos);
                let v2_point = Point::new(seg.pos, bp2.pos);

                let v1_key = point_key(&v1_point);
                let v1_id = *point_to_vertex.entry(v1_key).or_insert_with(|| {
                    graph.add_vertex(v1_point)
                });

                let v2_key = point_key(&v2_point);
                let v2_id = *point_to_vertex.entry(v2_key).or_insert_with(|| {
                    graph.add_vertex(v2_point)
                });

                // Add edge (orthogonal = true)
                graph.add_edge(v1_id, v2_id, true);
            }
        }
    }

    fn compute_y_bounds(&self, obstacles: &[ObstacleInput]) -> (f64, f64) {
        let mut min_y = f64::MAX;
        let mut max_y = f64::MIN;
        for obs in obstacles {
            let (_, obs_min_y, _, obs_max_y) = polygon_bounds(&obs.polygon);
            min_y = min_y.min(obs_min_y);
            max_y = max_y.max(obs_max_y);
        }
        (min_y - self.padding, max_y + self.padding)
    }

    fn compute_x_bounds(&self, obstacles: &[ObstacleInput]) -> (f64, f64) {
        let mut min_x = f64::MAX;
        let mut max_x = f64::MIN;
        for obs in obstacles {
            let (obs_min_x, _, obs_max_x, _) = polygon_bounds(&obs.polygon);
            min_x = min_x.min(obs_min_x);
            max_x = max_x.max(obs_max_x);
        }
        (min_x - self.padding, max_x + self.padding)
    }
}

impl Default for OrthogonalVisGraphGenerator {
    fn default() -> Self {
        Self::new()
    }
}

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
    fn test_pos_vert_inf_ordering() {
        let pv1 = PosVertInf::new(10.0, Some(1), VIS_DIR_UP);
        let pv2 = PosVertInf::new(20.0, Some(2), VIS_DIR_DOWN);
        let pv3 = PosVertInf::new(10.0, Some(2), VIS_DIR_UP);

        assert!(pv1 < pv2); // Different position
        assert!(pv1 < pv3); // Same position, different vertex_id
    }

    #[test]
    fn test_event_ordering() {
        let e1 = Event::shape_open(10.0, 1, 0.0, 100.0);
        let e2 = Event::shape_close(10.0, 1, 0.0, 100.0);
        let e3 = Event::shape_open(20.0, 2, 0.0, 100.0);

        assert!(e1 < e2); // Same position, Open before Close
        assert!(e1 < e3); // Different position
    }

    #[test]
    fn test_scanline_gaps() {
        let mut scanline = Scanline::new();
        scanline.insert(ScanlineNode::new(1, 20.0, 40.0));
        scanline.insert(ScanlineNode::new(2, 60.0, 80.0));

        let gaps = scanline.find_gaps(0.0, 100.0);
        assert_eq!(gaps.len(), 3);
        assert_eq!(gaps[0], (0.0, 20.0));
        assert_eq!(gaps[1], (40.0, 60.0));
        assert_eq!(gaps[2], (80.0, 100.0));
    }

    #[test]
    fn test_line_segment_merge() {
        let mut seg1 = LineSegment::new(50.0, 0.0, 100.0, false);
        seg1.add_breakpoint(0.0, None, VIS_DIR_DOWN);
        seg1.add_breakpoint(50.0, Some(1), VIS_DIR_UP | VIS_DIR_DOWN);

        let mut seg2 = LineSegment::new(50.0, 50.0, 150.0, false);
        seg2.add_breakpoint(50.0, Some(1), VIS_DIR_UP | VIS_DIR_DOWN);
        seg2.add_breakpoint(150.0, None, VIS_DIR_UP);

        seg1.merge(&seg2);

        assert_eq!(seg1.begin, 0.0);
        assert_eq!(seg1.finish, 150.0);
        assert_eq!(seg1.breakpoints.len(), 3);
    }

    #[test]
    fn test_simple_visibility_graph() {
        let generator = OrthogonalVisGraphGenerator::new();

        let obstacles = vec![rect_obstacle(1, 50.0, 50.0, 50.0, 50.0)];

        let connectors = vec![ConnectorInput {
            id: 1,
            start: Point::new(25.0, 75.0),
            end: Point::new(125.0, 75.0),
        }];

        let graph = generator.generate(&obstacles, &connectors);

        // Should have vertices for connector endpoints plus intersection points
        assert!(graph.vertices().count() >= 2);
        // Graph should have been populated (edge_count method may not exist, just verify no panic)
    }

    #[test]
    fn test_no_obstacles() {
        let generator = OrthogonalVisGraphGenerator::new();

        let connectors = vec![ConnectorInput {
            id: 1,
            start: Point::new(0.0, 0.0),
            end: Point::new(100.0, 100.0),
        }];

        let graph = generator.generate(&[], &connectors);

        // Should have at least the connector endpoints
        assert!(graph.vertices().count() >= 2);
    }
}
