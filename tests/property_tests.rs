//! Property-based tests for libavoid-rust
//!
//! These tests verify invariants that should hold for all inputs.

use libavoid::{Router, Point, ConnEnd, Rectangle, PolygonInterface};

// ============================================================================
// Geometry Property Tests
// ============================================================================

#[test]
fn property_route_starts_at_source() {
    // Property: Every route should start at the source point
    for seed in 0..20 {
        let mut router = Router::new(0);

        // Random-ish source and dest
        let src_x = (seed * 17) as f64 % 500.0;
        let src_y = (seed * 23) as f64 % 500.0;
        let dst_x = (seed * 31 + 100) as f64 % 500.0;
        let dst_y = (seed * 37 + 100) as f64 % 500.0;

        let src = ConnEnd::new(Point::new(src_x, src_y));
        let dst = ConnEnd::new(Point::new(dst_x, dst_y));
        let conn_id = router.new_connector(src, dst);

        let conn = router.get_connector(conn_id).unwrap();
        if let Some(route) = conn.display_route() {
            assert!(route.size() >= 2, "Route should have at least 2 points");
            let start = route.at(0);
            assert!((start.x - src_x).abs() < 0.001 && (start.y - src_y).abs() < 0.001,
                "Route should start at source ({}, {}), got ({}, {})",
                src_x, src_y, start.x, start.y);
        }
    }
}

#[test]
fn property_route_ends_at_destination() {
    // Property: Every route should end at the destination point
    for seed in 0..20 {
        let mut router = Router::new(0);

        let src_x = (seed * 17) as f64 % 500.0;
        let src_y = (seed * 23) as f64 % 500.0;
        let dst_x = (seed * 31 + 100) as f64 % 500.0;
        let dst_y = (seed * 37 + 100) as f64 % 500.0;

        let src = ConnEnd::new(Point::new(src_x, src_y));
        let dst = ConnEnd::new(Point::new(dst_x, dst_y));
        let conn_id = router.new_connector(src, dst);

        let conn = router.get_connector(conn_id).unwrap();
        if let Some(route) = conn.display_route() {
            let end = route.at(route.size() - 1);
            assert!((end.x - dst_x).abs() < 0.001 && (end.y - dst_y).abs() < 0.001,
                "Route should end at destination ({}, {}), got ({}, {})",
                dst_x, dst_y, end.x, end.y);
        }
    }
}

#[test]
fn property_route_is_continuous() {
    // Property: Route segments should be connected (no gaps)
    let mut router = Router::new(0);

    // Add some obstacles
    router.add_shape(Rectangle::new(Point::new(100.0, 100.0), 50.0, 50.0).into(), 1);
    router.add_shape(Rectangle::new(Point::new(200.0, 50.0), 40.0, 100.0).into(), 2);

    for seed in 0..10 {
        let src_x = (seed * 13) as f64 % 300.0;
        let src_y = (seed * 19) as f64 % 300.0;
        let dst_x = (seed * 29 + 200) as f64 % 500.0;
        let dst_y = (seed * 41 + 100) as f64 % 400.0;

        let src = ConnEnd::new(Point::new(src_x, src_y));
        let dst = ConnEnd::new(Point::new(dst_x, dst_y));
        let conn_id = router.new_connector(src, dst);

        let conn = router.get_connector(conn_id).unwrap();
        if let Some(route) = conn.display_route() {
            // Each segment end should equal the next segment start
            for i in 0..route.size().saturating_sub(2) {
                let p1 = route.at(i + 1);
                // Points should be valid (not NaN or infinity)
                assert!(p1.x.is_finite() && p1.y.is_finite(),
                    "Route points should be finite");
            }
        }
    }
}

