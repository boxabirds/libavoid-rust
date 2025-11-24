//! Geometric types and operations for libavoid
//!
//! This module provides the core geometric primitives used throughout libavoid,
//! including points, boxes, polygons, and edges.

use std::ops::{Add, Sub, Index};

/// A 2D point with x and y coordinates
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub id: u32,
    pub vn: u32,
}

impl Point {
    /// Creates a new point with the given coordinates
    pub fn new(x: f64, y: f64) -> Self {
        Point {
            x,
            y,
            id: 0,
            vn: crate::UNASSIGNED_VERTEX_NUMBER,
        }
    }

    /// Creates a new point with coordinates and ID
    pub fn with_id(x: f64, y: f64, id: u32) -> Self {
        Point {
            x,
            y,
            id,
            vn: crate::UNASSIGNED_VERTEX_NUMBER,
        }
    }

    /// Checks if two points are equal within epsilon tolerance
    pub fn equals(&self, other: &Point) -> bool {
        const EPSILON: f64 = 1e-10;
        (self.x - other.x).abs() < EPSILON && (self.y - other.y).abs() < EPSILON
    }

    /// Returns the distance between two points
    pub fn distance(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Returns the squared distance (faster when actual distance not needed)
    pub fn distance_squared(&self, other: &Point) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }
}

impl Default for Point {
    fn default() -> Self {
        Point::new(0.0, 0.0)
    }
}

impl Add for Point {
    type Output = Point;

    fn add(self, other: Point) -> Point {
        Point::new(self.x + other.x, self.y + other.y)
    }
}

impl Sub for Point {
    type Output = Point;

    fn sub(self, other: Point) -> Point {
        Point::new(self.x - other.x, self.y - other.y)
    }
}

impl Index<usize> for Point {
    type Output = f64;

    fn index(&self, index: usize) -> &f64 {
        match index {
            0 => &self.x,
            1 => &self.y,
            _ => panic!("Point index out of bounds: {}", index),
        }
    }
}

impl PartialOrd for Point {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.x != other.x {
            self.x.partial_cmp(&other.x)
        } else {
            self.y.partial_cmp(&other.y)
        }
    }
}

/// Type alias for Vector (same as Point)
pub type Vector = Point;

/// An axis-aligned bounding box
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Box {
    pub min: Point,
    pub max: Point,
}

impl Box {
    /// Creates a new box from two corner points
    pub fn new(min: Point, max: Point) -> Self {
        Box { min, max }
    }

    /// Creates a box from coordinates
    pub fn from_coords(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Box {
            min: Point::new(x1.min(x2), y1.min(y2)),
            max: Point::new(x1.max(x2), y1.max(y2)),
        }
    }

    /// Returns the width of the box
    pub fn width(&self) -> f64 {
        self.max.x - self.min.x
    }

    /// Returns the height of the box
    pub fn height(&self) -> f64 {
        self.max.y - self.min.y
    }

    /// Returns the length (max of width and height)
    pub fn length(&self) -> f64 {
        self.width().max(self.height())
    }

    /// Checks if the box contains a point
    pub fn contains(&self, point: &Point) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Checks if this box intersects another box
    pub fn intersects(&self, other: &Box) -> bool {
        !(self.max.x < other.min.x
            || self.min.x > other.max.x
            || self.max.y < other.min.y
            || self.min.y > other.max.y)
    }
}

/// Trait for polygon-like objects
pub trait PolygonInterface {
    /// Clears all points from the polygon
    fn clear(&mut self);

    /// Returns true if the polygon is empty
    fn empty(&self) -> bool;

    /// Returns the number of points in the polygon
    fn size(&self) -> usize;

    /// Returns the polygon's ID
    fn id(&self) -> u32;

    /// Returns a reference to the point at the given index
    fn at(&self, index: usize) -> &Point;

    /// Returns the bounding rectangle of the polygon
    fn bounding_rect(&self) -> Box;

    /// Returns a polygon offset by the given amount
    fn offset_polygon(&self, offset: f64) -> Polygon;
}

/// A polygon defined by a sequence of points
#[derive(Debug, Clone)]
pub struct Polygon {
    pub id: u32,
    pub ps: Vec<Point>,
    pub ts: Vec<char>,
    pub checkpoints_on_route: usize,
}

impl Polygon {
    /// Creates an empty polygon
    pub fn new() -> Self {
        Polygon {
            id: 0,
            ps: Vec::new(),
            ts: Vec::new(),
            checkpoints_on_route: 0,
        }
    }

    /// Creates a polygon with the given capacity
    pub fn with_capacity(n: usize) -> Self {
        Polygon {
            id: 0,
            ps: Vec::with_capacity(n),
            ts: Vec::with_capacity(n),
            checkpoints_on_route: 0,
        }
    }

    /// Creates a polygon with an ID
    pub fn with_id(id: u32) -> Self {
        Polygon {
            id,
            ps: Vec::new(),
            ts: Vec::new(),
            checkpoints_on_route: 0,
        }
    }

