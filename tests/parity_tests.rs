//! Parity tests comparing libavoid-rust behavior with libavoid-js/C++
//!
//! These tests verify that routing results match expected behavior from the
//! original C++ libavoid implementation.

use libavoid::{Router, Point, Polygon, ConnEnd, ConnType, Rectangle, PolygonInterface};

/// Helper to create a rectangle polygon
fn rect(x: f64, y: f64, w: f64, h: f64) -> Polygon {
    Rectangle::new(Point::new(x, y), w, h).into()
}

// ============================================================================
// Basic Routing Parity Tests
// ============================================================================

#[test]
fn parity_simple_obstacle_avoidance() {
    // Test case from libavoid-js: simple routing around a single obstacle
    let mut router = Router::new(0);

    // Add a 50x50 obstacle centered at (100, 100)
    // Bounds: x: 75-125, y: 75-125
    let obstacle = rect(100.0, 100.0, 50.0, 50.0);
    router.add_shape(obstacle, 1);

    // Route from left of obstacle to right of obstacle at y=100 (through center)
    let src = ConnEnd::new(Point::new(50.0, 100.0));
    let dst = ConnEnd::new(Point::new(200.0, 100.0));
    let conn_id = router.new_connector(src, dst);

    // Process transaction to compute routes
    router.process_transaction();

    // Get the route
    let conn = router.get_connector(conn_id).unwrap();
    let route = conn.display_route().unwrap();

    // Route MUST have more than 2 points - a direct line would go through the obstacle
    assert!(route.size() > 2,
        "Route should avoid obstacle and have more than 2 points, got {}",
        route.size());

    // Verify no route point is inside the obstacle (allowing boundary touches)
    const TOLERANCE: f64 = 0.1;
    for i in 0..route.size() {
        let p = route.at(i);
        let inside = p.x > 75.0 + TOLERANCE && p.x < 125.0 - TOLERANCE
                  && p.y > 75.0 + TOLERANCE && p.y < 125.0 - TOLERANCE;
        assert!(!inside,
            "Route point ({}, {}) is inside obstacle bounds (75-125, 75-125)",
            p.x, p.y);
    }
}

#[test]
fn parity_multiple_obstacles() {
    // Test routing through a maze of obstacles
    let mut router = Router::new(0);

    // Create a simple maze
    router.add_shape(rect(50.0, 50.0, 30.0, 100.0), 1);   // Left wall
    router.add_shape(rect(120.0, 0.0, 30.0, 100.0), 2);   // Right wall (top)
    router.add_shape(rect(120.0, 150.0, 30.0, 100.0), 3); // Right wall (bottom)

    // Route from top-left to bottom-right
    let src = ConnEnd::new(Point::new(10.0, 100.0));
    let dst = ConnEnd::new(Point::new(200.0, 100.0));
    let conn_id = router.new_connector(src, dst);

    // Process transaction to compute routes
    router.process_transaction();

    let conn = router.get_connector(conn_id).unwrap();
    let route = conn.display_route().unwrap();

    // Route should exist and have reasonable length
    assert!(route.size() >= 2, "Route should have at least 2 points");

    // Start and end points should be correct
    assert!(route.at(0).equals(&Point::new(10.0, 100.0)), "Route should start at source");
    assert!(route.at(route.size() - 1).equals(&Point::new(200.0, 100.0)), "Route should end at destination");
}

#[test]
fn parity_orthogonal_routing() {
    // Test orthogonal routing produces only H/V segments
    let mut router = Router::new(0);

    router.add_shape(rect(100.0, 100.0, 50.0, 50.0), 1);

    let src = ConnEnd::new(Point::new(50.0, 80.0));
    let dst = ConnEnd::new(Point::new(200.0, 180.0));

    let mut conn = libavoid::ConnRef::with_type(
        1,
        src,
        dst,
        ConnType::Orthogonal,
    );
    conn.set_routing_type(ConnType::Orthogonal);
    router.add_connector(conn);

    // Process transaction to compute routes
    router.process_transaction();

    let conn = router.get_connector(1).unwrap();
    let route = conn.display_route().unwrap();

    // Verify all segments are orthogonal (horizontal or vertical)
    for i in 0..route.size().saturating_sub(1) {
        let p1 = route.at(i);
        let p2 = route.at(i + 1);

        let is_horizontal = (p1.y - p2.y).abs() < 0.001;
        let is_vertical = (p1.x - p2.x).abs() < 0.001;

        assert!(is_horizontal || is_vertical,
            "Orthogonal route segment should be horizontal or vertical, got ({}, {}) to ({}, {})",
            p1.x, p1.y, p2.x, p2.y);
    }
}

