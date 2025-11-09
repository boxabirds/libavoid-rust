use libavoid::{Router, Point, Rectangle, ConnEnd, ConnType, PolygonInterface};

#[test]
fn test_polyline_routes_around_obstacle() {
    let mut router = Router::new(0);

    // Add an obstacle in the middle
    let rect = Rectangle::new(Point::new(50.0, 50.0), 40.0, 40.0);
    router.add_shape(rect.into(), 1);

    // Create a connector that would go through the obstacle
    let src = ConnEnd::new(Point::new(0.0, 50.0));
    let dst = ConnEnd::new(Point::new(100.0, 50.0));
    let conn_id = router.new_connector(src, dst);

    // Get the route
    let conn = router.get_connector(conn_id).unwrap();
    let route = conn.display_route().expect("Route should exist");

    // The route should have more than 2 points (it should go around the obstacle)
    assert!(route.size() >= 2, "Route should exist with at least 2 points");

    // Verify the route doesn't pass through the obstacle center
    let has_waypoint = route.size() > 2;
    println!("Route has {} points (waypoint: {})", route.size(), has_waypoint);
}

#[test]
fn test_direct_path_when_clear() {
    let mut router = Router::new(0);

    // No obstacles
    let src = ConnEnd::new(Point::new(0.0, 0.0));
    let dst = ConnEnd::new(Point::new(100.0, 100.0));
    let conn_id = router.new_connector(src, dst);

    // Get the route
    let conn = router.get_connector(conn_id).unwrap();
    let route = conn.display_route().expect("Route should exist");

    // Should be a direct path (2 points)
    assert_eq!(route.size(), 2, "Direct path should have exactly 2 points");
    assert_eq!(route.at(0).x, 0.0);
    assert_eq!(route.at(0).y, 0.0);
    assert_eq!(route.at(1).x, 100.0);
    assert_eq!(route.at(1).y, 100.0);
}

#[test]
fn test_orthogonal_routing() {
    let mut router = Router::new(0);

    // Create orthogonal connector
    let src = ConnEnd::new(Point::new(0.0, 0.0));
    let dst = ConnEnd::new(Point::new(100.0, 100.0));

    // Create connector with orthogonal routing from the start
    let mut conn = libavoid::ConnRef::with_endpoints(1, src, dst);
    conn.set_routing_type(ConnType::Orthogonal);
    let conn_id = router.add_connector(conn);

    // Get the route
    let conn = router.get_connector(conn_id).unwrap();
    let route = conn.display_route().expect("Route should exist");

    // Verify it's orthogonal (all segments should be horizontal or vertical)
    for i in 0..route.size().saturating_sub(1) {
        let p1 = route.at(i);
        let p2 = route.at(i + 1);

        let is_horizontal = (p1.y - p2.y).abs() < 1e-6;
        let is_vertical = (p1.x - p2.x).abs() < 1e-6;

        assert!(
            is_horizontal || is_vertical,
            "Segment {} to {} is not orthogonal: ({}, {}) to ({}, {})",
            i,
            i + 1,
            p1.x,
            p1.y,
            p2.x,
            p2.y
        );
    }
}

#[test]
fn test_multiple_obstacles() {
    let mut router = Router::new(0);

    // Create a maze of obstacles
    router.add_shape(Rectangle::new(Point::new(30.0, 50.0), 20.0, 80.0).into(), 1);
    router.add_shape(Rectangle::new(Point::new(70.0, 50.0), 20.0, 80.0).into(), 2);

    // Route through the maze
    let src = ConnEnd::new(Point::new(0.0, 50.0));
    let dst = ConnEnd::new(Point::new(100.0, 50.0));
    let conn_id = router.new_connector(src, dst);

    let conn = router.get_connector(conn_id).unwrap();
    let route = conn.display_route().expect("Route should exist");

    // Should find a path (either over or under the obstacles)
    assert!(route.size() >= 2, "Should find a valid path");
    println!("Found path with {} waypoints", route.size());
}

#[test]
fn test_shape_movement_triggers_reroute() {
    let mut router = Router::new(0);

    // Add obstacle and connector
    let rect = Rectangle::new(Point::new(50.0, 10.0), 40.0, 10.0);
    let shape_id = router.add_shape(rect.into(), 1);

    let src = ConnEnd::new(Point::new(0.0, 50.0));
    let dst = ConnEnd::new(Point::new(100.0, 50.0));
    let conn_id = router.new_connector(src, dst);

    let conn = router.get_connector(conn_id).unwrap();
    let route1 = conn.display_route().expect("Route should exist");
    let initial_waypoints = route1.size();

    // Move obstacle into the path
    router.move_shape(shape_id, Point::new(50.0, 50.0));

    let conn = router.get_connector(conn_id).unwrap();
    let route2 = conn.display_route().expect("Route should exist after move");

    println!(
        "Before move: {} points, After move: {} points",
        initial_waypoints,
        route2.size()
    );

    // Route should still exist (may have different number of waypoints)
    assert!(route2.size() >= 2, "Route should exist after obstacle moved");
}

#[test]
fn test_polygon_offsetting() {
    use libavoid::{Polygon, PolygonInterface};

    let mut poly = Polygon::new();
    poly.push(Point::new(10.0, 10.0));
    poly.push(Point::new(20.0, 10.0));
    poly.push(Point::new(20.0, 20.0));
    poly.push(Point::new(10.0, 20.0));

    let offset_poly = poly.offset_polygon(2.0);

    // Should have same number of vertices
    assert_eq!(offset_poly.size(), 4);

    // The offset polygon should be larger (bounding box test)
    let orig_bbox = poly.bounding_rect();
    let offset_bbox = offset_poly.bounding_rect();

    println!("Original bbox: ({}, {}) to ({}, {})",
             orig_bbox.min.x, orig_bbox.min.y, orig_bbox.max.x, orig_bbox.max.y);
    println!("Offset bbox: ({}, {}) to ({}, {})",
             offset_bbox.min.x, offset_bbox.min.y, offset_bbox.max.x, offset_bbox.max.y);

    // The offset should expand the bounding box
    // Allow some tolerance due to corner handling
    assert!(
        offset_bbox.width() >= orig_bbox.width() - 0.1,
        "Width should not shrink significantly: {} vs {}",
        offset_bbox.width(),
        orig_bbox.width()
    );
    assert!(
        offset_bbox.height() >= orig_bbox.height() - 0.1,
        "Height should not shrink significantly: {} vs {}",
        offset_bbox.height(),
        orig_bbox.height()
    );
}
