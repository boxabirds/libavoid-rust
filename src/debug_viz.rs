//! Debug visualization helpers (Task #22)
//!
//! Provides SVG export functionality for debugging routing:
//! - Visibility graphs
//! - Routes and paths
//! - Channel information

use crate::geometry::{Point, Polygon, PolygonInterface};
use crate::visibility::VisibilityGraph;
use crate::channel::ShiftSegment;

/// SVG visualization configuration
pub struct SvgConfig {
    pub width: f64,
    pub height: f64,
    pub padding: f64,
    pub scale: f64,
}

impl Default for SvgConfig {
    fn default() -> Self {
        SvgConfig {
            width: 800.0,
            height: 600.0,
            padding: 50.0,
            scale: 1.0,
        }
    }
}

/// Export visibility graph to SVG format
pub fn export_visibility_graph_svg(graph: &VisibilityGraph, config: &SvgConfig) -> String {
    let mut svg = String::new();

    // SVG header
    svg.push_str(&format!(
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
"#,
        config.width, config.height
    ));

    // Background
    svg.push_str(&format!(
        r#"  <rect width="100%" height="100%" fill="white"/>
"#
    ));

    // Draw edges first (so they appear behind vertices)
    svg.push_str(r#"  <g id="edges" stroke="blue" stroke-width="1" opacity="0.3">
"#);

    for vertex in graph.vertices() {
        for edge in vertex.edges.iter().chain(vertex.orthogonal_edges.iter()) {
            if let Some(target) = graph.get_vertex(edge.target_id) {
                let (x1, y1) = transform_point(&vertex.point, config);
                let (x2, y2) = transform_point(&target.point, config);

                let color = if edge.orthogonal { "green" } else { "blue" };
                svg.push_str(&format!(
                    r#"    <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="{}"/>
"#,
                    x1, y1, x2, y2, color
                ));
            }
        }
    }
    svg.push_str("  </g>\n");

    // Draw vertices
    svg.push_str(r#"  <g id="vertices" fill="red">
"#);

    for vertex in graph.vertices() {
        let (x, y) = transform_point(&vertex.point, config);
        svg.push_str(&format!(
            r#"    <circle cx="{}" cy="{}" r="3"/>
"#,
            x, y
        ));
    }
    svg.push_str("  </g>\n");

    svg.push_str("</svg>\n");
    svg
}

/// Export routes to SVG format
pub fn export_routes_svg(routes: &[Polygon], config: &SvgConfig) -> String {
    let mut svg = String::new();

    // SVG header
    svg.push_str(&format!(
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
"#,
        config.width, config.height
    ));

    // Background
    svg.push_str(r#"  <rect width="100%" height="100%" fill="white"/>
"#);

    // Draw each route with different color
    let colors = ["red", "blue", "green", "purple", "orange", "brown"];

    for (idx, route) in routes.iter().enumerate() {
        let color = colors[idx % colors.len()];
        svg.push_str(&format!(
            r#"  <g id="route_{}" stroke="{}" stroke-width="2" fill="none">
"#,
            idx, color
        ));

        if route.size() > 1 {
            svg.push_str("    <path d=\"");

            for i in 0..route.size() {
                let point = route.at(i);
                let (x, y) = transform_point(&point, config);

                if i == 0 {
                    svg.push_str(&format!("M {} {} ", x, y));
                } else {
                    svg.push_str(&format!("L {} {} ", x, y));
                }
            }

            svg.push_str("\"/>\n");
        }

        // Draw vertices
        for i in 0..route.size() {
            let point = route.at(i);
            let (x, y) = transform_point(&point, config);
            svg.push_str(&format!(
                r#"    <circle cx="{}" cy="{}" r="4" fill="{}"/>
"#,
                x, y, color
            ));
        }

        svg.push_str("  </g>\n");
    }

    svg.push_str("</svg>\n");
    svg
}

/// Export channel information to SVG format
pub fn export_channel_info_svg(
    segments: &[ShiftSegment],
    dimension: usize,
    config: &SvgConfig,
) -> String {
    let mut svg = String::new();

    // SVG header
    svg.push_str(&format!(
        r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
"#,
        config.width, config.height
    ));

    // Background
    svg.push_str(r#"  <rect width="100%" height="100%" fill="white"/>
"#);

    // Draw segments with their limits
    svg.push_str(r#"  <g id="segments">
"#);

    for (idx, segment) in segments.iter().enumerate() {
        let color = if segment.fixed { "red" } else { "blue" };
        let opacity = if segment.fixed { "0.3" } else { "0.6" };

        // Note: position and limits would need to be calculated from route points
        // For now, use placeholder values since segment doesn't store position directly
        let pos = (idx as f64 * 50.0) + 50.0;
        let min_lim = segment.min_limit * config.scale + config.padding;
        let max_lim = segment.max_limit * config.scale + config.padding;

        // Draw segment as a rectangle showing its range and limits
        if dimension == 0 {
            // Horizontal segment
            let x1 = pos * config.scale + config.padding;
            let y1 = min_lim;
            let y2 = max_lim;

            svg.push_str(&format!(
                r#"    <rect x="{}" y="{}" width="3" height="{}" fill="{}" opacity="{}"/>
"#,
                x1 - 1.5, y1, y2 - y1, color, opacity
            ));
        } else {
            // Vertical segment
            let x1 = min_lim;
            let x2 = max_lim;
            let y1 = pos * config.scale + config.padding;

            svg.push_str(&format!(
                r#"    <rect x="{}" y="{}" width="{}" height="3" fill="{}" opacity="{}"/>
"#,
                x1, y1 - 1.5, x2 - x1, color, opacity
            ));
        }

        // Add label
        let label_x = if dimension == 0 {
            pos * config.scale + config.padding + 5.0
        } else {
            (min_lim + max_lim) / 2.0
        };

        let label_y = if dimension == 0 {
            (min_lim + max_lim) / 2.0
        } else {
            pos * config.scale + config.padding + 5.0
        };

        svg.push_str(&format!(
            r#"    <text x="{}" y="{}" font-size="10" fill="black">Seg {}</text>
"#,
            label_x, label_y, idx
        ));
    }

    svg.push_str("  </g>\n");
    svg.push_str("</svg>\n");
    svg
}

/// Transform a point from world coordinates to SVG coordinates
fn transform_point(point: &Point, config: &SvgConfig) -> (f64, f64) {
    let x = point.x * config.scale + config.padding;
    let y = point.y * config.scale + config.padding;
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::visibility::VisibilityGraph;
    use crate::geometry::Point;

    #[test]
    fn test_svg_export_basic() {
        let graph = VisibilityGraph::new();
        let config = SvgConfig::default();
        let svg = export_visibility_graph_svg(&graph, &config);

        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_routes_svg_export() {
        let mut route = Polygon::new();
        route.push(Point::new(0.0, 0.0));
        route.push(Point::new(100.0, 100.0));

        let config = SvgConfig::default();
        let svg = export_routes_svg(&[route], &config);

        assert!(svg.contains("<svg"));
        assert!(svg.contains("<path"));
    }
}
