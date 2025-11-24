# Orthogonal Visibility Graph Implementation Tasks

**Date:** 2024-11-24
**Based on:** C++ libavoid `orthogonal.cpp` (3259 lines)
**Goal:** Replace A* hack with proper orthogonal visibility graph routing

## Summary

**Completed Tasks:** 11 ✓ | **Skipped/Deferred:** 5 [S] | **In Progress:** 1 [~] | **Pending:** 1
**New File:** `src/orthogonal_visgraph.rs` - Orthogonal visibility graph generator (900 lines)

### Phase 1 (Foundation): 5/5 ✓
### Phase 2 (Sweep-Line): 5/5 ✓
### Phase 3 (Nudging): 4/4 [S] - Deferred, using existing ChannelRouter
### Phase 4 (Integration): 1/4 ✓ | 1/4 [S] | 1/4 [~] | 1/4 pending

---

## Status Legend
- [ ] Not started
- [~] In progress
- [x] Completed
- [S] Skipped (not needed)

---

## Phase 1 - Foundation Data Structures

### #1 LineSegment Structure
- [x] **Create `LineSegment` struct**
  - C++ ref: `orthogonal.cpp:100-300`
  - Fields: `begin`, `end`, `vertices: Vec<PosVertInf>`, `breakpoints: BTreeSet<PosVertInf>`
  - Purpose: Represents horizontal/vertical visibility line during sweep

### #2 PosVertInf Structure
- [x] **Create `PosVertInf` struct**
  - C++ ref: `orthogonal.cpp:50-90`
  - Fields: `vertex_id`, `position: f64`, `directions: ConnDirFlags`
  - Purpose: Vertex with position and visibility direction flags
  - Needs: Ord/PartialOrd for BTreeSet storage

### #3 Event Structure
- [x] **Create `Event` enum and queue**
  - C++ ref: `orthogonal.cpp:400-450`
  - Variants: `Open { pos, shape_id }`, `Close { pos, shape_id }`, `ConnPoint { pos, conn_id }`
  - Purpose: Sweep-line event queue (sorted by position)

### #4 Node Structure (Scanline State)
- [x] **Create `Node` struct for scanline**
  - C++ ref: `orthogonal.cpp:350-400`
  - Fields: `min`, `max`, `shape_id`, `is_shape_edge`
  - Purpose: Tracks active shape boundaries during sweep

### #5 SegmentListWrapper
- [x] **Create `SegmentListWrapper` for segment merging**
  - C++ ref: `orthogonal.cpp:250-300`
  - Purpose: Container that merges overlapping LineSegments
  - Methods: `insert()`, `commit_to_graph()`

---

## Phase 2 - Sweep-Line Algorithm

### #6 Vertical Sweep (processEventVert)
- [x] **Implement vertical sweep for horizontal segments**
  - C++ ref: `orthogonal.cpp:600-800`
  - Input: Sorted events by X coordinate
  - Output: Horizontal visibility candidate segments
  - Algorithm:
    1. Process Open events: add shape edge to scanline
    2. Process Close events: remove shape edge, emit segment
    3. Process ConnPoint events: add connector visibility region

### #7 Horizontal Sweep (processEventHori)
- [x] **Implement horizontal sweep for vertical segments**
  - C++ ref: `orthogonal.cpp:800-1000`
  - Input: Sorted events by Y coordinate
  - Output: Vertical visibility edges
  - Algorithm: Mirror of vertical sweep, intersects with horizontal segments

### #8 Segment Intersection
- [x] **Implement `intersect_segments()`**
  - C++ ref: `orthogonal.cpp:500-600`
  - Purpose: Compute intersection points between H and V segments
  - Creates breakpoints for visibility edge generation

### #9 Edge Generation from Breakpoints
- [x] **Implement `generate_visibility_edges_from_breakpoints()`**
  - C++ ref: `orthogonal.cpp:300-400`
  - Purpose: Convert LineSegment breakpoints to EdgeInf visibility edges
  - Handles: Direction constraints, connector endpoint special cases

### #10 Main Entry Point
- [x] **Implement `generate_static_orthogonal_vis_graph()`**
  - C++ ref: `orthogonal.cpp:1000-1100`
  - Orchestrates: Event queue build -> Vertical sweep -> Horizontal sweep
  - Output: Populated VisibilityGraph with orthogonal edges

---

