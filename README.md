# libavoid-rust

A Rust port of **libavoid** - Fast, object-avoiding connector routing for interactive diagram editors. Includes WebAssembly bindings for JavaScript/TypeScript.

## Overview

libavoid is a cross-platform library providing fast, object-avoiding connector routing for use in interactive diagram editors. This is a Rust implementation of the original C++ library by Michael Wybrow from Monash University.

### Features

- **Fast incremental routing** - Efficient updates when shapes move
- **Multiple routing modes**:
  - Polyline routing (direct diagonal paths)
  - Orthogonal routing (rectilinear/Manhattan routing)
- **Object avoidance** - Automatically routes around obstacles
- **Transaction support** - Batch multiple operations for performance
- **Connection pins** - Attach connectors to specific points on shapes
- **Configurable parameters** - Shape buffer distance, nudging, penalties
- **WebAssembly support** - Use in browsers and Node.js

## Installation

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
libavoid = "0.1.0"
```

### JavaScript/TypeScript (WASM)

Build the WASM module:

```bash
# Install wasm-pack if you haven't already
cargo install wasm-pack

# Build for web
wasm-pack build --target web --features wasm

# Or build for Node.js
wasm-pack build --target nodejs --features wasm
```

## Quick Start

### Rust Usage

```rust
use libavoid::{Router, Point, Rectangle, ConnRef, ConnEnd, ConnType, PolygonInterface};

fn main() {
    // Create a router with orthogonal routing
    let mut router = Router::new(ConnType::Orthogonal as u32);
    router.set_transaction_use(true);

    // Add an obstacle
    let obstacle = Rectangle::new(Point::new(100.0, 100.0), 80.0, 60.0);
    router.add_shape(obstacle.into(), 1);

    // Create a connector
    let src = ConnEnd::new(Point::new(50.0, 100.0));
    let dst = ConnEnd::new(Point::new(200.0, 100.0));
    let mut conn = ConnRef::with_endpoints(1, src, dst);
    conn.set_routing_type(ConnType::Orthogonal);
    router.add_connector(conn);

    // Process routing
    router.process_transaction();

    // Get the route
    if let Some(conn) = router.get_connector(1) {
        if let Some(route) = conn.display_route() {
            for i in 0..route.size() {
                let pt = route.at(i);
                println!("Point: ({}, {})", pt.x, pt.y);
            }
        }
    }
}
```

### JavaScript Usage (Browser)

```javascript
import init, { Router, Point, Rectangle, ConnRef, ConnEnd, ShapeRef } from './pkg/libavoid.js';

const ORTHOGONAL_ROUTING = 2;

async function main() {
  await init();

  // Create router
  const router = new Router(ORTHOGONAL_ROUTING);

  // Add obstacle
  const rect = new Rectangle(new Point(100, 100), 80, 60);
  const shape = new ShapeRef(router, rect.toPolygon());
  router.addShape(shape);

  // Create connector
  const srcEnd = new ConnEnd(new Point(50, 100));
  const dstEnd = new ConnEnd(new Point(200, 100));
  const conn = ConnRef.createWithEndpoints(router, srcEnd, dstEnd);
  conn.setRoutingType(ORTHOGONAL_ROUTING);
  router.addConnector(conn);

  // Process and get route
  router.processTransaction();
  const route = router.getConnectorRoute(conn.id());

  for (let i = 0; i < route.size(); i++) {
    const pt = route.at(i);
    console.log(`Point: (${pt.x}, ${pt.y})`);
  }
}
```

## Examples

### Running Examples

```bash
# Rust example
cargo run --example simple_routing

