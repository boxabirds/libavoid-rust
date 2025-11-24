# libavoid-rust Parity Audit Tasks

**Date:** 2024-11-24
**Based on:** docs/reports/20251124-1941-parity-audit.md
**Last Updated:** 2024-11-24

---

## CRITICAL: Authoritative Reference

**The C++ libavoid source at `../adaptagrams/cola/libavoid/` is GOSPEL.**

- Every algorithm, data structure, and approach must match the C++ implementation
- Do NOT reinvent wheels - read the C++ code and port it faithfully
- When in doubt, the C++ behavior is correct by definition
- Copy the logic, translate the idioms, preserve the semantics

This is a **port**, not a reimplementation. The C++ code has been battle-tested for years. Our job is to translate it to Rust, not to "improve" it with different approaches.

---

## Summary

**Completed Tasks:** 0 of 42 tasks
**Tests:** TBD

### P0 (Critical - Correctness): 0/5
### P1 (High - Route Quality): 0/8
### P2 (Medium - Feature Completeness): 0/12
### P3 (Low - Performance/Advanced): 0/11
### Test Coverage: 0/6

---

## Status Legend
- [ ] Not started
- [~] In progress
- [x] Completed
- [S] Skipped (not applicable)

---

## P0 - Critical (Correctness Issues)

These issues can cause incorrect routing results.

### Containment Map

- [ ] #1 **Implement containment map (`router->contains[pID]`)**
  - C++ ref: `libavoid/router.cpp` - `generateContains()`, `contains` map
  - C++ ref: `libavoid/visibility.cpp:129-132` - containment check in `vertexVisibility()`
  - Purpose: Track which connector endpoints are inside which shapes
  - Implementation:
    - Add `contains: HashMap<VertexId, HashSet<ObstacleId>>` to Router
    - Add `generate_contains(vertex: &VertInf)` method
    - Use point-in-polygon test for each shape
    - Call during visibility computation for connector endpoints
  - Impact: Required for correct routing when endpoints are near/inside shapes
  - Test: Connector endpoint inside shape boundary routes correctly

- [ ] #2 **Use containment map in visibility computation**
  - C++ ref: `libavoid/visibility.cpp:385-408` - `sweepVisible()` containment check
  - File: `src/visibility.rs:476-561`
  - Implementation:
    - Pass containment set to `compute_vertex_visibility()`
    - Skip edges of containing shapes when computing visibility
    - Filter blocking edges that belong to containing shapes
  - Impact: Prevents false blocking by containing shape edges

### Visibility Cones

- [ ] #3 **Implement `inValidRegion()` for visibility cones**
  - C++ ref: `libavoid/geometry.cpp` - `inValidRegion()`
  - C++ ref: `libavoid/visibility.cpp:591-602` - cone checks in sweep
  - Purpose: Shape corners have limited visibility based on adjacent edges
  - Implementation:
    - Add `in_valid_region(prev: Point, center: Point, next: Point, test: Point) -> bool`
    - Check if `test` point is in the valid visibility region of `center`
    - Valid region is the exterior angle formed by prev→center→next
  - Note: Partially exists at `src/geometry.rs:950-993` but not integrated
  - Test: Routing around concave polygon corners

- [ ] #4 **Integrate visibility cones into sweep algorithm**
  - C++ ref: `libavoid/visibility.cpp:591-602`
  - File: `src/visibility.rs:650-810`
  - Implementation:
    - For shape corner vertices, check `in_valid_region()` before adding edge
    - Both source and target visibility cones must permit the edge
    - Add `shape_edge_before` / `shape_edge_after` to `VertInf` (already exists)
  - Impact: Prevents routes that would clip through shape corners

### A* Determinism

- [ ] #5 **Add timestamp tie-breaking for deterministic A***
  - C++ ref: `libavoid/makepath.cpp:72-85` - `ANode::timeStamp`
  - C++ ref: `libavoid/makepath.cpp:293-295` - tie-breaking comparison
  - File: `src/graph.rs`
  - Implementation:
    - Add `timestamp: u32` field to `SearchState`
    - Increment global timestamp counter for each node expansion
    - Use timestamp as secondary sort key in BinaryHeap
    - When f-scores equal, prefer lower timestamp (earlier discovered)
  - Impact: Ensures identical routes across runs for same input
  - Test: Run same routing 100 times, verify identical results

