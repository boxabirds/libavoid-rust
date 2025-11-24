# Orthogonal Visibility Graph Implementation Tasks

**Date:** 2024-11-24
**Based on:** docs/20251124-2153-visibility-graph.md
**Goal:** Replace grid-based A* with scanline visibility graph (verbatim C++ port)

## Summary

**Total Tasks:** 42
**Phases:** 6

### Phase 1: Data Structures (8 tasks)
### Phase 2: Event Generation (5 tasks)
### Phase 3: Vertical Sweep (8 tasks)
### Phase 4: Horizontal Sweep (4 tasks)
### Phase 5: Edge Creation & Routing (7 tasks)
### Phase 6: Cleanup & Delete Old Code (5 tasks)
### Test Tasks (5 tasks)

---

## Status Legend
- [ ] Not started
- [~] In progress
- [x] Completed

---

## Phase 1: Data Structures

Port C++ data structures verbatim. All structures go in `src/orthogonal_visgraph.rs`.

### 1.1 ScanlineNode

- [ ] **#1 Port ScanlineNode struct**
  - C++ ref: `scanline.h:78-104` - `class Node`
  - Fields to port:
    ```rust
    struct ScanlineNode {
        obstacle: Option<ObstacleRef>,  // C++: Obstacle *v
        conn_vert: Option<VertInfRef>,  // C++: VertInf *c
        shift_segment: Option<ShiftSegmentRef>,  // C++: ShiftSegment *ss
        pos: f64,                       // C++: double pos
        min: [f64; 2],                  // C++: double min[2]
        max: [f64; 2],                  // C++: double max[2]
        first_above: Option<NodeIdx>,   // C++: Node *firstAbove
        first_below: Option<NodeIdx>,   // C++: Node *firstBelow
        iter: Option<ScanlineIter>,     // C++: NodeSet::iterator iter
    }
    ```
  - Implement constructors matching C++:
    - `Node(Obstacle *v, const double p)` - lines 53-67
    - `Node(VertInf *c, const double p)` - lines 69-79
    - `Node(ShiftSegment *ss, const double p)` - lines 81-91

### 1.2 Node Methods

- [ ] **#2 Port Node::firstObstacleAbove/Below**
  - C++ ref: `scanline.cpp:100-131`
  - Purpose: Find first obstacle edge above/below in scanline
  - Returns `f64` (position or -DBL_MAX/DBL_MAX)

- [ ] **#3 Port Node::firstPointAbove/Below**
  - C++ ref: `scanline.cpp:219-261`
  - Purpose: Find first point above/below, ignoring inline edges
  - Key logic: `inLineWithEdge` check at lines 230-231, 252-253

- [ ] **#4 Port Node::findFirstPointAboveAndBelow**
  - C++ ref: `scanline.cpp:163-217`
  - Purpose: Find visibility limits in both directions
  - Complex logic handling overlapping shapes

- [ ] **#5 Port Node::isInsideShape**
  - C++ ref: `scanline.cpp:265-282`
  - Purpose: Check if node position is inside any shape

### 1.3 Event Types

- [ ] **#6 Port EventType enum**
  - C++ ref: `scanline.h:107-114`
  - Values (order matters for sorting!):
    ```rust
    enum EventType {
        Open = 1,      // Shape edge opens
        SegOpen = 2,   // Segment opens (nudging)
        ConnPoint = 3, // Connector endpoint
        SegClose = 4,  // Segment closes
        Close = 5,     // Shape edge closes
    }
    ```

- [ ] **#7 Port Event struct**
  - C++ ref: `scanline.h:117-124`, `scanline.cpp:285-290`
  - Fields:
    ```rust
    struct Event {
        event_type: EventType,
        node: NodeIdx,
        pos: f64,
    }
    ```

### 1.4 LineSegment

