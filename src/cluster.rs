//! Cluster support for grouping shapes
//!
//! Clusters allow shapes to be grouped together, with connectors routing
//! around the cluster boundary rather than individual shapes.

use crate::geometry::{Point, Polygon, PolygonInterface, Rectangle};

/// A cluster reference that groups shapes together
#[derive(Debug, Clone)]
pub struct ClusterRef {
    /// Unique identifier
    id: u32,
    /// The polygon defining the cluster boundary
    polygon: Polygon,
    /// Rectangular bounding polygon
    rectangular_polygon: Polygon,
    /// Whether this cluster is active
    active: bool,
    /// IDs of shapes contained in this cluster
    contained_shapes: Vec<u32>,
}

impl ClusterRef {
    /// Creates a new cluster with the given polygon boundary
    pub fn new(id: u32, polygon: Polygon) -> Self {
        let rectangular_polygon = Self::compute_bounding_rect(&polygon);
        ClusterRef {
            id,
            polygon,
            rectangular_polygon,
            active: false,
            contained_shapes: Vec::new(),
        }
    }

    /// Creates a new cluster with a rectangular boundary
    pub fn from_rectangle(id: u32, center: Point, width: f64, height: f64) -> Self {
        let rect = Rectangle::new(center, width, height);
        let polygon: Polygon = rect.into();
        Self::new(id, polygon)
    }

    /// Computes the bounding rectangle polygon for a given polygon
    fn compute_bounding_rect(poly: &Polygon) -> Polygon {
        if poly.size() == 0 {
            return Polygon::new();
        }

        let mut min_x = f64::INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut max_y = f64::NEG_INFINITY;

        for i in 0..poly.size() {
            let p = poly.at(i);
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        let mut rect_poly = Polygon::new();
        rect_poly.push(Point::new(min_x, min_y));
        rect_poly.push(Point::new(max_x, min_y));
        rect_poly.push(Point::new(max_x, max_y));
        rect_poly.push(Point::new(min_x, max_y));
        rect_poly
    }

    /// Returns the cluster ID
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns whether the cluster is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Makes the cluster active
    pub fn make_active(&mut self) {
        self.active = true;
    }

    /// Makes the cluster inactive
    pub fn make_inactive(&mut self) {
        self.active = false;
    }

    /// Returns the cluster polygon
    pub fn polygon(&self) -> &Polygon {
        &self.polygon
    }

    /// Returns the rectangular bounding polygon
    pub fn rectangular_polygon(&self) -> &Polygon {
        &self.rectangular_polygon
    }

    /// Sets a new polygon for the cluster
    pub fn set_polygon(&mut self, polygon: Polygon) {
        self.rectangular_polygon = Self::compute_bounding_rect(&polygon);
        self.polygon = polygon;
    }

    /// Adds a shape to this cluster
    pub fn add_shape(&mut self, shape_id: u32) {
        if !self.contained_shapes.contains(&shape_id) {
            self.contained_shapes.push(shape_id);
        }
    }

    /// Removes a shape from this cluster
    pub fn remove_shape(&mut self, shape_id: u32) {
        self.contained_shapes.retain(|&id| id != shape_id);
    }

    /// Returns the IDs of shapes in this cluster
    pub fn contained_shapes(&self) -> &[u32] {
        &self.contained_shapes
    }

    /// Checks if a shape is in this cluster
    pub fn contains_shape(&self, shape_id: u32) -> bool {
        self.contained_shapes.contains(&shape_id)
    }

    /// Checks if a point is inside the cluster boundary
    pub fn contains_point(&self, point: &Point) -> bool {
        use crate::geometry::point_in_polygon;
        point_in_polygon(point, &self.polygon)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_creation() {
        let cluster = ClusterRef::from_rectangle(1, Point::new(100.0, 100.0), 50.0, 50.0);
        assert_eq!(cluster.id(), 1);
        assert!(!cluster.is_active());
    }

    #[test]
    fn test_cluster_activation() {
        let mut cluster = ClusterRef::from_rectangle(1, Point::new(100.0, 100.0), 50.0, 50.0);

        cluster.make_active();
        assert!(cluster.is_active());

        cluster.make_inactive();
        assert!(!cluster.is_active());
    }

    #[test]
    fn test_cluster_shape_management() {
        let mut cluster = ClusterRef::from_rectangle(1, Point::new(100.0, 100.0), 50.0, 50.0);

        cluster.add_shape(10);
        cluster.add_shape(20);

        assert!(cluster.contains_shape(10));
        assert!(cluster.contains_shape(20));
        assert!(!cluster.contains_shape(30));

        cluster.remove_shape(10);
        assert!(!cluster.contains_shape(10));
    }

    #[test]
    fn test_cluster_bounding_rect() {
        let mut poly = Polygon::new();
        poly.push(Point::new(0.0, 0.0));
        poly.push(Point::new(100.0, 50.0));
        poly.push(Point::new(50.0, 100.0));

        let cluster = ClusterRef::new(1, poly);
        let rect = cluster.rectangular_polygon();

        // Check bounding rect
        assert_eq!(rect.size(), 4);
        assert_eq!(rect.at(0).x, 0.0);  // min_x
        assert_eq!(rect.at(0).y, 0.0);  // min_y
        assert_eq!(rect.at(2).x, 100.0); // max_x
        assert_eq!(rect.at(2).y, 100.0); // max_y
    }
}
