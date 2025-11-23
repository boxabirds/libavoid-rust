# libavoid-rust API Parity Implementation Plan

**Goal:** 100% drop-in compatibility with libavoid-js
**Created:** 2025-11-23
**Status:** In Progress

---

## Progress Overview

| Phase | Description | Tasks | Complete | Status |
|-------|-------------|-------|----------|--------|
| 1 | WASM Infrastructure | 8 | 0 | Not Started |
| 2 | Geometry Types | 12 | 0 | Not Started |
| 3 | Router Enhancements | 14 | 0 | Not Started |
| 4 | Connector System | 18 | 0 | Not Started |
| 5 | Shape & Obstacle System | 10 | 0 | Not Started |
| 6 | Junction System | 8 | 0 | Not Started |
| 7 | Connection Pins | 12 | 0 | Not Started |
| 8 | Hyperedge Support | 6 | 0 | Not Started |
| 9 | Constants & Enums | 10 | 0 | Not Started |
| 10 | Utility Functions | 6 | 0 | Not Started |
| 11 | TypeScript Definitions | 4 | 0 | Not Started |
| 12 | Integration Tests | 10 | 0 | Not Started |
| **Total** | | **118** | **0** | **0%** |

---

## Phase 1: WASM Infrastructure (CRITICAL)

**Priority:** P0 - Blocking
**Dependencies:** None
**Estimated effort:** 2-3 days

### 1.1 AvoidLib Restructure

- [ ] **1.1.1** Change `AvoidLib::load()` to return `Promise<JsValue>`
  - File: `src/wasm.rs`
  - Use `wasm_bindgen_futures` for async support
  - Accept optional `file_path: Option<String>` parameter
  - Acceptance: `await AvoidLib.load()` works in JS

- [ ] **1.1.2** Create `Avoid` namespace struct
  - File: `src/wasm.rs`
  - New struct that holds all class constructors and constants
  - Acceptance: `AvoidLib.getInstance()` returns object with all exports

- [ ] **1.1.3** Change `getInstance()` to return `Avoid` namespace
  - File: `src/wasm.rs`
  - Must return object, not struct instance
  - Acceptance: `Avoid.Point`, `Avoid.Router` etc. accessible

- [ ] **1.1.4** Add initialization state tracking
  - File: `src/wasm.rs`
  - Track if `load()` was called
  - Throw error if `getInstance()` called before `load()`
  - Acceptance: Proper error message on premature access

### 1.2 Build Configuration

- [ ] **1.2.1** Add `wasm-bindgen-futures` dependency
  - File: `Cargo.toml`
  - Required for async WASM functions
  - Acceptance: `cargo build --features wasm` succeeds

- [ ] **1.2.2** Add `js-sys` dependency
  - File: `Cargo.toml`
  - Required for Function callbacks and JsValue handling
  - Acceptance: Can use `js_sys::Function` in wasm.rs

- [ ] **1.2.3** Add `web-sys` dependency (optional features)
  - File: `Cargo.toml`
  - For console logging if needed
  - Acceptance: Optional debug logging works

- [ ] **1.2.4** Configure wasm-pack output format
  - File: `Cargo.toml` or package.json
  - Ensure ESM output matches libavoid-js
  - Acceptance: `import { AvoidLib } from 'libavoid-rust'` works

---

## Phase 2: Geometry Types

**Priority:** P0 - Blocking
**Dependencies:** Phase 1.1
**Estimated effort:** 2 days

### 2.1 Point Enhancements

- [ ] **2.1.1** Add `id` getter/setter to WASM Point
  - File: `src/wasm.rs`
  - Acceptance: `point.id = 5; console.log(point.id)` works

- [ ] **2.1.2** Add `vn` getter/setter to WASM Point
  - File: `src/wasm.rs`
  - Acceptance: `point.vn = 2; console.log(point.vn)` works

- [ ] **2.1.3** Add `equal(other)` method to WASM Point
  - File: `src/wasm.rs`
  - Acceptance: `point1.equal(point2)` returns boolean