#[test]
fn parity_direct_path_when_clear() {
    // When path is clear, should use direct line
    let mut router = Router::new(0);

    // No obstacles - path should be direct
    let src = ConnEnd::new(Point::new(0.0, 0.0));
    let dst = ConnEnd::new(Point::new(100.0, 100.0));
    let conn_id = router.new_connector(src, dst);

    // Process transaction to compute routes
    router.process_transaction();

    let conn = router.get_connector(conn_id).unwrap();
    let route = conn.display_route().unwrap();

    // Direct path should have exactly 2 points
    assert_eq!(route.size(), 2, "Direct path should have exactly 2 points");
    assert!(route.at(0).equals(&Point::new(0.0, 0.0)));
    assert!(route.at(1).equals(&Point::new(100.0, 100.0)));
}

#[test]
fn parity_transaction_batching() {
    // Test that transaction batching produces same results as immediate mode
    use libavoid::ROUTER_FLAG_USE_TRANSACTIONS;

    // Create two routers - one with transactions, one without
    let mut router_immediate = Router::new(0);
    let mut router_batch = Router::new(ROUTER_FLAG_USE_TRANSACTIONS);

    // Add same shapes to both
    let shape = rect(100.0, 100.0, 50.0, 50.0);
    router_immediate.add_shape(shape.clone(), 1);
    router_batch.add_shape(shape, 1);

    // Add same connectors to both
    let src = ConnEnd::new(Point::new(50.0, 125.0));
    let dst = ConnEnd::new(Point::new(200.0, 125.0));

    router_immediate.new_connector(src.clone(), dst.clone());
    router_batch.new_connector(src, dst);

    // Process transactions for both
    router_immediate.process_transaction();
    router_batch.process_transaction();

    // Both should produce routes
    let route1 = router_immediate.get_connector(1).unwrap().display_route().unwrap();
    let route2 = router_batch.get_connector(1).unwrap().display_route().unwrap();

    // Routes should be similar (not necessarily identical due to floating point)
    assert_eq!(route1.size(), route2.size(),
        "Transaction mode should produce same route structure");
}

// ============================================================================
// Edge Case Parity Tests
// ============================================================================

#[test]
fn parity_route_along_obstacle_edge() {
    // Route that needs to go along an obstacle edge
    let mut router = Router::new(0);

    router.add_shape(rect(100.0, 0.0, 50.0, 200.0), 1);

    // Source and dest on same side of obstacle
    let src = ConnEnd::new(Point::new(50.0, 50.0));
    let dst = ConnEnd::new(Point::new(50.0, 150.0));
    let conn_id = router.new_connector(src, dst);

    // Process transaction to compute routes
    router.process_transaction();

    let conn = router.get_connector(conn_id).unwrap();
    let route = conn.display_route().unwrap();

    // Route should exist
    assert!(route.size() >= 2);

    // Route should not cross the obstacle
    for i in 0..route.size() {
        let p = route.at(i);
        // No point should be inside the obstacle
        let inside = p.x > 100.0 && p.x < 150.0 && p.y > 0.0 && p.y < 200.0;
        assert!(!inside, "Route point should not be inside obstacle");
    }
}

#[test]
fn parity_endpoint_on_obstacle_boundary() {
    // Endpoint exactly on obstacle boundary
    let mut router = Router::new(0);

    router.add_shape(rect(100.0, 100.0, 50.0, 50.0), 1);

    // Source on obstacle boundary
    let src = ConnEnd::new(Point::new(100.0, 125.0)); // Left edge of obstacle
    let dst = ConnEnd::new(Point::new(200.0, 125.0));
    let conn_id = router.new_connector(src, dst);

    // Process transaction to compute routes
    router.process_transaction();

    let conn = router.get_connector(conn_id).unwrap();
    let route = conn.display_route();

    // Should still produce a valid route
    assert!(route.is_some(), "Should produce route even with endpoint on boundary");
}

