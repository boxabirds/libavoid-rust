# Testing Strategy for libavoid-rust Parity

## Overview

Testing WASM/JS bindings requires a multi-layered approach:

1. **Rust Unit Tests** - Core library logic (existing)
2. **WASM Integration Tests** - JS bindings work correctly
3. **API Compatibility Tests** - Same API as libavoid-js
4. **Behavioral Parity Tests** - Same outputs for same inputs
5. **Side-by-Side Comparison** - Run both libraries, compare results

---

## Test Infrastructure Setup

### Required Tools

```bash
# Node.js test runner
npm init -y
npm install --save-dev vitest @vitest/browser playwright

# WASM build tool
cargo install wasm-pack

# For running both libraries side-by-side
npm install --save-dev libavoid-js
```

### Project Structure

```
libavoid-rust/
├── src/                    # Rust source
├── tests/                  # Rust integration tests (existing)
├── js-tests/               # NEW: JavaScript test suite
│   ├── package.json
│   ├── vitest.config.js
│   ├── setup.js            # Load WASM before tests
│   ├── unit/               # Unit tests for each class
│   │   ├── point.test.js
│   │   ├── polygon.test.js
│   │   ├── router.test.js
│   │   ├── connref.test.js
│   │   └── ...
│   ├── integration/        # Full workflow tests
│   │   ├── routing.test.js
│   │   ├── callbacks.test.js
│   │   └── ...
│   ├── compatibility/      # API compatibility with libavoid-js
│   │   ├── api-surface.test.js
│   │   └── signatures.test.js
│   └── parity/             # Behavioral comparison tests
│       ├── compare-routes.test.js
│       └── compare-outputs.test.js
├── pkg/                    # wasm-pack output (generated)
└── docs/
```

---

## Layer 1: Rust Unit Tests

**Purpose:** Verify core Rust logic works correctly.
**Location:** `src/*.rs` (inline) and `tests/*.rs`
**Runner:** `cargo test`

Already have 43 tests. Continue adding as features are implemented.

```bash
# Run all Rust tests
cargo test

# Run with feature flags
cargo test --features wasm
```

---

## Layer 2: WASM Integration Tests

**Purpose:** Verify JS bindings expose correct APIs.
**Location:** `js-tests/unit/`
**Runner:** `vitest`

### Setup Files

**js-tests/package.json:**
```json
{
  "name": "libavoid-rust-tests",
  "type": "module",
  "scripts": {
    "build:wasm": "cd .. && wasm-pack build --target web --features wasm --out-dir js-tests/pkg",
    "test": "vitest run",
    "test:watch": "vitest",
    "test:ui": "vitest --ui"
  },
  "devDependencies": {
    "vitest": "^1.0.0",
    "libavoid-js": "^0.4.5"
  }
}
```

**js-tests/vitest.config.js:**
```javascript
import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    environment: 'node',
    setupFiles: ['./setup.js'],
    testTimeout: 10000,
    globals: true
  }
});
```

**js-tests/setup.js:**
```javascript
import { beforeAll, afterAll } from 'vitest';

// Global Avoid instance for tests
let Avoid = null;
let AvoidJS = null;

beforeAll(async () => {
  // Load libavoid-rust (our implementation)
  const { AvoidLib } = await import('./pkg/libavoid.js');
  await AvoidLib.load();
  Avoid = AvoidLib.getInstance();
  globalThis.Avoid = Avoid;

  // Load libavoid-js (reference implementation)
  const { AvoidLib: AvoidLibJS } = await import('libavoid-js');
  await AvoidLibJS.load();
  AvoidJS = AvoidLibJS.getInstance();
  globalThis.AvoidJS = AvoidJS;
});

afterAll(() => {
  // Cleanup if needed
});
```

### Example Unit Tests