---

## P1 - High (Route Quality Issues)

These issues affect aesthetic quality of routes.

### Segment Merging for Nudging

- [ ] #6 **Implement `shouldAlignWith()` for segment alignment detection**
  - C++ ref: `libavoid/orthogonal.cpp:383-440` - `NudgingShiftSegment::shouldAlignWith()`
  - File: `src/channel.rs`
  - Implementation:
    - Add method to `ShiftSegment`: `should_align_with(&self, other: &ShiftSegment) -> bool`
    - Return true if:
      - Same connector AND both are final segments AND overlapping
      - Same connector AND one has checkpoints, touching at non-checkpoint
    - Consider `endsInShape` flag for stronger alignment preference
  - Impact: Segments that should appear as one get aligned

- [ ] #7 **Implement `canAlignWith()` for optional alignment**
  - C++ ref: `libavoid/orthogonal.cpp:361-381` - `canAlignWith()`
  - File: `src/channel.rs`
  - Implementation:
    - Add method: `can_align_with(&self, other: &ShiftSegment) -> bool`
    - Return true if same connector and neither has checkpoints
    - These segments are allowed to drift together but don't have to
  - Purpose: Allows aesthetic consolidation without forcing it

- [ ] #8 **Implement `mergeWith()` for segment consolidation**
  - C++ ref: `libavoid/orthogonal.cpp:443-478` - `mergeWith()`
  - File: `src/channel.rs`
  - Implementation:
    - Add method: `merge_with(&mut self, other: &ShiftSegment)`
    - Adjust limits: `min_limit = max(self.min, other.min)`, `max_limit = min(self.max, other.max)`
    - Compute merged position as average, clamped to limits
    - Merge index lists and sort by position
    - Apply merged position to all points
  - Impact: Aligned segments move together during nudging

- [ ] #9 **Implement segment merging in nudging pipeline**
  - C++ ref: `libavoid/orthogonal.cpp:1200-1250` - segment merging loop
  - File: `src/channel.rs:317-395`
  - Implementation:
    - Before building VPSC problem, iterate segments
    - For each pair where `should_align_with()` returns true, call `merge_with()`
    - For pairs where `can_align_with()` and close together, optionally merge
    - Update segment list after merging
  - Test: Two final segments of same connector merge into one

### Ordering and Limits

- [ ] #10 **Implement `fixedOrder()` for segment ordering**
  - C++ ref: `libavoid/orthogonal.cpp:266-287` - `fixedOrder()`
  - File: `src/channel.rs`
  - Implementation:
    - Add method: `fixed_order(&self) -> (bool, i32)` returning (is_fixed, order)
    - Check if segment is within `nudge_distance` of min or max limit
    - If at min limit: order = 1 (must be below others)
    - If at max limit: order = -1 (must be above others)
    - If at both or fixed: is_fixed = true
  - Purpose: Ensures limit-constrained segments maintain relative positions

- [ ] #11 **Use `fixedOrder()` in constraint generation**
  - C++ ref: `libavoid/orthogonal.cpp:1350-1400`
  - File: `src/channel.rs:789-843`
  - Implementation:
    - When building constraints between overlapping segments
    - Check `fixed_order()` for both segments
    - If orders conflict or both fixed at same position, handle specially
    - Generate constraints respecting the required ordering
  - Impact: Prevents segments from being nudged past their limits

### Default Parameter Alignment

- [ ] #12 **Align default bend penalty with C++**
  - C++ ref: `libavoid/router.cpp` - `bendPenalty` default = 0.0
  - Rust current: `src/orthogonal.rs:89` - `bend_penalty: 50.0`
  - Implementation:
    - Change default `bend_penalty` to 0.0 in `OrthogonalRouter::new()`
    - Update `OrthogonalAStarRouter::new()` similarly
    - Add parameter to Router for global bend penalty control
  - Impact: Routes will have more bends but match C++ behavior
  - Note: Consider making this configurable rather than hardcoded

