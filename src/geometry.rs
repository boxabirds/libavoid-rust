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
        // Simple implementation: offset each point outward from centroid
        let mut result = self.clone();

        if self.ps.is_empty() {
            return result;
        }

        // Calculate centroid
        let mut cx = 0.0;
        let mut cy = 0.0;
        for p in &self.ps {
            cx += p.x;
            cy += p.y;
        }
        cx /= self.ps.len() as f64;
        cy /= self.ps.len() as f64;
        let centroid = Point::new(cx, cy);

        // Offset each point away from centroid
        for p in &mut result.ps {
            let dx = p.x - centroid.x;
            let dy = p.y - centroid.y;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist > 1e-10 {
                p.x += (dx / dist) * offset;
                p.y += (dy / dist) * offset;
            }
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
}
