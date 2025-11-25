/**
 * libavoid-rust Examples Gallery
 *
 * MECE (Mutually Exclusive, Collectively Exhaustive) demonstration of
 * connector routing capabilities.
 */

import init, {
  Router,
  Point,
  Polygon,
  ConnRef,
  ConnEnd,
  ShapeRef,
  Rectangle
} from './pkg/libavoid.js';

// Routing type constants
const POLY_LINE_ROUTING = 1;
const ORTHOGONAL_ROUTING = 2;

// Colors for visualization
const COLORS = {
  obstacle: '#ef4444',
  obstacleFill: 'rgba(239, 68, 68, 0.2)',
  polylineRoute: '#3b82f6',
  orthogonalRoute: '#8b5cf6',
  multiRoute: '#10b981',
  crossing: '#f59e0b',
  interactive: '#06b6d4'
};

// Highly visible color palette for cycling through connectors
const CONNECTOR_PALETTE = [
  '#2563eb', // blue
  '#dc2626', // red
  '#16a34a', // green
  '#9333ea', // purple
  '#ea580c', // orange
  '#0891b2', // cyan
  '#c026d3', // fuchsia
  '#65a30d', // lime
  '#0d9488', // teal
  '#e11d48', // rose
];

// Routing options (boolean flags) - indices match Rust RoutingOption enum
const RoutingOption = {
  NUDGE_ORTHOGONAL_ROUTES: 0,
  PENALISE_ORTHOGONAL_SHARED_PATHS: 1,
  NUDGE_ORTHOGONAL_TOUCHING_COLINEAR: 2,
  PERFORM_UNIFYING_NUDGING_PREPROCESSING: 3,
  IMPROVE_HYPEREDGE_ORTHOGONAL_ROUTES: 4,
  PENALISE_CROSSING_SHARED_PATHS: 5,
  NUDGE_PREPROCESSED_TOUCHING_PATHS: 6,
  NUDGE_ORTHOGONAL_SEGMENTS_CONNECTED_TO_SHAPES: 7,
  NUDGE_SEGMENT_IF_TOUCHING_OBSTACLE: 8,
};

// Routing parameters (numeric values) - indices match Rust RoutingParameter enum
const RoutingParameter = {
  SEGMENT_PENALTY: 0,
  ANGLE_PENALTY: 1,
  CROSSING_PENALTY: 2,
  CLUSTER_CROSSING_PENALTY: 3,
  FIXED_SHARED_PATH_PENALTY: 4,
  PORT_DIRECTION_PENALTY: 5,
  SHAPE_BUFFER_DISTANCE: 6,
  IDEAL_NUDGING_DISTANCE: 7,
  REVERSE_DIRECTION_PENALTY: 8,
};

// SVG namespace
const SVG_NS = 'http://www.w3.org/2000/svg';

// Global examples object
window.examples = {};

// Track which examples have been run (for re-triggering on global nudge change)
window.runExamples = new Set();

// Global parameters (used by examples that support nudging)
window.globalNudgeDistance = 10;
window.globalShapeBuffer = 4;

// Apply global parameters and re-run all previously run examples
window.applyGlobalSettings = function() {
  const nudgeDistance = parseFloat(document.getElementById('global-nudge-input').value) || 10;
  const shapeBuffer = parseFloat(document.getElementById('global-buffer-input').value) || 4;

  window.globalNudgeDistance = nudgeDistance;
  window.globalShapeBuffer = shapeBuffer;

  console.log(`applyGlobalSettings: nudge=${nudgeDistance}, buffer=${shapeBuffer}`);

  // Re-run all previously run examples
  const rerunCount = window.runExamples.size;
  console.log(`applyGlobalSettings: ${rerunCount} examples to re-run:`, [...window.runExamples]);
  if (rerunCount > 0) {
    window.runExamples.forEach(exampleName => {
      const example = window.examples[exampleName];
      if (example && typeof example.run === 'function') {
        console.log(`  Re-running example: ${exampleName}`);
        example.run();
      }
    });
    document.getElementById('nudge-status').textContent =
      `Applied nudge=${nudgeDistance}px, buffer=${shapeBuffer}px to ${rerunCount} example(s)`;
  } else {
    document.getElementById('nudge-status').textContent =
      'No examples run yet. Run some examples first!';
  }
};

// Legacy function for backwards compatibility
window.applyGlobalNudge = function(value) {
  document.getElementById('global-nudge-input').value = value;
  window.applyGlobalSettings();
};

// Helper to mark an example as run
window.markExampleRun = function(name) {
  window.runExamples.add(name);
};

// Helper to create a rectangle polygon from top-left corner
function createRectPolygon(x, y, width, height) {
  const centerX = x + width / 2;
  const centerY = y + height / 2;
  const rect = new Rectangle(new Point(centerX, centerY), width, height);
  return rect.toPolygon();
}

// ============================================================================
// SVG Helper Functions
// ============================================================================

function clearSvg(svgId) {
  const svg = document.getElementById(svgId);
  while (svg.firstChild) {
    svg.removeChild(svg.firstChild);
  }
}

function drawRect(svgId, x, y, width, height, fillColor, strokeColor) {
  const svg = document.getElementById(svgId);
  const rect = document.createElementNS(SVG_NS, 'rect');
  rect.setAttribute('x', x);
  rect.setAttribute('y', y);
  rect.setAttribute('width', width);
  rect.setAttribute('height', height);
  rect.setAttribute('fill', fillColor);
  rect.setAttribute('stroke', strokeColor);
  rect.setAttribute('stroke-width', '2');
  svg.appendChild(rect);
  return rect;
}

function drawPolygon(svgId, points, fillColor, strokeColor) {
  const svg = document.getElementById(svgId);
  const polygon = document.createElementNS(SVG_NS, 'polygon');
  polygon.setAttribute('points', points.map(p => `${p.x},${p.y}`).join(' '));
  polygon.setAttribute('fill', fillColor);
  polygon.setAttribute('stroke', strokeColor);
  polygon.setAttribute('stroke-width', '2');
  svg.appendChild(polygon);
  return polygon;
}

