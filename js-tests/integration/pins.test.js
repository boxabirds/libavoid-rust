/**
 * Connection Pin Tests
 *
 * Tests for ShapeConnectionPin functionality
 */

import { describe, it, expect, beforeEach } from 'vitest';

describe('Connection Pins', () => {
  let router;
  let shape;

  beforeEach(({ skip }) => {
    if (!globalThis.Avoid?.ShapeConnectionPin) skip();
    router = new globalThis.Avoid.Router(globalThis.Avoid.PolyLineRouting);

    // Create a square shape for pin tests
    const poly = new globalThis.Avoid.Polygon(4);
    poly.set_ps(0, new globalThis.Avoid.Point(0, 0));
    poly.set_ps(1, new globalThis.Avoid.Point(100, 0));
    poly.set_ps(2, new globalThis.Avoid.Point(100, 100));
    poly.set_ps(3, new globalThis.Avoid.Point(0, 100));
    shape = new globalThis.Avoid.ShapeRef(router, poly);
  });

  describe('Pin Creation', () => {
    it('creates pin on shape with all directions', () => {
      const pin = new globalThis.Avoid.ShapeConnectionPin(
        shape,
        1, // class ID
        50, // x offset
        0, // y offset
        0, // inside offset
        globalThis.Avoid.ConnDirAll
      );

      expect(pin.directions()).toBe(globalThis.Avoid.ConnDirAll);
    });

    it('creates pin with specific direction', () => {
      const pin = new globalThis.Avoid.ShapeConnectionPin(
        shape,
        1,
        50, 0, 0,
        globalThis.Avoid.ConnDirUp
      );

      expect(pin.directions()).toBe(globalThis.Avoid.ConnDirUp);
    });

    it('creates pin at different positions', () => {
      // Top pin
      const topPin = new globalThis.Avoid.ShapeConnectionPin(
        shape, 1, 50, 0, 0, globalThis.Avoid.ConnDirUp
      );

      // Bottom pin
      const bottomPin = new globalThis.Avoid.ShapeConnectionPin(
        shape, 2, 50, 100, 0, globalThis.Avoid.ConnDirDown
      );

      // Left pin
      const leftPin = new globalThis.Avoid.ShapeConnectionPin(
        shape, 3, 0, 50, 0, globalThis.Avoid.ConnDirLeft
      );

      // Right pin
      const rightPin = new globalThis.Avoid.ShapeConnectionPin(
        shape, 4, 100, 50, 0, globalThis.Avoid.ConnDirRight
      );

      expect(topPin.directions()).toBe(globalThis.Avoid.ConnDirUp);
      expect(bottomPin.directions()).toBe(globalThis.Avoid.ConnDirDown);
      expect(leftPin.directions()).toBe(globalThis.Avoid.ConnDirLeft);
      expect(rightPin.directions()).toBe(globalThis.Avoid.ConnDirRight);
    });
  });

  describe('Pin Exclusivity', () => {
    it('pins are non-exclusive by default', () => {
      const pin = new globalThis.Avoid.ShapeConnectionPin(
        shape, 1, 50, 0, 0, globalThis.Avoid.ConnDirAll
      );

      expect(pin.isExclusive()).toBe(false);
    });

    it('can set pin to exclusive', () => {
      const pin = new globalThis.Avoid.ShapeConnectionPin(
        shape, 1, 50, 0, 0, globalThis.Avoid.ConnDirAll
      );

      pin.setExclusive(true);
      expect(pin.isExclusive()).toBe(true);
    });

    it('can toggle pin exclusivity', () => {
      const pin = new globalThis.Avoid.ShapeConnectionPin(
        shape, 1, 50, 0, 0, globalThis.Avoid.ConnDirAll
      );

      pin.setExclusive(true);
      expect(pin.isExclusive()).toBe(true);

      pin.setExclusive(false);
      expect(pin.isExclusive()).toBe(false);
    });
  });

  describe('Pin Connection Cost', () => {
    it('can set connection cost', () => {
      const pin = new globalThis.Avoid.ShapeConnectionPin(
        shape, 1, 50, 0, 0, globalThis.Avoid.ConnDirAll
      );

      expect(() => pin.setConnectionCost(10.5)).not.toThrow();
    });
  });

  describe('ConnEnd from Pin', () => {
    it('creates ConnEnd from shape pin class', () => {
      // Create a pin on the shape
      const pin = new globalThis.Avoid.ShapeConnectionPin(
        shape, 1, 50, 0, 0, globalThis.Avoid.ConnDirAll
      );

      // Create ConnEnd referencing the pin class
      const connEnd = globalThis.Avoid.ConnEnd.fromShapePin(shape, 1);

      expect(connEnd).toBeDefined();
    });

    it('can route connector to pin', () => {
      // Create pin on shape
      const pin = new globalThis.Avoid.ShapeConnectionPin(
        shape, 1, 50, 0, 0, globalThis.Avoid.ConnDirAll
      );

      router.processTransaction();

      // Create connector from external point to pin
      const srcEnd = new globalThis.Avoid.ConnEnd(new globalThis.Avoid.Point(-50, 50));
      const dstEnd = globalThis.Avoid.ConnEnd.fromShapePin(shape, 1);
      const conn = globalThis.Avoid.ConnRef.createWithEndpoints(router, srcEnd, dstEnd);

      router.processTransaction();

      // Connector should be created
      expect(conn.id()).toBeGreaterThan(0);
    });
  });

  describe('Multiple Pins on Shape', () => {
    it('supports multiple pins with different class IDs', () => {
      const CLASS_ID_TOP = 1;
      const CLASS_ID_BOTTOM = 2;
      const CLASS_ID_LEFT = 3;
      const CLASS_ID_RIGHT = 4;

      const topPin = new globalThis.Avoid.ShapeConnectionPin(
        shape, CLASS_ID_TOP, 50, 0, 0, globalThis.Avoid.ConnDirUp
      );
      const bottomPin = new globalThis.Avoid.ShapeConnectionPin(
        shape, CLASS_ID_BOTTOM, 50, 100, 0, globalThis.Avoid.ConnDirDown
      );
      const leftPin = new globalThis.Avoid.ShapeConnectionPin(
        shape, CLASS_ID_LEFT, 0, 50, 0, globalThis.Avoid.ConnDirLeft
      );
      const rightPin = new globalThis.Avoid.ShapeConnectionPin(
        shape, CLASS_ID_RIGHT, 100, 50, 0, globalThis.Avoid.ConnDirRight
      );

      // Can create ConnEnds for each pin class
      const topEnd = globalThis.Avoid.ConnEnd.fromShapePin(shape, CLASS_ID_TOP);
      const bottomEnd = globalThis.Avoid.ConnEnd.fromShapePin(shape, CLASS_ID_BOTTOM);
      const leftEnd = globalThis.Avoid.ConnEnd.fromShapePin(shape, CLASS_ID_LEFT);
      const rightEnd = globalThis.Avoid.ConnEnd.fromShapePin(shape, CLASS_ID_RIGHT);

      expect(topEnd).toBeDefined();
      expect(bottomEnd).toBeDefined();
      expect(leftEnd).toBeDefined();
      expect(rightEnd).toBeDefined();
    });
  });
});