# Web examples (start server from examples directory)
cd examples
python3 -m http.server 8080
# Then open http://localhost:8080/web/gallery.html
```

### Available Examples

| Example | Description | Location |
|---------|-------------|----------|
| **Simple Routing** | Basic Rust connector routing | `examples/simple_routing.rs` |
| **Web Gallery** | Interactive demo of all routing features | `examples/web/gallery.html` |
| **Zombie Chase** | Game demo using pathfinding | `examples/zombies/index.html` |
| **Node.js** | Server-side routing example | `examples/node/main.mjs` |

### Web Gallery Features

The web gallery (`examples/web/gallery.html`) demonstrates:

1. **Basic Polyline Routing** - Direct paths around obstacles
2. **Orthogonal Routing** - Manhattan-style H/V only paths
3. **Multiple Obstacles** - Routing through complex layouts
4. **Shape Operations** - Add, move, delete shapes dynamically
5. **Transaction Batching** - Performance comparison
6. **Multiple Connectors** - Many simultaneous routes
7. **Crossing Detection** - Visualize route intersections
8. **Interactive Demo** - Click to add shapes and connectors
9. **Route Nudging** - Automatic separation of overlapping routes
10. **Options Comparison** - Side-by-side parameter effects

### Zombie Chase Game

A playable game demonstrating real-time pathfinding:

- Zombies chase the player using libavoid routing
- Multiple rounds with increasing difficulty
- Shows polyline routing for direct pursuit paths
- Toggle debug view to see connector paths

## Routing Parameters

Configure routing behavior:

```rust
use libavoid::RoutingParameter;

// Distance to keep between routes and obstacles
router.set_routing_parameter(RoutingParameter::ShapeBufferDistance, 10.0);

// Distance between parallel route segments (nudging)
router.set_routing_parameter(RoutingParameter::IdealNudgingDistance, 10.0);

// Penalty for route segments (affects path complexity)
router.set_routing_parameter(RoutingParameter::SegmentPenalty, 10.0);

// Penalty for route crossings
router.set_routing_parameter(RoutingParameter::CrossingPenalty, 200.0);
```

## Routing Options

Enable/disable routing features:

```rust
use libavoid::RoutingOption;

// Enable nudging (separates overlapping orthogonal routes)
router.set_routing_option(RoutingOption::NudgeOrthogonalRoutes, true);

// Penalize shared path segments
router.set_routing_option(RoutingOption::PenaliseOrthogonalSharedPaths, true);
```

## Testing

```bash
# Run Rust tests
cargo test

# Run JavaScript tests
cd js-tests
npm install
npm run build:wasm  # Build WASM for tests
npm test
```

The JS test suite includes:
- Unit tests for individual classes
- API compatibility tests with libavoid-js
- Behavioral parity tests
- Integration tests

## Building

### Rust Library

```bash
cargo build --release
```

### WASM Module

```bash
# For web (ES modules)
wasm-pack build --target web --features wasm

# For Node.js
wasm-pack build --target nodejs --features wasm

# For bundlers (webpack, etc.)
wasm-pack build --target bundler --features wasm
```

### Running Benchmarks

```bash
cargo bench
```

## Architecture

The library is organized into modules:

| Module | Description |
|--------|-------------|
| `geometry` | Core types: Point, Polygon, Rectangle, Box |
| `router` | Main routing engine and API |
| `connector` | Connector definitions and routing |
| `shape` | Shape/obstacle representation |
| `visibility` | Visibility graph computation |
| `orthogonal` | Orthogonal routing with visibility graphs |
| `vpsc` | Variable Placement with Separation Constraints (nudging) |
| `wasm` | WebAssembly bindings |

## Routing Algorithms

### Polyline Routing
Uses visibility graphs to find shortest paths that avoid obstacles. Paths can have any angle.

### Orthogonal Routing
Generates rectilinear (horizontal/vertical only) paths using:
1. Orthogonal visibility graph generation
2. A* pathfinding on the visibility graph
3. VPSC-based nudging to separate overlapping segments

## Original C++ Library

This is a port of the original libavoid C++ library:

- **Author**: Michael Wybrow, Monash University
- **Repository**: https://github.com/mjwybrow/adaptagrams
- **License**: LGPL 2.1
- **Research**: Based on peer-reviewed graph drawing research

## License

LGPL-2.1 (same as the original library)

## Status

Core functionality implemented:

- [x] Basic geometry types
- [x] Router core
- [x] Polyline routing
- [x] Orthogonal routing
- [x] Visibility graph
- [x] A* pathfinding
- [x] Shape management
- [x] Connector management
- [x] Transaction support
- [x] Connection pins (basic)
- [x] Route nudging (VPSC)
- [x] WebAssembly bindings
- [ ] Hyperedge routing
- [ ] Junction support
- [ ] Cluster support

## Contributing

Contributions welcome! Areas of interest:
- Performance optimizations
- Missing features from C++ version
- Additional examples and documentation
- Test coverage improvements

## Acknowledgments

- Michael Wybrow for the original C++ libavoid library
- Monash University's Adaptive Diagrams and Documents lab