- [ ] #13 **Add `endsInShape` detection for final segments**
  - C++ ref: `libavoid/orthogonal.cpp:126-130` - `endsInShape` flag
  - File: `src/channel.rs`
  - Implementation:
    - Add `ends_in_shape: bool` to `ShiftSegment`
    - Set true if segment's endpoint is a shape connection pin
    - Check in `build_shift_segments()` by examining route endpoint type
  - Purpose: Final segments ending in shapes get stronger alignment preference

---

## P2 - Medium (Feature Completeness)

### Orthogonal Visibility Graph

- [ ] #14 **Replace grid-based orthogonal A* with visibility graph approach**
  - C++ ref: `libavoid/orthogonal.cpp:generateStaticOrthogonalVisGraph()`
  - File: `src/orthogonal.rs`, new `src/orthogonal_visgraph.rs`
  - Implementation:
    - Create separate orthogonal visibility graph (`vis_orth_graph`)
    - Two-pass scanline sweep (X then Y dimensions)
    - Create dummy vertices on shape edges at breakpoints
    - Connect vertices with H/V edges only
  - Impact: Continuous coordinates instead of fixed grid

- [ ] #15 **Implement orthogonal visibility scanline events**
  - C++ ref: `libavoid/scanline.cpp` - `SegmentListWrapper`, `Event`
  - Implementation:
    - `ShapeOpen` / `ShapeClose` events for entering/leaving shape range
    - `ConnPoint` events for connector endpoints
    - Maintain active shape list during sweep
    - Compute visibility segments at each event
  - Depends on: #14

- [ ] #16 **Connect orthogonal visibility graph to A* router**
  - File: `src/orthogonal.rs`
  - Implementation:
    - Use `vis_orth_graph` for orthogonal routing instead of grid A*
    - Add connector endpoints to graph before routing
    - Remove endpoints after routing (or mark inactive)
  - Depends on: #14, #15

### VPSC Solver Improvements

- [ ] #17 **Implement block splitting in VPSC solver**
  - C++ ref: `libavoid/vpsc.cpp:420-500` - `Block::split()`
  - File: `src/vpsc.rs`
  - Implementation:
    - Add `split_block(block_id: usize, constraint_id: usize)` method
    - When a constraint within a block has negative Lagrange multiplier
    - Split block at that constraint into two independent blocks
    - Recompute optimal positions for each new block
  - Impact: Finds globally optimal solutions instead of local optima

- [ ] #18 **Implement Lagrange multiplier tracking**
  - C++ ref: `libavoid/vpsc.cpp:350-380` - `Constraint::lm`, `Block::compute_lm()`
  - File: `src/vpsc.rs`
  - Implementation:
    - Add `lagrange_multiplier: f64` to `Constraint`
    - After solving, compute LM for each active constraint
    - LM < 0 indicates constraint should be deactivated
    - Use in `split_block()` decision
  - Depends on: #17

- [ ] #19 **Implement heap-based constraint activation**
  - C++ ref: `libavoid/vpsc.cpp:580-650` - `Blocks::mergeLeft/mergeRight`
  - File: `src/vpsc.rs`
  - Implementation:
    - Add `in_heap: BinaryHeap<ConstraintRef>` and `out_heap` to Block
    - Heaps ordered by violation amount
    - Pop most-violated constraint for activation
    - More efficient than iterating all constraints
  - Impact: Better performance for large constraint sets

### Hyperedge Improvements

- [ ] #20 **Implement hyperedge tree structure**
  - C++ ref: `libavoid/hyperedgeimprover.cpp:50-100` - `HyperedgeTreeNode`, `HyperedgeTreeEdge`
  - File: `src/hyperedge.rs`
  - Implementation:
    ```rust
    pub struct HyperedgeTreeNode {
        junction_id: Option<u32>,
        terminal: Option<ConnEnd>,
        edges: Vec<HyperedgeTreeEdgeRef>,
    }
    pub struct HyperedgeTreeEdge {
        connector_id: u32,
        nodes: [HyperedgeTreeNodeRef; 2],
    }
    ```
  - Purpose: Proper bidirectional tree for hyperedge traversal

- [ ] #21 **Implement hyperedge segment merging**
  - C++ ref: `libavoid/hyperedgeimprover.cpp:300-400`
  - File: `src/hyperedge.rs`
  - Implementation:
    - Traverse hyperedge tree, extract shift segments
    - Identify collinear segments from different connectors
    - Merge into single segment for nudging
    - Apply nudged position to all merged segments
  - Impact: Hyperedge branches don't overlap unnecessarily

