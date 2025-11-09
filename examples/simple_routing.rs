//! Simple connector routing example
//!
//! This example demonstrates basic usage of libavoid:
//! - Creating a router
//! - Adding shapes (obstacles)
//! - Creating connectors between points
//! - Routing connectors around obstacles
//! - Outputting the result to SVG

use libavoid::{Router, Point, Rectangle, ConnEnd, ConnType, PolygonInterface};

fn main() {
    println!("libavoid-rust: Simple Routing Example");
    println!("======================================\n");

    // Create a router instance
    let mut router = Router::new(0);

    // Create some shapes (obstacles)
    println!("Adding shapes...");

    // Shape 1: Rectangle at (50, 50)
    let rect1 = Rectangle::new(Point::new(50.0, 50.0), 80.0, 60.0);
    let shape1_id = router.add_shape(rect1.into(), 1);
    println!("  Shape {}: Rectangle at (50, 50), size 80x60", shape1_id);

    // Shape 2: Rectangle at (200, 100)
    let rect2 = Rectangle::new(Point::new(200.0, 100.0), 100.0, 80.0);
    let shape2_id = router.add_shape(rect2.into(), 2);
    println!("  Shape {}: Rectangle at (200, 100), size 100x80", shape2_id);

    // Shape 3: Rectangle at (150, 250)
    let rect3 = Rectangle::new(Point::new(150.0, 250.0), 70.0, 70.0);
    let shape3_id = router.add_shape(rect3.into(), 3);
    println!("  Shape {}: Rectangle at (150, 250), size 70x70\n", shape3_id);

    // Create connectors
    println!("Adding connectors...");

    // Connector 1: From bottom-left to top-right (polyline)
    let src1 = ConnEnd::new(Point::new(0.0, 0.0));
    let dst1 = ConnEnd::new(Point::new(300.0, 300.0));
    let conn1_id = router.new_connector(src1, dst1);
    println!("  Connector {}: Polyline from (0,0) to (300,300)", conn1_id);

    // Connector 2: From left to right (orthogonal)
    let src2 = ConnEnd::new(Point::new(0.0, 150.0));
    let dst2 = ConnEnd::new(Point::new(350.0, 150.0));
    let conn2_id = router.new_connector(src2, dst2);

    // Set connector 2 to use orthogonal routing
    if let Some(conn) = router.get_connector_mut(conn2_id) {
        conn.set_routing_type(ConnType::Orthogonal);
    }
    println!("  Connector {}: Orthogonal from (0,150) to (350,150)", conn2_id);

    // Connector 3: Vertical connection (orthogonal)
    let src3 = ConnEnd::new(Point::new(100.0, 0.0));
    let dst3 = ConnEnd::new(Point::new(100.0, 350.0));
    let conn3_id = router.new_connector(src3, dst3);

    if let Some(conn) = router.get_connector_mut(conn3_id) {
        conn.set_routing_type(ConnType::Orthogonal);
    }
    println!("  Connector {}: Orthogonal from (100,0) to (100,350)\n", conn3_id);

    // Display routing information
    println!("Routing results:");
    for conn in router.connectors() {
        if let Some(route) = conn.display_route() {
            println!("  Connector {}: {} segments, {} points",
                conn.id(),
                route.size().saturating_sub(1),
                route.size()
            );
        }
    }

    // Output to SVG
    let svg_path = "routing_example.svg";
    match router.output_instance_to_svg(svg_path) {
        Ok(_) => println!("\n✓ SVG output written to: {}", svg_path),
        Err(e) => println!("\n✗ Error writing SVG: {}", e),
    }

    println!("\nExample complete!");
}