- [ ] **2.1.4** Add default constructor `Point()`
  - File: `src/wasm.rs`
  - Creates point at (0, 0)
  - Acceptance: `new Avoid.Point()` creates (0,0) point

### 2.2 Polygon Enhancements

- [ ] **2.2.1** Add `clear()` method
  - File: `src/wasm.rs`
  - Acceptance: `polygon.clear()` empties polygon

- [ ] **2.2.2** Add `empty()` method
  - File: `src/wasm.rs`
  - Acceptance: `polygon.empty()` returns boolean

- [ ] **2.2.3** Add `id()` method
  - File: `src/wasm.rs`
  - Acceptance: `polygon.id()` returns number

- [ ] **2.2.4** Add `at(index)` method
  - File: `src/wasm.rs`
  - Acceptance: `polygon.at(0)` returns Point

- [ ] **2.2.5** Add `setPoint(index, point)` method
  - File: `src/wasm.rs`
  - Alternative name for set_ps
  - Acceptance: `polygon.setPoint(0, pt)` works

- [ ] **2.2.6** Add `boundingRectPolygon()` method
  - File: `src/wasm.rs`
  - Acceptance: Returns Polygon representing bounding rect

- [ ] **2.2.7** Add `offsetBoundingBox(offset)` method
  - File: `src/wasm.rs`
  - Acceptance: Returns Box

- [ ] **2.2.8** Add `offsetPolygon(offset)` method
  - File: `src/wasm.rs`
  - Acceptance: Returns Polygon

### 2.3 Rectangle (NEW)

- [ ] **2.3.1** Create Rectangle WASM struct
  - File: `src/wasm.rs`
  - Extends/wraps Polygon
  - Acceptance: `new Avoid.Rectangle(...)` works

- [ ] **2.3.2** Add constructor `Rectangle(centre, width, height)`
  - File: `src/wasm.rs`
  - Acceptance: Creates rectangle centered at point

- [ ] **2.3.3** Add constructor `Rectangle(topLeft, bottomRight)`
  - File: `src/wasm.rs`
  - Note: WASM doesn't support true overloading, use different method
  - Acceptance: `Avoid.Rectangle.fromCorners(tl, br)` or similar

### 2.4 Box (NEW)

- [ ] **2.4.1** Create Box WASM struct
  - File: `src/wasm.rs`
  - Properties: min, max (Points)
  - Acceptance: `new Avoid.Box()` works

- [ ] **2.4.2** Add `min` and `max` getters/setters
  - File: `src/wasm.rs`
  - Acceptance: `box.min`, `box.max` return Points

- [ ] **2.4.3** Add `length(dimension)` method
  - File: `src/wasm.rs`
  - Acceptance: `box.length(0)` returns width

- [ ] **2.4.4** Add `width()` and `height()` methods
  - File: `src/wasm.rs`
  - Acceptance: `box.width()`, `box.height()` return numbers

---

## Phase 3: Router Enhancements

**Priority:** P0 - Blocking
**Dependencies:** Phase 2
**Estimated effort:** 2-3 days

### 3.1 Constructor Fix

- [ ] **3.1.1** Change Router constructor to accept number flag
  - File: `src/wasm.rs`
  - Current: `Router(RoutingType)` enum
  - Required: `Router(flags: u32)`
  - Acceptance: `new Avoid.Router(Avoid.PolyLineRouting)` works

### 3.2 Missing Methods

- [ ] **3.2.1** Add `printInfo()` method
  - File: `src/wasm.rs`
  - Logs router state to console
  - Acceptance: `router.printInfo()` outputs debug info

- [ ] **3.2.2** Add `deleteConnector(connRef)` method
  - File: `src/wasm.rs`
  - Acceptance: `router.deleteConnector(conn)` removes connector

- [ ] **3.2.3** Add `deleteShape(shapeRef)` method
  - File: `src/wasm.rs`
  - Acceptance: `router.deleteShape(shape)` removes shape