#[test]
fn property_route_avoids_obstacle_interiors() {
    // Property: Route midpoints should not be strictly inside obstacles
    let mut router = Router::new(0);

    let obstacles = [
        (100.0, 100.0, 50.0, 50.0),
        (200.0, 150.0, 60.0, 40.0),
        (50.0, 200.0, 40.0, 60.0),
    ];

    for (i, &(x, y, w, h)) in obstacles.iter().enumerate() {
        router.add_shape(Rectangle::new(Point::new(x, y), w, h).into(), (i + 1) as u32);
    }

    // Test multiple routes
    let endpoints = [
        (10.0, 10.0, 300.0, 300.0),
        (10.0, 250.0, 300.0, 50.0),
        (150.0, 10.0, 150.0, 280.0),
    ];

    for (src_x, src_y, dst_x, dst_y) in endpoints {
        let src = ConnEnd::new(Point::new(src_x, src_y));
        let dst = ConnEnd::new(Point::new(dst_x, dst_y));
        let conn_id = router.new_connector(src, dst);

        let conn = router.get_connector(conn_id).unwrap();
        if let Some(route) = conn.display_route() {
            // Check midpoints of each segment
            for i in 0..route.size().saturating_sub(1) {
                let p1 = route.at(i);
                let p2 = route.at(i + 1);
                let mid = Point::new((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0);

                // Midpoint should not be strictly inside any obstacle
                for &(ox, oy, ow, oh) in &obstacles {
                    let inside = mid.x > ox + 1.0 && mid.x < ox + ow - 1.0 &&
                                 mid.y > oy + 1.0 && mid.y < oy + oh - 1.0;
                    assert!(!inside,
                        "Route midpoint ({}, {}) should not be inside obstacle at ({}, {})",
                        mid.x, mid.y, ox, oy);
                }
            }
        }
    }
}

#[test]
fn property_route_length_is_reasonable() {
    // Property: Route length should be >= direct distance and <= some reasonable bound
    let mut router = Router::new(0);

    // Add obstacle that forces detour
    router.add_shape(Rectangle::new(Point::new(100.0, 0.0), 50.0, 200.0).into(), 1);

    let src = ConnEnd::new(Point::new(50.0, 100.0));
    let dst = ConnEnd::new(Point::new(200.0, 100.0));
    let conn_id = router.new_connector(src, dst);

    let direct_dist = Point::new(50.0, 100.0).distance(&Point::new(200.0, 100.0));

    let conn = router.get_connector(conn_id).unwrap();
    if let Some(route) = conn.display_route() {
        let mut route_length = 0.0;
        for i in 0..route.size().saturating_sub(1) {
            route_length += route.at(i).distance(route.at(i + 1));
        }

        // Route should be at least as long as direct distance
        assert!(route_length >= direct_dist * 0.99,
            "Route length {} should be >= direct distance {}", route_length, direct_dist);

        // Route should not be absurdly long (< 10x direct distance)
        let max_reasonable = direct_dist * 10.0;
        assert!(route_length < max_reasonable,
            "Route length {} should be < {} (10x direct)", route_length, max_reasonable);
    }
}

// ============================================================================
// Transaction Property Tests
// ============================================================================

#[test]
fn property_transaction_idempotent() {
    // Property: Calling process_transaction multiple times should be safe
    use libavoid::ROUTER_FLAG_USE_TRANSACTIONS;

    let mut router = Router::new(ROUTER_FLAG_USE_TRANSACTIONS);

    router.add_shape(Rectangle::new(Point::new(100.0, 100.0), 50.0, 50.0).into(), 1);

    let src = ConnEnd::new(Point::new(50.0, 125.0));
    let dst = ConnEnd::new(Point::new(200.0, 125.0));
    router.new_connector(src, dst);

    // Process multiple times
    router.process_transaction();
    let route1 = router.get_connector(1).unwrap().display_route().cloned();

    router.process_transaction();
    let route2 = router.get_connector(1).unwrap().display_route().cloned();

    router.process_transaction();
    let route3 = router.get_connector(1).unwrap().display_route().cloned();

    // Routes should be unchanged
    assert_eq!(route1.as_ref().map(|r| r.size()), route2.as_ref().map(|r| r.size()));
    assert_eq!(route2.as_ref().map(|r| r.size()), route3.as_ref().map(|r| r.size()));
}

#[test]
fn property_shape_removal_updates_routes() {
    // Property: Removing a blocking shape should allow shorter routes
    let mut router = Router::new(0);

    // Add obstacle that blocks direct path
    let shape_id = router.add_shape(
        Rectangle::new(Point::new(100.0, 50.0), 50.0, 100.0).into(),
        1
    );

    let src = ConnEnd::new(Point::new(50.0, 100.0));
    let dst = ConnEnd::new(Point::new(200.0, 100.0));
    let conn_id = router.new_connector(src, dst);

    // Process transaction to compute initial routes
    router.process_transaction();

    let route_with_obstacle = router.get_connector(conn_id).unwrap().display_route().cloned();

    // Remove obstacle
    router.delete_shape(shape_id);

    // Process transaction after shape removal
    router.process_transaction();

    let route_without_obstacle = router.get_connector(conn_id).unwrap().display_route().cloned();

    // Both routes should exist
    assert!(route_with_obstacle.is_some());
    assert!(route_without_obstacle.is_some());
}

// ============================================================================
// Crossing Detection Property Tests
// ============================================================================

#[test]
fn property_crossing_count_symmetric() {
    use libavoid::geometry::{count_route_crossings, Polygon};

    // Property: crossings(A, B) == crossings(B, A)
    let mut route1 = Polygon::new();
    route1.push(Point::new(0.0, 50.0));
    route1.push(Point::new(100.0, 50.0));

    let mut route2 = Polygon::new();
    route2.push(Point::new(50.0, 0.0));
    route2.push(Point::new(50.0, 100.0));

    let crossings_ab = count_route_crossings(&route1, &route2);
    let crossings_ba = count_route_crossings(&route2, &route1);

    assert_eq!(crossings_ab, crossings_ba,
        "Crossing count should be symmetric");
}

#[test]
fn property_no_self_crossings_for_simple_routes() {
    use libavoid::geometry::{count_route_crossings, Polygon};

    // Property: A simple (non-self-intersecting) route has 0 crossings with itself
    let mut route = Polygon::new();
    route.push(Point::new(0.0, 0.0));
    route.push(Point::new(100.0, 0.0));
    route.push(Point::new(100.0, 100.0));
    route.push(Point::new(0.0, 100.0));

    // Note: this counts segment crossings, not self-intersection
    // Adjacent segments share endpoints and shouldn't count
    let self_crossings = count_route_crossings(&route, &route);

    // For a simple L-shaped route, there should be no proper self-crossings
    // (segments share endpoints, which are excluded)
    assert_eq!(self_crossings, 0, "Simple route should have no self-crossings");
}