## Phase 3 - Proper Nudging (C++ Parity)

**Status:** DEFERRED - Using existing `src/channel.rs` ChannelRouter for nudging

### #11 NudgingShiftSegment
- [S] **Extend ShiftSegment with bend classification** - DEFERRED
  - C++ ref: `orthogonal.cpp:1500-1700`
  - Note: Using existing ShiftSegment in channel.rs for now

### #12 Segment Sorting (linesort)
- [S] **Implement partial-order segment sorting** - DEFERRED
  - C++ ref: `orthogonal.cpp:1800-2000`
  - Note: Using simpler position-based sorting in ChannelRouter

### #13 Route Simplification
- [S] **Implement `simplify_orthogonal_routes()`** - DEFERRED
  - C++ ref: `orthogonal.cpp:1300-1400`
  - Note: Can be added later if route quality needs improvement

### #14 Build Nudging Segments
- [S] **Implement `build_orthogonal_nudging_segments()`** - DEFERRED
  - C++ ref: `orthogonal.cpp:1700-1800`
  - Note: Using existing build_shift_segments_with_obstacles in ChannelRouter

---

## Phase 4 - Router Integration

### #15 Replace A* Hack in Router
- [x] **Use orthogonal visibility graph for routing**
  - File: `src/router.rs`
  - Replace: Current A* fallback in orthogonal routing
  - With: Pathfinding through orthogonal visibility graph

### #16 Proper Nudging Integration
- [S] **Call nudging with correct segment classification** - DEFERRED
  - Note: Using existing ChannelRouter.nudge_routes_with_obstacles() called from Router
  - Can enhance later with Phase 3 improvements

### #17 Incremental Graph Updates
- [ ] **Support incremental orthogonal graph updates**
  - C++ ref: `orthogonal.cpp` updates on shape changes
  - Purpose: Don't rebuild entire graph on every shape move
  - Reuse: Existing `dirty_shapes` tracking in Router
  - Status: Future optimization

### #18 Gallery Demo Verification
- [~] **Verify Example 9 (Route Nudging) works correctly**
  - File: `examples/web/gallery.js`
  - Test: Routes avoid obstacles
  - Test: Overlapping routes are nudged apart
  - Test: Performance is acceptable
  - Status: AWAITING USER TESTING at http://localhost:8080/gallery.html

---

## Dependencies

```
Phase 1 (Foundation)
├── #1 LineSegment
├── #2 PosVertInf
├── #3 Event
├── #4 Node
└── #5 SegmentListWrapper

Phase 2 (Sweep-Line) - depends on Phase 1
├── #6 processEventVert
├── #7 processEventHori
├── #8 intersect_segments
├── #9 generate_visibility_edges
└── #10 generate_static_orthogonal_vis_graph

Phase 3 (Nudging) - can parallelize with Phase 2
├── #11 NudgingShiftSegment
├── #12 linesort
├── #13 simplify_orthogonal_routes
└── #14 build_orthogonal_nudging_segments

Phase 4 (Integration) - depends on Phase 2 & 3
├── #15 Replace A* hack
├── #16 Proper nudging integration
├── #17 Incremental updates
└── #18 Gallery verification
```

---

## Implementation Notes

### Coordinate System
- C++ libavoid uses Y-down coordinates
- Rust implementation should match for consistency
- `getPosVertInfDirections()` maps scanline directions

### Separation Distance
- C++ uses reduction steps (divide by 10) if infeasible
- Start with base separation, reduce incrementally
- `double reductionSteps = 10.0`

### Checkpoint Handling
- Segments with checkpoints locked to position
- Unless `nudgeFinalSegments` is enabled
- Preserves user-specified waypoints

### File Organization
- New file: `src/orthogonal_visgraph.rs` for sweep-line algorithm
- Extend: `src/channel.rs` for improved nudging
- Modify: `src/router.rs` for integration

---

## Test Plan

1. **Unit tests per phase**
   - Phase 1: Data structure creation and ordering
   - Phase 2: Sweep-line produces correct segments
   - Phase 3: Nudging segment classification
   - Phase 4: End-to-end routing

2. **Parity tests**
   - Compare output with C++ libavoid on test cases
   - Verify route quality (length, crossings)

3. **Visual verification**
   - Gallery Example 9 shows correct behavior
   - No routes through obstacles
   - Overlapping routes separated