- [ ] **3.2.4** Add `moveShape(shape, newPolygon)` overload
  - File: `src/wasm.rs`
  - Different from existing offset version
  - Acceptance: `router.moveShape(shape, polygon)` works

- [ ] **3.2.5** Add `moveJunction(junction, point)` method
  - File: `src/wasm.rs`
  - Acceptance: `router.moveJunction(junc, pt)` works

- [ ] **3.2.6** Add `moveJunction(junction, xDiff, yDiff)` method
  - File: `src/wasm.rs`
  - Acceptance: `router.moveJunction(junc, 5, 10)` works

### 3.3 Configuration Methods

- [ ] **3.3.1** Add `setRoutingParameter(param, value)` to WASM
  - File: `src/wasm.rs`
  - Accept numeric parameter ID
  - Acceptance: `router.setRoutingParameter(Avoid.segmentPenalty, 50)` works

- [ ] **3.3.2** Add `setRoutingOption(option, value)` to WASM
  - File: `src/wasm.rs`
  - Accept numeric option ID
  - Acceptance: `router.setRoutingOption(Avoid.nudgeOrthogonalSegmentsConnectedToShapes, true)` works

### 3.4 Internal Router Improvements

- [ ] **3.4.1** Track shapes by reference in Router
  - File: `src/wasm.rs`
  - Router needs to own/track shapes for moveShape(shape, polygon)
  - Acceptance: Shape references remain valid after operations

- [ ] **3.4.2** Track connectors by reference in Router
  - File: `src/wasm.rs`
  - Router needs to own/track connectors
  - Acceptance: Connector references remain valid after operations

- [ ] **3.4.3** Track junctions by reference in Router
  - File: `src/wasm.rs`
  - Router needs to own/track junctions
  - Acceptance: Junction references remain valid after operations

- [ ] **3.4.4** Implement processTransaction return value
  - File: `src/wasm.rs`
  - Should return boolean
  - Acceptance: `const result = router.processTransaction()` returns boolean

---

## Phase 4: Connector System

**Priority:** P0 - Blocking
**Dependencies:** Phase 3
**Estimated effort:** 3-4 days

### 4.1 ConnRef Constructors

- [ ] **4.1.1** Add `ConnRef(router)` basic constructor
  - File: `src/wasm.rs`
  - Already exists, verify signature
  - Acceptance: `new Avoid.ConnRef(router)` works

- [ ] **4.1.2** Add `ConnRef(router, src, dst)` constructor
  - File: `src/wasm.rs`
  - Acceptance: `new Avoid.ConnRef(router, srcEnd, dstEnd)` works

- [ ] **4.1.3** Add `ConnRef(router, src, dst, id)` constructor
  - File: `src/wasm.rs`
  - With optional ID parameter
  - Acceptance: `new Avoid.ConnRef(router, srcEnd, dstEnd, 42)` works

### 4.2 ConnRef Methods

- [ ] **4.2.1** Implement functional `setCallback(callback, connRef)`
  - File: `src/wasm.rs`
  - Store js_sys::Function reference
  - Call on reroute events
  - Callback receives raw pointer (number)
  - Acceptance: Callback fires when route changes

- [ ] **4.2.2** Add `setSourceEndpoint(connEnd)` method
  - File: `src/wasm.rs`
  - Acceptance: `connRef.setSourceEndpoint(end)` works

- [ ] **4.2.3** Verify `setDestEndpoint(connEnd)` method
  - File: `src/wasm.rs`
  - Already exists, verify signature matches
  - Acceptance: `connRef.setDestEndpoint(end)` works

- [ ] **4.2.4** Add `routingType()` getter
  - File: `src/wasm.rs`
  - Returns ConnType enum value as number
  - Acceptance: `connRef.routingType()` returns 0, 1, or 2

- [ ] **4.2.5** Add `setRoutingType(type)` method
  - File: `src/wasm.rs`
  - Accepts ConnType enum value as number
  - Acceptance: `connRef.setRoutingType(Avoid.ConnType_Orthogonal)` works

