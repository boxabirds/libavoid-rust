# Migrating from libavoid-js to libavoid-rust

This guide provides exhaustive examples for migrating code from libavoid-js (Emscripten-based) to libavoid-rust (wasm-bindgen-based).

## Table of Contents

1. [Overview](#overview)
2. [Module Loading](#module-loading)
3. [Constructor Changes](#constructor-changes)
4. [Callback System](#callback-system)
5. [Pointer Functions](#pointer-functions)
6. [Memory Management](#memory-management)
7. [Complete Migration Examples](#complete-migration-examples)
8. [API Reference Comparison](#api-reference-comparison)

---

## Overview

### Key Differences

| Aspect | libavoid-js | libavoid-rust |
|--------|-------------|---------------|
| WASM toolchain | Emscripten | wasm-bindgen |
| Memory model | Manual (pointers) | Garbage collected |
| Constructor overloading | Supported | Factory methods |
| Callbacks | Pointer-based | Not supported* |
| Object destruction | `Avoid.destroy(obj)` | `obj.free()` or automatic GC |

*Callbacks require a different architectural approach in wasm-bindgen. See [Callback System](#callback-system).

---

## Module Loading

### libavoid-js

```javascript
// Browser (ES Module)
import { AvoidLib } from 'libavoid-js';

async function init() {
  await AvoidLib.load();
  const Avoid = AvoidLib.getInstance();
  // Use Avoid...
}

// Or with script tag
<script src="libavoid.js"></script>
<script>
  AvoidLib.load().then(() => {
    const Avoid = AvoidLib.getInstance();
  });
</script>
```

### libavoid-rust

```javascript
// Browser (ES Module)
import init, {
  Router, Point, Polygon, ConnRef, ConnEnd, ShapeRef,
  JunctionRef, ShapeConnectionPin, HyperedgeRerouter, Box, Rectangle
} from './pkg/libavoid.js';

async function main() {
  await init();  // Initialize WASM

  // Use classes directly - no getInstance() needed
  const router = new Router(1);  // PolyLineRouting
}

// Node.js
import { readFile } from 'fs/promises';
import { initSync, Router, Point, ... } from './pkg/libavoid.js';

const wasmBuffer = await readFile('./pkg/libavoid_bg.wasm');
initSync({ module: wasmBuffer });

const router = new Router(1);
```

### Migration Pattern

```javascript
// BEFORE (libavoid-js)
import { AvoidLib } from 'libavoid-js';
await AvoidLib.load();
const Avoid = AvoidLib.getInstance();
const router = new Avoid.Router(Avoid.PolyLineRouting);

// AFTER (libavoid-rust)
import init, { Router } from './pkg/libavoid.js';
await init();
const PolyLineRouting = 1;
const router = new Router(PolyLineRouting);
```

---

## Constructor Changes

libavoid-rust uses **factory methods** instead of constructor overloading because wasm-bindgen doesn't support multiple constructors with different signatures.

### ConnRef

```javascript
// BEFORE (libavoid-js) - Multiple constructor forms
const conn1 = new Avoid.ConnRef(router);
const conn2 = new Avoid.ConnRef(router, srcEnd, dstEnd);
const conn3 = new Avoid.ConnRef(router, srcEnd, dstEnd, 123);  // with ID

// AFTER (libavoid-rust) - Factory methods
const conn1 = new ConnRef(router);
const conn2 = ConnRef.createWithEndpoints(router, srcEnd, dstEnd);
const conn3 = ConnRef.createWithId(router, srcEnd, dstEnd, 123);
```

### ShapeRef

```javascript
// BEFORE (libavoid-js)
const shape1 = new Avoid.ShapeRef(router, polygon);
const shape2 = new Avoid.ShapeRef(router, polygon, 456);  // with ID

// AFTER (libavoid-rust)
const shape1 = new ShapeRef(router, polygon);
const shape2 = ShapeRef.createWithId(router, polygon, 456);
```

### JunctionRef

```javascript
// BEFORE (libavoid-js)
const junction1 = new Avoid.JunctionRef(router, position);
const junction2 = new Avoid.JunctionRef(router, position, 789);  // with ID

// AFTER (libavoid-rust)
const junction1 = new JunctionRef(router, position);
const junction2 = JunctionRef.createWithId(router, position, 789);
```

### ConnEnd

```javascript
// BEFORE (libavoid-js)
const end1 = new Avoid.ConnEnd(point);
const end2 = new Avoid.ConnEnd(shapeRef, pinClassId);

// AFTER (libavoid-rust)
const end1 = new ConnEnd(point);
const end2 = ConnEnd.fromShapePin(shapeRef, pinClassId);
```

### Rectangle

```javascript
// BEFORE (libavoid-js)
const rect1 = new Avoid.Rectangle(center, width, height);
const rect2 = new Avoid.Rectangle(topLeft, bottomRight);

// AFTER (libavoid-rust)
const rect1 = new Rectangle(center, width, height);
const rect2 = Rectangle.fromCorners(topLeft, bottomRight);
```

### Box

```javascript
// BEFORE (libavoid-js)
const box = new Avoid.Box(minPoint, maxPoint);

// AFTER (libavoid-rust)
const box = Box.fromCoords(minX, minY, maxX, maxY);
// Or construct empty and set properties
const box2 = new Box();
box2.min = minPoint;
box2.max = maxPoint;
```

### ShapeConnectionPin

```javascript
// BEFORE (libavoid-js)
const pin1 = new Avoid.ShapeConnectionPin(shape, classId, xOffset, yOffset, insideOffset, visDirs);
const pin2 = new Avoid.ShapeConnectionPin(junction, classId, visDirs);

// AFTER (libavoid-rust)
const pin1 = new ShapeConnectionPin(shape, classId, xOffset, yOffset, insideOffset, visDirs);
const pin2 = ShapeConnectionPin.createOnJunction(junction, classId, visDirs);
```

---

## Callback System

### The Problem

libavoid-js uses Emscripten's pointer-based callback system:

```javascript
// libavoid-js callback pattern
function connCallback(connRefPtr) {
  const connRef = Avoid.wrapPointer(connRefPtr, Avoid.ConnRef);
  console.log(`Connector ${connRef.id()} needs rerouting!`);
  const route = connRef.displayRoute();
  // Process route...
}

connRef.setCallback(connCallback, connRef);
router.processTransaction();  // Callbacks fire automatically
```

**This pattern cannot be directly replicated in wasm-bindgen** because:
1. wasm-bindgen uses garbage-collected JavaScript objects, not raw pointers
2. `wrapPointer()` and `getPointer()` are Emscripten-specific functions
3. Callbacks would need to store JavaScript closures in Rust, which is complex

### Migration Strategy: Poll-Based Updates

Instead of callbacks, poll for route changes after transactions:

```javascript
// AFTER (libavoid-rust) - Poll-based approach
function processAndCheckRoutes(router, connectors) {
  router.processTransaction();

  for (const conn of connectors) {
    const route = conn.displayRoute();
    if (route && route.size() > 0) {
      console.log(`Connector ${conn.id()} route:`);
      for (let i = 0; i < route.size(); i++) {
        const pt = route.get_ps(i);
        console.log(`  (${pt.x}, ${pt.y})`);
      }
    }
  }
}

// Usage
const connectors = [conn1, conn2, conn3];
processAndCheckRoutes(router, connectors);

// After making changes
router.moveShape(shape, 10, 20);
processAndCheckRoutes(router, connectors);
```

### Migration Strategy: Event Wrapper

Create a wrapper that tracks connectors and emits events:

```javascript
// AFTER (libavoid-rust) - Event-based wrapper
class RouterWrapper extends EventTarget {
  constructor(flags) {
    super();
    this.router = new Router(flags);
    this.connectors = new Map();
    this.previousRoutes = new Map();
  }

  addConnector(srcEnd, dstEnd) {
    const conn = ConnRef.createWithEndpoints(this.router, srcEnd, dstEnd);
    this.connectors.set(conn.id(), conn);
    return conn;
  }

  processTransaction() {
    this.router.processTransaction();

    // Check for route changes
    for (const [id, conn] of this.connectors) {
      const route = conn.displayRoute();
      const routeKey = this.routeToString(route);
      const previousKey = this.previousRoutes.get(id);

      if (routeKey !== previousKey) {
        this.previousRoutes.set(id, routeKey);
        this.dispatchEvent(new CustomEvent('routechange', {
          detail: { connectorId: id, connector: conn, route }
        }));
      }
    }
  }

  routeToString(route) {
    if (!route) return '';
    const points = [];
    for (let i = 0; i < route.size(); i++) {
      const pt = route.get_ps(i);
      points.push(`${pt.x},${pt.y}`);
    }
    return points.join('|');
  }
}

// Usage
const wrapper = new RouterWrapper(PolyLineRouting);

wrapper.addEventListener('routechange', (e) => {
  console.log(`Connector ${e.detail.connectorId} changed!`);
});

const conn = wrapper.addConnector(srcEnd, dstEnd);
wrapper.processTransaction();
```

---

## Pointer Functions

### getPointer() and wrapPointer()

These Emscripten functions **do not work** in libavoid-rust:

```javascript
// BEFORE (libavoid-js)
const ptr = Avoid.getPointer(connRef);  // Returns memory address
const obj = Avoid.wrapPointer(ptr, Avoid.ConnRef);  // Wraps pointer to object

// AFTER (libavoid-rust)
// These functions exist but return dummy values:
Avoid.getPointer(obj);  // Always returns 0
Avoid.wrapPointer(ptr, Type);  // Always returns null

// Instead, keep direct references to objects:
const connectors = new Map();
connectors.set(conn.id(), conn);

// Retrieve by ID
const conn = connectors.get(id);
```

### Migration Pattern

```javascript
// BEFORE (libavoid-js)
function storeConnector(conn) {
  const ptr = Avoid.getPointer(conn);
  pointerMap.set(ptr, true);
}

function getConnector(ptr) {
  return Avoid.wrapPointer(ptr, Avoid.ConnRef);
}

// AFTER (libavoid-rust)
const connectorMap = new Map();

function storeConnector(conn) {
  connectorMap.set(conn.id(), conn);
}

function getConnector(id) {
  return connectorMap.get(id);
}
```

---

## Memory Management

### libavoid-js (Emscripten)

```javascript
// Manual destruction required
const shape = new Avoid.ShapeRef(router, polygon);
// ... use shape ...
Avoid.destroy(shape);  // Must call to free memory
```

### libavoid-rust (wasm-bindgen)

```javascript
// Option 1: Automatic garbage collection (recommended)
const shape = new ShapeRef(router, polygon);
// shape is automatically freed when no longer referenced

// Option 2: Explicit free (if needed)
const shape = new ShapeRef(router, polygon);
shape.free();  // Explicitly free memory

// Option 3: Using Symbol.dispose (modern JS)
{
  using shape = new ShapeRef(router, polygon);
  // shape.free() called automatically at block end
}

// The destroy() function still works for compatibility:
Avoid.destroy(shape);  // Calls shape.free() internally
```

---

## Complete Migration Examples

### Example 1: Basic Routing

```javascript
// ==================== BEFORE (libavoid-js) ====================
import { AvoidLib } from 'libavoid-js';

async function main() {
  await AvoidLib.load();
  const Avoid = AvoidLib.getInstance();

  const router = new Avoid.Router(Avoid.PolyLineRouting);

  const srcPt = new Avoid.Point(0, 0);
  const dstPt = new Avoid.Point(100, 100);
  const srcEnd = new Avoid.ConnEnd(srcPt);
  const dstEnd = new Avoid.ConnEnd(dstPt);

  const conn = new Avoid.ConnRef(router, srcEnd, dstEnd);

  router.processTransaction();

  const route = conn.displayRoute();
  for (let i = 0; i < route.size(); i++) {
    console.log(route.get_ps(i).x, route.get_ps(i).y);
  }
}

// ==================== AFTER (libavoid-rust) ====================
import init, { Router, Point, ConnEnd, ConnRef } from './pkg/libavoid.js';

const PolyLineRouting = 1;

async function main() {
  await init();

  const router = new Router(PolyLineRouting);

  const srcPt = new Point(0, 0);
  const dstPt = new Point(100, 100);
  const srcEnd = new ConnEnd(srcPt);
  const dstEnd = new ConnEnd(dstPt);

  // Changed: Use factory method instead of constructor overload
  const conn = ConnRef.createWithEndpoints(router, srcEnd, dstEnd);

  router.processTransaction();

  const route = conn.displayRoute();
  if (route) {
    for (let i = 0; i < route.size(); i++) {
      console.log(route.get_ps(i).x, route.get_ps(i).y);
    }
  }
}
```

### Example 2: Shapes with Connection Pins

```javascript
// ==================== BEFORE (libavoid-js) ====================
const Avoid = AvoidLib.getInstance();
const router = new Avoid.Router(Avoid.OrthogonalRouting);

// Create shape
const poly = new Avoid.Polygon(4);
poly.set_ps(0, new Avoid.Point(0, 0));
poly.set_ps(1, new Avoid.Point(100, 0));
poly.set_ps(2, new Avoid.Point(100, 100));
poly.set_ps(3, new Avoid.Point(0, 100));
const shape = new Avoid.ShapeRef(router, poly);

// Add connection pins
const topPin = new Avoid.ShapeConnectionPin(shape, 1, 50, 0, 0, Avoid.ConnDirUp);
const bottomPin = new Avoid.ShapeConnectionPin(shape, 2, 50, 100, 0, Avoid.ConnDirDown);

// Connect to pin
const connEnd = new Avoid.ConnEnd(shape, 1);  // Connect to pin class 1
const conn = new Avoid.ConnRef(router, new Avoid.ConnEnd(new Avoid.Point(-50, 50)), connEnd);

router.processTransaction();

// ==================== AFTER (libavoid-rust) ====================
import init, {
  Router, Point, Polygon, ShapeRef, ShapeConnectionPin, ConnEnd, ConnRef
} from './pkg/libavoid.js';

const OrthogonalRouting = 2;
const ConnDirUp = 1;
const ConnDirDown = 2;

await init();

const router = new Router(OrthogonalRouting);

// Create shape (same as before)
const poly = new Polygon(4);
poly.set_ps(0, new Point(0, 0));
poly.set_ps(1, new Point(100, 0));
poly.set_ps(2, new Point(100, 100));
poly.set_ps(3, new Point(0, 100));
const shape = new ShapeRef(router, poly);

// Add connection pins (same as before)
const topPin = new ShapeConnectionPin(shape, 1, 50, 0, 0, ConnDirUp);
const bottomPin = new ShapeConnectionPin(shape, 2, 50, 100, 0, ConnDirDown);

// Connect to pin - Changed: Use factory method
const connEnd = ConnEnd.fromShapePin(shape, 1);
const srcEnd = new ConnEnd(new Point(-50, 50));
const conn = ConnRef.createWithEndpoints(router, srcEnd, connEnd);

router.processTransaction();
```

### Example 3: Callbacks to Polling

```javascript
// ==================== BEFORE (libavoid-js) ====================
const Avoid = AvoidLib.getInstance();
const router = new Avoid.Router(Avoid.PolyLineRouting);

function handleRouteChange(connRefPtr) {
  const conn = Avoid.wrapPointer(connRefPtr, Avoid.ConnRef);
  const route = conn.displayRoute();
  updateUI(conn.id(), route);
}

const conn = new Avoid.ConnRef(router, srcEnd, dstEnd);
conn.setCallback(handleRouteChange, conn);

// Changes automatically trigger callback
router.processTransaction();
addShape();
router.processTransaction();  // Callback fires

// ==================== AFTER (libavoid-rust) ====================
import init, { Router, ConnRef, ConnEnd, Point } from './pkg/libavoid.js';

await init();

const PolyLineRouting = 1;
const router = new Router(PolyLineRouting);

// Track connectors manually
const connectors = [];

function handleRouteChange(conn) {
  const route = conn.displayRoute();
  if (route) {
    updateUI(conn.id(), route);
  }
}

function processAndNotify() {
  router.processTransaction();
  // Poll all connectors for changes
  for (const conn of connectors) {
    handleRouteChange(conn);
  }
}

const srcEnd = new ConnEnd(new Point(0, 0));
const dstEnd = new ConnEnd(new Point(100, 100));
const conn = ConnRef.createWithEndpoints(router, srcEnd, dstEnd);
connectors.push(conn);

// Must explicitly check routes after each transaction
processAndNotify();
addShape();
processAndNotify();
```

### Example 4: Junctions and Hyperedges

```javascript
// ==================== BEFORE (libavoid-js) ====================
const Avoid = AvoidLib.getInstance();
const router = new Avoid.Router(Avoid.PolyLineRouting);

const junction = new Avoid.JunctionRef(router, new Avoid.Point(100, 100));
const rerouter = new Avoid.HyperedgeRerouter();
const hyperedgeId = rerouter.registerHyperedgeForRerouting(junction);

// ==================== AFTER (libavoid-rust) ====================
import init, { Router, Point, JunctionRef, HyperedgeRerouter } from './pkg/libavoid.js';

await init();

const PolyLineRouting = 1;
const router = new Router(PolyLineRouting);

const junction = new JunctionRef(router, new Point(100, 100));
const rerouter = new HyperedgeRerouter();
const hyperedgeId = rerouter.registerHyperedgeForRerouting(junction);
// Same API - no changes needed
```

---

## API Reference Comparison

### Constants

| libavoid-js | libavoid-rust | Value |
|-------------|---------------|-------|
| `Avoid.PolyLineRouting` | `PolyLineRouting` (define locally) | `1` |
| `Avoid.OrthogonalRouting` | `OrthogonalRouting` (define locally) | `2` |
| `Avoid.ConnDirNone` | `ConnDirNone` | `0` |
| `Avoid.ConnDirUp` | `ConnDirUp` | `1` |
| `Avoid.ConnDirDown` | `ConnDirDown` | `2` |
| `Avoid.ConnDirLeft` | `ConnDirLeft` | `4` |
| `Avoid.ConnDirRight` | `ConnDirRight` | `8` |
| `Avoid.ConnDirAll` | `ConnDirAll` | `15` |
| `Avoid.ConnType_PolyLine` | `ConnType_PolyLine` | `1` |
| `Avoid.ConnType_Orthogonal` | `ConnType_Orthogonal` | `2` |

### Router Parameters

| Constant | Value | Description |
|----------|-------|-------------|
| `segmentPenalty` | `0` | Cost per path segment |
| `anglePenalty` | `1` | Cost per bend |
| `crossingPenalty` | `2` | Cost per crossing |
| `clusterCrossingPenalty` | `3` | Cost for crossing cluster boundaries |
| `fixedSharedPathPenalty` | `4` | Cost for shared path segments |
| `portDirectionPenalty` | `5` | Cost for wrong port direction |
| `shapeBufferDistance` | `6` | Buffer around shapes |
| `idealNudgingDistance` | `7` | Preferred nudging distance |
| `reverseDirectionPenalty` | `8` | Cost for reverse direction |

### Class Method Changes

| Class | libavoid-js | libavoid-rust |
|-------|-------------|---------------|
| ConnRef | `new ConnRef(router, src, dst)` | `ConnRef.createWithEndpoints(router, src, dst)` |
| ConnRef | `new ConnRef(router, src, dst, id)` | `ConnRef.createWithId(router, src, dst, id)` |
| ShapeRef | `new ShapeRef(router, poly, id)` | `ShapeRef.createWithId(router, poly, id)` |
| JunctionRef | `new JunctionRef(router, pos, id)` | `JunctionRef.createWithId(router, pos, id)` |
| ConnEnd | `new ConnEnd(shape, pinClass)` | `ConnEnd.fromShapePin(shape, pinClass)` |
| Rectangle | `new Rectangle(p1, p2)` | `Rectangle.fromCorners(p1, p2)` |
| ShapeConnectionPin | `new ShapeConnectionPin(junction, ...)` | `ShapeConnectionPin.createOnJunction(junction, ...)` |

### Unsupported Functions

| Function | libavoid-js | libavoid-rust | Notes |
|----------|-------------|---------------|-------|
| `getPointer(obj)` | Returns memory address | Returns `0` | Not available in wasm-bindgen |
| `wrapPointer(ptr, Type)` | Wraps pointer to object | Returns `null` | Not available in wasm-bindgen |
| `setCallback(fn, ctx)` | Sets route change callback | No-op | Use polling instead |

---

## Summary

### Quick Reference Card

```javascript
// Import pattern
// BEFORE: const Avoid = AvoidLib.getInstance();
// AFTER:  import { Router, Point, ... } from './pkg/libavoid.js';

// ConnRef with endpoints
// BEFORE: new Avoid.ConnRef(router, src, dst)
// AFTER:  ConnRef.createWithEndpoints(router, src, dst)

// ConnRef with ID
// BEFORE: new Avoid.ConnRef(router, src, dst, id)
// AFTER:  ConnRef.createWithId(router, src, dst, id)

// ConnEnd from pin
// BEFORE: new Avoid.ConnEnd(shape, pinClass)
// AFTER:  ConnEnd.fromShapePin(shape, pinClass)

// ShapeRef with ID
// BEFORE: new Avoid.ShapeRef(router, poly, id)
// AFTER:  ShapeRef.createWithId(router, poly, id)

// Rectangle from corners
// BEFORE: new Avoid.Rectangle(p1, p2)
// AFTER:  Rectangle.fromCorners(p1, p2)

// Memory cleanup
// BEFORE: Avoid.destroy(obj)
// AFTER:  obj.free() or let GC handle it

// Callbacks
// BEFORE: conn.setCallback(fn, ctx)
// AFTER:  Poll after processTransaction()
```

### Checklist for Migration

- [ ] Change import pattern from `AvoidLib.getInstance()` to direct imports
- [ ] Replace constructor overloads with factory methods
- [ ] Remove callback-based code; implement polling
- [ ] Remove `getPointer()` and `wrapPointer()` calls
- [ ] Track objects by ID instead of pointer
- [ ] Define constants locally (or import from a constants file)
- [ ] Replace `Avoid.destroy()` with `obj.free()` or rely on GC
- [ ] Test all route retrieval code handles null/undefined routes
