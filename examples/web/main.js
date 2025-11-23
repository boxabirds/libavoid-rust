/**
 * libavoid-rust Web Example
 *
 * This example demonstrates basic connector routing around obstacles.
 * It mirrors the libavoid-js example but uses the wasm-bindgen API.
 */

// Import the WASM module
import init, {
  Router,
  Point,
  Polygon,
  ConnRef,
  ConnEnd,
  ShapeRef
} from './pkg/libavoid.js';

// Constants (matching libavoid-js)
const PolyLineRouting = 1;
const OrthogonalRouting = 2;

const output = document.getElementById('output');
const canvas = document.getElementById('canvas');

function log(msg) {
  output.textContent += msg + '\n';
  console.log(msg);
}

function drawRoute(route, color = 'blue') {
  if (!route || route.size() === 0) return;

  let pathData = '';
  for (let i = 0; i < route.size(); i++) {
    const pt = route.get_ps(i);
    const cmd = i === 0 ? 'M' : 'L';
    pathData += `${cmd} ${pt.x * 50} ${pt.y * 50} `;
  }

  const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
  path.setAttribute('d', pathData);
  path.setAttribute('stroke', color);
  path.setAttribute('stroke-width', '2');
  path.setAttribute('fill', 'none');
  canvas.appendChild(path);
}

function drawShape(polygon, color = 'rgba(255,0,0,0.3)') {
  let pathData = '';
  for (let i = 0; i < polygon.size(); i++) {
    const pt = polygon.get_ps(i);
    const cmd = i === 0 ? 'M' : 'L';
    pathData += `${cmd} ${pt.x * 50} ${pt.y * 50} `;
  }
  pathData += 'Z';

  const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
  path.setAttribute('d', pathData);
  path.setAttribute('fill', color);
  path.setAttribute('stroke', 'red');
  canvas.appendChild(path);
}

async function main() {
  output.textContent = '';

  // Initialize WASM module
  log('Initializing WASM module...');
  await init();
  log('WASM module loaded.\n');

  // Create router with polyline routing
  const router = new Router(PolyLineRouting);
  log('Created router with PolyLineRouting.');

  // Define source and destination points
  const srcPt = new Point(1.2, 0.5);
  const dstPt = new Point(1.5, 4);
  log(`Source: (${srcPt.x}, ${srcPt.y})`);
  log(`Destination: (${dstPt.x}, ${dstPt.y})`);

  // Create connection endpoints
  const srcConnEnd = new ConnEnd(srcPt);
  const dstConnEnd = new ConnEnd(dstPt);

  // Create connector using factory method (libavoid-rust API)
  // Note: libavoid-js uses: new ConnRef(router, srcConnEnd, dstConnEnd)
  const connRef = ConnRef.createWithEndpoints(router, srcConnEnd, dstConnEnd);
  log(`\nCreated connector with ID: ${connRef.id()}`);

  // Process initial routing
  router.processTransaction();
  log('Processed initial transaction.');

  // Display initial route
  let route = connRef.displayRoute();
  if (route && route.size() > 0) {
    log('\nInitial route:');
    log('----------');
    for (let i = 0; i < route.size(); i++) {
      const pt = route.get_ps(i);
      log(`  (${pt.x.toFixed(2)}, ${pt.y.toFixed(2)})`);
    }
    log('----------');
    drawRoute(route, 'lightblue');
  }

  // Add an obstacle shape
  log('\nAdding a triangular obstacle...');
  const shapePoly = new Polygon(3);
  shapePoly.set_ps(0, new Point(1, 1));
  shapePoly.set_ps(1, new Point(2.5, 1.5));
  shapePoly.set_ps(2, new Point(1.5, 2.5));
  const shapeRef = new ShapeRef(router, shapePoly);
  log(`Created shape with ID: ${shapeRef.id()}`);
  drawShape(shapePoly);

  // Process routing after adding shape
  router.processTransaction();
  log('Processed transaction after adding shape.');

  // Display route after shape added
  route = connRef.displayRoute();
  if (route && route.size() > 0) {
    log('\nRoute after adding shape:');
    log('----------');
    for (let i = 0; i < route.size(); i++) {
      const pt = route.get_ps(i);
      log(`  (${pt.x.toFixed(2)}, ${pt.y.toFixed(2)})`);
    }
    log('----------');
    drawRoute(route, 'green');
  }

  // Update destination endpoint
  log('\nShifting destination endpoint...');
  const dstPt2 = new Point(6, 4.5);
  connRef.setDestEndpoint(new ConnEnd(dstPt2));
  router.processTransaction();
  log(`New destination: (${dstPt2.x}, ${dstPt2.y})`);

  // Display route after endpoint shift
  route = connRef.displayRoute();
  if (route && route.size() > 0) {
    log('\nRoute after shifting endpoint:');
    log('----------');
    for (let i = 0; i < route.size(); i++) {
      const pt = route.get_ps(i);
      log(`  (${pt.x.toFixed(2)}, ${pt.y.toFixed(2)})`);
    }
    log('----------');
    drawRoute(route, 'blue');
  }

  // Move the shape
  log('\nMoving shape right by 0.5...');
  router.moveShape(shapeRef, 0.5, 0);
  router.processTransaction();

  // Display final route
  route = connRef.displayRoute();
  if (route && route.size() > 0) {
    log('\nFinal route after moving shape:');
    log('----------');
    for (let i = 0; i < route.size(); i++) {
      const pt = route.get_ps(i);
      log(`  (${pt.x.toFixed(2)}, ${pt.y.toFixed(2)})`);
    }
    log('----------');
  }

  log('\nExample complete!');
}

main().catch(err => {
  log('Error: ' + err.message);
  console.error(err);
});
