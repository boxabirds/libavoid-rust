/**
 * Point class unit tests
 */

import { describe, it, expect, beforeEach } from 'vitest';

describe('Point', () => {
  beforeEach(({ skip }) => {
    if (!globalThis.Avoid?.Point) {
      skip();
    }
  });

  describe('constructor', () => {
    it('creates point with specified coordinates', () => {
      const pt = new globalThis.Avoid.Point(3.5, 7.2);
      expect(pt.x).toBeCloseTo(3.5);
      expect(pt.y).toBeCloseTo(7.2);
    });

    it('handles negative coordinates', () => {
      const pt = new globalThis.Avoid.Point(-10, -20);
      expect(pt.x).toBe(-10);
      expect(pt.y).toBe(-20);
    });
  });

  describe('properties', () => {
    it('allows setting x and y', () => {
      const pt = new globalThis.Avoid.Point(0, 0);
      pt.x = 10;
      pt.y = 20;
      expect(pt.x).toBe(10);
      expect(pt.y).toBe(20);
    });
  });
});
