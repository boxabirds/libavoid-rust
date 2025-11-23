/**
 * libavoid-rust Node.js Example
 *
 * This example demonstrates basic connector routing around obstacles.
 * It mirrors the libavoid-js node-standalone example but uses the wasm-bindgen API.
 *
 * Run with: node main.mjs
 */

import { readFile } from 'fs/promises';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __dirname = dirname(fileURLToPath(import.meta.url));

// Constants (matching libavoid-js)
const PolyLineRouting = 1;
const OrthogonalRouting = 2;

async function main() {
  console.log('libavoid-rust Node.js Example\n');
  console.log('================================\n');

  // Load WASM module
  const wasmPath = join(__dirname, '../../js-tests/pkg/libavoid_bg.wasm');
  const wasmBuffer = await readFile(wasmPath);

  const libavoid = await import('../../js-tests/pkg/libavoid.js');

  // Initialize WASM
  if (libavoid.initSync) {
    libavoid.initSync({ module: wasmBuffer });
  }

  const {
    Router,
    Point,
    Polygon,
    ConnRef,
    ConnEnd,
    ShapeRef
  } = libavoid;

  // Create router with polyline routing
  const router = new Router(PolyLineRouting);
  console.log('Created router with PolyLineRouting.');

  // Define source and destination points
  const srcPt = new Point(1.2, 0.5);
  const dstPt = new Point(1.5, 4);
  console.log(`Source: (${srcPt.x}, ${srcPt.y})`);
  console.log(`Destination: (${dstPt.x}, ${dstPt.y})`);

  // Create connection endpoints
  const srcConnEnd = new ConnEnd(srcPt);
  const dstConnEnd = new ConnEnd(dstPt);

  // Create connector using factory method
  // Note: libavoid-js uses: new ConnRef(router, srcConnEnd, dstConnEnd)
  // libavoid-rust uses: ConnRef.createWithEndpoints(router, srcConnEnd, dstConnEnd)
  const connRef = ConnRef.createWithEndpoints(router, srcConnEnd, dstConnEnd);
  console.log(`\nCreated connector with ID: ${connRef.id()}`);

  // Process initial routing
  router.processTransaction();
  console.log('Processed initial transaction.');

  // Display initial route
  let route = connRef.displayRoute();
  if (route && route.size() > 0) {
    console.log('\nInitial route:');
    console.log('----------');
    for (let i = 0; i < route.size(); i++) {
      const pt = route.get_ps(i);
      console.log(`  (${pt.x.toFixed(2)}, ${pt.y.toFixed(2)})`);
    }
    console.log('----------');
  }

  // Add an obstacle shape
  console.log('\nAdding a triangular obstacle...');
  const shapePoly = new Polygon(3);
  shapePoly.set_ps(0, new Point(1, 1));
  shapePoly.set_ps(1, new Point(2.5, 1.5));
  shapePoly.set_ps(2, new Point(1.5, 2.5));
  const shapeRef = new ShapeRef(router, shapePoly);
  console.log(`Created shape with ID: ${shapeRef.id()}`);

  // Process routing after adding shape
  router.processTransaction();
  console.log('Processed transaction after adding shape.');

  // Display route after shape added
  route = connRef.displayRoute();
  if (route && route.size() > 0) {
    console.log('\nRoute after adding shape:');
    console.log('----------');
    for (let i = 0; i < route.size(); i++) {
      const pt = route.get_ps(i);
      console.log(`  (${pt.x.toFixed(2)}, ${pt.y.toFixed(2)})`);
    }
    console.log('----------');
  }

  // Update destination endpoint
  console.log('\nShifting destination endpoint...');
  const dstPt2 = new Point(6, 4.5);
  connRef.setDestEndpoint(new ConnEnd(dstPt2));
  router.processTransaction();
  console.log(`New destination: (${dstPt2.x}, ${dstPt2.y})`);

  // Display route after endpoint shift
  route = connRef.displayRoute();
  if (route && route.size() > 0) {
    console.log('\nRoute after shifting endpoint:');
    console.log('----------');
    for (let i = 0; i < route.size(); i++) {
      const pt = route.get_ps(i);
      console.log(`  (${pt.x.toFixed(2)}, ${pt.y.toFixed(2)})`);
    }
    console.log('----------');
  }

  // Move the shape
  console.log('\nMoving shape right by 0.5...');
  router.moveShape(shapeRef, 0.5, 0);
  router.processTransaction();

  // Display final route
  route = connRef.displayRoute();
  if (route && route.size() > 0) {
    console.log('\nFinal route after moving shape:');
    console.log('----------');
    for (let i = 0; i < route.size(); i++) {
      const pt = route.get_ps(i);
      console.log(`  (${pt.x.toFixed(2)}, ${pt.y.toFixed(2)})`);
    }
    console.log('----------');
  }

  console.log('\nExample complete!');
}

main().catch(err => {
  console.error('Error:', err.message);
  console.error(err);
  process.exit(1);
});