- [ ] **4.2.6** Add `setHateCrossings(value)` method
  - File: `src/wasm.rs`
  - Also add to core Rust ConnRef if missing
  - Acceptance: `connRef.setHateCrossings(true)` works

- [ ] **4.2.7** Add `doesHateCrossings()` method
  - File: `src/wasm.rs`
  - Acceptance: `connRef.doesHateCrossings()` returns boolean

- [ ] **4.2.8** Verify `displayRoute()` returns correct type
  - File: `src/wasm.rs`
  - Should return PolyLine (alias for Polygon)
  - Acceptance: `connRef.displayRoute().size()` works

### 4.3 ConnEnd Constructors

- [ ] **4.3.1** Verify `ConnEnd(point)` constructor
  - File: `src/wasm.rs`
  - Already exists, verify signature
  - Acceptance: `new Avoid.ConnEnd(point)` works

- [ ] **4.3.2** Add `ConnEnd(shapeRef, connectionPinClassID)` constructor
  - File: `src/wasm.rs`
  - For connecting to shape pins
  - Acceptance: `new Avoid.ConnEnd(shape, pinClassId)` works

- [ ] **4.3.3** Add `createConnEndFromJunctionRef(junctionRef)` static method
  - File: `src/wasm.rs`
  - Factory method for junction-based endpoints
  - Acceptance: `Avoid.ConnEnd.createConnEndFromJunctionRef(junc)` works

### 4.4 Core Rust ConnRef Additions

- [ ] **4.4.1** Add `hate_crossings` field to ConnRef
  - File: `src/connector.rs`
  - Boolean field with getter/setter
  - Acceptance: Rust tests pass

- [ ] **4.4.2** Add callback storage to ConnRef
  - File: `src/connector.rs`
  - Store callback function reference
  - Acceptance: Callback mechanism works in Rust

- [ ] **4.4.3** Trigger callbacks on reroute
  - File: `src/router.rs`
  - Call connector callbacks after processTransaction
  - Acceptance: Callbacks fire at appropriate times

### 4.5 PolyLine Type Alias

- [ ] **4.5.1** Export PolyLine as alias for Polygon
  - File: `src/wasm.rs`
  - `type PolyLine = Polygon` or similar
  - Acceptance: `Avoid.PolyLine` exists and is usable

### 4.6 Checkpoint Class

- [ ] **4.6.1** Create Checkpoint WASM struct
  - File: `src/wasm.rs`
  - Constructor: `Checkpoint(point)`
  - Acceptance: `new Avoid.Checkpoint(pt)` works

---

## Phase 5: Shape & Obstacle System

**Priority:** P1 - High
**Dependencies:** Phase 3
**Estimated effort:** 2 days

### 5.1 ShapeRef Enhancements

- [ ] **5.1.1** Add optional `id` parameter to constructor
  - File: `src/wasm.rs`
  - `ShapeRef(router, polygon, id?)`
  - Acceptance: `new Avoid.ShapeRef(router, poly, 42)` works

- [ ] **5.1.2** Add `polygon()` method
  - File: `src/wasm.rs`
  - Returns copy of shape's polygon
  - Acceptance: `shape.polygon()` returns Polygon

- [ ] **5.1.3** Add `position()` method
  - File: `src/wasm.rs`
  - Returns shape's position Point
  - Acceptance: `shape.position()` returns Point

- [ ] **5.1.4** Add `setNewPoly(polygon)` method
  - File: `src/wasm.rs`
  - Updates shape's polygon
  - Acceptance: `shape.setNewPoly(newPoly)` works

### 5.2 Obstacle Interface

- [ ] **5.2.1** Create Obstacle WASM interface/trait
  - File: `src/wasm.rs`
  - Base for ShapeRef and JunctionRef
  - Acceptance: Both ShapeRef and JunctionRef implement it

- [ ] **5.2.2** Add `id()` to Obstacle interface
  - File: `src/wasm.rs`
  - Acceptance: `obstacle.id()` works for any obstacle

- [ ] **5.2.3** Add `polygon()` to Obstacle interface
  - File: `src/wasm.rs`
  - Acceptance: `obstacle.polygon()` works