**js-tests/unit/point.test.js:**
```javascript
import { describe, it, expect } from 'vitest';

describe('Point', () => {
  describe('constructor', () => {
    it('creates point with default values', () => {
      const pt = new Avoid.Point();
      expect(pt.x).toBe(0);
      expect(pt.y).toBe(0);
    });

    it('creates point with specified coordinates', () => {
      const pt = new Avoid.Point(3.5, 7.2);
      expect(pt.x).toBe(3.5);
      expect(pt.y).toBe(7.2);
    });
  });

  describe('properties', () => {
    it('allows setting x and y', () => {
      const pt = new Avoid.Point(0, 0);
      pt.x = 10;
      pt.y = 20;
      expect(pt.x).toBe(10);
      expect(pt.y).toBe(20);
    });

    it('has id property', () => {
      const pt = new Avoid.Point(0, 0);
      pt.id = 42;
      expect(pt.id).toBe(42);
    });

    it('has vn property', () => {
      const pt = new Avoid.Point(0, 0);
      pt.vn = 5;
      expect(pt.vn).toBe(5);
    });
  });

  describe('methods', () => {
    it('equal() compares points', () => {
      const pt1 = new Avoid.Point(1, 2);
      const pt2 = new Avoid.Point(1, 2);
      const pt3 = new Avoid.Point(1, 3);
      expect(pt1.equal(pt2)).toBe(true);
      expect(pt1.equal(pt3)).toBe(false);
    });
  });
});
```

**js-tests/unit/router.test.js:**
```javascript
import { describe, it, expect } from 'vitest';

describe('Router', () => {
  describe('constructor', () => {
    it('creates router with PolyLineRouting', () => {
      const router = new Avoid.Router(Avoid.PolyLineRouting);
      expect(router).toBeDefined();
    });

    it('creates router with OrthogonalRouting', () => {
      const router = new Avoid.Router(Avoid.OrthogonalRouting);
      expect(router).toBeDefined();
    });
  });

  describe('processTransaction', () => {
    it('returns boolean', () => {
      const router = new Avoid.Router(Avoid.PolyLineRouting);
      const result = router.processTransaction();
      expect(typeof result).toBe('boolean');
    });
  });

  describe('setRoutingParameter', () => {
    it('accepts parameter and value', () => {
      const router = new Avoid.Router(Avoid.PolyLineRouting);
      expect(() => {
        router.setRoutingParameter(Avoid.segmentPenalty, 50);
      }).not.toThrow();
    });
  });

  describe('setRoutingOption', () => {
    it('accepts option and boolean', () => {
      const router = new Avoid.Router(Avoid.PolyLineRouting);
      expect(() => {
        router.setRoutingOption(Avoid.nudgeOrthogonalSegmentsConnectedToShapes, true);
      }).not.toThrow();
    });
  });
});
```

**js-tests/unit/connref.test.js:**
```javascript
import { describe, it, expect, vi } from 'vitest';

describe('ConnRef', () => {
  let router;

  beforeEach(() => {
    router = new Avoid.Router(Avoid.PolyLineRouting);
  });

  describe('constructor', () => {
    it('creates with router only', () => {
      const conn = new Avoid.ConnRef(router);
      expect(conn).toBeDefined();
      expect(typeof conn.id()).toBe('number');
    });

    it('creates with router, src, dst', () => {
      const src = new Avoid.ConnEnd(new Avoid.Point(0, 0));
      const dst = new Avoid.ConnEnd(new Avoid.Point(10, 10));
      const conn = new Avoid.ConnRef(router, src, dst);
      expect(conn).toBeDefined();
    });

    it('creates with router, src, dst, id', () => {
      const src = new Avoid.ConnEnd(new Avoid.Point(0, 0));
      const dst = new Avoid.ConnEnd(new Avoid.Point(10, 10));
      const conn = new Avoid.ConnRef(router, src, dst, 42);
      expect(conn.id()).toBe(42);
    });
  });

  describe('setCallback', () => {
    it('registers callback that fires on reroute', async () => {
      const callback = vi.fn();
      const src = new Avoid.ConnEnd(new Avoid.Point(0, 0));
      const dst = new Avoid.ConnEnd(new Avoid.Point(10, 10));
      const conn = new Avoid.ConnRef(router, src, dst);

      conn.setCallback(callback, conn);
      router.processTransaction();

      expect(callback).toHaveBeenCalled();
    });

    it('callback receives pointer that can be wrapped', () => {
      let receivedPtr = null;
      const callback = (ptr) => { receivedPtr = ptr; };

      const src = new Avoid.ConnEnd(new Avoid.Point(0, 0));
      const dst = new Avoid.ConnEnd(new Avoid.Point(10, 10));
      const conn = new Avoid.ConnRef(router, src, dst);

      conn.setCallback(callback, conn);
      router.processTransaction();

      expect(typeof receivedPtr).toBe('number');

      const wrapped = Avoid.wrapPointer(receivedPtr, Avoid.ConnRef);
      expect(wrapped.id()).toBe(conn.id());
    });
  });

  describe('displayRoute', () => {
    it('returns route after processTransaction', () => {
      const src = new Avoid.ConnEnd(new Avoid.Point(0, 0));
      const dst = new Avoid.ConnEnd(new Avoid.Point(10, 10));
      const conn = new Avoid.ConnRef(router, src, dst);

      router.processTransaction();

      const route = conn.displayRoute();
      expect(route).toBeDefined();
      expect(route.size()).toBeGreaterThanOrEqual(2);
    });
  });

  describe('setHateCrossings', () => {
    it('sets and gets hate crossings flag', () => {
      const conn = new Avoid.ConnRef(router);
      conn.setHateCrossings(true);
      expect(conn.doesHateCrossings()).toBe(true);
      conn.setHateCrossings(false);
      expect(conn.doesHateCrossings()).toBe(false);
    });
  });
});
```

