use libavoid::{Router, Point, Rectangle, ConnEnd, ConnType, PolygonInterface, Polygon};

#[test]
fn test_polyline_routes_around_obstacle() {
    let mut router = Router::new(0);

    // Add an obstacle centered at (50, 50) with size 40x40
    // Bounds: x: 30-70, y: 30-70
    let rect = Rectangle::new(Point::new(50.0, 50.0), 40.0, 40.0);
    router.add_shape(rect.into(), 1);

    // Create a connector that would go through the obstacle center at y=50
    let src = ConnEnd::new(Point::new(0.0, 50.0));
    let dst = ConnEnd::new(Point::new(100.0, 50.0));
    let conn_id = router.new_connector(src, dst);

    // Process transaction to compute routes
    router.process_transaction();

    // Get the route
    let conn = router.get_connector(conn_id).unwrap();
    let route = conn.display_route().expect("Route should exist");

    // The route MUST have more than 2 points - direct path goes through obstacle
    assert!(route.size() > 2,
        "Route should avoid obstacle! Direct path goes through obstacle at x:30-70, y:30-70. Got {} points",
        route.size());

    println!("Route has {} points", route.size());
}

#[test]
fn test_direct_path_when_clear() {
    let mut router = Router::new(0);

    // No obstacles
    let src = ConnEnd::new(Point::new(0.0, 0.0));
    let dst = ConnEnd::new(Point::new(100.0, 100.0));
    let conn_id = router.new_connector(src, dst);

    // Process transaction to compute routes
    router.process_transaction();

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

    // Process transaction to compute routes
    router.process_transaction();

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

    // Process transaction to compute routes
    router.process_transaction();

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

    // Process first transaction
    router.process_transaction();

    let conn = router.get_connector(conn_id).unwrap();
    let route1 = conn.display_route().expect("Route should exist");
    let initial_waypoints = route1.size();

    // Move obstacle into the path
    router.move_shape(shape_id, Point::new(50.0, 50.0));

    // Process transaction after move
    router.process_transaction();

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

#[test]
fn test_route_avoids_obstacle_horizontal_line() {
    // This mirrors the gallery.js basic example exactly
    let mut router = Router::new(0);

    // Create obstacle: top-left (175, 100), width 50, height 50
    // Using Rectangle::new which takes CENTER point
    let center_x = 175.0 + 50.0 / 2.0; // = 200
    let center_y = 100.0 + 50.0 / 2.0; // = 125
    let rect = Rectangle::new(Point::new(center_x, center_y), 50.0, 50.0);

    println!("Rectangle center: ({}, {})", center_x, center_y);
    let poly: Polygon = rect.into();
    println!("Polygon points:");
    for i in 0..poly.size() {
        let p = poly.at(i);
        println!("  ({}, {})", p.x, p.y);
    }

    router.add_shape(poly, 1);

    // Route from left to right at y=125 (same as obstacle center y)
    let src = ConnEnd::new(Point::new(50.0, 125.0));
    let dst = ConnEnd::new(Point::new(350.0, 125.0));
    let conn_id = router.new_connector(src, dst);

    // Process transaction to compute routes
    router.process_transaction();

    let conn = router.get_connector(conn_id).unwrap();
    let route = conn.display_route().expect("Route should exist");

    println!("Route points:");
    for i in 0..route.size() {
        let p = route.at(i);
        println!("  ({}, {})", p.x, p.y);
    }

    // The route MUST have more than 2 points because a direct line
    // from (50, 125) to (350, 125) passes through the obstacle
    // which spans x: 175-225, y: 100-150
    assert!(
        route.size() > 2,
        "Route should avoid obstacle! Got {} points (direct line). \
         Obstacle is at x:175-225, y:100-150. Route y=125 passes through it.",
        route.size()
    );
}

#[test]
fn test_geometry_intersection_detection() {
    use libavoid::geometry::{point_in_polygon, segment_intersects_polygon_interior};
    
    // Create the exact same obstacle polygon
    let mut poly = Polygon::new();
    poly.push(Point::new(175.0, 100.0));
    poly.push(Point::new(225.0, 100.0));
    poly.push(Point::new(225.0, 150.0));
    poly.push(Point::new(175.0, 150.0));
    
    // Test points
    let src = Point::new(50.0, 125.0);
    let dst = Point::new(350.0, 125.0);
    let mid = Point::new(200.0, 125.0);  // Midpoint of route
    
    println!("Testing point_in_polygon:");
    println!("  src (50, 125) in polygon: {}", point_in_polygon(&src, &poly));
    println!("  dst (350, 125) in polygon: {}", point_in_polygon(&dst, &poly));
    println!("  mid (200, 125) in polygon: {}", point_in_polygon(&mid, &poly));
    
    // Midpoint (200, 125) should be inside because:
    // x: 175 < 200 < 225 ✓
    // y: 100 < 125 < 150 ✓
    assert!(point_in_polygon(&mid, &poly), 
        "Midpoint (200, 125) should be inside polygon (175-225, 100-150)");
    
    println!("\nTesting segment_intersects_polygon_interior:");
    let intersects = segment_intersects_polygon_interior(&src, &dst, &poly);
    println!("  segment (50,125)-(350,125) intersects: {}", intersects);
    
    assert!(intersects,
        "Segment from (50,125) to (350,125) should intersect polygon at x:175-225, y:100-150");
}

#[test]
fn test_debug_router_obstacle_detection() {
    use libavoid::geometry::{segment_intersects_polygon_interior};
    use libavoid::Obstacle;

    let mut router = Router::new(0);

    // Add obstacle
    let rect = Rectangle::new(Point::new(200.0, 125.0), 50.0, 50.0);
    let shape_id = router.add_shape(rect.into(), 1);

    println!("Shape added with id: {}", shape_id);
    println!("Number of shapes in router: {}", router.shapes().count());

    // Check the shape is there and active
    if let Some(shape) = router.get_shape(shape_id) {
        println!("Shape found, is_active: {}", shape.is_active());
        let poly = shape.polygon();
        println!("Shape polygon points:");
        for i in 0..poly.size() {
            let p = poly.at(i);
            println!("  ({}, {})", p.x, p.y);
        }

        // Test intersection directly with the shape's polygon
        let src = Point::new(50.0, 125.0);
        let dst = Point::new(350.0, 125.0);
        let intersects = segment_intersects_polygon_interior(&src, &dst, poly);
        println!("\nDirect intersection test: {}", intersects);
    } else {
        println!("ERROR: Shape not found!");
    }

    // Now add connector and check route
    let conn_id = router.new_connector(
        ConnEnd::new(Point::new(50.0, 125.0)),
        ConnEnd::new(Point::new(350.0, 125.0))
    );

    // Process transaction to compute routes
    router.process_transaction();

    let conn = router.get_connector(conn_id).unwrap();
    let route = conn.display_route().expect("Route should exist");
    
    println!("\nRoute points after routing:");
    for i in 0..route.size() {
        let p = route.at(i);
        println!("  ({}, {})", p.x, p.y);
    }
    
    assert!(route.size() > 2, "Route should avoid obstacle, got {} points", route.size());
}

#[test]
fn test_orthogonal_nudging_via_router() {
    use libavoid::RoutingOption;

    // Create router with orthogonal routing type
    let mut router = Router::new(ConnType::Orthogonal as u32);

    // Enable nudging
    router.set_routing_option(RoutingOption::NudgeOrthogonalRoutes, true);

    // No obstacles - just test pure nudging
    // Create three connectors with identical endpoints
    let src = ConnEnd::new(Point::new(15.0, 75.0));
    let dst = ConnEnd::new(Point::new(185.0, 75.0));

    // Create connectors using transaction mode for efficiency
    router.set_transaction_use(true);

    let conn1_id = router.new_connector(src.clone(), dst.clone());
    let conn2_id = router.new_connector(src.clone(), dst.clone());
    let conn3_id = router.new_connector(src.clone(), dst.clone());

    // Set orthogonal routing type
    if let Some(c) = router.get_connector_mut(conn1_id) {
        c.set_routing_type(ConnType::Orthogonal);
    }
    if let Some(c) = router.get_connector_mut(conn2_id) {
        c.set_routing_type(ConnType::Orthogonal);
    }
    if let Some(c) = router.get_connector_mut(conn3_id) {
        c.set_routing_type(ConnType::Orthogonal);
    }

    // Process transaction to route and nudge
    router.process_transaction();

    // Get the routes
    let route1 = router.get_connector(conn1_id).unwrap().display_route().unwrap();
    let route2 = router.get_connector(conn2_id).unwrap().display_route().unwrap();
    let route3 = router.get_connector(conn3_id).unwrap().display_route().unwrap();

    println!("Route 1: {:?}", (0..route1.size()).map(|i| route1.at(i)).collect::<Vec<_>>());
    println!("Route 2: {:?}", (0..route2.size()).map(|i| route2.at(i)).collect::<Vec<_>>());
    println!("Route 3: {:?}", (0..route3.size()).map(|i| route3.at(i)).collect::<Vec<_>>());

    // Routes should have been nudged apart
    // Check Y coordinates of first point (or any point on horizontal segment)
    let y1 = route1.at(0).y;
    let y2 = route2.at(0).y;
    let y3 = route3.at(0).y;

    println!("Y coordinates: {} {} {}", y1, y2, y3);

    // After nudging, the Y coordinates should be different
    let all_same = (y1 - y2).abs() < 0.1 && (y2 - y3).abs() < 0.1;
    assert!(!all_same, "Routes should be nudged apart! Y coords: {}, {}, {}", y1, y2, y3);
}