    /// Adds a point to the polygon
    pub fn push(&mut self, point: Point) {
        self.ps.push(point);
    }

    /// Sets a point at the given index
    pub fn set_point(&mut self, index: usize, point: Point) {
        if index < self.ps.len() {
            self.ps[index] = point;
        }
    }

    /// Translates the polygon by the given offset
    pub fn translate(&mut self, offset: Point) {
        for p in &mut self.ps {
            *p = *p + offset;
        }
    }

    /// Returns an iterator over the points
    pub fn points(&self) -> impl Iterator<Item = &Point> {
        self.ps.iter()
    }

    /// Returns a mutable iterator over the points
    pub fn points_mut(&mut self) -> impl Iterator<Item = &mut Point> {
        self.ps.iter_mut()
    }

    /// Simplifies the polygon by removing collinear points
    pub fn simplify(&mut self) {
        if self.ps.len() < 3 {
            return;
        }

        let mut simplified = Vec::new();
        simplified.push(self.ps[0]);

        for i in 1..self.ps.len() - 1 {
            let p1 = self.ps[i - 1];
            let p2 = self.ps[i];
            let p3 = self.ps[i + 1];

            // Check if points are collinear
            let cross = (p2.x - p1.x) * (p3.y - p1.y) - (p2.y - p1.y) * (p3.x - p1.x);
            if cross.abs() > 1e-10 {
                simplified.push(p2);
            }
        }

        simplified.push(self.ps[self.ps.len() - 1]);
        self.ps = simplified;
    }
}

impl Default for Polygon {
    fn default() -> Self {
        Polygon::new()
    }
}

impl PolygonInterface for Polygon {
    fn clear(&mut self) {
        self.ps.clear();
        self.ts.clear();
    }

    fn empty(&self) -> bool {
        self.ps.is_empty()
    }

    fn size(&self) -> usize {
        self.ps.len()
    }

    fn id(&self) -> u32 {
        self.id
    }

    fn at(&self, index: usize) -> &Point {
        &self.ps[index]
    }

    fn bounding_rect(&self) -> Box {
        if self.ps.is_empty() {
            return Box::new(Point::new(0.0, 0.0), Point::new(0.0, 0.0));
        }

        let mut min_x = self.ps[0].x;
        let mut min_y = self.ps[0].y;
        let mut max_x = self.ps[0].x;
        let mut max_y = self.ps[0].y;

        for p in &self.ps {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        Box::new(Point::new(min_x, min_y), Point::new(max_x, max_y))
    }

    fn offset_polygon(&self, offset: f64) -> Polygon {
        // Proper polygon offsetting using edge normals
        let mut result = Polygon::with_id(self.id);

        let n = self.ps.len();
        if n < 3 {
            // For degenerate polygons, just return a copy
            return self.clone();
        }

        // For each vertex, compute the offset based on the normals of adjacent edges
        for i in 0..n {
            let prev_i = if i == 0 { n - 1 } else { i - 1 };
            let next_i = if i == n - 1 { 0 } else { i + 1 };

            let prev = &self.ps[prev_i];
            let curr = &self.ps[i];
            let next = &self.ps[next_i];

            // Compute edge vectors
            let edge1_x = curr.x - prev.x;
            let edge1_y = curr.y - prev.y;
            let edge2_x = next.x - curr.x;
            let edge2_y = next.y - curr.y;

            // Compute perpendicular normals (rotate 90° clockwise for outward)
            // For a typical clockwise wound polygon, outward is to the right
            let mut normal1_x = edge1_y;
            let mut normal1_y = -edge1_x;
            let mut normal2_x = edge2_y;
            let mut normal2_y = -edge2_x;

            // Normalize the normals
            let len1 = (normal1_x * normal1_x + normal1_y * normal1_y).sqrt();
            if len1 > 1e-10 {
                normal1_x /= len1;
                normal1_y /= len1;
            }

            let len2 = (normal2_x * normal2_x + normal2_y * normal2_y).sqrt();
            if len2 > 1e-10 {
                normal2_x /= len2;
                normal2_y /= len2;
            }

            // Average the normals
            let avg_normal_x = (normal1_x + normal2_x) * 0.5;
            let avg_normal_y = (normal1_y + normal2_y) * 0.5;

            // Normalize the averaged normal
            let avg_len = (avg_normal_x * avg_normal_x + avg_normal_y * avg_normal_y).sqrt();
            let (final_normal_x, final_normal_y) = if avg_len > 1e-10 {
                (avg_normal_x / avg_len, avg_normal_y / avg_len)
            } else {
                (normal1_x, normal1_y)
            };

            // Offset the vertex
            let new_point = Point::new(
                curr.x + final_normal_x * offset,
                curr.y + final_normal_y * offset,
            );
            result.push(new_point);
        }

        result
    }
}

/// Type alias for PolyLine
pub type PolyLine = Polygon;

/// A rectangle defined by two corner points
#[derive(Debug, Clone)]
pub struct Rectangle {
    polygon: Polygon,
}

impl Rectangle {
    /// Creates a rectangle from two corner points
    pub fn new_from_points(p1: Point, p2: Point) -> Self {
        let mut polygon = Polygon::with_capacity(4);

        let min_x = p1.x.min(p2.x);
        let min_y = p1.y.min(p2.y);
        let max_x = p1.x.max(p2.x);
        let max_y = p1.y.max(p2.y);

        polygon.push(Point::new(min_x, min_y));
        polygon.push(Point::new(max_x, min_y));
        polygon.push(Point::new(max_x, max_y));
        polygon.push(Point::new(min_x, max_y));

        Rectangle { polygon }
    }