- [ ] **#8 Port LineSegment struct**
  - C++ ref: `orthogonal.cpp:1050-1150` (approximate, class is inline)
  - Fields:
    ```rust
    struct LineSegment {
        begin: f64,           // Start on parallel axis
        finish: f64,          // End on parallel axis
        pos: f64,             // Position on perpendicular axis
        fixed: bool,          // Is this segment fixed?
        vert_infs: BTreeSet<PosVertInf>,  // Vertices on segment
    }
    ```
  - Methods:
    - `new(begin, finish, pos, fixed, v1, v2)`
    - `contains_point(pos) -> bool`
    - Implement `Ord` for sorting by `(pos, begin)`

---

## Phase 2: Event Generation

### 2.1 Event Creation

- [ ] **#9 Port obstacle event generation**
  - C++ ref: `orthogonal.cpp:1738-1760`
  - For each obstacle:
    - Create `Open` event at `bbox.min.y` with node at `midX`
    - Create `Close` event at `bbox.max.y` with same node
  - Node stores `min[XDIM], max[XDIM], min[YDIM], max[YDIM]`

- [ ] **#10 Port connector endpoint event generation**
  - C++ ref: `orthogonal.cpp:1785-1801`
  - For each connector vertex where `visDirections != ConnDirNone`:
    - Create `ConnPoint` event at `point.y`
    - Node position is `point.x`

- [ ] **#11 Port event sorting**
  - C++ ref: `scanline.cpp:294-308` - `compare_events()`
  - Sort order:
    1. By position (`pos`)
    2. By event type (Open < SegOpen < ConnPoint < SegClose < Close)
    3. By node pointer (for stability)

### 2.2 Visibility Correction

- [ ] **#12 Port fixConnectionPointVisibilityOnOutsideOfVisibilityGraph**
  - C++ ref: `orthogonal.cpp:1255-1290`
  - Purpose: Fix visibility for endpoints on graph boundary
  - Called twice: once with `ConnDirLeft | ConnDirRight`, once with `ConnDirUp | ConnDirDown`

- [ ] **#13 Port SegmentListWrapper**
  - C++ ref: `orthogonal.cpp:1160-1250` (approximate)
  - Methods:
    - `insert(segment) -> &mut LineSegment` - insert or merge
    - `list() -> &Vec<LineSegment>`
  - Key behavior: segments at same `pos` get merged

---

## Phase 3: Vertical Sweep (Horizontal Segments)

### 3.1 Main Sweep Loop

- [ ] **#14 Port vertical sweep main loop structure**
  - C++ ref: `orthogonal.cpp:1811-1854`
  - Structure:
    ```
    for each event (sorted by Y):
        if new Y position:
            process passes 2 and 3 for previous Y
        process pass 1 for current event
    ```
  - Three passes per Y position:
    - Pass 1: Add Open events to scanline
    - Pass 2: Process all events (create segments)
    - Pass 3: Remove Close events from scanline

### 3.2 processEventVert Implementation

- [ ] **#15 Port processEventVert pass 1 (scanline insertion)**
  - C++ ref: `orthogonal.cpp:1373-1395`
  - Insert node into scanline (BTreeSet ordered by X position)
  - Set up `firstAbove`/`firstBelow` neighbor pointers

- [ ] **#16 Port processEventVert pass 2 - Open/Close events**
  - C++ ref: `orthogonal.cpp:1397-1460`
  - For shape edges:
    - Determine `lineY` (min_y for Open, max_y for Close)
    - Call `findFirstPointAboveAndBelow()` to get visibility limits
    - Create vertices at shape corners `(minShape, lineY)` and `(maxShape, lineY)`
    - Insert segments:
      - `(minLimit, minShape)` if visible left
      - `(minShape, maxShape)` along shape edge
      - `(maxShape, maxLimit)` if visible right
    - Handle overlapping shapes case (lines 1437-1459)

- [ ] **#17 Port processEventVert pass 2 - ConnPoint events**
  - C++ ref: `orthogonal.cpp:1461-1512`
  - For connector endpoints:
    - Get visibility limits from `firstPointAbove/Below`
    - Check `isInsideShape`
    - Create segments based on `visDirections`:
      - If `ConnDirLeft` and `minLimit < cp.x`: segment `(minLimit, cp.x)`
      - If `ConnDirRight` and `cp.x < maxLimit`: segment `(cp.x, maxLimit)`
    - Add dummy vertex if not inside shape (lines 1494-1510)