- [ ] **5.2.4** Add `router()` to Obstacle interface
  - File: `src/wasm.rs`
  - Returns reference to owning router
  - Acceptance: `obstacle.router()` returns Router

- [ ] **5.2.5** Add `position()` to Obstacle interface
  - File: `src/wasm.rs`
  - Acceptance: `obstacle.position()` returns Point

- [ ] **5.2.6** Add `setNewPoly(polygon)` to Obstacle interface
  - File: `src/wasm.rs`
  - Acceptance: `obstacle.setNewPoly(poly)` works

---

## Phase 6: Junction System

**Priority:** P1 - High
**Dependencies:** Phase 5
**Estimated effort:** 2 days

### 6.1 JunctionRef WASM Export

- [ ] **6.1.1** Create JunctionRef WASM struct
  - File: `src/wasm.rs`
  - Wrap existing RustJunctionRef
  - Acceptance: `new Avoid.JunctionRef(...)` works

- [ ] **6.1.2** Add `JunctionRef(router, point)` constructor
  - File: `src/wasm.rs`
  - Acceptance: `new Avoid.JunctionRef(router, pt)` works

- [ ] **6.1.3** Add `JunctionRef(router, point, id)` constructor
  - File: `src/wasm.rs`
  - With optional ID
  - Acceptance: `new Avoid.JunctionRef(router, pt, 42)` works

### 6.2 JunctionRef Methods

- [ ] **6.2.1** Add `position()` method
  - File: `src/wasm.rs`
  - Returns junction position
  - Acceptance: `junction.position()` returns Point

- [ ] **6.2.2** Add `setPositionFixed(fixed)` method
  - File: `src/wasm.rs`
  - Also add to core Rust JunctionRef
  - Acceptance: `junction.setPositionFixed(true)` works

- [ ] **6.2.3** Add `positionFixed()` method
  - File: `src/wasm.rs`
  - Acceptance: `junction.positionFixed()` returns boolean

- [ ] **6.2.4** Add `recommendedPosition()` method
  - File: `src/wasm.rs`
  - Returns optimized position suggestion
  - Acceptance: `junction.recommendedPosition()` returns Point

### 6.3 Core Rust Junction Additions

- [ ] **6.3.1** Add `position_fixed` field to JunctionRef
  - File: `src/junction.rs`
  - Boolean field with getter/setter
  - Acceptance: Rust tests pass

---

## Phase 7: Connection Pins

**Priority:** P1 - High
**Dependencies:** Phase 5, Phase 6
**Estimated effort:** 2-3 days

### 7.1 ShapeConnectionPin WASM Export

- [ ] **7.1.1** Create ShapeConnectionPin WASM struct
  - File: `src/wasm.rs`
  - Wrap existing Rust ConnectionPin
  - Acceptance: `new Avoid.ShapeConnectionPin(...)` works

- [ ] **7.1.2** Add constructor `(shape, classId, xOffset, yOffset, proportional, insideOffset, visDirs)`
  - File: `src/wasm.rs`
  - Full constructor with all parameters
  - Acceptance: Full constructor works

- [ ] **7.1.3** Add constructor `(shape, classId, xOffset, yOffset, insideOffset, visDirs)`
  - File: `src/wasm.rs`
  - Without proportional parameter
  - Acceptance: Simplified constructor works

- [ ] **7.1.4** Add constructor `(junction, classId, visDirs?)`
  - File: `src/wasm.rs`
  - For junction-attached pins
  - Acceptance: Junction pin constructor works

### 7.2 ShapeConnectionPin Methods

- [ ] **7.2.1** Add `setConnectionCost(cost)` method
  - File: `src/wasm.rs`
  - Acceptance: `pin.setConnectionCost(5.0)` works

- [ ] **7.2.2** Add `position(newPoly?)` method
  - File: `src/wasm.rs`
  - Optional polygon parameter
  - Acceptance: `pin.position()` returns Point

