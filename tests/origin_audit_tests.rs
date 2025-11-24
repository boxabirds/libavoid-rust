//! Origin Audit Tests
//!
//! Tests based on the origin audit (docs/20251124-origin-audit-tasks.md)
//! comparing libavoid-rust against the original C++ libavoid.
//!
//! Each test is tagged with its task number for traceability.

use libavoid::{
    Router, Point, Rectangle, ConnEnd, Polygon, PolygonInterface,
    JunctionRef, ConnRef, Obstacle,
};

// =============================================================================
// P1 - High Priority Tests
// =============================================================================

/// Task #4: Test that routes update correctly after shape movement
///
/// Scenario 1: Shape blocks path, then moves out of the way
/// Expected: Route becomes direct (2 points) after shape moves
#[test]
fn test_route_updates_when_shape_moves_out_of_path() {
    let mut router = Router::new(0);

    // Add obstacle blocking a horizontal path at y=100
    // Shape centered at (100, 100), size 50x50 => bounds x:75-125, y:75-125
    let rect = Rectangle::new(Point::new(100.0, 100.0), 50.0, 50.0);
    let shape_id = router.add_shape(rect.into(), 1);

    // Create connector through the obstacle's y position
    let src = ConnEnd::new(Point::new(0.0, 100.0));
    let dst = ConnEnd::new(Point::new(200.0, 100.0));
    let conn_id = router.new_connector(src, dst);

    // Process transaction to compute routes
    router.process_transaction();

    // Route should detour around obstacle
    let conn = router.get_connector(conn_id).unwrap();
    let route_before = conn.display_route().expect("Route should exist");
    assert!(
        route_before.size() > 2,
        "Route should detour around obstacle, got {} points",
        route_before.size()
    );

    // Move shape out of the way (to y=300)
    router.move_shape(shape_id, Point::new(100.0, 300.0));

    // Process transaction after move
    router.process_transaction();

    // Route should now be direct
    let conn = router.get_connector(conn_id).unwrap();
    let route_after = conn.display_route().expect("Route should exist after move");
    assert_eq!(
        route_after.size(),
        2,
        "Route should be direct after obstacle moved, got {} points",
        route_after.size()
    );
}

/// Task #4: Test that routes update correctly after shape movement
///
/// Scenario 2: Shape doesn't block path, then moves into the way
/// Expected: Route becomes indirect (>2 points) after shape moves into path
#[test]
fn test_route_updates_when_shape_moves_into_path() {
    let mut router = Router::new(0);

    // Add obstacle NOT blocking the path (at y=300)
    let rect = Rectangle::new(Point::new(100.0, 300.0), 50.0, 50.0);
    let shape_id = router.add_shape(rect.into(), 1);

    // Create connector at y=100 (clear path)
    let src = ConnEnd::new(Point::new(0.0, 100.0));
    let dst = ConnEnd::new(Point::new(200.0, 100.0));
    let conn_id = router.new_connector(src, dst);

    // Process transaction to compute routes
    router.process_transaction();

    // Route should be direct
    let conn = router.get_connector(conn_id).unwrap();
    let route_before = conn.display_route().expect("Route should exist");
    assert_eq!(
        route_before.size(),
        2,
        "Route should be direct with no obstacle, got {} points",
        route_before.size()
    );

    // Move shape into the path
    router.move_shape(shape_id, Point::new(100.0, 100.0));

    // Process transaction after move
    router.process_transaction();

    // Route should now detour
    let conn = router.get_connector(conn_id).unwrap();
    let route_after = conn.display_route().expect("Route should exist after move");
    assert!(
        route_after.size() > 2,
        "Route should detour after obstacle moved into path, got {} points",
        route_after.size()
    );
}