    /// Creates a rectangle from center point, width, and height
    pub fn new(center: Point, width: f64, height: f64) -> Self {
        let half_width = width / 2.0;
        let half_height = height / 2.0;

        let mut polygon = Polygon::with_capacity(4);
        polygon.push(Point::new(center.x - half_width, center.y - half_height));
        polygon.push(Point::new(center.x + half_width, center.y - half_height));
        polygon.push(Point::new(center.x + half_width, center.y + half_height));
        polygon.push(Point::new(center.x - half_width, center.y + half_height));

        Rectangle { polygon }
    }

    /// Returns the width of the rectangle
    pub fn width(&self) -> f64 {
        (self.polygon.ps[1].x - self.polygon.ps[0].x).abs()
    }

    /// Returns the height of the rectangle
    pub fn height(&self) -> f64 {
        (self.polygon.ps[2].y - self.polygon.ps[1].y).abs()
    }

    /// Returns the center point of the rectangle
    pub fn center(&self) -> Point {
        let min_x = self.polygon.ps[0].x;
        let min_y = self.polygon.ps[0].y;
        let max_x = self.polygon.ps[2].x;
        let max_y = self.polygon.ps[2].y;

        Point::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0)
    }
}

impl From<Rectangle> for Polygon {
    fn from(rect: Rectangle) -> Polygon {
        rect.polygon
    }
}

/// An edge between two points
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Edge {
    pub a: Point,
    pub b: Point,
}

impl Edge {
    /// Creates a new edge
    pub fn new(a: Point, b: Point) -> Self {
        Edge { a, b }
    }

    /// Returns the length of the edge
    pub fn length(&self) -> f64 {
        self.a.distance(&self.b)
    }

    /// Returns the squared length (faster when actual length not needed)
    pub fn length_squared(&self) -> f64 {
        self.a.distance_squared(&self.b)
    }
}

// ============================================================================
// Geometry utility functions
// ============================================================================

/// Tolerance for floating point comparisons
pub const EPSILON: f64 = 1e-10;

/// Computes the counter-clockwise orientation of three points.
/// Returns positive if CCW (left turn), negative if CW (right turn), zero if collinear.
pub fn ccw(a: &Point, b: &Point, c: &Point) -> f64 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

/// Checks if points are collinear (within epsilon tolerance)
pub fn collinear(a: &Point, b: &Point, c: &Point) -> bool {
    ccw(a, b, c).abs() < EPSILON
}

/// Checks if point q lies on segment pr (assumes collinearity)
pub fn on_segment(p: &Point, q: &Point, r: &Point) -> bool {
    q.x >= p.x.min(r.x) - EPSILON
        && q.x <= p.x.max(r.x) + EPSILON
        && q.y >= p.y.min(r.y) - EPSILON
        && q.y <= p.y.max(r.y) + EPSILON
}

/// Checks if two line segments intersect.
/// Uses the CCW orientation test for robustness.
/// Returns true only if segments properly cross or overlap.
pub fn segments_intersect(a1: &Point, a2: &Point, b1: &Point, b2: &Point) -> bool {
    let ccw1 = ccw(a1, a2, b1);
    let ccw2 = ccw(a1, a2, b2);
    let ccw3 = ccw(b1, b2, a1);
    let ccw4 = ccw(b1, b2, a2);

    // Standard crossing case: segments straddle each other
    if ccw1 * ccw2 < 0.0 && ccw3 * ccw4 < 0.0 {
        return true;
    }

    // Collinear cases - check if one endpoint lies on the other segment
    if ccw1.abs() < EPSILON && on_segment(a1, b1, a2) {
        return true;
    }
    if ccw2.abs() < EPSILON && on_segment(a1, b2, a2) {
        return true;
    }
    if ccw3.abs() < EPSILON && on_segment(b1, a1, b2) {
        return true;
    }
    if ccw4.abs() < EPSILON && on_segment(b1, a2, b2) {
        return true;
    }

    false
}