#[test]
fn parity_coincident_endpoints() {
    // Source and destination at same point
    let mut router = Router::new(0);

    let src = ConnEnd::new(Point::new(100.0, 100.0));
    let dst = ConnEnd::new(Point::new(100.0, 100.0));
    let conn_id = router.new_connector(src, dst);

    // Process transaction to compute routes
    router.process_transaction();

    let conn = router.get_connector(conn_id).unwrap();
    let route = conn.display_route();

    // Should handle gracefully (either 1 point or 2 coincident points)
    assert!(route.is_some());
}

// ============================================================================
// Parameter and Option Parity Tests
// ============================================================================

#[test]
fn test_default_parameters_match_cpp() {
    // Verify all default parameter values match C++ libavoid
    // C++ Reference: libavoid/router.cpp:85-91
    use libavoid::RoutingParameter;

    let router = Router::new(0);

    // Verify routing parameters match C++ defaults
    assert_eq!(router.routing_parameter(RoutingParameter::SegmentPenalty), 10.0,
        "SegmentPenalty should default to 10.0 (C++ router.cpp:89)");

    assert_eq!(router.routing_parameter(RoutingParameter::BendPenalty), 0.0,
        "BendPenalty should default to 0.0 (C++ implicit default)");

    assert_eq!(router.routing_parameter(RoutingParameter::CrossingPenalty), 0.0,
        "CrossingPenalty should default to 0.0 (C++ implicit default)");

    assert_eq!(router.routing_parameter(RoutingParameter::ClusterCrossingPenalty), 4000.0,
        "ClusterCrossingPenalty should default to 4000.0 (C++ router.cpp:90)");

    assert_eq!(router.routing_parameter(RoutingParameter::IdealNudgingDistance), 4.0,
        "IdealNudgingDistance should default to 4.0 (C++ router.cpp:91)");

    assert_eq!(router.routing_parameter(RoutingParameter::ShapeBufferDistance), 0.0,
        "ShapeBufferDistance should default to 0.0 (C++ implicit default)");

    // Note: Other parameters (FixedSharedPathPenalty, PortDirectionPenalty, ReverseDirectionPenalty)
    // are not set by default in C++ and will return 0.0 when queried
}

#[test]
fn test_default_options_match_cpp() {
    // Verify all default routing option values match C++ libavoid
    // C++ Reference: libavoid/router.cpp:93-101
    use libavoid::RoutingOption;

    let router = Router::new(0);

    // Verify routing options match C++ defaults
    assert_eq!(router.routing_option(RoutingOption::NudgeOrthogonalRoutes), false,
        "NudgeOrthogonalRoutes should default to false");

    assert_eq!(router.routing_option(RoutingOption::ImproveHyperedgeRoutes), true,
        "ImproveHyperedgeRoutes should default to true (C++ router.cpp:94)");

    assert_eq!(router.routing_option(RoutingOption::PenalisePortDirections), false,
        "PenalisePortDirections should default to false");

    assert_eq!(router.routing_option(RoutingOption::NudgeSharedPathsWithCommonEndPoint), false,
        "NudgeSharedPathsWithCommonEndPoint should default to false");

    assert_eq!(router.routing_option(RoutingOption::NudgeOrthogonalSegmentsConnectedToShapes), false,
        "NudgeOrthogonalSegmentsConnectedToShapes should default to false");

    assert_eq!(router.routing_option(RoutingOption::PenaliseOrthogonalSharedPathsAtConnEnds), false,
        "PenaliseOrthogonalSharedPathsAtConnEnds should default to false");

    assert_eq!(router.routing_option(RoutingOption::NudgeOrthogonalTouchingColinearSegments), false,
        "NudgeOrthogonalTouchingColinearSegments should default to false");

    assert_eq!(router.routing_option(RoutingOption::PerformUnifyingNudgingPreprocessingStep), true,
        "PerformUnifyingNudgingPreprocessingStep should default to true (C++ router.cpp:98)");

    assert_eq!(router.routing_option(RoutingOption::ImproveHyperedgeRoutesMovingAddingAndDeletingJunctions), false,
        "ImproveHyperedgeRoutesMovingAddingAndDeletingJunctions should default to false");
}

#[test]
fn test_transaction_mode_default() {
    // Verify transaction mode defaults to true for C++ parity
    // C++ Reference: libavoid/router.cpp:62 (m_consolidate_actions = true)
    let router = Router::new(0);

    assert_eq!(router.transaction_use(), true,
        "Transaction mode should default to true (C++ m_consolidate_actions = true)");
}
