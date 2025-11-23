/**
 * Test setup - loads both libavoid-rust (our impl) and libavoid-js (reference)
 *
 * After setup, tests can access:
 * - globalThis.Avoid      - Our implementation (namespace with classes)
 * - globalThis.AvoidJS    - Reference implementation (libavoid-js)
 * - globalThis.AvoidLib   - Our AvoidLib class
 * - globalThis.AvoidLibJS - Reference AvoidLib namespace
 */

import { beforeAll, afterAll } from 'vitest';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { readFile } from 'fs/promises';

const __dirname = dirname(fileURLToPath(import.meta.url));

beforeAll(async () => {
  console.log('Setup: Starting beforeAll...');

  // Load libavoid-rust (our implementation)
  try {
    const wasmPath = join(__dirname, 'pkg', 'libavoid_bg.wasm');
    console.log('Setup: WASM path:', wasmPath);

    // Read WASM file
    const wasmBuffer = await readFile(wasmPath);

    // Dynamic import of our WASM module
    const libavoidRust = await import('./pkg/libavoid.js');

    // Initialize WASM with the buffer (using initSync for Node.js)
    if (libavoidRust.initSync) {
      libavoidRust.initSync({ module: wasmBuffer });
    } else if (libavoidRust.default) {
      await libavoidRust.default({ module: wasmBuffer });
    }

    // Create an Avoid-like namespace object with all exports
    globalThis.Avoid = {
      // Classes
      Point: libavoidRust.Point,
      Polygon: libavoidRust.Polygon,
      Router: libavoidRust.Router,
      ConnRef: libavoidRust.ConnRef,
      ConnEnd: libavoidRust.ConnEnd,
      ShapeRef: libavoidRust.ShapeRef,
      AvoidLib: libavoidRust.AvoidLib,

      // Enums/Constants (from RoutingType)
      PolyLineRouting: libavoidRust.RoutingType?.PolyLineRouting ?? 0,
      OrthogonalRouting: libavoidRust.RoutingType?.OrthogonalRouting ?? 1,

      // TODO: These need to be added to wasm.rs
      // Direction flags
      ConnDirNone: undefined,
      ConnDirUp: undefined,
      ConnDirDown: undefined,
      ConnDirLeft: undefined,
      ConnDirRight: undefined,
      ConnDirAll: undefined,

      // Connection types
      ConnType_None: undefined,
      ConnType_PolyLine: undefined,
      ConnType_Orthogonal: undefined,

      // Routing parameters - need to be exported
      segmentPenalty: undefined,
      anglePenalty: undefined,
      crossingPenalty: undefined,

      // Utility functions (not yet implemented)
      destroy: undefined,
      getPointer: undefined,
      wrapPointer: undefined,
    };

    globalThis.AvoidLib = libavoidRust.AvoidLib;
    console.log('libavoid-rust loaded successfully');
  } catch (err) {
    console.warn('Could not load libavoid-rust:', err.message);
    console.warn('Run `npm run build:wasm` first');
    globalThis.Avoid = null;
    globalThis.AvoidLib = null;
  }

  // Load libavoid-js (reference implementation)
  try {
    const { AvoidLib: AvoidLibJS } = await import('libavoid-js');
    await AvoidLibJS.load();
    globalThis.AvoidJS = AvoidLibJS.getInstance();
    globalThis.AvoidLibJS = AvoidLibJS;
    console.log('libavoid-js (reference) loaded successfully');
  } catch (err) {
    console.warn('Could not load libavoid-js:', err.message);
    console.warn('Run `npm install` to install dependencies');
    globalThis.AvoidJS = null;
    globalThis.AvoidLibJS = null;
  }
});

afterAll(() => {
  // Cleanup if needed
  globalThis.Avoid = null;
  globalThis.AvoidJS = null;
  globalThis.AvoidLib = null;
  globalThis.AvoidLibJS = null;
});