---

## Layer 3: API Compatibility Tests

**Purpose:** Verify our API surface matches libavoid-js exactly.
**Location:** `js-tests/compatibility/`

**js-tests/compatibility/api-surface.test.js:**
```javascript
import { describe, it, expect } from 'vitest';

describe('API Surface Compatibility', () => {
  describe('AvoidLib namespace', () => {
    it('has load() function', () => {
      expect(typeof globalThis.AvoidLib?.load).toBe('function');
    });

    it('has getInstance() function', () => {
      expect(typeof globalThis.AvoidLib?.getInstance).toBe('function');
    });
  });

  describe('Avoid namespace has all classes', () => {
    const requiredClasses = [
      'Point', 'Polygon', 'Rectangle', 'Box',
      'Router', 'ConnRef', 'ConnEnd', 'ShapeRef',
      'JunctionRef', 'ShapeConnectionPin', 'HyperedgeRerouter'
    ];

    requiredClasses.forEach(className => {
      it(`has ${className} class`, () => {
        expect(Avoid[className]).toBeDefined();
        expect(typeof Avoid[className]).toBe('function');
      });
    });
  });

  describe('Avoid namespace has all constants', () => {
    const requiredConstants = [
      // Router flags
      'PolyLineRouting', 'OrthogonalRouting',
      // Direction flags
      'ConnDirNone', 'ConnDirUp', 'ConnDirDown',
      'ConnDirLeft', 'ConnDirRight', 'ConnDirAll',
      // Connection types
      'ConnType_None', 'ConnType_PolyLine', 'ConnType_Orthogonal',
      // Routing parameters
      'segmentPenalty', 'anglePenalty', 'crossingPenalty',
      'clusterCrossingPenalty', 'fixedSharedPathPenalty',
      'portDirectionPenalty', 'shapeBufferDistance',
      'idealNudgingDistance', 'reverseDirectionPenalty',
      // Routing options
      'nudgeOrthogonalSegmentsConnectedToShapes',
      'improveHyperedgeRoutesMovingJunctions',
      'penaliseOrthogonalSharedPathsAtConnEnds',
      'nudgeOrthogonalTouchingColinearSegments',
      'performUnifyingNudgingPreprocessingStep',
      'improveHyperedgeRoutesMovingAddingAndDeletingJunctions',
      'nudgeSharedPathsWithCommonEndPoint'
    ];

    requiredConstants.forEach(constName => {
      it(`has ${constName} constant`, () => {
        expect(Avoid[constName]).toBeDefined();
        expect(typeof Avoid[constName]).toBe('number');
      });
    });
  });

  describe('Avoid namespace has utility functions', () => {
    it('has destroy() function', () => {
      expect(typeof Avoid.destroy).toBe('function');
    });

    it('has getPointer() function', () => {
      expect(typeof Avoid.getPointer).toBe('function');
    });

    it('has wrapPointer() function', () => {
      expect(typeof Avoid.wrapPointer).toBe('function');
    });
  });
});
```