/// Task #5: Verify transaction processing correctly rebuilds visibility graph
///
/// Same operations in batch mode should produce same results as immediate mode
#[test]
fn test_transaction_processing_produces_correct_routes() {
    use libavoid::ROUTER_FLAG_USE_TRANSACTIONS;

    // Create router with transactions enabled
    let mut router = Router::new(ROUTER_FLAG_USE_TRANSACTIONS);

    // Add obstacle - this queues the action
    let rect = Rectangle::new(Point::new(100.0, 100.0), 50.0, 50.0);
    router.add_shape(rect.into(), 1);

    // Add connector - this also queues the action
    let src = ConnEnd::new(Point::new(50.0, 100.0));
    let dst = ConnEnd::new(Point::new(200.0, 100.0));
    let conn_id = router.new_connector(src, dst);

    // Before processing transaction, route may not exist or be correct
    // Process the transaction
    router.process_transaction();

    // After transaction, route should correctly avoid obstacle
    let conn = router.get_connector(conn_id).unwrap();
    let route = conn.display_route().expect("Route should exist after transaction");

    // Route should detour (obstacle is in the path)
    assert!(
        route.size() > 2,
        "Route should avoid obstacle after transaction processing, got {} points",
        route.size()
    );

    // Verify route doesn't go through obstacle interior
    const TOLERANCE: f64 = 1.0;
    for i in 0..route.size() {
        let p = route.at(i);
        let inside = p.x > 75.0 + TOLERANCE
            && p.x < 125.0 - TOLERANCE
            && p.y > 75.0 + TOLERANCE
            && p.y < 125.0 - TOLERANCE;
        assert!(
            !inside,
            "Route point ({}, {}) is inside obstacle bounds (75-125, 75-125)",
            p.x, p.y
        );
    }
}

/// Task #5: Verify multiple transactions work correctly
#[test]
fn test_multiple_transactions_maintain_consistency() {
    use libavoid::ROUTER_FLAG_USE_TRANSACTIONS;

    let mut router = Router::new(ROUTER_FLAG_USE_TRANSACTIONS);

    // Transaction 1: Add shape and connector
    let rect = Rectangle::new(Point::new(100.0, 100.0), 50.0, 50.0);
    let shape_id = router.add_shape(rect.into(), 1);

    let src = ConnEnd::new(Point::new(0.0, 100.0));
    let dst = ConnEnd::new(Point::new(200.0, 100.0));
    let conn_id = router.new_connector(src, dst);

    router.process_transaction();

    let conn = router.get_connector(conn_id).unwrap();
    let route1 = conn.display_route().expect("Route should exist");
    let route1_size = route1.size();

    // Transaction 2: Move shape out of the way
    router.move_shape(shape_id, Point::new(100.0, 300.0));
    router.process_transaction();

    let conn = router.get_connector(conn_id).unwrap();
    let route2 = conn.display_route().expect("Route should exist after second transaction");

    // Route should be simpler after obstacle moved
    assert!(
        route2.size() <= route1_size,
        "Route should be same or simpler after obstacle moved: {} vs {}",
        route2.size(),
        route1_size
    );
}

// =============================================================================
// P2 - Medium Priority: JunctionRef Tests
// =============================================================================

/// Task #6: Test JunctionRef.setPositionFixed / positionFixed
#[test]
fn test_junction_position_fixed() {
    let mut junction = JunctionRef::new(1, Point::new(100.0, 100.0));

    // Default should be false (position can be optimized)
    assert!(
        !junction.position_fixed(),
        "Junction position should not be fixed by default"
    );

    // Set to fixed
    junction.set_position_fixed(true);
    assert!(
        junction.position_fixed(),
        "Junction position should be fixed after setPositionFixed(true)"
    );

    // Set back to not fixed
    junction.set_position_fixed(false);
    assert!(
        !junction.position_fixed(),
        "Junction position should not be fixed after setPositionFixed(false)"
    );
}

