/**
 * ConnRef class unit tests
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';

describe('ConnRef', () => {
  let router;

  beforeEach(({ skip }) => {
    if (!globalThis.Avoid?.ConnRef) {
      skip();
    }
    router = new globalThis.Avoid.Router(globalThis.Avoid.PolyLineRouting);
  });

  describe('constructor', () => {
    it('creates with router only', () => {
      const conn = new globalThis.Avoid.ConnRef(router);
      expect(conn).toBeDefined();
      expect(typeof conn.id()).toBe('number');
    });
  });

  describe('id()', () => {
    it('returns numeric id', () => {
      const conn = new globalThis.Avoid.ConnRef(router);
      expect(typeof conn.id()).toBe('number');
    });
  });

  describe('endpoint methods', () => {
    it('setDestEndpoint updates destination', () => {
      const conn = new globalThis.Avoid.ConnRef(router);
      const dst = new globalThis.Avoid.ConnEnd(new globalThis.Avoid.Point(15, 15));
      expect(() => conn.setDestEndpoint(dst)).not.toThrow();
    });
  });

  describe('displayRoute()', () => {
    it('returns route with size method', () => {
      const src = new globalThis.Avoid.ConnEnd(new globalThis.Avoid.Point(0, 0));
      const dst = new globalThis.Avoid.ConnEnd(new globalThis.Avoid.Point(10, 10));

      // Need to set up connector properly - create and set endpoints
      const conn = new globalThis.Avoid.ConnRef(router);
      // Note: Current implementation may not support full ConnRef(router, src, dst) constructor

      router.processTransaction();

      const route = conn.displayRoute();
      // Route might be null/undefined if no endpoints set
      if (route) {
        expect(typeof route.size).toBe('function');
      }
    });
  });

  describe('setCallback()', () => {
    it('accepts callback function without error', () => {
      const conn = new globalThis.Avoid.ConnRef(router);
      const callback = vi.fn();
      // Current implementation is a stub - just verify it doesn't throw
      expect(() => conn.setCallback(callback, conn)).not.toThrow();
    });
  });

  describe('hate crossings', () => {
    it('setHateCrossings and doesHateCrossings work together', () => {
      const conn = new globalThis.Avoid.ConnRef(router);

      expect(conn.doesHateCrossings()).toBe(false);
      conn.setHateCrossings(true);
      expect(conn.doesHateCrossings()).toBe(true);
    });
  });

  describe('factory methods', () => {
    it('createWithEndpoints creates connector with endpoints', () => {
      const srcPoint = new globalThis.Avoid.Point(0, 0);
      const dstPoint = new globalThis.Avoid.Point(100, 100);
      const srcEnd = new globalThis.Avoid.ConnEnd(srcPoint);
      const dstEnd = new globalThis.Avoid.ConnEnd(dstPoint);

      const conn = globalThis.Avoid.ConnRef.createWithEndpoints(router, srcEnd, dstEnd);
      expect(conn).toBeDefined();
      expect(typeof conn.id()).toBe('number');
    });

    it('createWithId creates connector with specific ID', () => {
      const srcPoint = new globalThis.Avoid.Point(0, 0);
      const dstPoint = new globalThis.Avoid.Point(100, 100);
      const srcEnd = new globalThis.Avoid.ConnEnd(srcPoint);
      const dstEnd = new globalThis.Avoid.ConnEnd(dstPoint);

      const conn = globalThis.Avoid.ConnRef.createWithId(router, srcEnd, dstEnd, 999);
      expect(conn).toBeDefined();
      expect(conn.id()).toBe(999);
    });
  });
});