- [ ] #22 **Implement junction add/delete optimization**
  - C++ ref: `libavoid/hyperedgeimprover.cpp:600-800`
  - File: `src/hyperedge.rs`
  - Implementation:
    - `try_add_junction()`: Test if adding junction reduces total length
    - `try_remove_junction()`: Test if removing junction is beneficial
    - Iterate until no improvement possible
  - Depends on: #20

### Missing Router Features

- [ ] #23 **Implement InvisibilityGraph caching**
  - C++ ref: `libavoid/router.h:InvisibilityGrph` flag
  - C++ ref: `libavoid/graph.cpp` - `EdgeInf::addBlocker()`
  - File: `src/visibility.rs`, `src/router.rs`
  - Implementation:
    - Add `invisibility_graph: bool` option to Router
    - Instead of deleting blocked edges, mark them with blocker ID
    - When obstacle removed, recheck edges blocked by that obstacle
    - Reactivate edges that become visible
  - Impact: Faster incremental updates when obstacles removed

- [ ] #24 **Implement progress callbacks for transactions**
  - C++ ref: `libavoid/router.h` - `shouldContinueTransactionWithProgress()`
  - File: `src/router.rs`
  - Implementation:
    - Add `progress_callback: Option<Box<dyn Fn(TransactionProgress)>>` to Router
    - Define `TransactionProgress { phase: u8, total_phases: u8, proportion: f64 }`
    - Call callback at each phase transition and periodically within phases
    - Allow callback to return `false` to abort transaction
  - Purpose: UI responsiveness for large routing operations

- [ ] #25 **Implement phased transaction processing**
  - C++ ref: `libavoid/router.cpp` - `TransactionPhases` enum
  - File: `src/router.rs`
  - Implementation:
    - Split `process_transaction()` into discrete phases:
      1. OrthogonalVisibilityGraphScanX
      2. OrthogonalVisibilityGraphScanY
      3. RouteSearch
      4. CrossingDetection
      5. RerouteSearch
      6. OrthogonalNudgingX
      7. OrthogonalNudgingY
      8. Completed
    - Store current phase in Router state
    - Support resuming from interrupted phase
  - Depends on: #24

---

## P3 - Low (Performance & Advanced)

### Memory Optimization

- [ ] #26 **Implement block allocator for A* nodes**
  - C++ ref: `libavoid/makepath.cpp:40-60` - `AStarPathPrivate::m_available_nodes`
  - File: `src/graph.rs`
  - Implementation:
    - Create `NodePool` with pre-allocated blocks of 5000 nodes
    - Reuse nodes across searches instead of allocating new
    - Clear pool between transactions
  - Impact: Reduced allocation overhead for large graphs
  - Note: May not be significant in Rust due to allocator efficiency

### Sweep Algorithm Improvements

- [ ] #27 **Use balanced tree for sweepline active edges**
  - C++ ref: `libavoid/visibility.cpp` - `SweepEdgeList` (std::list with sorted insert)
  - File: `src/visibility.rs:650-810`
  - Implementation:
    - Replace `Vec<SweepEdge>` with `BTreeSet<SweepEdge>` or custom balanced tree
    - Implement proper `Ord` for `SweepEdge` based on intersection distance
    - O(log n) insert/remove instead of O(n)
  - Impact: O(n log n) sweep instead of O(n²) in practice

- [ ] #28 **Implement proper edge event handling in sweep**
  - C++ ref: `libavoid/visibility.cpp:556-670` - edge add/remove with `vecDir`
  - File: `src/visibility.rs:851-885`
  - Implementation:
    - Track which edges start/end at each vertex
    - On visiting vertex, remove edges ending here, add edges starting here
    - Use `vec_dir()` to determine AHEAD/BEHIND relationship
    - Currently simplified in `sweep_update_active_edges()`
  - Depends on: #27

### Debug and Development Features

- [ ] #29 **Implement SVG debug output**
  - C++ ref: `libavoid/debughandler.h` - `DebugHandler` interface
  - File: New `src/debug_handler.rs`
  - Implementation:
    - Trait `DebugHandler` with methods for logging routing steps
    - SVG writer implementation for visualization
    - Output visibility graph, routes, obstacles
    - Configurable via Router option
  - Purpose: Debugging complex routing issues