function drawRoute(svgId, route, color, strokeWidth = 2) {
  if (!route || route.size() === 0) return null;

  const svg = document.getElementById(svgId);
  let pathData = '';

  for (let i = 0; i < route.size(); i++) {
    const pt = route.at(i);
    if (pt) {
      const cmd = i === 0 ? 'M' : 'L';
      pathData += `${cmd} ${pt.x} ${pt.y} `;
    }
  }

  const path = document.createElementNS(SVG_NS, 'path');
  path.setAttribute('d', pathData);
  path.setAttribute('stroke', color);
  path.setAttribute('stroke-width', strokeWidth);
  path.setAttribute('fill', 'none');
  path.setAttribute('stroke-linecap', 'round');
  path.setAttribute('stroke-linejoin', 'round');
  svg.appendChild(path);
  return path;
}

function drawPoint(svgId, x, y, radius, color) {
  const svg = document.getElementById(svgId);
  const circle = document.createElementNS(SVG_NS, 'circle');
  circle.setAttribute('cx', x);
  circle.setAttribute('cy', y);
  circle.setAttribute('r', radius);
  circle.setAttribute('fill', color);
  svg.appendChild(circle);
  return circle;
}

function log(outputId, msg) {
  const output = document.getElementById(outputId);
  output.textContent += msg + '\n';
  output.scrollTop = output.scrollHeight;
}

function clearLog(outputId) {
  document.getElementById(outputId).textContent = '';
}

// ============================================================================
// Example 1: Basic Polyline Routing
// ============================================================================

window.examples.basic = {
  run: function() {
    window.markExampleRun('basic');
    clearSvg('canvas-basic');
    clearLog('output-basic');
    log('output-basic', 'Creating router with polyline routing...');

    const router = new Router(POLY_LINE_ROUTING);
    // Apply global shape buffer
    router.setRoutingParameter(RoutingParameter.SHAPE_BUFFER_DISTANCE, window.globalShapeBuffer);

    // Define obstacle (centered at 200, 125)
    const obstacleX = 175;
    const obstacleY = 100;
    const obstacleW = 50;
    const obstacleH = 50;

    // Draw obstacle
    drawRect('canvas-basic', obstacleX, obstacleY, obstacleW, obstacleH,
             COLORS.obstacleFill, COLORS.obstacle);

    // Create shape using helper function
    const shapePoly = createRectPolygon(obstacleX, obstacleY, obstacleW, obstacleH);

    // Debug: verify polygon was created correctly
    console.log('Polygon size:', shapePoly.size());
    for (let i = 0; i < shapePoly.size(); i++) {
      const pt = shapePoly.at(i);
      console.log(`  Point ${i}: (${pt?.x}, ${pt?.y})`);
    }

    const shapeRef = new ShapeRef(router, shapePoly);
    router.addShape(shapeRef);
    log('output-basic', `Added obstacle at (${obstacleX}, ${obstacleY})`);
    log('output-basic', `Polygon has ${shapePoly.size()} points`);

    // Create connector from left to right
    const srcPt = new Point(50, 125);
    const dstPt = new Point(350, 125);

    drawPoint('canvas-basic', srcPt.x, srcPt.y, 5, COLORS.polylineRoute);
    drawPoint('canvas-basic', dstPt.x, dstPt.y, 5, COLORS.polylineRoute);

    const srcEnd = new ConnEnd(srcPt);
    const dstEnd = new ConnEnd(dstPt);
    const connRef = ConnRef.createWithEndpoints(router, srcEnd, dstEnd);
    router.addConnector(connRef);

    log('output-basic', `Created connector: (${srcPt.x}, ${srcPt.y}) → (${dstPt.x}, ${dstPt.y})`);

    // Process and draw route
    router.processTransaction();
    const route = router.getConnectorRoute(connRef.id());

    if (route && route.size() > 0) {
      drawRoute('canvas-basic', route, COLORS.polylineRoute);
      log('output-basic', `Route computed with ${route.size()} points`);

      // Debug: print all route points
      for (let i = 0; i < route.size(); i++) {
        const pt = route.at(i);
        console.log(`Route point ${i}: (${pt?.x}, ${pt?.y})`);
        log('output-basic', `  Point ${i}: (${pt?.x?.toFixed(1)}, ${pt?.y?.toFixed(1)})`);
      }

      if (route.size() === 2) {
        log('output-basic', 'WARNING: Direct route - obstacle not avoided!');
      } else {
        log('output-basic', 'Route avoids obstacle via polyline path');
      }
    } else {
      log('output-basic', 'No route found!');
    }
  },

  reset: function() {
    clearSvg('canvas-basic');
    clearLog('output-basic');
    log('output-basic', 'Click "Run Example" to start');
  }
};

// ============================================================================
// Example 2: Orthogonal Routing
// ============================================================================

window.examples.orthogonal = {
  run: function() {
    window.markExampleRun('orthogonal');
    clearSvg('canvas-orthogonal');
    clearLog('output-orthogonal');
    log('output-orthogonal', 'Creating router with orthogonal routing...');

    const router = new Router(ORTHOGONAL_ROUTING);
    router.setRoutingParameter(RoutingParameter.SHAPE_BUFFER_DISTANCE, window.globalShapeBuffer);

    // Obstacle
    const obstacleX = 150;
    const obstacleY = 75;
    const obstacleW = 80;
    const obstacleH = 80;

    drawRect('canvas-orthogonal', obstacleX, obstacleY, obstacleW, obstacleH,
             COLORS.obstacleFill, COLORS.obstacle);

    const shapePoly = createRectPolygon(obstacleX, obstacleY, obstacleW, obstacleH);
    const shapeRef = new ShapeRef(router, shapePoly);
    router.addShape(shapeRef);
    log('output-orthogonal', 'Added rectangular obstacle');

    // Route from top-left to bottom-right
    const srcPt = new Point(50, 50);
    const dstPt = new Point(350, 200);

    drawPoint('canvas-orthogonal', srcPt.x, srcPt.y, 5, COLORS.orthogonalRoute);
    drawPoint('canvas-orthogonal', dstPt.x, dstPt.y, 5, COLORS.orthogonalRoute);

    const srcEnd = new ConnEnd(srcPt);
    const dstEnd = new ConnEnd(dstPt);
    const connRef = ConnRef.createWithEndpoints(router, srcEnd, dstEnd);
    connRef.setRoutingType(ORTHOGONAL_ROUTING);
    router.addConnector(connRef);

    log('output-orthogonal', 'Created orthogonal connector');

    router.processTransaction();
    const route = router.getConnectorRoute(connRef.id());

    if (route && route.size() > 0) {
      drawRoute('canvas-orthogonal', route, COLORS.orthogonalRoute);
      log('output-orthogonal', `Route has ${route.size()} points`);

      // Verify orthogonality
      let isOrthogonal = true;
      for (let i = 0; i < route.size() - 1; i++) {
        const p1 = route.at(i);
        const p2 = route.at(i + 1);
        if (p1 && p2) {
          const isH = Math.abs(p1.y - p2.y) < 0.1;
          const isV = Math.abs(p1.x - p2.x) < 0.1;
          if (!isH && !isV) isOrthogonal = false;
        }
      }
      log('output-orthogonal', isOrthogonal ?
        '✓ All segments are orthogonal (H/V only)' :
        '✗ Some segments are not orthogonal');
    }
  },

  reset: function() {
    clearSvg('canvas-orthogonal');
    clearLog('output-orthogonal');
    log('output-orthogonal', 'Click "Run Example" to start');
  }
};