- [ ] **#18 Port processEventVert pass 3 (scanline removal)**
  - C++ ref: `orthogonal.cpp:1515-1542`
  - Update neighbor pointers before removal
  - Remove node from scanline
  - Delete node for ConnPoint events

### 3.3 Scanline Management

- [ ] **#19 Port NodeSet (sorted scanline)**
  - C++ ref: `scanline.h:70-76` - `CmpNodePos` comparator
  - Use `BTreeSet<NodeIdx>` with custom ordering by `node.pos`
  - Tie-breaker: compare by node index

- [ ] **#20 Port neighbor pointer maintenance**
  - C++ ref: `orthogonal.cpp:1380-1394` (insertion), `1519-1527` (removal)
  - On insert: link to adjacent nodes in scanline
  - On remove: update neighbors to skip removed node

- [ ] **#21 Port markShiftSegmentsAbove/Below**
  - C++ ref: `scanline.cpp:135-164`
  - Purpose: Mark shift segments with space limits during nudging
  - Updates `minSpaceLimit`/`maxSpaceLimit`

---

## Phase 4: Horizontal Sweep (Vertical Segments)

### 4.1 Main Sweep

- [ ] **#22 Port horizontal sweep setup**
  - C++ ref: `orthogonal.cpp:1863-1903`
  - Same as vertical but:
    - Node position is `midY` (not `midX`)
    - Events sorted by X (not Y)
    - Open at `bbox.min.x`, Close at `bbox.max.x`

- [ ] **#23 Port processEventHori**
  - C++ ref: `orthogonal.cpp:1551-1715`
  - Mirror of `processEventVert` but creates vertical segments
  - Key differences:
    - `lineX` instead of `lineY`
    - `firstPointAbove/Below` in Y dimension
    - Visibility directions: `ConnDirUp`/`ConnDirDown`

### 4.2 Segment Processing

- [ ] **#24 Port segment sorting**
  - C++ ref: `orthogonal.cpp:1861` - `segments.list().sort()`
  - Sort horizontal segments by `(pos, begin)`
  - Sort vertical segments by `(pos, begin)`

- [ ] **#25 Port segment merging during insertion**
  - C++ ref: `SegmentListWrapper::insert()`
  - When inserting segment at existing `pos`:
    - Merge `begin`/`finish` ranges
    - Merge vertex sets

---

## Phase 5: Edge Creation & Routing

### 5.1 Vertex and Edge Creation

- [ ] **#26 Port VertInf creation from segments**
  - C++ ref: `orthogonal.cpp:1418-1421` (shape corners), `1446-1448`, `1455-1457`
  - Create vertex with `dummyOrthogShapeID` for shape corners
  - Create vertex with `dummyOrthogID` for internal points
  - Track all vertices in segment's `vert_infs` set

- [ ] **#27 Port edge creation from segments**
  - C++ ref: implicit in segment processing
  - For each segment:
    - Sort `vert_infs` by position
    - Create edge between each adjacent pair
    - Set `orthogonal = true` on edges

- [ ] **#28 Port orthogonal edge list management**
  - C++ ref: `vertices.h:151-152` - `orthogVisList`, `orthogVisListSize`
  - Each VertInf maintains list of orthogonal edges
  - Separate from polyline edges (`visList`)

### 5.2 Integration with Existing Graph

- [ ] **#29 Create OrthogonalVisibilityGraphBuilder**
  - New struct to encapsulate the algorithm
  - Methods:
    - `new(obstacles, connectors) -> Self`
    - `build() -> VisibilityGraph`
  - Owns all temporary state (nodes, events, segments)

- [ ] **#30 Integrate with Router::rebuild_visibility_graph**
  - Replace current implementation that uses `OrthogonalAStarRouter`
  - Call `OrthogonalVisibilityGraphBuilder::build()`
  - Store result in router's visibility graph

### 5.3 A* Routing on Orthogonal Graph

- [ ] **#31 Port orthogonal A* path finding**
  - C++ ref: `makepath.cpp:200-500` (approximate)
  - Key differences from polyline A*:
    - Use `orthogVisList` instead of `visList`
    - Direction tracking for bend penalties
    - Handle `visDirections` constraints at endpoints

