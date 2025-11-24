//! Tests for VPSC weight stratification (Task #2)

use libavoid::channel::{SegmentType, ShiftSegment};
use libavoid::{Point, Polygon};

#[test]
fn test_segment_type_classification() {
    // Test that SegmentType enum has all variants
    let _fixed = SegmentType::Fixed;
    let _final = SegmentType::Final;
    let _cbend = SegmentType::CBend;
    let _zigzag = SegmentType::ZigZag;
    let _regular = SegmentType::Regular;
}

#[test]
fn test_shift_segment_has_connected_to_shape_field() {
    // Verify ShiftSegment has the connected_to_shape field (Task #10)
    let seg = ShiftSegment::new(0, 0, 1, 0, 50.0, 0.0, 100.0);
    assert_eq!(seg.connected_to_shape, false); // Default should be false
}

#[test]
fn test_shift_segment_classification_types() {
    // Create a simple L-shaped route
    let mut route = Polygon::new();
    route.ps.push(Point::new(0.0, 0.0));
    route.ps.push(Point::new(100.0, 0.0)); // Horizontal segment
    route.ps.push(Point::new(100.0, 100.0)); // Vertical segment

    // Create segments
    let mut seg1 = ShiftSegment::new(0, 0, 1, 0, 0.0, -50.0, 50.0);
    let mut seg2 = ShiftSegment::new(0, 1, 2, 1, 100.0, 50.0, 150.0);

    // Classify them
    seg1.classify_segment(&route);
    seg2.classify_segment(&route);

    // First segment (at start) should be Final
    assert_eq!(seg1.segment_type, SegmentType::Final);

    // Last segment (at end) should be Final
    assert_eq!(seg2.segment_type, SegmentType::Final);
}

#[test]
fn test_fixed_segment_type() {
    // Fixed segments should have SegmentType::Fixed
    let seg = ShiftSegment::fixed(0, 0, 1, 0, 50.0);

    assert_eq!(seg.fixed, true);
    assert_eq!(seg.segment_type, SegmentType::Fixed);
    assert_eq!(seg.min_limit, seg.max_limit); // Fixed segments have no movement range
}

#[test]
fn test_single_segment_route_flag() {
    // Test the is_single_segment_route flag
    let mut seg = ShiftSegment::new(0, 0, 1, 0, 50.0, 0.0, 100.0);

    // Default should be false
    assert_eq!(seg.is_single_segment_route, false);

    // Can be set
    seg.is_single_segment_route = true;
    assert_eq!(seg.is_single_segment_route, true);
}
