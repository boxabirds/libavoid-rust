# Orthogonal Visibility Graph: Verbatim Algorithmic Port

**Date:** 2024-11-24
**Attempt:** #4
**Goal:** Port C++ libavoid's orthogonal visibility graph algorithm verbatim

## Problem Statement

The current Rust implementation uses grid-based A* routing (`GRID_RESOLUTION = 4.0`) which produces zigzag paths with many unnecessary bends. Example:

```
Current (grid A*): 12 points with multiple small steps
  (30,90) -> (110,90) -> (110,86) -> (118,86) -> (118,82) -> (146,82) -> ...

Expected (visibility graph): 6 points with clean path
  (30,90) -> (150,90) -> (150,76) -> (250,76) -> (250,90) -> (370,90)
```

## C++ Algorithm Overview

The C++ libavoid uses a **scanline sweep algorithm** to build an orthogonal visibility graph. This is fundamentally different from grid-based A* - instead of exploring a grid cell by cell, it:

1. Creates visibility segments by sweeping across the plane
2. Builds a sparse graph where vertices are obstacle corners and connector endpoints
3. Routes using A* on this sparse graph

### Source Files Reference

| C++ File | Purpose |
|----------|---------|
| `orthogonal.cpp:1730-1960` | `generateStaticOrthogonalVisGraph()` - main entry point |
| `orthogonal.cpp:1368-1543` | `processEventVert()` - vertical sweep processing |
| `orthogonal.cpp:1551-1715` | `processEventHori()` - horizontal sweep processing |
| `scanline.cpp` | Node, Event, and scanline data structures |
| `scanline.h` | ShiftSegment interface and EventType enum |
| `vertices.h:115-172` | VertInf - vertex in visibility graph |
| `graph.h:46-106` | EdgeInf - edge in visibility graph |
| `makepath.cpp` | A* routing on visibility graph |

## Algorithm Detail

### Phase 1: Event Generation

```
For each obstacle:
    Create Open event at obstacle.min_y with (min_x, max_x)
    Create Close event at obstacle.max_y with (min_x, max_x)

For each connector endpoint:
    Create ConnPoint event at endpoint.y with endpoint.x

Sort events by position (y for vertical sweep, x for horizontal sweep)
```

### Phase 2: Vertical Sweep (creates horizontal segments)

```cpp
// C++ ref: generateStaticOrthogonalVisGraph() lines 1811-1854
NodeSet scanline;  // Sorted by x-position
SegmentList segments;

for each event in sorted_events:
    if event.type == Open:
        // Add obstacle to scanline
        insert_to_scanline(event.node)

    if event.type == Open or Close:
        // Create horizontal visibility segments at shape edge
        lineY = (event.type == Open) ? shape.min_y : shape.max_y

        // Find visibility limits by scanning above/below in scanline
        minLimit = first_obstacle_above(scanline, shape.min_x)
        maxLimit = first_obstacle_below(scanline, shape.max_x)

        // Create segments:
        // 1. From minLimit to shape.min_x (if visible)
        // 2. Along shape edge from shape.min_x to shape.max_x
        // 3. From shape.max_x to maxLimit (if visible)

        // Insert vertices at shape corners
        vI1 = new VertInf(shape.min_x, lineY)  // Left corner
        vI2 = new VertInf(shape.max_x, lineY)  // Right corner

    if event.type == ConnPoint:
        // Create horizontal segments from connector endpoint
        minLimit = first_point_above(scanline)
        maxLimit = first_point_below(scanline)

        if connector.directions & LEFT:
            segments.insert(minLimit, endpoint.x, endpoint.y)
        if connector.directions & RIGHT:
            segments.insert(endpoint.x, maxLimit, endpoint.y)

    if event.type == Close:
        // Remove obstacle from scanline
        remove_from_scanline(event.node)
```

### Phase 3: Horizontal Sweep (creates vertical segments)

Same as Phase 2 but sweeping in X direction to create vertical visibility segments.

### Phase 4: Segment Merging and Edge Creation

```cpp
// C++ ref: orthogonal.cpp LineSegment class
// Segments with same position are merged
segments.sort()

for each segment in segments:
    // Create vertices at segment endpoints
    // Create edges between adjacent vertices on segment
    for i in 0..segment.vertInfs.size()-1:
        v1 = segment.vertInfs[i]
        v2 = segment.vertInfs[i+1]
        new EdgeInf(v1, v2, orthogonal=true)
```

### Phase 5: A* Routing on Visibility Graph

```cpp
// C++ ref: makepath.cpp
// Standard A* but on sparse visibility graph
// Uses orthogVisList (orthogonal edges) not visList (polyline edges)
```

## Data Structures

### Node (Scanline Element)