// ============================================================================
// Example 3: Multiple Obstacles
// ============================================================================

window.examples.multi = {
  run: function() {
    window.markExampleRun('multi');
    clearSvg('canvas-multi');
    clearLog('output-multi');
    log('output-multi', 'Creating maze with multiple obstacles...');

    const router = new Router(POLY_LINE_ROUTING);
    router.setRoutingParameter(RoutingParameter.SHAPE_BUFFER_DISTANCE, window.globalShapeBuffer);

    // Define multiple obstacles to create a maze
    const obstacles = [
      { x: 80, y: 40, w: 60, h: 80 },
      { x: 180, y: 100, w: 40, h: 100 },
      { x: 260, y: 20, w: 50, h: 60 },
      { x: 100, y: 160, w: 80, h: 40 },
      { x: 280, y: 120, w: 40, h: 80 }
    ];

    obstacles.forEach((obs, i) => {
      drawRect('canvas-multi', obs.x, obs.y, obs.w, obs.h,
               COLORS.obstacleFill, COLORS.obstacle);

      const poly = createRectPolygon(obs.x, obs.y, obs.w, obs.h);
      const shape = new ShapeRef(router, poly);
      router.addShape(shape);
    });

    log('output-multi', `Added ${obstacles.length} obstacles`);

    // Route through the maze
    const srcPt = new Point(20, 125);
    const dstPt = new Point(380, 125);

    drawPoint('canvas-multi', srcPt.x, srcPt.y, 5, COLORS.multiRoute);
    drawPoint('canvas-multi', dstPt.x, dstPt.y, 5, COLORS.multiRoute);

    const connRef = ConnRef.createWithEndpoints(router,
      new ConnEnd(srcPt), new ConnEnd(dstPt));
    router.addConnector(connRef);

    router.processTransaction();
    const route = router.getConnectorRoute(connRef.id());

    if (route && route.size() > 0) {
      drawRoute('canvas-multi', route, COLORS.multiRoute);
      log('output-multi', `Route computed: ${route.size()} points`);

      // Calculate route length
      let length = 0;
      for (let i = 0; i < route.size() - 1; i++) {
        const p1 = route.at(i);
        const p2 = route.at(i + 1);
        if (p1 && p2) {
          length += Math.sqrt((p2.x - p1.x) ** 2 + (p2.y - p1.y) ** 2);
        }
      }
      log('output-multi', `Total route length: ${length.toFixed(1)}px`);
    }
  },

  reset: function() {
    clearSvg('canvas-multi');
    clearLog('output-multi');
    log('output-multi', 'Click "Run Example" to start');
  }
};

// ============================================================================
// Example 4: Shape Operations
// ============================================================================