- [ ] **7.2.3** Add `directions()` method
  - File: `src/wasm.rs`
  - Returns ConnDirFlags
  - Acceptance: `pin.directions()` returns number

- [ ] **7.2.4** Add `setExclusive(exclusive)` method
  - File: `src/wasm.rs`
  - Acceptance: `pin.setExclusive(true)` works

- [ ] **7.2.5** Add `isExclusive()` method
  - File: `src/wasm.rs`
  - Acceptance: `pin.isExclusive()` returns boolean

- [ ] **7.2.6** Add `updatePosition(newPosition)` method
  - File: `src/wasm.rs`
  - Acceptance: `pin.updatePosition(pt)` works

### 7.3 Core Rust Connection Pin Additions

- [ ] **7.3.1** Add `connection_cost` field
  - File: `src/shape.rs`
  - With getter/setter
  - Acceptance: Rust tests pass

- [ ] **7.3.2** Add `exclusive` field
  - File: `src/shape.rs`
  - Boolean with getter/setter
  - Acceptance: Rust tests pass

---

## Phase 8: Hyperedge Support

**Priority:** P2 - Medium
**Dependencies:** Phase 6
**Estimated effort:** 1-2 days

### 8.1 HyperedgeRerouter WASM Export

- [ ] **8.1.1** Create HyperedgeRerouter WASM struct
  - File: `src/wasm.rs`
  - Wrap existing Rust HyperedgeRerouter
  - Acceptance: `new Avoid.HyperedgeRerouter()` works

- [ ] **8.1.2** Add default constructor
  - File: `src/wasm.rs`
  - Acceptance: `new Avoid.HyperedgeRerouter()` works

- [ ] **8.1.3** Add `registerHyperedgeForRerouting(junction)` method
  - File: `src/wasm.rs`
  - Acceptance: `rerouter.registerHyperedgeForRerouting(junc)` returns ID

### 8.2 Core Rust Hyperedge Additions

- [ ] **8.2.1** Implement `registerHyperedgeForRerouting` in Rust
  - File: `src/hyperedge.rs`
  - Acceptance: Rust tests pass

- [ ] **8.2.2** Add `HyperedgeNewAndDeletedObjectLists` struct
  - File: `src/hyperedge.rs`
  - For tracking hyperedge changes
  - Acceptance: Struct exists with correct fields

- [ ] **8.2.3** Integrate HyperedgeRerouter with Router
  - File: `src/router.rs`
  - Acceptance: Router can use HyperedgeRerouter

---

## Phase 9: Constants & Enums

**Priority:** P0 - Blocking
**Dependencies:** Phase 1.2
**Estimated effort:** 1 day

### 9.1 Router Flags

- [ ] **9.1.1** Export `PolyLineRouting` constant
  - File: `src/wasm.rs`
  - Value: `0` or appropriate flag value
  - Acceptance: `Avoid.PolyLineRouting` equals expected value

- [ ] **9.1.2** Export `OrthogonalRouting` constant
  - File: `src/wasm.rs`
  - Acceptance: `Avoid.OrthogonalRouting` equals expected value

### 9.2 Connection Direction Flags

- [ ] **9.2.1** Export `ConnDirNone` constant (value: 0)
- [ ] **9.2.2** Export `ConnDirUp` constant (value: 1)
- [ ] **9.2.3** Export `ConnDirDown` constant (value: 2)
- [ ] **9.2.4** Export `ConnDirLeft` constant (value: 4)
- [ ] **9.2.5** Export `ConnDirRight` constant (value: 8)
- [ ] **9.2.6** Export `ConnDirAll` constant (value: 15)
  - File: `src/wasm.rs`
  - Acceptance: All direction flags accessible on Avoid namespace

### 9.3 Connection Types

- [ ] **9.3.1** Export `ConnType_None` constant (value: 0)
- [ ] **9.3.2** Export `ConnType_PolyLine` constant (value: 1)
- [ ] **9.3.3** Export `ConnType_Orthogonal` constant (value: 2)
  - File: `src/wasm.rs`
  - Acceptance: All connection types accessible