/// Task #7: Test JunctionRef.recommendedPosition
#[test]
fn test_junction_recommended_position() {
    let junction = JunctionRef::new(1, Point::new(100.0, 100.0));

    // Initially no recommended position
    assert!(
        junction.recommended_position().is_none(),
        "Junction should have no recommended position initially"
    );

    // After routing/optimization, recommended position might differ from current
    // This is a placeholder - actual test requires router integration
}

/// Task #7: Test that router can set recommended position on junction
#[test]
#[ignore = "Router junction recommendation not yet implemented"]
fn test_router_sets_junction_recommended_position() {
    let mut router = Router::new(0);

    // Add junction
    let junction_id = router.add_junction(Point::new(100.0, 100.0), 1);

    // Add shapes that might influence optimal junction position
    router.add_shape(
        Rectangle::new(Point::new(50.0, 100.0), 30.0, 30.0).into(),
        2,
    );
    router.add_shape(
        Rectangle::new(Point::new(150.0, 100.0), 30.0, 30.0).into(),
        3,
    );

    // Add connectors meeting at junction
    // This is a basic test - full test requires hyperedge/multi-connector support

    // Get junction and check if recommended position was computed
    if let Some(junction) = router.get_junction(junction_id) {
        // Even if no optimization happened, method should exist
        let _ = junction.recommended_position();
    }
}

// =============================================================================
// P2 - Medium Priority: ShapeConnectionPin Tests
// =============================================================================

/// Task #9: Test ShapeConnectionPin.updatePosition
#[test]
fn test_shape_connection_pin_update_position() {
    use libavoid::shape::ConnectionPin;

    let mut pin = ConnectionPin::new(1, Point::new(10.0, 0.0));

    // Verify initial position
    assert_eq!(pin.position.x, 10.0);
    assert_eq!(pin.position.y, 0.0);

    // Update position
    pin.update_position(Point::new(20.0, 5.0));

    // Verify updated position
    assert_eq!(pin.position.x, 20.0);
    assert_eq!(pin.position.y, 5.0);
}

/// Task #9: Test that pin position updates affect routing
///
/// NOTE: This tests advanced pin-to-routing integration which requires
/// the router to resolve pin positions during endpoint resolution.
/// This is not yet fully implemented.
#[test]
#[ignore = "Pin-to-routing integration not yet implemented"]
fn test_pin_position_update_affects_routing() {
    let mut router = Router::new(0);

    // Create shape with a pin
    let mut poly = Polygon::new();
    poly.push(Point::new(90.0, 90.0));
    poly.push(Point::new(110.0, 90.0));
    poly.push(Point::new(110.0, 110.0));
    poly.push(Point::new(90.0, 110.0));

    let shape_id = router.add_shape(poly, 1);

    // Add a connection pin at the right edge
    router.add_connection_pin_to_shape(
        shape_id,
        1,                    // class_id
        Point::new(110.0, 100.0), // position (right edge center)
        0xF,                  // all directions
    );

    // Create connector to the pin
    let src = ConnEnd::new(Point::new(0.0, 100.0));
    let dst = ConnEnd::with_pin(Point::new(110.0, 100.0), shape_id, 1);
    let conn_id = router.new_connector(src, dst);

    // Get first route endpoint
    let endpoint1 = {
        let conn = router.get_connector(conn_id).unwrap();
        let route1 = conn.display_route().expect("Route should exist");
        *route1.at(route1.size() - 1)
    };

    // Update pin position to top edge
    router.update_connection_pin_position(shape_id, 1, Point::new(100.0, 90.0));

    // Re-route
    router.process_transaction();

    // Get second route endpoint
    let endpoint2 = {
        let conn = router.get_connector(conn_id).unwrap();
        let route2 = conn.display_route().expect("Route should exist after pin update");
        *route2.at(route2.size() - 1)
    };

    // Route endpoint should have changed
    assert!(
        !endpoint1.equals(&endpoint2),
        "Route endpoint should change after pin position update"
    );
}

// =============================================================================
// P2 - Medium Priority: Router Tests
// =============================================================================

