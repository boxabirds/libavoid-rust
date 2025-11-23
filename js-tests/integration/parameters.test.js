/**
 * Routing Parameter and Option Tests
 *
 * Tests for Router parameter and option settings
 */

import { describe, it, expect, beforeEach } from 'vitest';

describe('Routing Parameters', () => {
  let router;

  beforeEach(({ skip }) => {
    if (!globalThis.Avoid?.Router) skip();
    router = new globalThis.Avoid.Router(globalThis.Avoid.PolyLineRouting);
  });

  describe('Routing Parameter Constants', () => {
    it('has correct parameter constant values', () => {
      expect(globalThis.Avoid.segmentPenalty).toBe(0);
      expect(globalThis.Avoid.anglePenalty).toBe(1);
      expect(globalThis.Avoid.crossingPenalty).toBe(2);
      expect(globalThis.Avoid.clusterCrossingPenalty).toBe(3);
      expect(globalThis.Avoid.fixedSharedPathPenalty).toBe(4);
      expect(globalThis.Avoid.portDirectionPenalty).toBe(5);
      expect(globalThis.Avoid.shapeBufferDistance).toBe(6);
      expect(globalThis.Avoid.idealNudgingDistance).toBe(7);
      expect(globalThis.Avoid.reverseDirectionPenalty).toBe(8);
    });
  });

  describe('Routing Option Constants', () => {
    it('has correct option constant values', () => {
      expect(globalThis.Avoid.nudgeOrthogonalSegmentsConnectedToShapes).toBe(0);
      expect(globalThis.Avoid.improveHyperedgeRoutesMovingJunctions).toBe(1);
      expect(globalThis.Avoid.penaliseOrthogonalSharedPathsAtConnEnds).toBe(2);
      expect(globalThis.Avoid.nudgeOrthogonalColinearSegments).toBe(3);
      expect(globalThis.Avoid.performUnifyingNudgingPreprocessingStep).toBe(4);
      expect(globalThis.Avoid.improveHyperedgeRoutesMovingAddingAndDeletingJunctions).toBe(5);
      expect(globalThis.Avoid.nudgeSharedPathsWithCommonEndPoint).toBe(6);
    });
  });

  describe('Set and Get Parameters', () => {
    it('sets and gets segment penalty', () => {
      router.setRoutingParameter(globalThis.Avoid.segmentPenalty, 5.0);
      expect(router.routingParameter(globalThis.Avoid.segmentPenalty)).toBe(5.0);
    });

    it('sets and gets crossing penalty', () => {
      router.setRoutingParameter(globalThis.Avoid.crossingPenalty, 100.0);
      expect(router.routingParameter(globalThis.Avoid.crossingPenalty)).toBe(100.0);
    });

    it('sets and gets shape buffer distance', () => {
      router.setRoutingParameter(globalThis.Avoid.shapeBufferDistance, 10.0);
      expect(router.routingParameter(globalThis.Avoid.shapeBufferDistance)).toBe(10.0);
    });

    it('sets and gets ideal nudging distance', () => {
      router.setRoutingParameter(globalThis.Avoid.idealNudgingDistance, 4.0);
      expect(router.routingParameter(globalThis.Avoid.idealNudgingDistance)).toBe(4.0);
    });
  });

  describe('Set and Get Options', () => {
    it('sets and gets nudge orthogonal option', () => {
      router.setRoutingOption(globalThis.Avoid.nudgeOrthogonalSegmentsConnectedToShapes, true);
      expect(router.routingOption(globalThis.Avoid.nudgeOrthogonalSegmentsConnectedToShapes)).toBe(true);

      router.setRoutingOption(globalThis.Avoid.nudgeOrthogonalSegmentsConnectedToShapes, false);
      expect(router.routingOption(globalThis.Avoid.nudgeOrthogonalSegmentsConnectedToShapes)).toBe(false);
    });

    it('sets and gets improve hyperedge routes option', () => {
      router.setRoutingOption(globalThis.Avoid.improveHyperedgeRoutesMovingJunctions, true);
      expect(router.routingOption(globalThis.Avoid.improveHyperedgeRoutesMovingJunctions)).toBe(true);
    });
  });

  describe('Router Flags', () => {
    it('creates polyline router', () => {
      const polyRouter = new globalThis.Avoid.Router(globalThis.Avoid.PolyLineRouting);
      expect(polyRouter).toBeDefined();
    });

    it('creates orthogonal router', () => {
      const orthoRouter = new globalThis.Avoid.Router(globalThis.Avoid.OrthogonalRouting);
      expect(orthoRouter).toBeDefined();
    });

    it('has correct routing flag values', () => {
      expect(globalThis.Avoid.PolyLineRouting).toBe(1);
      expect(globalThis.Avoid.OrthogonalRouting).toBe(2);
    });
  });

  describe('Connection Direction Constants', () => {
    it('has correct direction constant values', () => {
      expect(globalThis.Avoid.ConnDirNone).toBe(0);
      expect(globalThis.Avoid.ConnDirUp).toBe(1);
      expect(globalThis.Avoid.ConnDirDown).toBe(2);
      expect(globalThis.Avoid.ConnDirLeft).toBe(4);
      expect(globalThis.Avoid.ConnDirRight).toBe(8);
      expect(globalThis.Avoid.ConnDirAll).toBe(15);
    });

    it('direction constants can be combined', () => {
      const upDown = globalThis.Avoid.ConnDirUp | globalThis.Avoid.ConnDirDown;
      expect(upDown).toBe(3);

      const leftRight = globalThis.Avoid.ConnDirLeft | globalThis.Avoid.ConnDirRight;
      expect(leftRight).toBe(12);

      const all = globalThis.Avoid.ConnDirUp | globalThis.Avoid.ConnDirDown |
                  globalThis.Avoid.ConnDirLeft | globalThis.Avoid.ConnDirRight;
      expect(all).toBe(globalThis.Avoid.ConnDirAll);
    });
  });

  describe('Connection Type Constants', () => {
    it('has correct connection type values', () => {
      expect(globalThis.Avoid.ConnType_None).toBe(0);
      expect(globalThis.Avoid.ConnType_PolyLine).toBe(1);
      expect(globalThis.Avoid.ConnType_Orthogonal).toBe(2);
    });
  });
});
