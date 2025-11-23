/**
 * Integration test - basic workflow test
 */

import { describe, it, expect, beforeEach } from 'vitest';

describe('Basic Integration', () => {
  beforeEach(({ skip }) => {
    if (!globalThis.Avoid?.Router) skip();
  });

  it('creates router and processes transaction', () => {
    const router = new globalThis.Avoid.Router(globalThis.Avoid.PolyLineRouting);
    expect(() => router.processTransaction()).not.toThrow();
  });

  it('creates shape and adds to router', () => {
    const router = new globalThis.Avoid.Router(globalThis.Avoid.PolyLineRouting);

    const poly = new globalThis.Avoid.Polygon(4);
    poly.set_ps(0, new globalThis.Avoid.Point(0, 0));
    poly.set_ps(1, new globalThis.Avoid.Point(10, 0));
    poly.set_ps(2, new globalThis.Avoid.Point(10, 10));
    poly.set_ps(3, new globalThis.Avoid.Point(0, 10));

    const shape = new globalThis.Avoid.ShapeRef(router, poly);
    expect(shape).toBeDefined();
    expect(typeof shape.id()).toBe('number');

    router.processTransaction();
  });

  it('creates connector', () => {
    const router = new globalThis.Avoid.Router(globalThis.Avoid.PolyLineRouting);
    const conn = new globalThis.Avoid.ConnRef(router);

    expect(conn).toBeDefined();
    expect(typeof conn.id()).toBe('number');
  });

  it('moves shape after creation', () => {
    const router = new globalThis.Avoid.Router(globalThis.Avoid.PolyLineRouting);

    const poly = new globalThis.Avoid.Polygon(4);
    poly.set_ps(0, new globalThis.Avoid.Point(0, 0));
    poly.set_ps(1, new globalThis.Avoid.Point(10, 0));
    poly.set_ps(2, new globalThis.Avoid.Point(10, 10));
    poly.set_ps(3, new globalThis.Avoid.Point(0, 10));

    const shape = new globalThis.Avoid.ShapeRef(router, poly);
    router.processTransaction();

    expect(() => {
      router.moveShape(shape, 5, 5);
      router.processTransaction();
    }).not.toThrow();
  });
});
