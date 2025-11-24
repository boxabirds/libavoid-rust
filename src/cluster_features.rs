//! Cluster features (Task #17)
//!
//! Handles cluster crossing detection and penalties for pathfinding.
//! C++ Reference: libavoid/makepath.cpp:480-500

use crate::geometry::Point;
use crate::cluster::ClusterRef;

/// Cluster feature enhancements for routing
pub struct ClusterFeatures {
    // Reserved for future cluster-specific state
}

impl ClusterFeatures {
    /// Create new cluster features handler
    pub fn new() -> Self {
        ClusterFeatures {}
    }

    /// Check if a line segment crosses any cluster boundary
    pub fn edge_crosses_cluster(
        from: &Point,
        to: &Point,
        clusters: &[ClusterRef],
    ) -> bool {
        for cluster in clusters {
            if !cluster.is_active() {
                continue;
            }

            if Self::line_intersects_polygon(from, to, cluster.polygon()) {
                return true;
            }
        }
        false
    }

    /// Check if a line segment intersects with a polygon boundary
    fn line_intersects_polygon(
        p1: &Point,
        p2: &Point,
        polygon: &crate::geometry::Polygon,
    ) -> bool {
        use crate::geometry::PolygonInterface;

        let n = polygon.size();
        if n < 2 {
            return false;
        }

        // Check if line intersects any edge of the polygon
        for i in 0..n {
            let v1 = polygon.at(i);
            let v2 = polygon.at((i + 1) % n);

            if Self::line_segments_intersect(p1, p2, &v1, &v2) {
                return true;
            }
        }

        false
    }

    /// Check if two line segments intersect
    fn line_segments_intersect(
        p1: &Point,
        p2: &Point,
        p3: &Point,
        p4: &Point,
    ) -> bool {
        let d1 = Self::direction(p3, p4, p1);
        let d2 = Self::direction(p3, p4, p2);
        let d3 = Self::direction(p1, p2, p3);
        let d4 = Self::direction(p1, p2, p4);

        if ((d1 > 0.0 && d2 < 0.0) || (d1 < 0.0 && d2 > 0.0))
            && ((d3 > 0.0 && d4 < 0.0) || (d3 < 0.0 && d4 > 0.0))
        {
            return true;
        }

        // Check for collinear cases
        if d1.abs() < 1e-10 && Self::on_segment(p3, p1, p4) {
            return true;
        }
        if d2.abs() < 1e-10 && Self::on_segment(p3, p2, p4) {
            return true;
        }
        if d3.abs() < 1e-10 && Self::on_segment(p1, p3, p2) {
            return true;
        }
        if d4.abs() < 1e-10 && Self::on_segment(p1, p4, p2) {
            return true;
        }

        false
    }

    /// Compute direction using cross product
    fn direction(p1: &Point, p2: &Point, p3: &Point) -> f64 {
        (p3.x - p1.x) * (p2.y - p1.y) - (p2.x - p1.x) * (p3.y - p1.y)
    }

    /// Check if point q lies on segment pr
    fn on_segment(p: &Point, q: &Point, r: &Point) -> bool {
        q.x <= p.x.max(r.x)
            && q.x >= p.x.min(r.x)
            && q.y <= p.y.max(r.y)
            && q.y >= p.y.min(r.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Polygon, Rectangle};

    #[test]
    fn test_line_segments_intersect() {
        let p1 = Point::new(0.0, 0.0);
        let p2 = Point::new(10.0, 10.0);
        let p3 = Point::new(0.0, 10.0);
        let p4 = Point::new(10.0, 0.0);

        // These two segments should intersect at (5, 5)
        assert!(ClusterFeatures::line_segments_intersect(&p1, &p2, &p3, &p4));
    }

    #[test]
    fn test_edge_crosses_cluster() {
        let rect = Rectangle::new(Point::new(50.0, 50.0), 20.0, 20.0);
        let mut cluster = ClusterRef::new(1, rect.into());
        cluster.make_active();

        // Line that crosses the cluster
        let from = Point::new(0.0, 50.0);
        let to = Point::new(100.0, 50.0);
        assert!(ClusterFeatures::edge_crosses_cluster(&from, &to, &[cluster.clone()]));

        // Line that doesn't cross the cluster
        let from2 = Point::new(0.0, 0.0);
        let to2 = Point::new(10.0, 10.0);
        assert!(!ClusterFeatures::edge_crosses_cluster(&from2, &to2, &[cluster]));
    }
}