```rust
/// C++ ref: scanline.h:78-104
struct ScanlineNode {
    /// Obstacle reference (None for connector points)
    obstacle_id: Option<u32>,
    /// Connector vertex (None for obstacles)
    conn_vertex: Option<u32>,
    /// Position along sweep axis
    pos: f64,
    /// Bounding box on perpendicular axis
    perp_min: f64,
    perp_max: f64,
    /// Pointers to neighbors in scanline
    first_above: Option<usize>,
    first_below: Option<usize>,
}
```

### Event

```rust
/// C++ ref: scanline.h:107-124
enum EventType {
    Open = 1,     // Shape edge opens
    SegOpen = 2,  // Segment opens (for nudging)
    ConnPoint = 3, // Connector endpoint
    SegClose = 4, // Segment closes
    Close = 5,    // Shape edge closes
}

struct Event {
    event_type: EventType,
    node_idx: usize,  // Index into nodes array
    pos: f64,         // Position along sweep axis
}
```

### LineSegment

```rust
/// C++ ref: orthogonal.cpp LineSegment class (around line 1100)
struct LineSegment {
    /// Start position on parallel axis
    begin: f64,
    /// End position on parallel axis
    finish: f64,
    /// Position on perpendicular axis
    pos: f64,
    /// Is this a horizontal segment?
    is_horizontal: bool,
    /// Vertices on this segment, sorted by position
    vert_infs: BTreeSet<PosVertInf>,
}
```

### SegmentListWrapper

```rust
/// C++ ref: orthogonal.cpp SegmentListWrapper class
/// Manages segment merging during sweep
struct SegmentListWrapper {
    segments: Vec<LineSegment>,
    /// Map from position to segment index for O(1) lookup
    pos_to_segment: HashMap<OrderedFloat<f64>, usize>,
}

impl SegmentListWrapper {
    /// Insert or merge a segment at the given position
    /// C++ ref: SegmentListWrapper::insert()
    fn insert(&mut self, segment: LineSegment) -> &mut LineSegment {
        // If segment at same position exists, merge
        // Otherwise insert new segment
    }
}
```

## Implementation Plan

### Step 1: Port Data Structures

Port these from C++ verbatim:
- [ ] `ScanlineNode` from `scanline.h:78-104`
- [ ] `Event` and `EventType` from `scanline.h:107-124`
- [ ] `LineSegment` from `orthogonal.cpp:~1100`
- [ ] `SegmentListWrapper` from `orthogonal.cpp`
- [ ] `PosVertInf` ordering (already exists but verify)

### Step 2: Port Event Generation

- [ ] `generateStaticOrthogonalVisGraph()` lines 1736-1801
- [ ] Event sorting with `compare_events()` from `scanline.cpp:294-308`

### Step 3: Port Vertical Sweep

- [ ] `processEventVert()` from `orthogonal.cpp:1368-1543`
- [ ] Node neighbor finding (firstAbove/firstBelow)
- [ ] `findFirstPointAboveAndBelow()` from `scanline.cpp`
- [ ] Segment insertion with vertex creation

### Step 4: Port Horizontal Sweep

- [ ] `processEventHori()` from `orthogonal.cpp:1551-1715`
- [ ] Same structure as vertical but perpendicular

### Step 5: Port Segment Processing

- [ ] Segment merging during sweep
- [ ] Edge creation from segments
- [ ] Integration with existing `VisibilityGraph`

### Step 6: Port A* on Visibility Graph

- [ ] Modify existing A* to use orthogonal edges
- [ ] Handle direction constraints at endpoints
- [ ] Path reconstruction

## Key Differences from Current Implementation

| Aspect | Current (Grid A*) | Target (Visibility Graph) |
|--------|-------------------|---------------------------|
| Graph size | O(area / resolution^2) | O(n + m) where n=obstacles, m=endpoints |
| Path quality | Many small steps | Minimal bends at obstacle corners |
| Memory | Large implicit grid | Sparse explicit graph |
| Build time | None (implicit) | O(n log n) sweep |
| Query time | O(area) per query | O(edges) per query |

## Testing Strategy

1. **Unit tests** for each data structure
2. **Integration test** matching C++ output exactly:
   ```rust
   #[test]
   fn test_vis_graph_matches_cpp() {
       // Same obstacle and connector setup as C++ test
       // Assert same vertices and edges created
   }
   ```
3. **Regression test** for Example 10 webdemo scenario

## Post-Implementation Cleanup

Once visibility graph is working:
- **DELETE** `OrthogonalAStarRouter` entirely
- **DELETE** `GRID_RESOLUTION` constant
- **DELETE** grid-based `route_astar()` and related methods
- **DELETE** `simple_l_route()` fallback

No fallbacks. One algorithm. Clean code.

## References

- C++ libavoid source: `/Users/julian/expts/adaptagrams/cola/libavoid/`
- Existing Rust attempt: `/Users/julian/expts/libavoid-rust/src/orthogonal_visgraph.rs`
- Paper: "Orthogonal Connector Routing" by Wybrow et al.