### 9.4 Routing Parameters

- [ ] **9.4.1** Export `segmentPenalty` constant (value: 0)
- [ ] **9.4.2** Export `anglePenalty` constant (value: 1)
- [ ] **9.4.3** Export `crossingPenalty` constant (value: 2)
- [ ] **9.4.4** Export `clusterCrossingPenalty` constant (value: 3)
- [ ] **9.4.5** Export `fixedSharedPathPenalty` constant (value: 4)
- [ ] **9.4.6** Export `portDirectionPenalty` constant (value: 5)
- [ ] **9.4.7** Export `shapeBufferDistance` constant (value: 6)
- [ ] **9.4.8** Export `idealNudgingDistance` constant (value: 7)
- [ ] **9.4.9** Export `reverseDirectionPenalty` constant (value: 8)
  - File: `src/wasm.rs`
  - Acceptance: All parameters accessible

### 9.5 Routing Options

- [ ] **9.5.1** Export `nudgeOrthogonalSegmentsConnectedToShapes` constant (value: 0)
- [ ] **9.5.2** Export `improveHyperedgeRoutesMovingJunctions` constant (value: 1)
- [ ] **9.5.3** Export `penaliseOrthogonalSharedPathsAtConnEnds` constant (value: 2)
- [ ] **9.5.4** Export `nudgeOrthogonalTouchingColinearSegments` constant (value: 3)
- [ ] **9.5.5** Export `performUnifyingNudgingPreprocessingStep` constant (value: 4)
- [ ] **9.5.6** Export `improveHyperedgeRoutesMovingAddingAndDeletingJunctions` constant (value: 5)
- [ ] **9.5.7** Export `nudgeSharedPathsWithCommonEndPoint` constant (value: 6)
  - File: `src/wasm.rs`
  - Acceptance: All options accessible

---

## Phase 10: Utility Functions

**Priority:** P1 - High
**Dependencies:** Phase 1
**Estimated effort:** 1-2 days

### 10.1 Pointer Management

- [ ] **10.1.1** Implement `destroy(obj)` function
  - File: `src/wasm.rs`
  - Manually free WASM object
  - Acceptance: `Avoid.destroy(shape)` cleans up object

- [ ] **10.1.2** Implement `getPointer(obj)` function
  - File: `src/wasm.rs`
  - Returns raw pointer as number
  - Acceptance: `Avoid.getPointer(connRef)` returns number

- [ ] **10.1.3** Implement `wrapPointer(ptr, Class)` function
  - File: `src/wasm.rs`
  - Wraps raw pointer back to JS object
  - Required for callback interop
  - Acceptance: `Avoid.wrapPointer(ptr, Avoid.ConnRef)` returns ConnRef

### 10.2 Object Tracking

- [ ] **10.2.1** Create pointer registry for tracking live objects
  - File: `src/wasm.rs`
  - HashMap<u32, Box<dyn Any>> or similar
  - Acceptance: Objects can be retrieved by pointer

- [ ] **10.2.2** Implement pointer allocation
  - File: `src/wasm.rs`
  - Assign unique IDs to objects
  - Acceptance: Each object gets unique pointer

- [ ] **10.2.3** Implement pointer deallocation
  - File: `src/wasm.rs`
  - Clean up on destroy()
  - Acceptance: Memory properly freed

---

## Phase 11: TypeScript Definitions

**Priority:** P1 - High
**Dependencies:** All previous phases
**Estimated effort:** 1 day

### 11.1 Type Definition File

- [ ] **11.1.1** Create `dist/index.d.ts`
  - File: `dist/index.d.ts` or `typings/libavoid.d.ts`
  - Match libavoid-js type definitions exactly
  - Acceptance: TypeScript compilation succeeds with types

- [ ] **11.1.2** Add interface definitions for all classes
  - Acceptance: All classes have proper type definitions

- [ ] **11.1.3** Add enum/constant type definitions
  - Acceptance: All constants properly typed

- [ ] **11.1.4** Add AvoidLib namespace definition
  - Acceptance: `AvoidLib.load()` and `AvoidLib.getInstance()` typed correctly

