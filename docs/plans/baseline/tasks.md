# libavoid-rust Baseline Implementation Tasks

## Overview

This document catalogues all tasks required to complete the baseline libavoid-rust implementation with full parity to the C++ version.

## Task Status Legend

- **pending**: Not started
- **in_progress**: Currently being worked on
- **done**: Completed and tested

---

## Task Catalogue

| ID | Name | Status | Dependencies | Estimate |
|----|------|--------|--------------|----------|
| 1 | Fix geometry segment intersection | **done** | - | S |
| 2 | Implement proper polygon intersection test | **done** | 1 | S |
| 3 | Fix is_direct_path_clear to use polygon test | **done** | 2 | S |
| 4 | Implement VertInf with full C++ fields | **done** | - | M |
| 5 | Implement EdgeInf with full C++ fields | **done** | 4 | S |
| 6 | Rewrite VisibilityGraph with proper structure | **done** | 4, 5 | L |
| 7 | Implement visibility computation (scanline) | **done** | 6 | L |
| 8 | Implement A* with cost function | **done** | 6 | M |
| 9 | Add segment penalty to path cost | **done** | 8 | S |
| 10 | Add angle penalty to path cost | **done** | 8 | S |
| 11 | Add crossing penalty to path cost | **done** | 8, 22 | M |
| 12 | Implement ConnEnd with all endpoint types | **done** | - | M |
| 13 | Implement ShapeConnectionPin fully | **done** | 12 | M |
| 14 | Implement pin selection algorithm | **done** | 13 | M |
| 15 | Implement Checkpoint routing | **done** | 8 | M |
| 16 | Implement ActionInfo transaction queue | **done** | - | S |
| 17 | Implement proper transaction processing | **done** | 16 | M |
| 18 | Implement shape add/remove/move actions | **done** | 17 | M |
| 19 | Implement connector reroute queue | **done** | 17 | S |
| 20 | Implement JunctionRef fully | **done** | 4 | M |
| 21 | Implement junction routing | **done** | 20, 8 | M |
| 22 | Implement connector crossing detection | **done** | 2 | M |
| 23 | Implement orthogonal graph building | **done** | 6 | L |
| 24 | Implement orthogonal A* routing | **done** | 23, 8 | M |
| 25 | Implement route nudging | **done** | 24 | L |
| 26 | Implement HyperedgeRerouter stub | **done** | 21 | M |
| 27 | Implement hyperedge improvement | **done** | 26 | L |
| 28 | Add router configuration parameters | **done** | - | S |
| 29 | Add router configuration options | **done** | 28 | S |
| 30 | Implement fixed route support | **done** | 8 | S |
| 31 | Implement connector callbacks | **done** | - | S |
| 32 | Update WASM bindings for new types | **done** | All core | M |
| 33 | Create unit test suite for geometry | **done** | 1, 2 | M |
| 34 | Create unit test suite for visibility | **done** | 6, 7 | M |
| 35 | Create unit test suite for path finding | **done** | 8-11 | M |
| 36 | Create integration test suite | **done** | 17-21 | L |
| 37 | Create parity test suite vs libavoid-js | pending | 32 | L |
| 38 | Create property-based tests | pending | 36 | M |
| 39 | Create performance benchmarks | pending | 36 | M |
| 40 | Documentation and examples | pending | All | M |

---

## Size Estimates

- **S (Small)**: < 100 lines, < 1 hour
- **M (Medium)**: 100-500 lines, 1-4 hours
- **L (Large)**: 500+ lines, 4+ hours

---

## Dependency Graph

```
1 → 2 → 3
        ↓
4 → 5 → 6 → 7
        ↓
        8 → 9, 10, 11, 15, 30
        ↓
       14 ← 13 ← 12

16 → 17 → 18, 19
          ↓
20 → 21 → 26 → 27

6 → 23 → 24 → 25

22 → 11

All core → 32 → 37
33-39 (tests can run in parallel)
```

---

## Phase Breakdown

### Phase 1: Core Geometry (Tasks 1-3)
Fix fundamental geometry operations that are broken.

### Phase 2: Visibility Graph (Tasks 4-7)
Rebuild visibility graph with proper structure.

### Phase 3: Path Finding (Tasks 8-11)
Implement A* with full cost function.

### Phase 4: Connection System (Tasks 12-15)
Full endpoint and pin support.

### Phase 5: Transaction System (Tasks 16-19)
Proper action queue and processing.

### Phase 6: Junction Support (Tasks 20-21)
Full junction routing.

### Phase 7: Orthogonal Routing (Tasks 23-25)
Complete orthogonal routing system.

### Phase 8: Advanced Features (Tasks 22, 26-31)
Crossings, hyperedges, configuration.

### Phase 9: WASM & Testing (Tasks 32-39)
Bindings and comprehensive tests.

### Phase 10: Polish (Task 40)
Documentation and examples.
