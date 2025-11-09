# libavoid-rust

A Rust port of **libavoid** - Fast, object-avoiding connector routing for interactive diagram editors.

## Overview

libavoid is a cross-platform library providing fast, object-avoiding connector routing for use in interactive diagram editors. This is a Rust implementation of the original C++ library by Michael Wybrow from Monash University.

### Features

- 🚀 **Fast incremental routing** - Efficient updates when shapes move
- 📐 **Multiple routing modes**:
  - Polyline routing (direct paths)
  - Orthogonal routing (rectilinear/Manhattan routing)
- 🎯 **Object avoidance** - Automatically routes around obstacles
- 🔄 **Transaction support** - Batch multiple operations for performance
- 📍 **Connection pins** - Attach connectors to specific points on shapes
- ⚙️ **Configurable parameters** - Fine-tune routing behavior

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
libavoid = "0.1.0"
```

## Quick Start

```rust
use libavoid::{Router, Point, Rectangle, ConnEnd, ConnType};

// Create a router
let mut router = Router::new(0);

// Add a shape (obstacle)
let rect = Rectangle::new(Point::new(100.0, 100.0), 80.0, 60.0);
let shape_id = router.add_shape(rect.into(), 1);

// Create a connector
let src = ConnEnd::new(Point::new(0.0, 0.0));
let dst = ConnEnd::new(Point::new(200.0, 200.0));
let conn_id = router.new_connector(src, dst);

// The connector is automatically routed around the shape!

// Get the routed path
if let Some(conn) = router.get_connector(conn_id) {
    if let Some(route) = conn.display_route() {
        for point in route.points() {
            println!("Point: ({}, {})", point.x, point.y);
        }
    }
}
```

## Examples

### Basic Routing

```rust
use libavoid::{Router, Point, Rectangle, ConnEnd};

let mut router = Router::new(0);

// Add obstacles
let rect1 = Rectangle::new(Point::new(50.0, 50.0), 80.0, 60.0);
router.add_shape(rect1.into(), 1);

// Create connector
let src = ConnEnd::new(Point::new(0.0, 0.0));
let dst = ConnEnd::new(Point::new(150.0, 150.0));
router.new_connector(src, dst);
```

### Orthogonal Routing

```rust
use libavoid::{Router, Point, ConnEnd, ConnType};

let mut router = Router::new(0);

let src = ConnEnd::new(Point::new(0.0, 100.0));
let dst = ConnEnd::new(Point::new(200.0, 100.0));
let conn_id = router.new_connector(src, dst);

// Set to orthogonal (rectilinear) routing
if let Some(conn) = router.get_connector_mut(conn_id) {
    conn.set_routing_type(ConnType::Orthogonal);
}
```

### Using Transactions

For better performance when making multiple changes:

```rust
use libavoid::{Router, ROUTER_FLAG_USE_TRANSACTIONS};

let mut router = Router::new(ROUTER_FLAG_USE_TRANSACTIONS);

// Make multiple changes
router.add_shape(shape1.into(), 1);
router.add_shape(shape2.into(), 2);
router.new_connector(src1, dst1);
router.new_connector(src2, dst2);

// Process all changes at once
router.process_transaction();
```

### Moving Shapes

```rust
// Move a shape to a new position
router.move_shape(shape_id, Point::new(150.0, 200.0));

// Connectors are automatically rerouted
```

### Routing Parameters

Customize routing behavior:

```rust
use libavoid::{Router, RoutingParameter};

let mut router = Router::new(0);

// Increase penalty for bends (prefer straighter routes)
router.set_routing_parameter(RoutingParameter::BendPenalty, 100.0);

// Adjust shape buffer distance
router.set_routing_parameter(RoutingParameter::ShapeBufferDistance, 10.0);
```

### Debug Visualization

Export to SVG for debugging:

```rust
router.output_instance_to_svg("output.svg")?;
```

## Running Examples

Run the included example:

```bash
cargo run --example simple_routing
```

This creates a `routing_example.svg` file showing the routed connectors.

## Core Concepts

### Router

The `Router` is the main entry point. It manages all shapes and connectors and performs routing calculations.

### Shapes

Shapes are obstacles that connectors must route around. They are defined by polygons.

### Connectors

Connectors (also called "connections" or "edges") are the lines that connect two endpoints and are automatically routed around shapes.

### Connection Endpoints

Each connector has a source and destination endpoint. Endpoints can be:
- Free points in space
- Attached to shapes
- Attached to specific connection pins on shapes

### Routing Types

- **PolyLine**: Direct paths with arbitrary angles
- **Orthogonal**: Only horizontal and vertical segments (Manhattan routing)

## Architecture

The library is organized into several modules:

- `geometry` - Core geometric types (Point, Polygon, Rectangle, etc.)
- `router` - Main routing engine and API
- `connector` - Connector definitions and routing
- `obstacle` - Obstacle/shape representation
- `shape` - Shape-specific functionality
- `visibility` - Visibility graph computation
- `graph` - Pathfinding algorithms (A*)
- `orthogonal` - Orthogonal routing algorithms

## Differences from C++ libavoid

This Rust port maintains the core algorithms and API design of the original library while adapting to Rust idioms:

- Uses Rust's ownership system instead of manual memory management
- Uses `Option` and `Result` instead of null pointers
- Uses trait objects for polymorphism
- Simplified some internal implementations while maintaining API compatibility

## Performance

The library is designed for interactive applications:

- Incremental updates when shapes move
- Visibility graph caching
- Transaction support for batch operations
- Efficient pathfinding with A* algorithm

## Original C++ Library

This is a port of the original libavoid C++ library:

- **Author**: Michael Wybrow, Monash University
- **Repository**: https://github.com/mjwybrow/adaptagrams
- **License**: LGPL 2.1
- **Research**: Based on peer-reviewed graph drawing research

## License

LGPL-2.1 (same as the original library)

## Contributing

Contributions are welcome! This is a port in progress and there are many opportunities to:

- Improve routing algorithms
- Add missing features from the C++ version
- Optimize performance
- Add more examples and documentation
- Write more comprehensive tests

## Acknowledgments

- Michael Wybrow for the original C++ libavoid library
- Monash University's Adaptive Diagrams and Documents lab

## Status

This is an initial port implementing the core functionality:

- ✅ Basic geometry types
- ✅ Router core
- ✅ Polyline routing
- ✅ Orthogonal routing
- ✅ Visibility graph
- ✅ A* pathfinding
- ✅ Shape management
- ✅ Connector management
- ✅ Transaction support
- ⚠️ Connection pins (basic support)
- ❌ Hyperedge routing
- ❌ Junction support
- ❌ Advanced orthogonal improvements
- ❌ Cluster support

## Resources

- [Original libavoid documentation](http://www.adaptagrams.org/documentation/libavoid.html)
- [Example gallery](examples/)
