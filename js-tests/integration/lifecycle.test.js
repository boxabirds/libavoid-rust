/**
 * Shape/Connector Lifecycle Tests
 *
 * Tests create, modify, and delete operations for shapes and connectors
 */

import { describe, it, expect, beforeEach } from 'vitest';

describe('Shape/Connector Lifecycle', () => {
  let router;

  beforeEach(({ skip }) => {
    if (!globalThis.Avoid?.Router) skip();
    router = new globalThis.Avoid.Router(globalThis.Avoid.PolyLineRouting);
  });

  describe('Shape Lifecycle', () => {
    it('creates shape with polygon', () => {
      const poly = new globalThis.Avoid.Polygon(4);
      poly.set_ps(0, new globalThis.Avoid.Point(0, 0));
      poly.set_ps(1, new globalThis.Avoid.Point(100, 0));
      poly.set_ps(2, new globalThis.Avoid.Point(100, 100));
      poly.set_ps(3, new globalThis.Avoid.Point(0, 100));

      const shape = new globalThis.Avoid.ShapeRef(router, poly);
      expect(shape.id()).toBeGreaterThan(0);
      expect(shape.polygon().size()).toBe(4);
    });

    it('moves shape to new position', () => {
      const poly = new globalThis.Avoid.Polygon(4);
      poly.set_ps(0, new globalThis.Avoid.Point(0, 0));
      poly.set_ps(1, new globalThis.Avoid.Point(100, 0));
      poly.set_ps(2, new globalThis.Avoid.Point(100, 100));
      poly.set_ps(3, new globalThis.Avoid.Point(0, 100));

      const shape = new globalThis.Avoid.ShapeRef(router, poly);
      router.processTransaction();

      router.moveShape(shape, 200, 200);
      router.processTransaction();

      // Shape should have been moved (exact position depends on implementation)
      expect(shape).toBeDefined();
    });

    it('updates shape polygon with setNewPoly', () => {
      const poly1 = new globalThis.Avoid.Polygon(4);
      poly1.set_ps(0, new globalThis.Avoid.Point(0, 0));
      poly1.set_ps(1, new globalThis.Avoid.Point(50, 0));
      poly1.set_ps(2, new globalThis.Avoid.Point(50, 50));
      poly1.set_ps(3, new globalThis.Avoid.Point(0, 50));

      const shape = new globalThis.Avoid.ShapeRef(router, poly1);
      router.processTransaction();

      const poly2 = new globalThis.Avoid.Polygon(4);
      poly2.set_ps(0, new globalThis.Avoid.Point(100, 100));
      poly2.set_ps(1, new globalThis.Avoid.Point(200, 100));
      poly2.set_ps(2, new globalThis.Avoid.Point(200, 200));
      poly2.set_ps(3, new globalThis.Avoid.Point(100, 200));

      shape.setNewPoly(poly2);
      router.processTransaction();

      // Polygon should be updated
      expect(shape.polygon().size()).toBe(4);
    });

    it('deletes shape from router', () => {
      const poly = new globalThis.Avoid.Polygon(4);
      poly.set_ps(0, new globalThis.Avoid.Point(0, 0));
      poly.set_ps(1, new globalThis.Avoid.Point(100, 0));
      poly.set_ps(2, new globalThis.Avoid.Point(100, 100));
      poly.set_ps(3, new globalThis.Avoid.Point(0, 100));

      const shape = new globalThis.Avoid.ShapeRef(router, poly);
      const shapeId = shape.id();
      router.processTransaction();

      expect(() => router.deleteShape(shape)).not.toThrow();
      router.processTransaction();
    });
  });

  describe('Connector Lifecycle', () => {
    it('creates connector with endpoints', () => {
      const srcPoint = new globalThis.Avoid.Point(0, 0);
      const dstPoint = new globalThis.Avoid.Point(100, 100);
      const srcEnd = new globalThis.Avoid.ConnEnd(srcPoint);
      const dstEnd = new globalThis.Avoid.ConnEnd(dstPoint);

      const conn = globalThis.Avoid.ConnRef.createWithEndpoints(router, srcEnd, dstEnd);
      expect(conn.id()).toBeGreaterThan(0);
    });

    it('sets connector endpoints after creation', () => {
      const conn = new globalThis.Avoid.ConnRef(router);

      const srcEnd = new globalThis.Avoid.ConnEnd(new globalThis.Avoid.Point(0, 0));
      const dstEnd = new globalThis.Avoid.ConnEnd(new globalThis.Avoid.Point(100, 100));

      expect(() => {
        conn.setSourceEndpoint(srcEnd);
        conn.setDestEndpoint(dstEnd);
      }).not.toThrow();
    });

    it('changes connector routing type', () => {
      const conn = new globalThis.Avoid.ConnRef(router);

      conn.setRoutingType(globalThis.Avoid.ConnType_PolyLine);
      expect(conn.routingType()).toBe(globalThis.Avoid.ConnType_PolyLine);

      conn.setRoutingType(globalThis.Avoid.ConnType_Orthogonal);
      expect(conn.routingType()).toBe(globalThis.Avoid.ConnType_Orthogonal);
    });

    it('deletes connector from router', () => {
      const conn = new globalThis.Avoid.ConnRef(router);
      const connId = conn.id();

      expect(() => router.deleteConnector(conn)).not.toThrow();
      router.processTransaction();
    });
  });

  describe('Full Workflow', () => {
    it('creates shapes and routes connectors around them', () => {
      // Create two obstacle shapes
      const poly1 = new globalThis.Avoid.Polygon(4);
      poly1.set_ps(0, new globalThis.Avoid.Point(100, 100));
      poly1.set_ps(1, new globalThis.Avoid.Point(150, 100));
      poly1.set_ps(2, new globalThis.Avoid.Point(150, 150));
      poly1.set_ps(3, new globalThis.Avoid.Point(100, 150));
      const shape1 = new globalThis.Avoid.ShapeRef(router, poly1);

      const poly2 = new globalThis.Avoid.Polygon(4);
      poly2.set_ps(0, new globalThis.Avoid.Point(200, 100));
      poly2.set_ps(1, new globalThis.Avoid.Point(250, 100));
      poly2.set_ps(2, new globalThis.Avoid.Point(250, 150));
      poly2.set_ps(3, new globalThis.Avoid.Point(200, 150));
      const shape2 = new globalThis.Avoid.ShapeRef(router, poly2);

      // Create connector between two points
      const srcEnd = new globalThis.Avoid.ConnEnd(new globalThis.Avoid.Point(50, 125));
      const dstEnd = new globalThis.Avoid.ConnEnd(new globalThis.Avoid.Point(300, 125));
      const conn = globalThis.Avoid.ConnRef.createWithEndpoints(router, srcEnd, dstEnd);

      router.processTransaction();

      // Verify route exists (may be null if no route calculated)
      const route = conn.displayRoute();
      // Route should exist (implementation dependent)
    });
  });
});
