//! Performance benchmarks for libavoid-rust
//!
//! Run with: cargo bench

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use libavoid::{Router, Point, Polygon, ConnEnd, ConnType, Rectangle, ROUTER_FLAG_USE_TRANSACTIONS};

/// Helper to create a rectangle polygon
fn rect(x: f64, y: f64, w: f64, h: f64) -> Polygon {
    Rectangle::new(Point::new(x, y), w, h).into()
}

// ============================================================================
// Basic Routing Benchmarks
// ============================================================================

fn bench_routing(c: &mut Criterion) {
    let mut group = c.benchmark_group("routing");

    // Direct path (no obstacles)
    group.bench_function("direct_path", |b| {
        b.iter(|| {
            let mut router = Router::new(0);
            let src = ConnEnd::new(Point::new(0.0, 0.0));
            let dst = ConnEnd::new(Point::new(100.0, 100.0));
            router.new_connector(black_box(src), black_box(dst));
        });
    });

    // Single obstacle
    group.bench_function("single_obstacle", |b| {
        b.iter(|| {
            let mut router = Router::new(0);
            router.add_shape(rect(100.0, 100.0, 50.0, 50.0), 1);

            let src = ConnEnd::new(Point::new(50.0, 125.0));
            let dst = ConnEnd::new(Point::new(200.0, 125.0));
            router.new_connector(black_box(src), black_box(dst));
        });
    });

    // Multiple obstacles
    for n in [5, 10, 20] {
        group.bench_with_input(BenchmarkId::new("obstacles", n), &n, |b, &n| {
            b.iter(|| {
                let mut router = Router::new(0);

                for i in 0..n {
                    let x = (i % 5 * 80) as f64;
                    let y = (i / 5 * 80 + 50) as f64;
                    router.add_shape(rect(x, y, 30.0, 30.0), (i + 1) as u32);
                }

                let src = ConnEnd::new(Point::new(0.0, 100.0));
                let dst = ConnEnd::new(Point::new(400.0, 200.0));
                router.new_connector(black_box(src), black_box(dst));
            });
        });
    }

    group.finish();
}

// ============================================================================
// Transaction Benchmarks
// ============================================================================

fn bench_transactions(c: &mut Criterion) {
    let mut group = c.benchmark_group("transactions");

    for n_connectors in [5, 10, 20] {
        group.bench_with_input(
            BenchmarkId::new("batch", n_connectors),
            &n_connectors,
            |b, &n| {
                b.iter(|| {
                    let mut router = Router::new(ROUTER_FLAG_USE_TRANSACTIONS);

                    for i in 0..5 {
                        router.add_shape(rect((i * 100) as f64, 100.0, 40.0, 40.0), (i + 1) as u32);
                    }

                    for i in 0..n {
                        let src = ConnEnd::new(Point::new((i * 50) as f64, 50.0));
                        let dst = ConnEnd::new(Point::new((i * 50) as f64, 200.0));
                        router.new_connector(src, dst);
                    }

                    router.process_transaction();
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("immediate", n_connectors),
            &n_connectors,
            |b, &n| {
                b.iter(|| {
                    let mut router = Router::new(0);

                    for i in 0..5 {
                        router.add_shape(rect((i * 100) as f64, 100.0, 40.0, 40.0), (i + 1) as u32);
                    }

                    for i in 0..n {
                        let src = ConnEnd::new(Point::new((i * 50) as f64, 50.0));
                        let dst = ConnEnd::new(Point::new((i * 50) as f64, 200.0));
                        router.new_connector(src, dst);
                    }
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// Orthogonal Routing Benchmarks
// ============================================================================

fn bench_orthogonal(c: &mut Criterion) {
    let mut group = c.benchmark_group("orthogonal");

    group.bench_function("simple", |b| {
        b.iter(|| {
            let mut router = Router::new(0);

            let src = ConnEnd::new(Point::new(0.0, 0.0));
            let dst = ConnEnd::new(Point::new(100.0, 100.0));

            let conn = libavoid::ConnRef::with_type(1, src, dst, ConnType::Orthogonal);
            router.add_connector(conn);
        });
    });

    group.bench_function("with_obstacles", |b| {
        b.iter(|| {
            let mut router = Router::new(0);

            router.add_shape(rect(50.0, 50.0, 30.0, 30.0), 1);
            router.add_shape(rect(100.0, 80.0, 30.0, 30.0), 2);

            let src = ConnEnd::new(Point::new(0.0, 60.0));
            let dst = ConnEnd::new(Point::new(150.0, 100.0));

            let conn = libavoid::ConnRef::with_type(1, src, dst, ConnType::Orthogonal);
            router.add_connector(conn);
        });
    });

    group.finish();
}

// ============================================================================
// Geometry Benchmarks
// ============================================================================

fn bench_geometry(c: &mut Criterion) {
    use libavoid::geometry::{count_route_crossings, point_in_polygon};

    let mut group = c.benchmark_group("geometry");

    // Crossing detection
    let mut route1 = Polygon::new();
    for i in 0..10 {
        route1.push(Point::new((i * 20) as f64, (i % 2 * 50) as f64));
    }

    let mut route2 = Polygon::new();
    for i in 0..10 {
        route2.push(Point::new((i * 20) as f64, 25.0 + (i % 2 * 50) as f64));
    }

    group.bench_function("crossing_detection", |b| {
        b.iter(|| count_route_crossings(black_box(&route1), black_box(&route2)));
    });

    // Point in polygon
    let mut polygon = Polygon::new();
    polygon.push(Point::new(0.0, 0.0));
    polygon.push(Point::new(100.0, 0.0));
    polygon.push(Point::new(100.0, 100.0));
    polygon.push(Point::new(50.0, 150.0));
    polygon.push(Point::new(0.0, 100.0));

    let test_point = Point::new(50.0, 50.0);

    group.bench_function("point_in_polygon", |b| {
        b.iter(|| point_in_polygon(black_box(&test_point), black_box(&polygon)));
    });

    group.finish();
}

// ============================================================================
// Visibility Graph Benchmarks
// ============================================================================

fn bench_visibility_graph(c: &mut Criterion) {
    let mut group = c.benchmark_group("visibility_graph");

    for n_shapes in [10, 25, 50] {
        group.bench_with_input(BenchmarkId::new("construction", n_shapes), &n_shapes, |b, &n| {
            b.iter(|| {
                let mut router = Router::new(0);

                for i in 0..n {
                    let x = (i % 5 * 80) as f64;
                    let y = (i / 5 * 80) as f64;
                    router.add_shape(rect(x, y, 30.0, 30.0), (i + 1) as u32);
                }
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_routing,
    bench_transactions,
    bench_orthogonal,
    bench_geometry,
    bench_visibility_graph,
);

criterion_main!(benches);