**js-tests/compatibility/signatures.test.js:**
```javascript
import { describe, it, expect } from 'vitest';

describe('Method Signature Compatibility', () => {
  describe('Router methods match libavoid-js', () => {
    let router;
    beforeEach(() => {
      router = new Avoid.Router(Avoid.PolyLineRouting);
    });

    it('processTransaction() returns boolean', () => {
      const result = router.processTransaction();
      expect(typeof result).toBe('boolean');
    });

    it('moveShape(shape, x, y) accepts shape and offsets', () => {
      const poly = new Avoid.Polygon(4);
      poly.set_ps(0, new Avoid.Point(0, 0));
      poly.set_ps(1, new Avoid.Point(10, 0));
      poly.set_ps(2, new Avoid.Point(10, 10));
      poly.set_ps(3, new Avoid.Point(0, 10));
      const shape = new Avoid.ShapeRef(router, poly);

      expect(() => router.moveShape(shape, 5, 5)).not.toThrow();
    });

    it('moveShape(shape, polygon) accepts shape and polygon', () => {
      const poly1 = new Avoid.Polygon(4);
      poly1.set_ps(0, new Avoid.Point(0, 0));
      poly1.set_ps(1, new Avoid.Point(10, 0));
      poly1.set_ps(2, new Avoid.Point(10, 10));
      poly1.set_ps(3, new Avoid.Point(0, 10));
      const shape = new Avoid.ShapeRef(router, poly1);

      const poly2 = new Avoid.Polygon(4);
      poly2.set_ps(0, new Avoid.Point(5, 5));
      poly2.set_ps(1, new Avoid.Point(15, 5));
      poly2.set_ps(2, new Avoid.Point(15, 15));
      poly2.set_ps(3, new Avoid.Point(5, 15));

      expect(() => router.moveShape(shape, poly2)).not.toThrow();
    });
  });

  describe('ConnRef methods match libavoid-js', () => {
    let router;
    beforeEach(() => {
      router = new Avoid.Router(Avoid.PolyLineRouting);
    });

    it('displayRoute() returns PolyLine with size() and get_ps()', () => {
      const src = new Avoid.ConnEnd(new Avoid.Point(0, 0));
      const dst = new Avoid.ConnEnd(new Avoid.Point(10, 10));
      const conn = new Avoid.ConnRef(router, src, dst);
      router.processTransaction();

      const route = conn.displayRoute();
      expect(typeof route.size).toBe('function');
      expect(typeof route.get_ps).toBe('function');

      const size = route.size();
      expect(typeof size).toBe('number');

      if (size > 0) {
        const pt = route.get_ps(0);
        expect(pt.x).toBeDefined();
        expect(pt.y).toBeDefined();
      }
    });
  });
});
```

---

## Layer 4: Behavioral Parity Tests

**Purpose:** Verify same inputs produce same outputs as libavoid-js.
**Location:** `js-tests/parity/`

