/**
 * Behavioral Parity Tests
 *
 * Compares output of libavoid-rust vs libavoid-js for same inputs
 */

import { describe, it, expect, beforeEach } from 'vitest';

describe('Behavioral Parity', () => {
  const bothLoaded = () => globalThis.Avoid && globalThis.AvoidJS;

  describe('Basic Routing', () => {
    beforeEach(({ skip }) => {
      if (!bothLoaded()) skip();
    });

    it('both libraries can create routers', () => {
      const router1 = new globalThis.Avoid.Router(globalThis.Avoid.PolyLineRouting);
      const router2 = new globalThis.AvoidJS.Router(globalThis.AvoidJS.PolyLineRouting);

      expect(router1).toBeDefined();
      expect(router2).toBeDefined();
    });

    it('both libraries can create shapes', () => {
      const router1 = new globalThis.Avoid.Router(globalThis.Avoid.PolyLineRouting);
      const router2 = new globalThis.AvoidJS.Router(globalThis.AvoidJS.PolyLineRouting);

      // Our implementation
      const poly1 = new globalThis.Avoid.Polygon(4);
      poly1.set_ps(0, new globalThis.Avoid.Point(0, 0));
      poly1.set_ps(1, new globalThis.Avoid.Point(10, 0));
      poly1.set_ps(2, new globalThis.Avoid.Point(10, 10));
      poly1.set_ps(3, new globalThis.Avoid.Point(0, 10));
      const shape1 = new globalThis.Avoid.ShapeRef(router1, poly1);

      // Reference implementation
      const poly2 = new globalThis.AvoidJS.Polygon(4);
      poly2.set_ps(0, new globalThis.AvoidJS.Point(0, 0));
      poly2.set_ps(1, new globalThis.AvoidJS.Point(10, 0));
      poly2.set_ps(2, new globalThis.AvoidJS.Point(10, 10));
      poly2.set_ps(3, new globalThis.AvoidJS.Point(0, 10));
      const shape2 = new globalThis.AvoidJS.ShapeRef(router2, poly2);

      // Both shapes should be created successfully
      expect(shape1).toBeDefined();
      expect(shape2).toBeDefined();
      // Our implementation exposes id() method
      expect(typeof shape1.id()).toBe('number');
    });
  });
});
