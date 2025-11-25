/**
 * Point Storage Tests
 *
 * Tests that Point objects stored in JavaScript arrays/objects retain their values.
 * This reproduces a bug where storing WASM Point objects directly led to incorrect
 * coordinate values when accessed later (the fix is to store raw x,y numbers instead).
 */

import { describe, it, expect, beforeEach } from 'vitest';

describe('Point Storage in JS Collections', () => {
  let Avoid;

  beforeEach(({ skip }) => {
    if (!globalThis.Avoid?.Point) skip();
    Avoid = globalThis.Avoid;
  });

  describe('Point object persistence', () => {
    it('Point coordinates remain valid after storage in array', () => {
      const pt = new Avoid.Point(123.5, 456.7);
      const arr = [pt];

      // Access later
      expect(arr[0].x).toBeCloseTo(123.5);
      expect(arr[0].y).toBeCloseTo(456.7);
    });

    it('Point coordinates remain valid in object property', () => {
      const pt = new Avoid.Point(100, 200);
      const obj = { src: pt };

      expect(obj.src.x).toBe(100);
      expect(obj.src.y).toBe(200);
    });

    it('multiple Points stored together retain correct values', () => {
      const p1 = new Avoid.Point(10, 20);
      const p2 = new Avoid.Point(30, 40);
      const p3 = new Avoid.Point(50, 60);

      const connectors = [
        { src: p1, dst: p2 },
        { src: p2, dst: p3 },
      ];

      // Verify each point
      expect(connectors[0].src.x).toBe(10);
      expect(connectors[0].src.y).toBe(20);
      expect(connectors[0].dst.x).toBe(30);
      expect(connectors[0].dst.y).toBe(40);
      expect(connectors[1].src.x).toBe(30);
      expect(connectors[1].src.y).toBe(40);
      expect(connectors[1].dst.x).toBe(50);
      expect(connectors[1].dst.y).toBe(60);
    });

    it('Point values survive after router operations', () => {
      const router = new Avoid.Router(Avoid.OrthogonalRouting);

      // Store points
      const srcPt = new Avoid.Point(50, 100);
      const dstPt = new Avoid.Point(200, 100);

      const stored = { src: srcPt, dst: dstPt };

      // Create connector (which uses the points internally)
      const srcEnd = new Avoid.ConnEnd(srcPt);
      const dstEnd = new Avoid.ConnEnd(dstPt);
      const conn = Avoid.ConnRef.createWithEndpoints(router, srcEnd, dstEnd);

      router.processTransaction();

      // Check stored points still have correct values
      expect(stored.src.x).toBe(50);
      expect(stored.src.y).toBe(100);
      expect(stored.dst.x).toBe(200);
      expect(stored.dst.y).toBe(100);
    });

    it('raw coordinate storage pattern works reliably', () => {
      // This is the recommended pattern: store raw numbers, create Points when needed
      const router = new Avoid.Router(Avoid.OrthogonalRouting);

      // Store raw coordinates (not Point objects)
      const connectorData = [];

      // Simulate adding connectors like gallery.js interactive example
      const coords = [
        { srcX: 50, srcY: 100, dstX: 200, dstY: 100 },
        { srcX: 50, srcY: 150, dstX: 200, dstY: 150 },
      ];

      coords.forEach(c => {
        // Create fresh Point objects for WASM API
        const srcPt = new Avoid.Point(c.srcX, c.srcY);
        const dstPt = new Avoid.Point(c.dstX, c.dstY);

        const srcEnd = new Avoid.ConnEnd(srcPt);
        const dstEnd = new Avoid.ConnEnd(dstPt);
        const conn = Avoid.ConnRef.createWithEndpoints(router, srcEnd, dstEnd);

        // Store raw numbers for later use (e.g., redrawing)
        connectorData.push({
          conn,
          srcX: c.srcX,
          srcY: c.srcY,
          dstX: c.dstX,
          dstY: c.dstY,
        });
      });

      router.processTransaction();

      // Verify stored raw coordinates are correct
      expect(connectorData[0].srcX).toBe(50);
      expect(connectorData[0].srcY).toBe(100);
      expect(connectorData[0].dstX).toBe(200);
      expect(connectorData[0].dstY).toBe(100);

      expect(connectorData[1].srcX).toBe(50);
      expect(connectorData[1].srcY).toBe(150);
      expect(connectorData[1].dstX).toBe(200);
      expect(connectorData[1].dstY).toBe(150);

      // Can recreate Points from stored coordinates
      const recreatedPt = new Avoid.Point(connectorData[0].srcX, connectorData[0].srcY);
      expect(recreatedPt.x).toBe(50);
      expect(recreatedPt.y).toBe(100);
    });
  });
});
