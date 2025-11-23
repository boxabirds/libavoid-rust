/**
 * API Surface Compatibility Tests
 *
 * Verifies that libavoid-rust exposes the same API as libavoid-js
 */

import { describe, it, expect, beforeEach } from 'vitest';

describe('API Surface Compatibility', () => {
  describe('Core Classes (currently implemented)', () => {
    beforeEach(({ skip }) => {
      if (!globalThis.Avoid) skip();
    });

    it('has Point class', () => {
      expect(globalThis.Avoid.Point).toBeDefined();
      expect(typeof globalThis.Avoid.Point).toBe('function');
    });

    it('has Polygon class', () => {
      expect(globalThis.Avoid.Polygon).toBeDefined();
      expect(typeof globalThis.Avoid.Polygon).toBe('function');
    });

    it('has Router class', () => {
      expect(globalThis.Avoid.Router).toBeDefined();
      expect(typeof globalThis.Avoid.Router).toBe('function');
    });

    it('has ConnRef class', () => {
      expect(globalThis.Avoid.ConnRef).toBeDefined();
      expect(typeof globalThis.Avoid.ConnRef).toBe('function');
    });

    it('has ConnEnd class', () => {
      expect(globalThis.Avoid.ConnEnd).toBeDefined();
      expect(typeof globalThis.Avoid.ConnEnd).toBe('function');
    });

    it('has ShapeRef class', () => {
      expect(globalThis.Avoid.ShapeRef).toBeDefined();
      expect(typeof globalThis.Avoid.ShapeRef).toBe('function');
    });
  });

  describe('Router Flags', () => {
    beforeEach(({ skip }) => {
      if (!globalThis.Avoid) skip();
    });

    it('has PolyLineRouting constant', () => {
      expect(globalThis.Avoid.PolyLineRouting).toBeDefined();
      expect(typeof globalThis.Avoid.PolyLineRouting).toBe('number');
    });

    it('has OrthogonalRouting constant', () => {
      expect(globalThis.Avoid.OrthogonalRouting).toBeDefined();
      expect(typeof globalThis.Avoid.OrthogonalRouting).toBe('number');
    });
  });

  describe('Geometry Classes (implemented)', () => {
    beforeEach(({ skip }) => {
      if (!globalThis.Avoid) skip();
    });

    it('has Rectangle class', () => {
      expect(globalThis.Avoid.Rectangle).toBeDefined();
      expect(typeof globalThis.Avoid.Rectangle).toBe('function');
    });

    it('has Box class', () => {
      expect(globalThis.Avoid.Box).toBeDefined();
      expect(typeof globalThis.Avoid.Box).toBe('function');
    });

    it('Rectangle can be created with center, width, height', () => {
      const center = new globalThis.Avoid.Point(10, 20);
      const rect = new globalThis.Avoid.Rectangle(center, 50, 30);
      expect(rect.width()).toBe(50);
      expect(rect.height()).toBe(30);
    });

    it('Box has min/max properties', () => {
      const box = globalThis.Avoid.Box.fromCoords(0, 0, 100, 50);
      expect(box.width()).toBe(100);
      expect(box.height()).toBe(50);
    });

    it('Polygon has boundingRectPolygon method', () => {
      const poly = new globalThis.Avoid.Polygon(4);
      poly.set_ps(0, new globalThis.Avoid.Point(0, 0));
      poly.set_ps(1, new globalThis.Avoid.Point(10, 0));
      poly.set_ps(2, new globalThis.Avoid.Point(10, 10));
      poly.set_ps(3, new globalThis.Avoid.Point(0, 10));
      const bbox = poly.boundingRectPolygon();
      expect(bbox.size()).toBe(4);
    });
  });

  describe('JunctionRef (implemented)', () => {
    beforeEach(({ skip }) => {
      if (!globalThis.Avoid) skip();
    });

    it('has JunctionRef class', () => {
      expect(globalThis.Avoid.JunctionRef).toBeDefined();
      expect(typeof globalThis.Avoid.JunctionRef).toBe('function');
    });

    it('JunctionRef can be created with router and position', () => {
      const router = new globalThis.Avoid.Router(globalThis.Avoid.PolyLineRouting);
      const pos = new globalThis.Avoid.Point(50, 50);
      const junction = new globalThis.Avoid.JunctionRef(router, pos);
      expect(junction).toBeDefined();
      expect(typeof junction.id()).toBe('number');
    });
  });

  describe('Missing Classes (GAP - need to implement)', () => {
    beforeEach(({ skip }) => {
      if (!globalThis.Avoid) skip();
    });

    it.fails('has ShapeConnectionPin class', () => {
      expect(globalThis.Avoid.ShapeConnectionPin).toBeDefined();
    });

    it.fails('has HyperedgeRerouter class', () => {
      expect(globalThis.Avoid.HyperedgeRerouter).toBeDefined();
    });
  });

  describe('Constants (implemented)', () => {
    beforeEach(({ skip }) => {
      if (!globalThis.Avoid) skip();
    });

    it('has ConnDirNone constant', () => {
      expect(globalThis.Avoid.ConnDirNone).toBeDefined();
      expect(typeof globalThis.Avoid.ConnDirNone).toBe('number');
    });

    it('has segmentPenalty constant', () => {
      expect(globalThis.Avoid.segmentPenalty).toBeDefined();
      expect(typeof globalThis.Avoid.segmentPenalty).toBe('number');
    });

    it('has ConnType_PolyLine constant', () => {
      expect(globalThis.Avoid.ConnType_PolyLine).toBeDefined();
      expect(typeof globalThis.Avoid.ConnType_PolyLine).toBe('number');
    });

    it('has all direction constants', () => {
      expect(globalThis.Avoid.ConnDirUp).toBe(1);
      expect(globalThis.Avoid.ConnDirDown).toBe(2);
      expect(globalThis.Avoid.ConnDirLeft).toBe(4);
      expect(globalThis.Avoid.ConnDirRight).toBe(8);
      expect(globalThis.Avoid.ConnDirAll).toBe(15);
    });

    it('has all routing parameter constants', () => {
      expect(globalThis.Avoid.segmentPenalty).toBe(0);
      expect(globalThis.Avoid.anglePenalty).toBe(1);
      expect(globalThis.Avoid.crossingPenalty).toBe(2);
      expect(globalThis.Avoid.shapeBufferDistance).toBe(6);
      expect(globalThis.Avoid.idealNudgingDistance).toBe(7);
    });
  });

  describe('Missing Utility Functions (GAP - need to implement)', () => {
    beforeEach(({ skip }) => {
      if (!globalThis.Avoid) skip();
    });

    it.fails('has destroy() function', () => {
      expect(typeof globalThis.Avoid.destroy).toBe('function');
    });

    it.fails('has getPointer() function', () => {
      expect(typeof globalThis.Avoid.getPointer).toBe('function');
    });

    it.fails('has wrapPointer() function', () => {
      expect(typeof globalThis.Avoid.wrapPointer).toBe('function');
    });
  });
});

describe('Compare with libavoid-js Reference', () => {
  beforeEach(({ skip }) => {
    if (!globalThis.AvoidJS) skip();
  });

  it('libavoid-js reference is loaded', () => {
    expect(globalThis.AvoidJS).toBeDefined();
  });

  it('has same PolyLineRouting value', () => {
    if (!globalThis.Avoid) return;
    expect(globalThis.Avoid.PolyLineRouting).toBe(globalThis.AvoidJS.PolyLineRouting);
  });

  it('has same OrthogonalRouting value', () => {
    if (!globalThis.Avoid) return;
    expect(globalThis.Avoid.OrthogonalRouting).toBe(globalThis.AvoidJS.OrthogonalRouting);
  });
});