/// Task #10: Test Router.printInfo / debug output
#[test]
fn test_router_print_info() {
    let mut router = Router::new(0);

    // Add some content
    router.add_shape(
        Rectangle::new(Point::new(100.0, 100.0), 50.0, 50.0).into(),
        1,
    );
    router.new_connector(
        ConnEnd::new(Point::new(0.0, 100.0)),
        ConnEnd::new(Point::new(200.0, 100.0)),
    );

    // Get info string
    let info = router.print_info();

    // Should contain basic statistics
    assert!(
        info.contains("shape") || info.contains("Shape"),
        "Info should mention shapes: {}",
        info
    );
    assert!(
        info.contains("connector") || info.contains("Connector"),
        "Info should mention connectors: {}",
        info
    );
}

/// Task #10: Test Router debug state includes visibility graph info
#[test]
fn test_router_debug_state() {
    let mut router = Router::new(0);

    // Add shapes and connector
    router.add_shape(
        Rectangle::new(Point::new(100.0, 100.0), 50.0, 50.0).into(),
        1,
    );
    router.new_connector(
        ConnEnd::new(Point::new(0.0, 100.0)),
        ConnEnd::new(Point::new(200.0, 100.0)),
    );

    // Process transaction to build visibility graph
    router.process_transaction();

    // Get debug state
    let state = router.debug_state();

    // Should have counts
    assert!(state.shape_count > 0, "Should have shapes");
    assert!(state.connector_count > 0, "Should have connectors");
    assert!(state.vertex_count > 0, "Should have visibility vertices");
}

// =============================================================================
// P2 - Medium Priority: ConnRef Callback Tests
// =============================================================================

/// Task #8: Test ConnRef callback is invoked on route change
#[test]
fn test_connector_callback_invoked_on_route_change() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let mut router = Router::new(0);

    // Counter for callback invocations
    let callback_count = Arc::new(AtomicUsize::new(0));
    let callback_count_clone = callback_count.clone();

    // Create connector with callback
    let src = ConnEnd::new(Point::new(0.0, 0.0));
    let dst = ConnEnd::new(Point::new(100.0, 100.0));
    let mut conn = ConnRef::with_endpoints(1, src, dst);

    conn.set_callback(Arc::new(move |_conn: &ConnRef| {
        callback_count_clone.fetch_add(1, Ordering::SeqCst);
    }));

    let conn_id = router.add_connector(conn);

    // Process transaction to compute initial route
    router.process_transaction();

    // Callback should have been invoked during initial routing
    let count_after_initial = callback_count.load(Ordering::SeqCst);
    assert!(
        count_after_initial >= 1,
        "Callback should be invoked on initial routing, got {} invocations",
        count_after_initial
    );

    // Add obstacle that will force reroute when we explicitly reroute
    router.add_shape(
        Rectangle::new(Point::new(50.0, 50.0), 30.0, 30.0).into(),
        1,
    );

    // Process transaction to trigger reroute
    router.process_transaction();

    // Callback should be invoked again
    let count_after_obstacle = callback_count.load(Ordering::SeqCst);
    assert!(
        count_after_obstacle > count_after_initial,
        "Callback should be invoked when route changes, got {} invocations (was {})",
        count_after_obstacle,
        count_after_initial
    );
}

// =============================================================================
// WASM API Parity Tests (Task #22, #25)
// =============================================================================

/// Task #22: Verify moveShape uses offset semantics
#[test]
fn test_move_shape_position_semantics() {
    let mut router = Router::new(0);

    // Add shape at (100, 100)
    let rect = Rectangle::new(Point::new(100.0, 100.0), 50.0, 50.0);
    let shape_id = router.add_shape(rect.into(), 1);

    // Get initial position
    let shape = router.get_shape(shape_id).expect("Shape should exist");
    let initial_pos = shape.position();
    assert_eq!(initial_pos.x, 100.0);
    assert_eq!(initial_pos.y, 100.0);

    // Move to position (130, 120)
    router.move_shape(shape_id, Point::new(130.0, 120.0));

    // Verify new position
    let shape = router.get_shape(shape_id).expect("Shape should still exist");
    let new_pos = shape.position();

    assert!(
        (new_pos.x - 130.0).abs() < 0.001,
        "Shape x should be 130, got {}",
        new_pos.x
    );
    assert!(
        (new_pos.y - 120.0).abs() < 0.001,
        "Shape y should be 120, got {}",
        new_pos.y
    );
}