window.examples.shapes = {
  router: null,
  shapes: [],
  connRef: null,

  run: async function() {
    window.markExampleRun('shapes');
    clearSvg('canvas-shapes');
    clearLog('output-shapes');

    this.router = new Router(POLY_LINE_ROUTING);
    this.router.setRoutingParameter(RoutingParameter.SHAPE_BUFFER_DISTANCE, window.globalShapeBuffer);
    this.shapes = [];

    log('output-shapes', '=== Shape Operations Sequence ===\n');

    // Initial setup
    const srcPt = new Point(30, 125);
    const dstPt = new Point(370, 125);

    drawPoint('canvas-shapes', srcPt.x, srcPt.y, 5, COLORS.polylineRoute);
    drawPoint('canvas-shapes', dstPt.x, dstPt.y, 5, COLORS.polylineRoute);

    this.connRef = ConnRef.createWithEndpoints(this.router,
      new ConnEnd(srcPt), new ConnEnd(dstPt));
    this.router.addConnector(this.connRef);

    log('output-shapes', '1. Direct route (no obstacles):');
    this.router.processTransaction();
    this._drawRoute();

    await this._delay(800);

    // Add first shape
    log('output-shapes', '\n2. Adding obstacle...');
    this.addShape();

    await this._delay(800);

    // Move shape
    log('output-shapes', '\n3. Moving obstacle...');
    this.moveShape();

    await this._delay(800);

    // Add another shape
    log('output-shapes', '\n4. Adding second obstacle...');
    this.addShape();

    log('output-shapes', '\n=== Sequence Complete ===');
  },

  addShape: function() {
    if (!this.router) {
      log('output-shapes', 'Run sequence first!');
      return;
    }

    const positions = [
      { x: 150, y: 80, w: 60, h: 90 },
      { x: 250, y: 70, w: 50, h: 80 }
    ];

    const idx = this.shapes.length % positions.length;
    const pos = positions[idx];

    const poly = createRectPolygon(pos.x, pos.y, pos.w, pos.h);
    const shape = new ShapeRef(this.router, poly);
    this.router.addShape(shape);
    this.shapes.push({ shape, pos });

    this.router.processTransaction();
    this._redraw();
    log('output-shapes', `   Added shape at (${pos.x}, ${pos.y})`);
  },

  moveShape: function() {
    if (!this.router || this.shapes.length === 0) {
      log('output-shapes', 'Add shapes first!');
      return;
    }

    const shapeData = this.shapes[0];
    const offsetX = 30;
    const offsetY = 20;

    this.router.moveShape(shapeData.shape, offsetX, offsetY);
    shapeData.pos.x += offsetX;
    shapeData.pos.y += offsetY;

    this.router.processTransaction();
    this._redraw();
    log('output-shapes', `   Moved shape by (${offsetX}, ${offsetY})`);
  },

  reset: function() {
    this.router = null;
    this.shapes = [];
    this.connRef = null;
    clearSvg('canvas-shapes');
    clearLog('output-shapes');
    log('output-shapes', 'Click "Run Sequence" to start');
  },

  _redraw: function() {
    clearSvg('canvas-shapes');

    // Redraw endpoints
    drawPoint('canvas-shapes', 30, 125, 5, COLORS.polylineRoute);
    drawPoint('canvas-shapes', 370, 125, 5, COLORS.polylineRoute);

    // Redraw shapes
    this.shapes.forEach(s => {
      drawRect('canvas-shapes', s.pos.x, s.pos.y, s.pos.w, s.pos.h,
               COLORS.obstacleFill, COLORS.obstacle);
    });

    this._drawRoute();
  },

  _drawRoute: function() {
    if (this.connRef) {
      const route = this.router.getConnectorRoute(this.connRef.id());
      if (route) {
        drawRoute('canvas-shapes', route, COLORS.polylineRoute);
      }
    }
  },

  _delay: function(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
};

// ============================================================================
// Example 5: Transaction Batching
// ============================================================================

window.examples.batch = {
  runBatch: function() {
    window.markExampleRun('batch');
    clearSvg('canvas-batch');
    clearLog('output-batch');

    log('output-batch', '=== Batched Transaction Mode ===\n');

    const router = new Router(POLY_LINE_ROUTING);
    router.setRoutingParameter(RoutingParameter.SHAPE_BUFFER_DISTANCE, window.globalShapeBuffer);
    const startTime = performance.now();

    // Add multiple shapes and connectors in a batch
    const shapes = [];
    const shapeCount = 5;

    log('output-batch', `Adding ${shapeCount} obstacles...`);

    for (let i = 0; i < shapeCount; i++) {
      const x = 50 + i * 70;
      const y = 80 + (i % 2) * 60;
      const w = 40;
      const h = 50;

      const poly = createRectPolygon(x, y, w, h);
      const shape = new ShapeRef(router, poly);
      router.addShape(shape);
      shapes.push({ x, y, w, h });

      drawRect('canvas-batch', x, y, w, h, COLORS.obstacleFill, COLORS.obstacle);
    }

    // Add multiple connectors
    const connectors = [];
    const connectorCount = 3;

    log('output-batch', `Adding ${connectorCount} connectors...`);

    const connectorDefs = [
      { src: { x: 20, y: 50 }, dst: { x: 380, y: 50 }, color: '#3b82f6' },
      { src: { x: 20, y: 125 }, dst: { x: 380, y: 125 }, color: '#10b981' },
      { src: { x: 20, y: 200 }, dst: { x: 380, y: 200 }, color: '#f59e0b' }
    ];

    connectorDefs.forEach(def => {
      drawPoint('canvas-batch', def.src.x, def.src.y, 4, def.color);
      drawPoint('canvas-batch', def.dst.x, def.dst.y, 4, def.color);

      const conn = ConnRef.createWithEndpoints(router,
        new ConnEnd(new Point(def.src.x, def.src.y)),
        new ConnEnd(new Point(def.dst.x, def.dst.y)));
      router.addConnector(conn);
      connectors.push({ conn, color: def.color });
    });

    // Single transaction processes all at once
    log('output-batch', '\nProcessing single transaction...');
    router.processTransaction();
    const elapsed = performance.now() - startTime;

    // Draw all routes
    connectors.forEach(({ conn, color }) => {
      const route = router.getConnectorRoute(conn.id());
      if (route) {
        drawRoute('canvas-batch', route, color);
      }
    });

    log('output-batch', `\n✓ Batch complete in ${elapsed.toFixed(2)}ms`);
    log('output-batch', `  ${shapeCount} shapes + ${connectorCount} connectors`);
    log('output-batch', '  All processed in 1 transaction');
  },

  runImmediate: function() {
    clearSvg('canvas-batch');
    clearLog('output-batch');

    log('output-batch', '=== Immediate Mode (for comparison) ===\n');

    const router = new Router(POLY_LINE_ROUTING);
    const startTime = performance.now();

    const shapeCount = 5;

    log('output-batch', `Adding ${shapeCount} obstacles one by one...`);

    for (let i = 0; i < shapeCount; i++) {
      const x = 50 + i * 70;
      const y = 80 + (i % 2) * 60;
      const w = 40;
      const h = 50;

      const poly = createRectPolygon(x, y, w, h);
      const shape = new ShapeRef(router, poly);
      router.addShape(shape);
      router.processTransaction(); // Process after each shape

      drawRect('canvas-batch', x, y, w, h, COLORS.obstacleFill, COLORS.obstacle);
    }

    const connectorDefs = [
      { src: { x: 20, y: 50 }, dst: { x: 380, y: 50 }, color: '#3b82f6' },
      { src: { x: 20, y: 125 }, dst: { x: 380, y: 125 }, color: '#10b981' },
      { src: { x: 20, y: 200 }, dst: { x: 380, y: 200 }, color: '#f59e0b' }
    ];

    log('output-batch', '\nAdding connectors one by one...');

    connectorDefs.forEach(def => {
      drawPoint('canvas-batch', def.src.x, def.src.y, 4, def.color);
      drawPoint('canvas-batch', def.dst.x, def.dst.y, 4, def.color);

      const conn = ConnRef.createWithEndpoints(router,
        new ConnEnd(new Point(def.src.x, def.src.y)),
        new ConnEnd(new Point(def.dst.x, def.dst.y)));
      router.addConnector(conn);
      router.processTransaction(); // Process after each connector

      const route = router.getConnectorRoute(conn.id());
      if (route) {
        drawRoute('canvas-batch', route, def.color);
      }
    });

    const elapsed = performance.now() - startTime;

    log('output-batch', `\n✓ Immediate complete in ${elapsed.toFixed(2)}ms`);
    log('output-batch', `  ${shapeCount + 3} individual transactions`);
    log('output-batch', '  (Compare timing with batched mode)');
  },

  reset: function() {
    clearSvg('canvas-batch');
    clearLog('output-batch');
    log('output-batch', 'Compare batched vs immediate routing');
  }
};

// ============================================================================
// Example 6: Multiple Connectors
// ============================================================================

window.examples.connectors = {
  run: function() {
    window.markExampleRun('connectors');
    clearSvg('canvas-connectors');
    clearLog('output-connectors');

    log('output-connectors', 'Creating multiple simultaneous connectors...\n');

    const router = new Router(POLY_LINE_ROUTING);
    router.setRoutingParameter(RoutingParameter.SHAPE_BUFFER_DISTANCE, window.globalShapeBuffer);

    // Central obstacle
    const obstacleX = 160;
    const obstacleY = 80;
    const obstacleW = 80;
    const obstacleH = 90;

    drawRect('canvas-connectors', obstacleX, obstacleY, obstacleW, obstacleH,
             COLORS.obstacleFill, COLORS.obstacle);

    const poly = createRectPolygon(obstacleX, obstacleY, obstacleW, obstacleH);
    const shape = new ShapeRef(router, poly);
    router.addShape(shape);

    // Define multiple connectors with different source/dest pairs
    const connectorDefs = [
      { src: { x: 30, y: 40 }, dst: { x: 370, y: 40 }, color: '#3b82f6' },
      { src: { x: 30, y: 80 }, dst: { x: 370, y: 200 }, color: '#10b981' },
      { src: { x: 30, y: 125 }, dst: { x: 370, y: 125 }, color: '#f59e0b' },
      { src: { x: 30, y: 170 }, dst: { x: 370, y: 60 }, color: '#8b5cf6' },
      { src: { x: 30, y: 210 }, dst: { x: 370, y: 210 }, color: '#ef4444' }
    ];

    const connectors = [];

    connectorDefs.forEach((def, i) => {
      drawPoint('canvas-connectors', def.src.x, def.src.y, 4, def.color);
      drawPoint('canvas-connectors', def.dst.x, def.dst.y, 4, def.color);

      const conn = ConnRef.createWithEndpoints(router,
        new ConnEnd(new Point(def.src.x, def.src.y)),
        new ConnEnd(new Point(def.dst.x, def.dst.y)));
      router.addConnector(conn);
      connectors.push({ conn, def });

      log('output-connectors', `Connector ${i + 1}: (${def.src.x},${def.src.y}) → (${def.dst.x},${def.dst.y})`);
    });

    router.processTransaction();

    log('output-connectors', '\nRoutes computed:');

    connectors.forEach(({ conn, def }, i) => {
      const route = router.getConnectorRoute(conn.id());
      if (route && route.size() > 0) {
        drawRoute('canvas-connectors', route, def.color);
        log('output-connectors', `  Connector ${i + 1}: ${route.size()} points`);
      }
    });

    log('output-connectors', `\nTotal: ${connectors.length} connectors routed`);
  },

  reset: function() {
    clearSvg('canvas-connectors');
    clearLog('output-connectors');
    log('output-connectors', 'Click "Run Example" to start');
  }
};

// ============================================================================
// Example 7: Crossing Visualization
// ============================================================================

window.examples.crossing = {
  run: function() {
    window.markExampleRun('crossing');
    clearSvg('canvas-crossing');
    clearLog('output-crossing');

    log('output-crossing', 'Routing connectors that will cross...\n');

    const router = new Router(POLY_LINE_ROUTING);
    router.setRoutingParameter(RoutingParameter.SHAPE_BUFFER_DISTANCE, window.globalShapeBuffer);

    // Two connectors that must cross
    const connectorDefs = [
      {
        src: { x: 30, y: 30 },
        dst: { x: 370, y: 220 },
        color: '#3b82f6',
        name: 'Diagonal A (↘)'
      },
      {
        src: { x: 30, y: 220 },
        dst: { x: 370, y: 30 },
        color: '#10b981',
        name: 'Diagonal B (↗)'
      }
    ];

    const connectors = [];
    const routes = [];

    connectorDefs.forEach((def, i) => {
      drawPoint('canvas-crossing', def.src.x, def.src.y, 6, def.color);
      drawPoint('canvas-crossing', def.dst.x, def.dst.y, 6, def.color);

      const conn = ConnRef.createWithEndpoints(router,
        new ConnEnd(new Point(def.src.x, def.src.y)),
        new ConnEnd(new Point(def.dst.x, def.dst.y)));
      router.addConnector(conn);
      connectors.push({ conn, def });

      log('output-crossing', `${def.name}: (${def.src.x},${def.src.y}) → (${def.dst.x},${def.dst.y})`);
    });

    router.processTransaction();

    // Draw routes and detect crossings
    connectors.forEach(({ conn, def }) => {
      const route = router.getConnectorRoute(conn.id());
      if (route && route.size() > 0) {
        drawRoute('canvas-crossing', route, def.color, 3);
        routes.push(route);
      }
    });

    // Find crossing point (for two straight lines)
    if (routes.length >= 2) {
      const route1 = routes[0];
      const route2 = routes[1];

      // For these direct routes, find intersection
      const p1 = route1.at(0);
      const p2 = route1.at(route1.size() - 1);
      const p3 = route2.at(0);
      const p4 = route2.at(route2.size() - 1);

      if (p1 && p2 && p3 && p4) {
        const crossing = this._lineIntersection(
          p1.x, p1.y, p2.x, p2.y,
          p3.x, p3.y, p4.x, p4.y
        );

        if (crossing) {
          // Draw crossing indicator
          drawPoint('canvas-crossing', crossing.x, crossing.y, 10, COLORS.crossing);
          drawPoint('canvas-crossing', crossing.x, crossing.y, 6, '#fff');
          drawPoint('canvas-crossing', crossing.x, crossing.y, 3, COLORS.crossing);

          log('output-crossing', `\n⚠ CROSSING DETECTED`);
          log('output-crossing', `  Location: (${crossing.x.toFixed(1)}, ${crossing.y.toFixed(1)})`);
        }
      }
    }

    log('output-crossing', '\nNote: Crossings can be penalized via');
    log('output-crossing', 'router.setRoutingParameter(CROSSING_PENALTY, value)');
  },

  _lineIntersection: function(x1, y1, x2, y2, x3, y3, x4, y4) {
    const denom = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
    if (Math.abs(denom) < 0.001) return null;

    const t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / denom;
    const u = -((x1 - x2) * (y1 - y3) - (y1 - y2) * (x1 - x3)) / denom;

    if (t >= 0 && t <= 1 && u >= 0 && u <= 1) {
      return {
        x: x1 + t * (x2 - x1),
        y: y1 + t * (y2 - y1)
      };
    }
    return null;
  },

  reset: function() {
    clearSvg('canvas-crossing');
    clearLog('output-crossing');
    log('output-crossing', 'Click "Run Example" to start');
  }
};

// ============================================================================
// Example 8: Interactive Demo
// ============================================================================

window.examples.interactive = {
  router: null,
  shapes: [],
  connectors: [],
  mode: 'shape',
  pendingConnector: null,
  nextShapeId: 1,
  nextColorIndex: 0,

  init: function() {
    this.router = new Router(ORTHOGONAL_ROUTING); // Use orthogonal for nudging

    // Enable transaction mode - REQUIRED for nudging to work
    this.router.setTransactionUse(true);

    // Enable nudging for PCB-style parallel lanes
    this.router.setRoutingOption(RoutingOption.NUDGE_ORTHOGONAL_ROUTES, true);
    // Set nudging distance from global setting
    this.router.setRoutingParameter(RoutingParameter.IDEAL_NUDGING_DISTANCE, window.globalNudgeDistance);
    // Set buffer distance between routes and obstacles
    this.router.setRoutingParameter(RoutingParameter.SHAPE_BUFFER_DISTANCE, window.globalShapeBuffer);

    // Debug: verify parameters were set
    console.log('Interactive init: nudge=' + this.router.routingParameter(RoutingParameter.IDEAL_NUDGING_DISTANCE) +
                ', buffer=' + this.router.routingParameter(RoutingParameter.SHAPE_BUFFER_DISTANCE));

    this.shapes = [];
    this.connectors = [];
    this.mode = 'shape';
    this.pendingConnector = null;
    this.nextShapeId = 1;
    this.nextColorIndex = 0;

    const canvas = document.getElementById('canvas-interactive');
    canvas.onclick = (e) => this._handleClick(e);

    this._updateModeDisplay();

    // Mark as run so global nudge changes will update it
    window.markExampleRun('interactive');
  },

  // Re-run with current global nudge distance
  run: function() {
    window.markExampleRun('interactive');
    // Re-initialize router with new nudge distance
    const oldShapes = [...this.shapes];
    const oldConnectors = [...this.connectors];

    this.router = new Router(ORTHOGONAL_ROUTING);
    this.router.setTransactionUse(true);
    this.router.setRoutingOption(RoutingOption.NUDGE_ORTHOGONAL_ROUTES, true);
    this.router.setRoutingParameter(RoutingParameter.IDEAL_NUDGING_DISTANCE, window.globalNudgeDistance);
    this.router.setRoutingParameter(RoutingParameter.SHAPE_BUFFER_DISTANCE, window.globalShapeBuffer);

    // Re-add shapes
    this.shapes = [];
    oldShapes.forEach(s => {
      const poly = createRectPolygon(s.x, s.y, s.w, s.h);
      const shape = new ShapeRef(this.router, poly);
      this.router.addShape(shape);
      this.shapes.push({ shape, x: s.x, y: s.y, w: s.w, h: s.h });
    });

    // Re-add connectors using stored raw coordinates
    this.connectors = [];
    oldConnectors.forEach(c => {
      const srcPt = new Point(c.srcX, c.srcY);
      const dstPt = new Point(c.dstX, c.dstY);
      const conn = ConnRef.createWithEndpoints(this.router,
        new ConnEnd(srcPt),
        new ConnEnd(dstPt));
      conn.setRoutingType(ORTHOGONAL_ROUTING);
      this.router.addConnector(conn);
      this.connectors.push({ conn, srcX: c.srcX, srcY: c.srcY, dstX: c.dstX, dstY: c.dstY, color: c.color });
    });

    this._rerouteAndRedraw();
    this._updateModeDisplay();
  },

  setMode: function(mode) {
    this.mode = mode;
    this.pendingConnector = null;
    this._updateModeDisplay();
  },

  _updateModeDisplay: function() {
    const output = document.getElementById('output-interactive');
    if (this.mode === 'shape') {
      output.textContent = 'Mode: SHAPE | Click to add 50x50 obstacles';
    } else {
      if (this.pendingConnector) {
        output.textContent = 'Mode: CONNECTOR | Click destination point...';
      } else {
        output.textContent = 'Mode: CONNECTOR | Click source point...';
      }
    }
  },

  _handleClick: function(e) {
    const svg = document.getElementById('canvas-interactive');
    const rect = svg.getBoundingClientRect();

    // Convert to SVG coordinates
    const svgWidth = 400;
    const svgHeight = 250;
    const x = (e.clientX - rect.left) * (svgWidth / rect.width);
    const y = (e.clientY - rect.top) * (svgHeight / rect.height);

    if (this.mode === 'shape') {
      this._addShape(x - 25, y - 25, 50, 50);
    } else {
      this._handleConnectorClick(x, y);
    }
  },

  _addShape: function(x, y, w, h) {
    // Clamp to canvas bounds
    x = Math.max(0, Math.min(x, 350));
    y = Math.max(0, Math.min(y, 200));

    const poly = createRectPolygon(x, y, w, h);
    const shape = new ShapeRef(this.router, poly);
    this.router.addShape(shape);
    this.shapes.push({ shape, x, y, w, h });

    this._rerouteAndRedraw();
  },

  _handleConnectorClick: function(x, y) {
    if (!this.pendingConnector) {
      // First click - set source, assign color now
      // Store raw coordinates, not WASM Point objects (which can become invalid)
      const color = CONNECTOR_PALETTE[this.nextColorIndex % CONNECTOR_PALETTE.length];
      this.pendingConnector = { srcX: x, srcY: y, color };
      drawPoint('canvas-interactive', x, y, 6, color);
    } else {
      // Second click - create connector using fresh Point objects
      const srcPt = new Point(this.pendingConnector.srcX, this.pendingConnector.srcY);
      const dstPt = new Point(x, y);

      const conn = ConnRef.createWithEndpoints(this.router,
        new ConnEnd(srcPt),
        new ConnEnd(dstPt));
      conn.setRoutingType(ORTHOGONAL_ROUTING);
      this.router.addConnector(conn);

      // Store raw coordinates for redrawing, not WASM objects
      this.connectors.push({
        conn,
        srcX: this.pendingConnector.srcX,
        srcY: this.pendingConnector.srcY,
        dstX: x,
        dstY: y,
        color: this.pendingConnector.color
      });

      this.nextColorIndex++;
      this.pendingConnector = null;
      this._rerouteAndRedraw();
    }

    this._updateModeDisplay();
  },

  _rerouteAndRedraw: function() {
    console.log('_rerouteAndRedraw: nudge=' + window.globalNudgeDistance + ', buffer=' + window.globalShapeBuffer);
    console.log('  connectors:', this.connectors.length, 'shapes:', this.shapes.length);
    this.router.processTransaction();

    clearSvg('canvas-interactive');

    // Draw shapes
    this.shapes.forEach(s => {
      drawRect('canvas-interactive', s.x, s.y, s.w, s.h,
               COLORS.obstacleFill, COLORS.obstacle);
    });

    // Draw connectors with their assigned colors
    this.connectors.forEach(({ conn, srcX, srcY, dstX, dstY, color }, idx) => {
      drawPoint('canvas-interactive', srcX, srcY, 5, color);
      drawPoint('canvas-interactive', dstX, dstY, 5, color);

      const route = this.router.getConnectorRoute(conn.id());
      if (route) {
        drawRoute('canvas-interactive', route, color, 3);
        // Debug: log first route's points
        if (idx === 0 && route.size() > 0) {
          const pts = [];
          for (let i = 0; i < route.size(); i++) {
            const p = route.at(i);
            if (p) pts.push(`(${p.x.toFixed(1)},${p.y.toFixed(1)})`);
          }
          console.log('Route 0:', pts.join(' -> '));
        }
      }
    });
  },

  reset: function() {
    this.router = new Router(POLY_LINE_ROUTING);
    this.shapes = [];
    this.connectors = [];
    this.pendingConnector = null;
    this.nextShapeId = 1;

    clearSvg('canvas-interactive');
    this.setMode('shape');
  }
};

// ============================================================================
// Example 9: Route Nudging (Overlap Prevention)
// ============================================================================

window.examples.nudging = {
  run: function() {
    window.markExampleRun('nudging');
    clearSvg('canvas-nudging');
    clearLog('output-nudging');

    const nudgeDist = window.globalNudgeDistance;
    log('output-nudging', `Demonstrating route nudging (distance: ${nudgeDist}px)...\n`);

    // Create router with orthogonal routing
    const router = new Router(ORTHOGONAL_ROUTING);

    // Enable transaction mode - required for nudging to work
    router.setTransactionUse(true);

    // Enable route nudging with global distance
    router.setRoutingOption(RoutingOption.NUDGE_ORTHOGONAL_ROUTES, true);
    router.setRoutingParameter(RoutingParameter.IDEAL_NUDGING_DISTANCE, nudgeDist);
    // Set buffer distance between routes and obstacles
    router.setRoutingParameter(RoutingParameter.SHAPE_BUFFER_DISTANCE, nudgeDist);

    // Create a simple obstacle
    const obstaclePoly = createRectPolygon(150, 80, 100, 90);
    const obstacle = new ShapeRef(router, obstaclePoly);
    router.addShape(obstacle);

    // Create multiple connectors with SAME endpoints - they will overlap before nudging
    // All routes start/end at same Y=125 and must route around obstacle at Y=80-170
    const connectorDefs = [
      { src: { x: 30, y: 125 }, dst: { x: 370, y: 125 }, color: '#3b82f6', name: 'Route 1' },
      { src: { x: 30, y: 125 }, dst: { x: 370, y: 125 }, color: '#10b981', name: 'Route 2' },
      { src: { x: 30, y: 125 }, dst: { x: 370, y: 125 }, color: '#8b5cf6', name: 'Route 3' },
    ];

    const connectors = [];

    // Add connectors
    connectorDefs.forEach((def, i) => {
      const conn = ConnRef.createWithEndpoints(router,
        new ConnEnd(new Point(def.src.x, def.src.y)),
        new ConnEnd(new Point(def.dst.x, def.dst.y)));
      conn.setRoutingType(ORTHOGONAL_ROUTING);
      router.addConnector(conn);
      connectors.push({ conn, def });

      log('output-nudging', `${def.name}: (${def.src.x},${def.src.y}) → (${def.dst.x},${def.dst.y})`);
    });

    router.processTransaction();

    // Draw obstacle
    drawRect('canvas-nudging', 150, 80, 100, 90, COLORS.obstacleFill, COLORS.obstacle);

    // Draw routes
    log('output-nudging', '\nRoutes after nudging:');
    connectors.forEach(({ conn, def }) => {
      drawPoint('canvas-nudging', def.src.x, def.src.y, 5, def.color);
      drawPoint('canvas-nudging', def.dst.x, def.dst.y, 5, def.color);

      const route = router.getConnectorRoute(conn.id());
      if (route && route.size() > 0) {
        drawRoute('canvas-nudging', route, def.color, 2);

        // Log route points
        let routeStr = `  ${def.name}: `;
        for (let i = 0; i < Math.min(route.size(), 4); i++) {
          const pt = route.at(i);
          if (pt) routeStr += `(${pt.x.toFixed(0)},${pt.y.toFixed(0)}) `;
        }
        if (route.size() > 4) routeStr += '...';
        log('output-nudging', routeStr);
      }
    });

    log('output-nudging', '\n✓ Overlapping segments are nudged apart');
    log('output-nudging', '  using VPSC constraint satisfaction');
  },

  reset: function() {
    clearSvg('canvas-nudging');
    clearLog('output-nudging');
    log('output-nudging', 'Click "Run Example" to see route nudging');
  }
};

// ============================================================================
// 10. Routing Options Comparison
// ============================================================================

window.examples.comparison = {
  // Panel config is generated dynamically based on global nudge distance
  getPanelConfig: function() {
    const base = window.globalNudgeDistance;
    return [
      { id: 'canvas-compare-1', enableNudge: false, nudgeDistance: 0, label: 'Nudging OFF' },
      { id: 'canvas-compare-2', enableNudge: true, nudgeDistance: Math.max(1, base * 0.5), label: `Nudge = ${Math.max(1, base * 0.5)}px` },
      { id: 'canvas-compare-3', enableNudge: true, nudgeDistance: base, label: `Nudge = ${base}px` },
      { id: 'canvas-compare-4', enableNudge: true, nudgeDistance: base * 2, label: `Nudge = ${base * 2}px` },
    ];
  },

  // Obstacle in the center to force routes around it
  OBSTACLE: { x: 70, y: 50, width: 60, height: 50 },

  // Three routes that must go around the obstacle
  // Without nudging: routes stack on same path
  // With nudging: they separate by IDEAL_NUDGING_DISTANCE
  ROUTES: [
    { src: { x: 15, y: 75 }, dst: { x: 185, y: 75 }, color: '#2563eb' },
    { src: { x: 15, y: 75 }, dst: { x: 185, y: 75 }, color: '#dc2626' },
    { src: { x: 15, y: 75 }, dst: { x: 185, y: 75 }, color: '#16a34a' },
  ],

  runPanel: function(panelConfig) {
    const router = new Router(ORTHOGONAL_ROUTING);

    // Enable transaction mode - required for nudging to work
    router.setTransactionUse(true);

    // Configure nudging and buffer distance
    if (panelConfig.enableNudge) {
      router.setRoutingOption(RoutingOption.NUDGE_ORTHOGONAL_ROUTES, true);
      router.setRoutingParameter(RoutingParameter.IDEAL_NUDGING_DISTANCE, panelConfig.nudgeDistance);
      router.setRoutingParameter(RoutingParameter.SHAPE_BUFFER_DISTANCE, panelConfig.nudgeDistance);
      console.log(`Panel ${panelConfig.id}: Nudging enabled, distance=${panelConfig.nudgeDistance}`);
    } else {
      console.log(`Panel ${panelConfig.id}: Nudging disabled`);
    }

    // Create obstacle if defined
    const obs = this.OBSTACLE;
    if (obs) {
      console.log(`Panel ${panelConfig.id}: Creating obstacle at (${obs.x}, ${obs.y}, ${obs.width}, ${obs.height})`);
      const obstaclePoly = createRectPolygon(obs.x, obs.y, obs.width, obs.height);
      const obstacle = new ShapeRef(router, obstaclePoly);
      router.addShape(obstacle);

      // Draw obstacle
      drawRect(panelConfig.id, obs.x, obs.y, obs.width, obs.height,
               COLORS.obstacleFill, COLORS.obstacle);
    } else {
      console.log(`Panel ${panelConfig.id}: No obstacle defined`);
    }

    // Create connectors using createWithEndpoints pattern
    const connectors = this.ROUTES.map(route => {
      const conn = ConnRef.createWithEndpoints(router,
        new ConnEnd(new Point(route.src.x, route.src.y)),
        new ConnEnd(new Point(route.dst.x, route.dst.y)));
      conn.setRoutingType(ORTHOGONAL_ROUTING);
      router.addConnector(conn);
      // Draw small endpoint markers
      drawPoint(panelConfig.id, route.src.x, route.src.y, 3, route.color);
      drawPoint(panelConfig.id, route.dst.x, route.dst.y, 3, route.color);
      return { conn, color: route.color };
    });

    // Route
    router.processTransaction();

    // Draw routes with debug logging
    connectors.forEach(({ conn, color }, idx) => {
      const route = router.getConnectorRoute(conn.id());
      if (route && route.size() > 0) {
        drawRoute(panelConfig.id, route, color, 1.5);
        // Log all route points for debugging
        const points = [];
        for (let i = 0; i < route.size(); i++) {
          const pt = route.at(i);
          if (pt) points.push(`(${pt.x.toFixed(1)},${pt.y.toFixed(1)})`);
        }
        console.log(`Panel ${panelConfig.id}: Route ${idx} [${points.join(' -> ')}]`);
      } else {
        console.log(`Panel ${panelConfig.id}: Route ${idx} has no route!`);
      }
    });
  },

  run: function() {
    window.markExampleRun('comparison');
    clearLog('output-comparison');
    log('output-comparison', `Routing Options Comparison (base: ${window.globalNudgeDistance}px):`);
    log('output-comparison', '');

    const panelConfig = this.getPanelConfig();

    // Update labels in DOM
    const labels = document.querySelectorAll('.comparison-label');
    panelConfig.forEach((config, i) => {
      if (labels[i]) {
        labels[i].textContent = config.label;
      }
    });

    // Run each panel
    panelConfig.forEach((config, i) => {
      clearSvg(config.id);
      this.runPanel(config);
      log('output-comparison', `Panel ${i + 1}: ${config.label}`);
    });

    log('output-comparison', '');
    log('output-comparison', '✓ Compare how IDEAL_NUDGING_DISTANCE affects route separation');
  },

  reset: function() {
    const panelConfig = this.getPanelConfig();
    panelConfig.forEach(config => clearSvg(config.id));
    clearLog('output-comparison');
    log('output-comparison', 'Click "Run Comparison" to see different nudging settings');
  }
};

// ============================================================================
// Initialize on WASM load
// ============================================================================

async function initializeGallery() {
  const loading = document.getElementById('loading');
  const gallery = document.getElementById('gallery');

  try {
    await init();

    // Initialize interactive example
    window.examples.interactive.init();

    // Set up global controls (must be done after module loads)
    const nudgeInput = document.getElementById('global-nudge-input');
    const bufferInput = document.getElementById('global-buffer-input');
    const applyBtn = document.getElementById('apply-nudge-btn');

    nudgeInput.addEventListener('change', () => window.applyGlobalSettings());
    bufferInput.addEventListener('change', () => window.applyGlobalSettings());
    applyBtn.addEventListener('click', () => window.applyGlobalSettings());

    loading.style.display = 'none';
    gallery.style.display = 'grid';

    console.log('libavoid-rust gallery initialized');
  } catch (err) {
    loading.textContent = 'Error loading WASM module: ' + err.message;
    console.error('Failed to load WASM:', err);
  }
}

initializeGallery();