- [ ] **#32 Port path reconstruction**
  - C++ ref: `makepath.cpp` path reconstruction
  - Build `Polygon` from vertex sequence
  - Ensure path is orthogonal (only horizontal/vertical segments)

---

## Phase 6: Cleanup & Delete Old Code

### 6.1 Delete Grid-Based A*

- [ ] **#33 Delete OrthogonalAStarRouter struct**
  - File: `src/orthogonal.rs:410-677`
  - Delete entire struct and impl block

- [ ] **#34 Delete GRID_RESOLUTION constant**
  - File: `src/orthogonal.rs:408-410`
  - Delete constant definition

- [ ] **#35 Delete grid-based helper methods**
  - File: `src/orthogonal.rs`
  - Delete: `encode_node()`, `AStarState`, `route_astar()`, `route_astar_with_directions()`
  - Delete: `heuristic()`, `is_blocked()`, `reconstruct_path()`, `simple_l_route()`

### 6.2 Update OrthogonalRouter

- [ ] **#36 Update OrthogonalRouter to use visibility graph**
  - File: `src/orthogonal.rs:78-129`
  - Change `route_orthogonal()` to:
    - Look up path in prebuilt visibility graph
    - Fall through to simple L-route only if graph not built

- [ ] **#37 Delete simple L-route fallbacks**
  - File: `src/orthogonal.rs:131-171`
  - Delete `route_h_v_simple()`, `route_v_h_simple()`
  - Delete `route_h_v()`, `route_v_h()` if unused

---

## Test Tasks

### Unit Tests

- [ ] **#38 Test: ScanlineNode ordering**
  - Test `CmpNodePos` ordering matches C++
  - Test neighbor pointer maintenance

- [ ] **#39 Test: Event sorting**
  - Test events sort correctly by (pos, type, node)
  - Test Open < ConnPoint < Close ordering

- [ ] **#40 Test: LineSegment merging**
  - Test segments at same position merge correctly
  - Test vertex sets combine properly

### Integration Tests

- [ ] **#41 Test: Visibility graph matches C++ output**
  - Create identical obstacle/connector setup as C++ test
  - Assert same number of vertices and edges
  - Assert same vertex positions

- [ ] **#42 Test: Example 10 webdemo produces clean routes**
  - Route 3 should have ~6 points, not 41
  - All segments must be orthogonal
  - No 1-pixel zigzag steps

---

## Implementation Order

1. #1-#5 ScanlineNode and methods
2. #6-#8 Event and LineSegment structs
3. #38-#40 Unit tests for data structures
4. #9-#13 Event generation
5. #14-#21 Vertical sweep (processEventVert)
6. #22-#25 Horizontal sweep (processEventHori)
7. #26-#28 Edge creation
8. #29-#30 Builder integration
9. #31-#32 A* routing
10. #33-#37 Delete old code
11. #41-#42 Integration tests

---

## Dependencies

```
#1 -> #2, #3, #4, #5 (Node methods need Node struct)
#6, #7 -> #9, #10, #11 (Events need EventType)
#8 -> #13 (SegmentListWrapper needs LineSegment)
#14 -> #15, #16, #17, #18 (Sweep loop needs passes)
#19, #20 -> #14 (Sweep needs scanline)
#26, #27 -> #16, #17, #23 (Edges need segments)
#29 -> all previous (Builder needs everything)
#30 -> #29 (Router integration needs builder)
#33-#37 -> #30 (Delete after new code works)
```

---

## Success Criteria

1. **Route quality**: Example 10 Route 3 has ≤8 points (not 41)
2. **Correctness**: All segments are orthogonal (no diagonals)
3. **Performance**: Build time O(n log n), routing time O(edges)
4. **Clean code**: Zero fallbacks, zero grid-based code remaining

---

## References

- C++ source: `/Users/julian/expts/adaptagrams/cola/libavoid/`
- Design doc: `docs/20251124-2153-visibility-graph.md`
- Current Rust: `src/orthogonal_visgraph.rs` (partial, to be completed)