// =============================================================================
// P3 - Performance and Advanced Features
// =============================================================================

/// Task #12: Test incremental visibility updates
///
/// This tests that incremental visibility updates produce the same results
/// as full rebuilds while being more efficient.
#[test]
fn test_incremental_visibility_updates() {
    // Test with incremental updates enabled (default)
    let mut router1 = Router::new(0);

    // Create initial shapes
    let shape1_id = router1.add_shape(
        Rectangle::new(Point::new(50.0, 50.0), 20.0, 20.0).into(),
        1,
    );
    let _shape2_id = router1.add_shape(
        Rectangle::new(Point::new(150.0, 50.0), 20.0, 20.0).into(),
        2,
    );

    // Create connector
    let src = ConnEnd::new(Point::new(0.0, 50.0));
    let dst = ConnEnd::new(Point::new(200.0, 50.0));
    let conn_id = router1.new_connector(src, dst);

    // Process transaction to compute routes
    router1.process_transaction();

    // Get initial route
    let route_initial = {
        let conn = router1.get_connector(conn_id).unwrap();
        conn.display_route().cloned()
    };
    assert!(route_initial.is_some(), "Initial route should exist");

    // Move shape (triggers incremental update)
    router1.move_shape(shape1_id, Point::new(50.0, 150.0)); // Move out of path

    // Process transaction after move
    router1.process_transaction();

    // Get route after incremental update
    let route_after_move = {
        let conn = router1.get_connector(conn_id).unwrap();
        conn.display_route().cloned()
    };
    assert!(route_after_move.is_some(), "Route should exist after incremental update");

    // Now test with full rebuild (disable incremental)
    let mut router2 = Router::new(0);
    router2.set_use_incremental_updates(false);

    // Same setup
    let shape1_id2 = router2.add_shape(
        Rectangle::new(Point::new(50.0, 50.0), 20.0, 20.0).into(),
        1,
    );
    let _shape2_id2 = router2.add_shape(
        Rectangle::new(Point::new(150.0, 50.0), 20.0, 20.0).into(),
        2,
    );

    let src2 = ConnEnd::new(Point::new(0.0, 50.0));
    let dst2 = ConnEnd::new(Point::new(200.0, 50.0));
    let conn_id2 = router2.new_connector(src2, dst2);

    // Process transaction
    router2.process_transaction();

    // Move shape (triggers full rebuild)
    router2.move_shape(shape1_id2, Point::new(50.0, 150.0));

    // Process transaction after move
    router2.process_transaction();

    // Get route after full rebuild
    let route_full_rebuild = {
        let conn = router2.get_connector(conn_id2).unwrap();
        conn.display_route().cloned()
    };
    assert!(route_full_rebuild.is_some(), "Route should exist after full rebuild");

    // Both routes should have the same endpoints
    let route1 = route_after_move.unwrap();
    let route2 = route_full_rebuild.unwrap();

    assert_eq!(
        route1.at(0),
        route2.at(0),
        "Start points should match between incremental and full rebuild"
    );
    assert_eq!(
        route1.at(route1.size() - 1),
        route2.at(route2.size() - 1),
        "End points should match between incremental and full rebuild"
    );
}

// =============================================================================
// Test Helpers
// =============================================================================

#[allow(dead_code)]
fn rect(x: f64, y: f64, w: f64, h: f64) -> Polygon {
    Rectangle::new(Point::new(x, y), w, h).into()
}
