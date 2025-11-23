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

  describe('Missing Classes (GAP - need to implement)', () => {
    beforeEach(({ skip }) => {
      if (!globalThis.Avoid) skip();
    });

    it.fails('has Rectangle class', () => {
      expect(globalThis.Avoid.Rectangle).toBeDefined();
    });

    it.fails('has Box class', () => {
      expect(globalThis.Avoid.Box).toBeDefined();
    });

    it.fails('has JunctionRef class', () => {
      expect(globalThis.Avoid.JunctionRef).toBeDefined();
    });

    it.fails('has ShapeConnectionPin class', () => {
      expect(globalThis.Avoid.ShapeConnectionPin).toBeDefined();
    });

    it.fails('has HyperedgeRerouter class', () => {
      expect(globalThis.Avoid.HyperedgeRerouter).toBeDefined();
    });
  });

  describe('Missing Constants (GAP - need to implement)', () => {
    beforeEach(({ skip }) => {
      if (!globalThis.Avoid) skip();
    });

    it.fails('has ConnDirNone constant', () => {
      expect(globalThis.Avoid.ConnDirNone).toBeDefined();
      expect(typeof globalThis.Avoid.ConnDirNone).toBe('number');
    });

    it.fails('has segmentPenalty constant', () => {
      expect(globalThis.Avoid.segmentPenalty).toBeDefined();
      expect(typeof globalThis.Avoid.segmentPenalty).toBe('number');
    });

    it.fails('has ConnType_PolyLine constant', () => {
      expect(globalThis.Avoid.ConnType_PolyLine).toBeDefined();
      expect(typeof globalThis.Avoid.ConnType_PolyLine).toBe('number');
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
