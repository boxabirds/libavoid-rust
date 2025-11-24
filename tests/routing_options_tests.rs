//! Tests for routing options functionality

use libavoid::{Router, Point, ConnEnd, RoutingOption, Rectangle, Polygon, PolygonInterface};

fn rect(x: f64, y: f64, w: f64, h: f64) -> Polygon {
    Rectangle::new(Point::new(x, y), w, h).into()
}

#[test]
fn test_nudge_orthogonal_segments_connected_to_shapes_default() {
    // Test that by default (option = false), endpoint segments are not nudged
    let mut router = Router::new(0);

    // Verify default is false
    assert_eq!(router.routing_option(RoutingOption::NudgeOrthogonalSegmentsConnectedToShapes), false);
}

#[test]
fn test_nudge_orthogonal_segments_connected_to_shapes_set() {
    // Test that we can enable the option
    let mut router = Router::new(0);

    router.set_routing_option(RoutingOption::NudgeOrthogonalSegmentsConnectedToShapes, true);
    assert_eq!(router.routing_option(RoutingOption::NudgeOrthogonalSegmentsConnectedToShapes), true);

    router.set_routing_option(RoutingOption::NudgeOrthogonalSegmentsConnectedToShapes, false);
    assert_eq!(router.routing_option(RoutingOption::NudgeOrthogonalSegmentsConnectedToShapes), false);
}

#[test]
fn test_all_routing_options_accessible() {
    // Verify all 9 routing options can be get/set
    let mut router = Router::new(0);

    // Test each option
    let options = vec![
        RoutingOption::NudgeOrthogonalRoutes,
        RoutingOption::ImproveHyperedgeRoutes,
        RoutingOption::PenalisePortDirections,
        RoutingOption::NudgeSharedPathsWithCommonEndPoint,
        RoutingOption::NudgeOrthogonalSegmentsConnectedToShapes,
        RoutingOption::PenaliseOrthogonalSharedPathsAtConnEnds,
        RoutingOption::NudgeOrthogonalTouchingColinearSegments,
        RoutingOption::PerformUnifyingNudgingPreprocessingStep,
        RoutingOption::ImproveHyperedgeRoutesMovingAddingAndDeletingJunctions,
    ];

    for option in options {
        // Get current value
        let current = router.routing_option(option);

        // Toggle it
        router.set_routing_option(option, !current);
        assert_eq!(router.routing_option(option), !current);

        // Toggle back
        router.set_routing_option(option, current);
        assert_eq!(router.routing_option(option), current);
    }
}

#[test]
fn test_routing_options_independent() {
    // Verify changing one option doesn't affect others
    let mut router = Router::new(0);

    // Get initial states
    let initial_nudge = router.routing_option(RoutingOption::NudgeOrthogonalRoutes);
    let initial_hyperedge = router.routing_option(RoutingOption::ImproveHyperedgeRoutes);
    let initial_penalise = router.routing_option(RoutingOption::PenalisePortDirections);

    // Change one option
    router.set_routing_option(RoutingOption::NudgeOrthogonalRoutes, !initial_nudge);

    // Others should be unchanged
    assert_eq!(router.routing_option(RoutingOption::ImproveHyperedgeRoutes), initial_hyperedge);
    assert_eq!(router.routing_option(RoutingOption::PenalisePortDirections), initial_penalise);
}
