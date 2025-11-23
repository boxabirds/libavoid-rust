/**
 * Router class unit tests
 */

import { describe, it, expect, beforeEach } from 'vitest';

describe('Router', () => {
  beforeEach(({ skip }) => {
    if (!globalThis.Avoid?.Router) {
      skip();
    }
  });

  describe('constructor', () => {
    it('creates router with PolyLineRouting', () => {
      const router = new globalThis.Avoid.Router(globalThis.Avoid.PolyLineRouting);
      expect(router).toBeDefined();
    });

    it('creates router with OrthogonalRouting', () => {
      const router = new globalThis.Avoid.Router(globalThis.Avoid.OrthogonalRouting);
      expect(router).toBeDefined();
    });
  });

  describe('processTransaction', () => {
    it('can be called without error', () => {
      const router = new globalThis.Avoid.Router(globalThis.Avoid.PolyLineRouting);
      expect(() => router.processTransaction()).not.toThrow();
    });

    it('can be called multiple times', () => {
      const router = new globalThis.Avoid.Router(globalThis.Avoid.PolyLineRouting);
      expect(() => {
        router.processTransaction();
        router.processTransaction();
        router.processTransaction();
      }).not.toThrow();
    });
  });

  describe('shape management', () => {
    it('moveShape accepts shape and offsets', () => {
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
});