**js-tests/parity/compare-routes.test.js:**
```javascript
import { describe, it, expect } from 'vitest';

// Tolerance for floating point comparison
const EPSILON = 0.0001;

function pointsEqual(p1, p2) {
  return Math.abs(p1.x - p2.x) < EPSILON && Math.abs(p1.y - p2.y) < EPSILON;
}

function routesEqual(route1, route2) {
  if (route1.size() !== route2.size()) return false;
  for (let i = 0; i < route1.size(); i++) {
    if (!pointsEqual(route1.get_ps(i), route2.get_ps(i))) return false;
  }
  return true;
}

describe('Behavioral Parity', () => {
  describe('Simple direct route', () => {
    it('produces same route as libavoid-js', () => {
      // Our implementation
      const router1 = new Avoid.Router(Avoid.PolyLineRouting);
      const src1 = new Avoid.ConnEnd(new Avoid.Point(0, 0));
      const dst1 = new Avoid.ConnEnd(new Avoid.Point(100, 100));
      const conn1 = new Avoid.ConnRef(router1, src1, dst1);
      router1.processTransaction();
      const route1 = conn1.displayRoute();

      // Reference implementation
      const router2 = new AvoidJS.Router(AvoidJS.PolyLineRouting);
      const src2 = new AvoidJS.ConnEnd(new AvoidJS.Point(0, 0));
      const dst2 = new AvoidJS.ConnEnd(new AvoidJS.Point(100, 100));
      const conn2 = new AvoidJS.ConnRef(router2, src2, dst2);
      router2.processTransaction();
      const route2 = conn2.displayRoute();

      expect(route1.size()).toBe(route2.size());
      expect(routesEqual(route1, route2)).toBe(true);
    });
  });

  describe('Route around obstacle', () => {
    it('produces same route as libavoid-js', () => {
      // Our implementation
      const router1 = new Avoid.Router(Avoid.PolyLineRouting);
      const shapePoly1 = new Avoid.Polygon(4);
      shapePoly1.set_ps(0, new Avoid.Point(40, 40));
      shapePoly1.set_ps(1, new Avoid.Point(60, 40));
      shapePoly1.set_ps(2, new Avoid.Point(60, 60));
      shapePoly1.set_ps(3, new Avoid.Point(40, 60));
      new Avoid.ShapeRef(router1, shapePoly1);

      const src1 = new Avoid.ConnEnd(new Avoid.Point(0, 50));
      const dst1 = new Avoid.ConnEnd(new Avoid.Point(100, 50));
      const conn1 = new Avoid.ConnRef(router1, src1, dst1);
      router1.processTransaction();
      const route1 = conn1.displayRoute();

      // Reference implementation
      const router2 = new AvoidJS.Router(AvoidJS.PolyLineRouting);
      const shapePoly2 = new AvoidJS.Polygon(4);
      shapePoly2.set_ps(0, new AvoidJS.Point(40, 40));
      shapePoly2.set_ps(1, new AvoidJS.Point(60, 40));
      shapePoly2.set_ps(2, new AvoidJS.Point(60, 60));
      shapePoly2.set_ps(3, new AvoidJS.Point(40, 60));
      new AvoidJS.ShapeRef(router2, shapePoly2);

      const src2 = new AvoidJS.ConnEnd(new AvoidJS.Point(0, 50));
      const dst2 = new AvoidJS.ConnEnd(new AvoidJS.Point(100, 50));
      const conn2 = new AvoidJS.ConnRef(router2, src2, dst2);
      router2.processTransaction();
      const route2 = conn2.displayRoute();

      // Routes should be similar (may not be identical due to implementation differences)
      // At minimum, both should have more than 2 points (went around obstacle)
      expect(route1.size()).toBeGreaterThan(2);
      expect(route2.size()).toBeGreaterThan(2);

      // Start and end points should match
      expect(pointsEqual(route1.get_ps(0), route2.get_ps(0))).toBe(true);
      expect(pointsEqual(
        route1.get_ps(route1.size() - 1),
        route2.get_ps(route2.size() - 1)
      )).toBe(true);
    });
  });

  describe('Orthogonal routing', () => {
    it('produces orthogonal route like libavoid-js', () => {
      // Our implementation
      const router1 = new Avoid.Router(Avoid.OrthogonalRouting);
      const src1 = new Avoid.ConnEnd(new Avoid.Point(0, 0));
      const dst1 = new Avoid.ConnEnd(new Avoid.Point(100, 100));
      const conn1 = new Avoid.ConnRef(router1, src1, dst1);
      router1.processTransaction();
      const route1 = conn1.displayRoute();

      // Verify our route is orthogonal
      for (let i = 0; i < route1.size() - 1; i++) {
        const p1 = route1.get_ps(i);
        const p2 = route1.get_ps(i + 1);
        const isHorizontal = Math.abs(p1.y - p2.y) < EPSILON;
        const isVertical = Math.abs(p1.x - p2.x) < EPSILON;
        expect(isHorizontal || isVertical).toBe(true);
      }
    });
  });
});
```

