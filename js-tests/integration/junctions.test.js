/**
 * Junction-Based Routing Tests
 *
 * Tests for JunctionRef and multi-connector routing through junctions
 */

import { describe, it, expect, beforeEach } from 'vitest';

describe('Junction-Based Routing', () => {
  let router;

  beforeEach(({ skip }) => {
    if (!globalThis.Avoid?.JunctionRef) skip();
    router = new globalThis.Avoid.Router(globalThis.Avoid.PolyLineRouting);
  });

  describe('Junction Creation', () => {
    it('creates junction at position', () => {
      const pos = new globalThis.Avoid.Point(100, 100);
      const junction = new globalThis.Avoid.JunctionRef(router, pos);

      expect(junction.id()).toBeGreaterThan(0);
      expect(junction.position().x).toBe(100);
      expect(junction.position().y).toBe(100);
    });

    it('creates junction with specific ID', () => {
      const pos = new globalThis.Avoid.Point(50, 50);
      const junction = globalThis.Avoid.JunctionRef.createWithId(router, pos, 999);

      expect(junction.id()).toBe(999);
    });

    it('updates junction position', () => {
      const pos1 = new globalThis.Avoid.Point(0, 0);
      const junction = new globalThis.Avoid.JunctionRef(router, pos1);

      const pos2 = new globalThis.Avoid.Point(200, 200);
      junction.setPosition(pos2);

      expect(junction.position().x).toBe(200);
      expect(junction.position().y).toBe(200);
    });
  });

  describe('Junction as ConnEnd', () => {
    it('can connect multiple connectors through a junction', () => {
      // Create a junction at the center
      const junctionPos = new globalThis.Avoid.Point(150, 150);
      const junction = new globalThis.Avoid.JunctionRef(router, junctionPos);

      // Create three endpoints
      const end1 = new globalThis.Avoid.ConnEnd(new globalThis.Avoid.Point(0, 150));
      const end2 = new globalThis.Avoid.ConnEnd(new globalThis.Avoid.Point(300, 150));
      const end3 = new globalThis.Avoid.ConnEnd(new globalThis.Avoid.Point(150, 0));
      const junctionEnd = new globalThis.Avoid.ConnEnd(junctionPos);

      // Create connectors to/from junction
      const conn1 = globalThis.Avoid.ConnRef.createWithEndpoints(router, end1, junctionEnd);
      const conn2 = globalThis.Avoid.ConnRef.createWithEndpoints(router, junctionEnd, end2);
      const conn3 = globalThis.Avoid.ConnRef.createWithEndpoints(router, junctionEnd, end3);

      expect(conn1.id()).not.toBe(conn2.id());
      expect(conn2.id()).not.toBe(conn3.id());

      router.processTransaction();
    });
  });

  describe('HyperedgeRerouter', () => {
    it('registers hyperedge for junction', () => {
      const junction = new globalThis.Avoid.JunctionRef(
        router,
        new globalThis.Avoid.Point(100, 100)
      );

      const rerouter = new globalThis.Avoid.HyperedgeRerouter();
      const id = rerouter.registerHyperedgeForRerouting(junction);

      expect(typeof id).toBe('number');
      expect(id).toBe(0); // First hyperedge gets ID 0
    });

    it('registers multiple hyperedges', () => {
      const junction1 = new globalThis.Avoid.JunctionRef(
        router,
        new globalThis.Avoid.Point(100, 100)
      );
      const junction2 = new globalThis.Avoid.JunctionRef(
        router,
        new globalThis.Avoid.Point(200, 200)
      );

      const rerouter = new globalThis.Avoid.HyperedgeRerouter();
      const id1 = rerouter.registerHyperedgeForRerouting(junction1);
      const id2 = rerouter.registerHyperedgeForRerouting(junction2);

      expect(id1).toBe(0);
      expect(id2).toBe(1);
    });
  });
});