- [ ] #30 **Implement SimpleRouting optimization**
  - C++ ref: `libavoid/router.h` - `SimpleRouting` flag
  - File: `src/router.rs`
  - Implementation:
    - For small graphs (< N shapes), use simpler algorithms
    - Skip full visibility graph construction
    - Direct pathfinding with obstacle collision detection
    - Threshold configurable (default: 3-5 shapes)
  - Impact: Faster routing for simple diagrams

### Checkpoint Improvements

- [ ] #31 **Full checkpoint integration in nudging**
  - C++ ref: `libavoid/orthogonal.cpp:206-209` - checkpoint weight
  - C++ ref: `libavoid/orthogonal.cpp:479-490` - `hasCheckpointAtPosition()`
  - File: `src/channel.rs`
  - Implementation:
    - Track checkpoint positions on each segment
    - Add `has_checkpoint_at_position(pos, dim)` method
    - Use in `should_align_with()` to prevent checkpoint modification
    - Apply `STRONG_WEIGHT` to segments with checkpoints
  - Currently: Checkpoints just mark segment as Fixed

### Cluster Features

- [ ] #32 **Complete cluster routing integration**
  - C++ ref: `libavoid/viscluster.cpp`
  - File: `src/cluster.rs`, `src/router.rs`
  - Implementation:
    - Add cluster boundary vertices to visibility graph
    - Apply `clusterCrossingPenalty` when route crosses cluster boundary
    - Support nested clusters
  - Currently: Basic `ClusterRef` structure only

### Additional Penalty Types

- [ ] #33 **Implement `portDirectionPenalty`**
  - C++ ref: `libavoid/router.h` - `portDirectionPenalty` parameter
  - C++ ref: `libavoid/makepath.cpp` - penalty application
  - File: `src/graph.rs`
  - Implementation:
    - Add penalty when edge direction doesn't match port direction cone
    - Use `inValidRegion()` to check if edge is in allowed direction cone
    - Apply multiplicative penalty to edge cost
  - Depends on: #3

- [ ] #34 **Implement `reverseDirectionPenalty`**
  - C++ ref: `libavoid/router.h` - `reverseDirectionPenalty`
  - File: `src/graph.rs`
  - Implementation:
    - Add penalty when edge goes away from destination
    - Compare edge direction to direct line to target
    - If angle > 90 degrees, apply penalty
  - Impact: Discourages routes that double back

### Multiple Cost Targets

- [ ] #35 **Support multiple heuristic targets in A***
  - C++ ref: `libavoid/makepath.cpp` - `m_cost_targets` vector
  - File: `src/graph.rs`
  - Implementation:
    - Allow specifying multiple goal vertices
    - Heuristic becomes minimum distance to any target
    - Useful for connection pins with multiple positions
  - Impact: Better routes when multiple valid endpoints exist

### Route Callbacks

- [ ] #36 **Implement route change notification system**
  - C++ ref: `libavoid/connector.cpp` - callback invocation on route change
  - File: `src/router.rs`, `src/connector.rs`
  - Implementation:
    - Router maintains list of route change callbacks
    - After transaction, notify for each connector whose route changed
    - Include old and new route in notification
  - Purpose: Application can react to routing changes

---

## Test Coverage Tasks

### Parity Verification Tests

- [ ] #37 **Test: Visibility with endpoint inside shape**
  - File: `tests/parity_audit_tests.rs`
  - Test: Create shape, place connector endpoint just inside boundary
  - Verify: Route correctly exits shape without clipping corners
  - Validates: #1, #2

- [ ] #38 **Test: Visibility cones at concave corners**
  - File: `tests/parity_audit_tests.rs`
  - Test: Create L-shaped obstacle, route around inner corner
  - Verify: Route doesn't cut through concave region
  - Validates: #3, #4

- [ ] #39 **Test: A* determinism**
  - File: `tests/parity_audit_tests.rs`
  - Test: Run identical routing 100 times
  - Verify: All routes are byte-identical
  - Validates: #5