/// Checks if two line segments intersect, excluding shared endpoints.
/// This is used for visibility tests where touching endpoints is allowed.
pub fn segments_intersect_excluding_endpoints(
    a1: &Point,
    a2: &Point,
    b1: &Point,
    b2: &Point,
) -> bool {
    // Check if any endpoints are the same (shared endpoint case)
    if a1.equals(b1) || a1.equals(b2) || a2.equals(b1) || a2.equals(b2) {
        // For shared endpoint, check if segments overlap beyond the shared point
        if a1.equals(b1) {
            // Segments share a1/b1, check if they overlap
            if collinear(a1, a2, b2) && (on_segment(a1, b2, a2) || on_segment(b1, a2, b2)) {
                // b2 is on segment a1-a2, or a2 is on segment b1-b2
                // This means overlap beyond the shared point
                return !a2.equals(b2); // Only intersect if they're different segments
            }
            return false;
        }
        if a1.equals(b2) {
            if collinear(a1, a2, b1) && (on_segment(a1, b1, a2) || on_segment(b2, a2, b1)) {
                return !a2.equals(b1);
            }
            return false;
        }
        if a2.equals(b1) {
            if collinear(a2, a1, b2) && (on_segment(a2, b2, a1) || on_segment(b1, a1, b2)) {
                return !a1.equals(b2);
            }
            return false;
        }
        if a2.equals(b2) {
            if collinear(a2, a1, b1) && (on_segment(a2, b1, a1) || on_segment(b2, a1, b1)) {
                return !a1.equals(b1);
            }
            return false;
        }
    }

    // No shared endpoints - use standard intersection test
    segments_intersect(a1, a2, b1, b2)
}

/// Checks if a point is inside a polygon using ray casting algorithm.
/// Returns true if the point is strictly inside (not on boundary).
pub fn point_in_polygon(point: &Point, polygon: &Polygon) -> bool {
    let n = polygon.size();
    if n < 3 {
        return false;
    }

    let mut inside = false;
    let mut j = n - 1;

    for i in 0..n {
        let pi = polygon.at(i);
        let pj = polygon.at(j);

        // Check if point is on an edge
        if collinear(pi, point, pj) && on_segment(pi, point, pj) {
            return false; // On boundary, not inside
        }

        if ((pi.y > point.y) != (pj.y > point.y))
            && (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x)
        {
            inside = !inside;
        }

        j = i;
    }

    inside
}