---

## Phase 12: Integration Tests

**Priority:** P1 - High
**Dependencies:** All previous phases
**Estimated effort:** 2 days

### 12.1 Port libavoid-js Examples

- [ ] **12.1.1** Port `examples/main.js` as integration test
  - File: `tests/wasm_integration.rs` or JS test file
  - Acceptance: Test passes with same behavior

- [ ] **12.1.2** Port `examples/node-standalone/main.mjs` as test
  - Acceptance: Node.js usage pattern works

### 12.2 API Compatibility Tests

- [ ] **12.2.1** Test callback mechanism
  - Verify callbacks fire correctly
  - Verify pointer wrapping works
  - Acceptance: Callback receives valid pointer, wrapping works

- [ ] **12.2.2** Test all constructor overloads
  - Each class with multiple constructors
  - Acceptance: All constructor patterns work

- [ ] **12.2.3** Test routing parameter/option setting
  - Acceptance: Parameters affect routing behavior

- [ ] **12.2.4** Test shape/connector lifecycle
  - Create, modify, delete
  - Acceptance: Full lifecycle works

- [ ] **12.2.5** Test junction-based routing
  - Acceptance: Junctions work as in libavoid-js

- [ ] **12.2.6** Test connection pins
  - Acceptance: Pin-based connections work

### 12.3 Behavioral Parity Tests

- [ ] **12.3.1** Test same input produces same output as libavoid-js
  - Run identical scenarios in both libraries
  - Compare route points
  - Acceptance: Routes match (within tolerance)

- [ ] **12.3.2** Performance benchmark comparison
  - Compare routing performance
  - Acceptance: Equal or better performance

---

## Appendix A: File Change Summary

| File | Changes Required |
|------|-----------------|
| `Cargo.toml` | Add dependencies: wasm-bindgen-futures, js-sys, web-sys |
| `src/wasm.rs` | Major rewrite (~800+ lines added) |
| `src/connector.rs` | Add hate_crossings, callback storage |
| `src/junction.rs` | Add position_fixed field |
| `src/shape.rs` | Add connection pin enhancements |
| `src/hyperedge.rs` | Add registerHyperedgeForRerouting |
| `src/router.rs` | Add callback triggering, pointer tracking |
| `dist/index.d.ts` | New file for TypeScript types |
| `tests/wasm_integration.rs` | New integration tests |

---

## Appendix B: Verification Checklist

Before marking complete, verify each phase against libavoid-js:

```javascript
// Phase 1 verification
await AvoidLib.load();
const Avoid = AvoidLib.getInstance();
assert(typeof Avoid.Router === 'function');
assert(typeof Avoid.PolyLineRouting === 'number');

// Phase 2 verification
const pt = new Avoid.Point(1, 2);
assert(pt.x === 1 && pt.y === 2);
const rect = new Avoid.Rectangle(pt, 10, 20);
assert(rect.size() === 4);

// Phase 3 verification
const router = new Avoid.Router(Avoid.PolyLineRouting);
router.setRoutingParameter(Avoid.segmentPenalty, 50);
assert(router.processTransaction() !== undefined);

// Phase 4 verification
const srcEnd = new Avoid.ConnEnd(new Avoid.Point(0, 0));
const dstEnd = new Avoid.ConnEnd(new Avoid.Point(10, 10));
const conn = new Avoid.ConnRef(router, srcEnd, dstEnd);
let callbackFired = false;
conn.setCallback((ptr) => { callbackFired = true; }, conn);
router.processTransaction();
assert(callbackFired);

// Continue for all phases...
```

---

## Appendix C: Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Callback complexity | Study wasm-bindgen closure patterns; test extensively |
| Memory leaks | Implement comprehensive destroy(); add leak detection tests |
| API signature mismatches | Create compatibility shims where needed |
| Performance regression | Benchmark after each phase |
| Breaking existing consumers | Maintain backwards compatibility during transition |

---

*Last updated: 2025-11-23*