---

## Layer 5: Port libavoid-js Examples as Tests

**Purpose:** Ensure the exact usage patterns from libavoid-js work.
**Location:** `js-tests/integration/`

**js-tests/integration/main-example.test.js:**
```javascript
import { describe, it, expect, vi } from 'vitest';

/**
 * This test ports the exact example from libavoid-js/examples/main.js
 */
describe('libavoid-js main.js example', () => {
  it('runs the complete example workflow', async () => {
    const router = new Avoid.Router(Avoid.PolyLineRouting);

    const srcPt = new Avoid.Point(1.2, 0.5);
    const dstPt = new Avoid.Point(1.5, 4);

    const srcConnEnd = new Avoid.ConnEnd(srcPt);
    const dstConnEnd = new Avoid.ConnEnd(dstPt);
    const connRef = new Avoid.ConnRef(router, srcConnEnd, dstConnEnd);

    const callbackCalls = [];
    function connCallback(connRefPtr) {
      const wrapped = Avoid.wrapPointer(connRefPtr, Avoid.ConnRef);
      const route = wrapped.displayRoute();
      callbackCalls.push({
        id: wrapped.id(),
        routeSize: route.size()
      });
    }

    connRef.setCallback(connCallback, connRef);

    // Force initial callback
    router.processTransaction();
    expect(callbackCalls.length).toBeGreaterThanOrEqual(1);

    // Adding a shape
    const shapePoly = new Avoid.Polygon(3);
    shapePoly.set_ps(0, new Avoid.Point(1, 1));
    shapePoly.set_ps(1, new Avoid.Point(2.5, 1.5));
    shapePoly.set_ps(2, new Avoid.Point(1.5, 2.5));
    const shapeRef = new Avoid.ShapeRef(router, shapePoly);
    router.processTransaction();

    // Shifting endpoint
    const dstPt2 = new Avoid.Point(6, 4.5);
    connRef.setDestEndpoint(new Avoid.ConnEnd(dstPt2));
    router.processTransaction();

    // Moving shape
    router.moveShape(shapeRef, 0.5, 0);
    router.processTransaction();

    // Verify callbacks were called appropriately
    expect(callbackCalls.length).toBeGreaterThan(1);
  });
});
```

---

## Running the Tests

### Command Summary

```bash
# 1. Build WASM
cd js-tests
npm run build:wasm

# 2. Run all JS tests
npm test

# 3. Run specific test file
npm test -- unit/point.test.js

# 4. Run with coverage
npm test -- --coverage

# 5. Run in watch mode during development
npm run test:watch

# 6. Run Rust tests (separate)
cd ..
cargo test
```

### CI/CD Integration

**.github/workflows/test.yml:**
```yaml
name: Tests

on: [push, pull_request]

jobs:
  rust-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test
      - run: cargo test --features wasm

  wasm-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - run: cargo install wasm-pack
      - run: cd js-tests && npm ci
      - run: cd js-tests && npm run build:wasm
      - run: cd js-tests && npm test
```

---

## Test Checklist per Phase

Use this checklist when implementing each phase:

### Phase N Completion Criteria

- [ ] All new public APIs have unit tests
- [ ] API signatures match libavoid-js (compatibility tests pass)
- [ ] Behavioral parity tests pass for affected functionality
- [ ] No regressions in existing tests
- [ ] CI passes

---

## Quick Start

```bash
# One-time setup
mkdir js-tests
cd js-tests
npm init -y
npm install --save-dev vitest libavoid-js
cargo install wasm-pack

# Create test files from templates above

# Run tests
npm run build:wasm
npm test
```

---

## Summary

| Layer | Purpose | Location | When to Run |
|-------|---------|----------|-------------|
| Rust Unit | Core logic | `tests/`, `src/` | Every commit |
| WASM Unit | JS bindings | `js-tests/unit/` | After WASM changes |
| API Compat | Surface match | `js-tests/compatibility/` | After API changes |
| Behavioral | Output match | `js-tests/parity/` | Before release |
| Integration | Full workflow | `js-tests/integration/` | Before release |