/// Checks if a line segment intersects a polygon.
/// Returns true if the segment crosses any edge of the polygon OR
/// if either endpoint is inside the polygon.
pub fn segment_intersects_polygon(p1: &Point, p2: &Point, polygon: &Polygon) -> bool {
    let n = polygon.size();
    if n < 3 {
        return false;
    }

    // Check if segment intersects any polygon edge
    for i in 0..n {
        let j = (i + 1) % n;
        let poly_p1 = polygon.at(i);
        let poly_p2 = polygon.at(j);

        if segments_intersect_excluding_endpoints(p1, p2, poly_p1, poly_p2) {
            return true;
        }
    }

    // Check if the midpoint of the segment is inside the polygon
    // (handles case where segment is entirely inside)
    let mid = Point::new((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0);
    if point_in_polygon(&mid, polygon) {
        return true;
    }

    // Also check both endpoints
    if point_in_polygon(p1, polygon) || point_in_polygon(p2, polygon) {
        return true;
    }

    false
}

/// Checks if a line segment passes through a polygon, excluding cases where
/// the segment merely touches a vertex of the polygon (for visibility tests).
pub fn segment_intersects_polygon_interior(p1: &Point, p2: &Point, polygon: &Polygon) -> bool {
    let n = polygon.size();
    if n < 3 {
        return false;
    }

    // Check if either endpoint is strictly inside the polygon
    if point_in_polygon(p1, polygon) || point_in_polygon(p2, polygon) {
        return true;
    }

    // Check if segment properly crosses any polygon edge
    for i in 0..n {
        let j = (i + 1) % n;
        let poly_p1 = polygon.at(i);
        let poly_p2 = polygon.at(j);

        // Skip if segment endpoint is a polygon vertex
        if p1.equals(poly_p1) || p1.equals(poly_p2) || p2.equals(poly_p1) || p2.equals(poly_p2) {
            continue;
        }

        // Check for proper crossing (not just touching)
        let ccw1 = ccw(p1, p2, poly_p1);
        let ccw2 = ccw(p1, p2, poly_p2);
        let ccw3 = ccw(poly_p1, poly_p2, p1);
        let ccw4 = ccw(poly_p1, poly_p2, p2);

        // Proper crossing: opposite signs on both tests
        if ccw1 * ccw2 < 0.0 && ccw3 * ccw4 < 0.0 {
            return true;
        }
    }

    // Check midpoint is inside (segment entirely within polygon)
    let mid = Point::new((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0);
    point_in_polygon(&mid, polygon)
}

// ============================================================================
// Connector Crossing Detection
// ============================================================================

/// Result of a crossing detection between two polyline routes
#[derive(Debug, Clone)]
pub struct CrossingInfo {
    /// Index of the segment in the first route where crossing occurs
    pub route1_segment: usize,
    /// Index of the segment in the second route where crossing occurs
    pub route2_segment: usize,
    /// The point where the crossing occurs
    pub crossing_point: Point,
}

/// Counts the number of crossings between two polyline routes.
/// A crossing occurs when segments from different routes intersect
/// at a point that is not a shared endpoint.
pub fn count_route_crossings(route1: &Polygon, route2: &Polygon) -> usize {
    let mut count = 0;

    if route1.size() < 2 || route2.size() < 2 {
        return 0;
    }

    for i in 0..route1.size() - 1 {
        let a1 = route1.at(i);
        let a2 = route1.at(i + 1);

        for j in 0..route2.size() - 1 {
            let b1 = route2.at(j);
            let b2 = route2.at(j + 1);

            if segments_intersect_proper(a1, a2, b1, b2) {
                count += 1;
            }
        }
    }

    count
}

/// Finds all crossings between two polyline routes.
pub fn find_route_crossings(route1: &Polygon, route2: &Polygon) -> Vec<CrossingInfo> {
    let mut crossings = Vec::new();

    if route1.size() < 2 || route2.size() < 2 {
        return crossings;
    }

    for i in 0..route1.size() - 1 {
        let a1 = route1.at(i);
        let a2 = route1.at(i + 1);

        for j in 0..route2.size() - 1 {
            let b1 = route2.at(j);
            let b2 = route2.at(j + 1);

            if let Some(point) = segment_intersection_point(a1, a2, b1, b2) {
                // Only count proper crossings, not shared endpoints
                if !point.equals(a1) && !point.equals(a2) && !point.equals(b1) && !point.equals(b2) {
                    crossings.push(CrossingInfo {
                        route1_segment: i,
                        route2_segment: j,
                        crossing_point: point,
                    });
                }
            }
        }
    }

    crossings
}

/// Checks if two segments have a proper intersection (cross each other,
/// not just touching at endpoints).
pub fn segments_intersect_proper(a1: &Point, a2: &Point, b1: &Point, b2: &Point) -> bool {
    // Skip if segments share an endpoint
    if a1.equals(b1) || a1.equals(b2) || a2.equals(b1) || a2.equals(b2) {
        return false;
    }

    let d1 = ccw(a1, a2, b1);
    let d2 = ccw(a1, a2, b2);
    let d3 = ccw(b1, b2, a1);
    let d4 = ccw(b1, b2, a2);

    // Proper crossing requires opposite signs on both sides
    if d1 * d2 < 0.0 && d3 * d4 < 0.0 {
        return true;
    }

    false
}

/// Computes the intersection point of two line segments, if they intersect.
pub fn segment_intersection_point(a1: &Point, a2: &Point, b1: &Point, b2: &Point) -> Option<Point> {
    let d1 = ccw(a1, a2, b1);
    let d2 = ccw(a1, a2, b2);
    let d3 = ccw(b1, b2, a1);
    let d4 = ccw(b1, b2, a2);

    // Check if segments intersect
    if !((d1 * d2 < 0.0 && d3 * d4 < 0.0) ||
         (d1 == 0.0 && on_segment(a1, b1, a2)) ||
         (d2 == 0.0 && on_segment(a1, b2, a2)) ||
         (d3 == 0.0 && on_segment(b1, a1, b2)) ||
         (d4 == 0.0 && on_segment(b1, a2, b2))) {
        return None;
    }

    // Compute intersection point using parametric form
    let dx1 = a2.x - a1.x;
    let dy1 = a2.y - a1.y;
    let dx2 = b2.x - b1.x;
    let dy2 = b2.y - b1.y;

    let denom = dx1 * dy2 - dy1 * dx2;

    if denom.abs() < 1e-10 {
        // Parallel or collinear - return midpoint of overlap if any
        return None;
    }

    let dx3 = b1.x - a1.x;
    let dy3 = b1.y - a1.y;

    let t = (dx3 * dy2 - dy3 * dx2) / denom;

    Some(Point::new(a1.x + t * dx1, a1.y + t * dy1))
}

/// Counts total crossings of a connector route with all other routes.
pub fn count_connector_crossings(route: &Polygon, other_routes: &[&Polygon]) -> usize {
    let mut total = 0;
    for other in other_routes {
        total += count_route_crossings(route, other);
    }
    total
}

// ============================================================================
// Sweep-line visibility algorithm helpers
// ============================================================================

/// Returns the rotational angle (0-360 degrees) of a point from the origin.
/// Used for sweep-line visibility algorithm.
pub fn rotational_angle(p: &Point) -> f64 {
    use std::f64::consts::PI;

    if p.y == 0.0 {
        return if p.x < 0.0 { 180.0 } else { 0.0 };
    } else if p.x == 0.0 {
        return if p.y < 0.0 { 270.0 } else { 90.0 };
    }

    let mut ang = (p.y / p.x).atan();
    ang = ang * 180.0 / PI;

    if p.x < 0.0 {
        ang += 180.0;
    } else if p.y < 0.0 {
        ang += 360.0;
    }

    debug_assert!(ang >= 0.0 && ang <= 360.0);
    ang
}

/// Direction constant: point is ahead (counter-clockwise) of the line
pub const VEC_DIR_AHEAD: i32 = 1;
/// Direction constant: point is behind (clockwise) of the line
pub const VEC_DIR_BEHIND: i32 = -1;
/// Direction constant: point is collinear with the line
pub const VEC_DIR_COLLINEAR: i32 = 0;

/// Returns the direction of point c relative to line ab.
/// Returns 1 (AHEAD/CCW), -1 (BEHIND/CW), or 0 (collinear).
/// This is the sign of the 2D cross product (b-a) × (c-a).
pub fn vec_dir(a: &Point, b: &Point, c: &Point) -> i32 {
    vec_dir_with_tolerance(a, b, c, 0.0)
}

/// Returns the direction of point c relative to line ab with tolerance.
pub fn vec_dir_with_tolerance(a: &Point, b: &Point, c: &Point, tolerance: f64) -> i32 {
    debug_assert!(tolerance >= 0.0);

    let area2 = (b.x - a.x) * (c.y - a.y) - (c.x - a.x) * (b.y - a.y);

    if area2 < -tolerance {
        VEC_DIR_BEHIND
    } else if area2 > tolerance {
        VEC_DIR_AHEAD
    } else {
        VEC_DIR_COLLINEAR
    }
}

/// Checks if point c lies on the open segment (a, b), assuming collinearity.
pub fn in_between(a: &Point, b: &Point, c: &Point) -> bool {
    if (a.x - b.x).abs() > f64::EPSILON {
        // Not vertical
        ((a.x < c.x) && (c.x < b.x)) || ((b.x < c.x) && (c.x < a.x))
    } else {
        ((a.y < c.y) && (c.y < b.y)) || ((b.y < c.y) && (c.y < a.y))
    }
}

/// Checks if point c lies on the closed segment [a, b].
pub fn point_on_line(a: &Point, b: &Point, c: &Point) -> bool {
    // Optimize for orthogonal segments
    if a.x == b.x {
        return (a.x == c.x) &&
            (((a.y < c.y) && (c.y < b.y)) || ((b.y < c.y) && (c.y < a.y)));
    } else if a.y == b.y {
        return (a.y == c.y) &&
            (((a.x < c.x) && (c.x < b.x)) || ((b.x < c.x) && (c.x < a.x)));
    }

    // General case
    vec_dir(a, b, c) == VEC_DIR_COLLINEAR && in_between(a, b, c)
}

/// Result codes for ray intersection
pub const DO_INTERSECT: i32 = 1;
pub const DONT_INTERSECT: i32 = 0;
pub const PARALLEL: i32 = 2;

/// Computes the intersection point of ray from `center` through `ray_point`
/// with segment from `seg1` to `seg2`.
/// Returns (result_code, intersection_point).
pub fn ray_intersect_point(
    seg1: &Point,
    seg2: &Point,
    center: &Point,
    ray_point: &Point,
) -> (i32, Point) {
    let ax = seg2.x - seg1.x;
    let ay = seg2.y - seg1.y;
    let bx = center.x - ray_point.x;
    let by = center.y - ray_point.y;
    let cx = seg1.x - center.x;
    let cy = seg1.y - center.y;

    let denom = ay * bx - ax * by;

    if denom.abs() < f64::EPSILON {
        return (PARALLEL, Point::new(0.0, 0.0));
    }

    let t = (ax * cy - ay * cx) / denom;

    // The ray extends from center through ray_point indefinitely
    // t >= 0 means intersection is in the ray direction
    if t < 0.0 {
        return (DONT_INTERSECT, Point::new(0.0, 0.0));
    }

    let s = if ax.abs() > ay.abs() {
        (bx * t + cx) / ax
    } else {
        (by * t + cy) / ay
    };

    // s must be in [0, 1] for intersection to be on the segment
    if s < 0.0 || s > 1.0 {
        return (DONT_INTERSECT, Point::new(0.0, 0.0));
    }

    let ix = center.x + t * (ray_point.x - center.x);
    let iy = center.y + t * (ray_point.y - center.y);

    (DO_INTERSECT, Point::new(ix, iy))
}

/// Checks if point b is in a valid region that can contain shortest paths.
/// a0, a1, a2 are ordered vertices of a shape (a1 is the corner being tested).
///
/// Based on the 'InCone' algorithm from computational geometry.
///
/// C++ ref: geometry.cpp:201 - inValidRegion()
///
/// # Arguments
/// * `ignore_regions` - If true, uses stricter visibility cone checks
/// * `a0` - Point before corner (on shape boundary)
/// * `a1` - The corner point
/// * `a2` - Point after corner (on shape boundary)
/// * `b` - The point to test
pub fn in_valid_region(
    ignore_regions: bool,
    a0: &Point,
    a1: &Point,
    a2: &Point,
    b: &Point,
) -> bool {
    // r is the edge a0--a1
    // s is the edge a1--a2
    // C++ ref: geometry.cpp:207-208
    let r_side = vec_dir(b, a0, a1);
    let s_side = vec_dir(b, a1, a2);

    // C++ ref: geometry.cpp:210-214
    let r_out_on = r_side <= 0;  // b is outside or on edge r
    let s_out_on = s_side <= 0;  // b is outside or on edge s
    let r_out = r_side < 0;      // b is strictly outside edge r
    let s_out = s_side < 0;      // b is strictly outside edge s

    // C++ ref: geometry.cpp:216
    if vec_dir(a0, a1, a2) > 0 {
        // Convex corner at a1
        // C++ ref: geometry.cpp:229-233
        if ignore_regions {
            (r_out_on && !s_out) || (!r_out && s_out_on)
        } else {
            r_out_on || s_out_on
        }
    } else {
        // Concave (reflex) corner at a1
        // C++ ref: geometry.cpp:248
        if ignore_regions {
            false
        } else {
            r_out_on && s_out_on
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_creation() {
        let p = Point::new(1.0, 2.0);
        assert_eq!(p.x, 1.0);
        assert_eq!(p.y, 2.0);
    }

    #[test]
    fn test_point_arithmetic() {
        let p1 = Point::new(1.0, 2.0);
        let p2 = Point::new(3.0, 4.0);
        let p3 = p1 + p2;
        assert_eq!(p3.x, 4.0);
        assert_eq!(p3.y, 6.0);

        let p4 = p2 - p1;
        assert_eq!(p4.x, 2.0);
        assert_eq!(p4.y, 2.0);
    }

    #[test]
    fn test_point_distance() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(3.0, 4.0);
        assert_eq!(p1.distance(&p2), 5.0);
    }

    #[test]
    fn test_box_operations() {
        let b = Box::from_coords(0.0, 0.0, 10.0, 10.0);
        assert_eq!(b.width(), 10.0);
        assert_eq!(b.height(), 10.0);

        let p = Point::new(5.0, 5.0);
        assert!(b.contains(&p));

        let p2 = Point::new(15.0, 15.0);
        assert!(!b.contains(&p2));
    }

    #[test]
    fn test_rectangle() {
        let rect = Rectangle::new(Point::new(0.0, 0.0), 20.0, 10.0);
        assert_eq!(rect.width(), 20.0);
        assert_eq!(rect.height(), 10.0);

        let center = rect.center();
        assert_eq!(center.x, 0.0);
        assert_eq!(center.y, 0.0);
    }

    #[test]
    fn test_polygon() {
        let mut poly = Polygon::new();
        poly.push(Point::new(0.0, 0.0));
        poly.push(Point::new(10.0, 0.0));
        poly.push(Point::new(10.0, 10.0));
        poly.push(Point::new(0.0, 10.0));

        assert_eq!(poly.size(), 4);
        assert!(!poly.empty());

        let bbox = poly.bounding_rect();
        assert_eq!(bbox.width(), 10.0);
        assert_eq!(bbox.height(), 10.0);
    }

    // ========================================================================
    // Geometry function tests
    // ========================================================================

    #[test]
    fn test_ccw() {
        let a = Point::new(0.0, 0.0);
        let b = Point::new(1.0, 0.0);
        let c = Point::new(1.0, 1.0);

        // CCW turn (left turn)
        assert!(ccw(&a, &b, &c) > 0.0);

        // CW turn (right turn)
        let d = Point::new(1.0, -1.0);
        assert!(ccw(&a, &b, &d) < 0.0);

        // Collinear
        let e = Point::new(2.0, 0.0);
        assert!(ccw(&a, &b, &e).abs() < EPSILON);
    }

    #[test]
    fn test_segments_intersect_crossing() {
        // X crossing
        let a1 = Point::new(0.0, 0.0);
        let a2 = Point::new(10.0, 10.0);
        let b1 = Point::new(0.0, 10.0);
        let b2 = Point::new(10.0, 0.0);

        assert!(segments_intersect(&a1, &a2, &b1, &b2));
    }

    #[test]
    fn test_segments_intersect_no_crossing() {
        // Parallel, no crossing
        let a1 = Point::new(0.0, 0.0);
        let a2 = Point::new(10.0, 0.0);
        let b1 = Point::new(0.0, 5.0);
        let b2 = Point::new(10.0, 5.0);

        assert!(!segments_intersect(&a1, &a2, &b1, &b2));
    }

    #[test]
    fn test_segments_intersect_shared_endpoint() {
        // Segments share an endpoint
        let a1 = Point::new(0.0, 0.0);
        let a2 = Point::new(5.0, 5.0);
        let b1 = Point::new(5.0, 5.0);
        let b2 = Point::new(10.0, 0.0);

        // Standard test considers this an intersection
        assert!(segments_intersect(&a1, &a2, &b1, &b2));
        // But excluding endpoints test does not
        assert!(!segments_intersect_excluding_endpoints(&a1, &a2, &b1, &b2));
    }

    #[test]
    fn test_segments_intersect_t_junction() {
        // T-junction
        let a1 = Point::new(0.0, 5.0);
        let a2 = Point::new(10.0, 5.0);
        let b1 = Point::new(5.0, 0.0);
        let b2 = Point::new(5.0, 5.0);

        assert!(segments_intersect(&a1, &a2, &b1, &b2));
    }

    #[test]
    fn test_point_in_polygon_inside() {
        let mut poly = Polygon::new();
        poly.push(Point::new(0.0, 0.0));
        poly.push(Point::new(10.0, 0.0));
        poly.push(Point::new(10.0, 10.0));
        poly.push(Point::new(0.0, 10.0));

        // Point clearly inside
        let inside = Point::new(5.0, 5.0);
        assert!(point_in_polygon(&inside, &poly));
    }

    #[test]
    fn test_point_in_polygon_outside() {
        let mut poly = Polygon::new();
        poly.push(Point::new(0.0, 0.0));
        poly.push(Point::new(10.0, 0.0));
        poly.push(Point::new(10.0, 10.0));
        poly.push(Point::new(0.0, 10.0));

        // Point clearly outside
        let outside = Point::new(15.0, 5.0);
        assert!(!point_in_polygon(&outside, &poly));
    }

    #[test]
    fn test_point_in_polygon_on_boundary() {
        let mut poly = Polygon::new();
        poly.push(Point::new(0.0, 0.0));
        poly.push(Point::new(10.0, 0.0));
        poly.push(Point::new(10.0, 10.0));
        poly.push(Point::new(0.0, 10.0));

        // Point on edge should return false (not strictly inside)
        let on_edge = Point::new(5.0, 0.0);
        assert!(!point_in_polygon(&on_edge, &poly));
    }

    #[test]
    fn test_segment_intersects_polygon_crossing() {
        let mut poly = Polygon::new();
        poly.push(Point::new(0.0, 0.0));
        poly.push(Point::new(10.0, 0.0));
        poly.push(Point::new(10.0, 10.0));
        poly.push(Point::new(0.0, 10.0));

        // Segment passes through polygon
        let p1 = Point::new(-5.0, 5.0);
        let p2 = Point::new(15.0, 5.0);
        assert!(segment_intersects_polygon(&p1, &p2, &poly));
    }

    #[test]
    fn test_segment_intersects_polygon_inside() {
        let mut poly = Polygon::new();
        poly.push(Point::new(0.0, 0.0));
        poly.push(Point::new(10.0, 0.0));
        poly.push(Point::new(10.0, 10.0));
        poly.push(Point::new(0.0, 10.0));

        // Segment entirely inside polygon
        let p1 = Point::new(2.0, 2.0);
        let p2 = Point::new(8.0, 8.0);
        assert!(segment_intersects_polygon(&p1, &p2, &poly));
    }

    #[test]
    fn test_segment_intersects_polygon_outside() {
        let mut poly = Polygon::new();
        poly.push(Point::new(0.0, 0.0));
        poly.push(Point::new(10.0, 0.0));
        poly.push(Point::new(10.0, 10.0));
        poly.push(Point::new(0.0, 10.0));

        // Segment entirely outside polygon
        let p1 = Point::new(15.0, 0.0);
        let p2 = Point::new(15.0, 10.0);
        assert!(!segment_intersects_polygon(&p1, &p2, &poly));
    }

    #[test]
    fn test_segment_along_edge_no_intersect() {
        let mut poly = Polygon::new();
        poly.push(Point::new(0.0, 0.0));
        poly.push(Point::new(10.0, 0.0));
        poly.push(Point::new(10.0, 10.0));
        poly.push(Point::new(0.0, 10.0));

        // Segment that goes along outside of polygon shouldn't intersect
        let p1 = Point::new(-5.0, 0.0);
        let p2 = Point::new(-5.0, 10.0);
        assert!(!segment_intersects_polygon(&p1, &p2, &poly));
    }

    #[test]
    fn test_segment_corner_touch() {
        let mut poly = Polygon::new();
        poly.push(Point::new(0.0, 0.0));
        poly.push(Point::new(10.0, 0.0));
        poly.push(Point::new(10.0, 10.0));
        poly.push(Point::new(0.0, 10.0));

        // Segment touches corner of polygon
        let p1 = Point::new(-5.0, -5.0);
        let p2 = Point::new(0.0, 0.0);

        // Interior test should not count corner touch as intersection
        assert!(!segment_intersects_polygon_interior(&p1, &p2, &poly));
    }
}