- [ ] #40 **Test: Segment merging in nudging**
  - File: `tests/parity_audit_tests.rs`
  - Test: Single connector with two final segments that should align
  - Verify: Segments end up at same position after nudging
  - Validates: #6, #7, #8, #9

- [ ] #41 **Test: Orthogonal routes match C++ output**
  - File: `tests/parity_audit_tests.rs`
  - Test: Define specific shape/connector configuration
  - Compare: Route points against known-good C++ output
  - Note: Requires C++ test harness or golden files
  - Validates: Overall parity

- [ ] #42 **Test: Hyperedge junction optimization**
  - File: `tests/parity_audit_tests.rs`
  - Test: 3-way hyperedge with suboptimal initial junction
  - Verify: Junction moves to reduce total length
  - Validates: #20, #21, #22

---

## Implementation Order

### Phase 1: Correctness (P0)
1. #1, #2 Containment map (foundation for visibility)
2. #3, #4 Visibility cones (correct corner handling)
3. #5 A* determinism (reproducible results)
4. #37, #38, #39 Tests for above

### Phase 2: Route Quality (P1)
5. #6, #7, #8, #9 Segment merging infrastructure
6. #10, #11 Fixed ordering
7. #12, #13 Parameter alignment
8. #40 Test for segment merging

### Phase 3: Core Feature Completeness (P2)
9. #17, #18, #19 VPSC solver improvements
10. #20, #21, #22 Hyperedge improvements
11. #14, #15, #16 Orthogonal visibility graph
12. #41, #42 Parity tests

### Phase 4: Extended Features (P2/P3)
13. #23 InvisibilityGraph
14. #24, #25 Transaction phases
15. #31 Checkpoint integration
16. #32 Cluster routing

### Phase 5: Optimization & Polish (P3)
17. #26 Block allocator
18. #27, #28 Sweep algorithm
19. #29, #30 Debug features
20. #33, #34, #35, #36 Additional penalties and callbacks

---

## C++ Reference Files (AUTHORITATIVE SOURCE)

**Location: `../adaptagrams/cola/libavoid/`**

This is the canonical implementation. Read these files. Understand them. Port them.

| Feature | C++ File | Lines |
|---------|----------|-------|
| Visibility sweep | `visibility.cpp` | 676 |
| Containment | `router.cpp` | (part of 3139) |
| A* pathfinding | `makepath.cpp` | 1556 |
| Orthogonal routing | `orthogonal.cpp` | 3259 |
| VPSC solver | `vpsc.cpp` | ~1500 |
| Hyperedge improver | `hyperedgeimprover.cpp` | 1232 |
| Geometry | `geometry.cpp` | (various) |
| Scanline | `scanline.cpp` | (orthogonal support) |
| Vertices | `vertices.cpp` | (vertex management) |
| Graph edges | `graph.cpp` | (edge management) |

**Before implementing any task:**
1. Open the corresponding C++ file
2. Read and understand the algorithm
3. Port the logic to Rust
4. Do not deviate from the C++ approach

---

## Notes

### Implementation Principle

**Port, don't reimplement.** The current Rust codebase has several places where a "simpler" or "different" approach was taken instead of faithfully porting the C++ logic. This created the parity gaps documented in this plan.

When implementing these tasks:
- The C++ code is the specification
- If the C++ does something that seems unnecessary, it's probably necessary
- If you think you have a "better" way, you're probably wrong - port the C++ way first, optimize later (if ever)

### Breaking Changes

The following tasks may cause route output changes:
- #3, #4 (visibility cones) - Routes near corners may change
- #12 (bend penalty) - Route shape preference will change
- #14-16 (orthogonal visgraph) - All orthogonal routes may change

Consider versioning or feature flags for gradual migration.

### Dependencies

```
#2 depends on #1
#4 depends on #3
#9 depends on #6, #7, #8
#11 depends on #10
#15, #16 depend on #14
#18 depends on #17
#21, #22 depend on #20
#25 depends on #24
#28 depends on #27
#33 depends on #3
```

### Partial Implementation Acceptable

Some features can be partially implemented for value:
- #14-16: Even partial orthogonal visgraph improves quality
- #17-19: Basic VPSC works; optimization is enhancement
- #20-22: Hyperedge tree nice-to-have for most uses
