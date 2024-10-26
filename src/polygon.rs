use iced::widget::canvas::{Fill, Frame, Path, Stroke, Text};
use iced::{color, Color};
use palette::{Darken, Srgba};

use crate::Point;

/// Static slice of pastelish colors for drawing polygons. Thanks, ChatGPT!
const COLORS: [Color; 16] = [
    color!(255, 179, 186), // Light Pink
    color!(255, 223, 186), // Peach
    color!(255, 255, 186), // Light Yellow
    color!(186, 255, 201), // Mint Green
    color!(186, 255, 255), // Light Cyan
    color!(186, 215, 255), // Light Blue
    color!(201, 186, 255), // Lavender
    color!(255, 186, 255), // Light Magenta
    color!(255, 186, 223), // Soft Rose
    color!(186, 199, 255), // Periwinkle
    color!(255, 219, 186), // Apricot
    color!(186, 242, 255), // Sky Blue
    color!(222, 255, 186), // Light Lime
    color!(255, 186, 219), // Blush
    color!(255, 242, 186), // Pale Gold
    color!(186, 255, 223), // Aqua Mint
];

/// Darkens a given [`Color`] by a percentage
fn darken(color: Color, factor: f32) -> Color {
    let srgba: Srgba = color.into();
    let darkened = srgba.darken(factor);
    Color::from(darkened)
}

/// Represents a convex [`Polygon`] obstacle on the board.
///
/// Vertices are stored in clockwise or counter-clockwise order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Polygon {
    /// The vertices that make up the [`Polygon`], stored in order
    vertices: Vec<Point>,
}

impl Polygon {
    /// Creates a new [`Polygon`] from a vector of [`Point`]s
    pub fn new(vertices: Vec<Point>) -> Self {
        Self { vertices }
    }

    /// Compute the center [`Point`] of the [`Polygon`] as the average of its
    /// vertices
    pub fn center(&self) -> Point {
        let n = self.vertices.len() as i32;
        let mut x = 0;
        let mut y = 0;

        for vertex in &self.vertices {
            x += vertex.x;
            y += vertex.y;
        }

        Point::new(x / n, y / n)
    }

    /// Returns an iterator over the vertices of the [`Polygon`]
    pub fn vertices(&self) -> impl Iterator<Item = &Point> {
        self.vertices.iter()
    }

    /// Returns all vertices as a vector of [`Point`]s
    pub fn vertices_vec(&self) -> Vec<Point> {
        self.vertices.clone()
    }

    /// Returns the outer [`Edge`]s of the [`Polygon`] as directed edges
    pub fn outer_edges(&self) -> Vec<Edge> {
        let vertices = &self.vertices;
        let n = vertices.len();
        let mut edges = Vec::with_capacity(n);

        for i in 0..n {
            let start = vertices[i];
            let end = vertices[(i + 1) % n];
            edges.push(Edge::new(start, end));
        }

        edges
    }

    /// Determine if a line segment intersects with the [`Polygon`]
    pub fn intersects_segment(&self, start: &Point, end: &Point) -> bool {
        let n = self.vertices.len();
        let test_edge = Edge::new(*start, *end);

        // First check if both points are vertices or the segment is along an edge
        let mut found_start = false;
        let mut found_end = false;
        for i in 0..n {
            let j = (i + 1) % n;
            let edge_start = &self.vertices[i];
            let edge_end = &self.vertices[j];
            let polygon_edge = Edge::new(*edge_start, *edge_end);

            // Check if points are vertices
            if !found_start {
                found_start = start == edge_start || start == edge_end;
            }
            if !found_end {
                found_end = end == edge_start || end == edge_end;
            }

            // If the test edge is collinear with a polygon edge and overlaps it,
            // we don't count it as an intersection
            if polygon_edge.contains_point(start) && polygon_edge.contains_point(end) {
                return false;
            }

            // Test for intersection with this edge
            if test_edge.intersects(&polygon_edge) {
                return true;
            }
        }

        // If either non-vertex point is inside the polygon, it intersects
        if !found_start && self.contains_point(start) {
            return true;
        }
        if !found_end && self.contains_point(end) {
            return true;
        }

        // Check midpoint
        let mid = Point::new((start.x + end.x) / 2, (start.y + end.y) / 2);
        !test_edge.contains_point(&mid) && self.contains_point(&mid)
    }

    /// Checks if a point lies inside the polygon using the ray casting algorithm
    fn contains_point(&self, point: &Point) -> bool {
        let mut inside = false;
        let mut j = self.vertices.len() - 1;

        for i in 0..self.vertices.len() {
            let vi = &self.vertices[i];
            let vj = &self.vertices[j];

            // Check if point is exactly on a vertex
            if point == vi || point == vj {
                return false; // Consider points on vertices as outside
            }

            if ((vi.y > point.y) != (vj.y > point.y))
                && (point.x < (vj.x - vi.x) * (point.y - vi.y) / (vj.y - vi.y) + vi.x)
            {
                inside = !inside;
            }

            j = i;
        }

        inside
    }

    /// Draw the [`Polygon`] on a canvas [`Frame`] at a given index
    pub fn draw(&self, index: usize, frame: &mut Frame) {
        let fill_color = COLORS[index % COLORS.len()];
        let stroke_color = darken(fill_color, 0.5);

        let path = Path::new(|p| {
            for (i, vertex) in self.vertices.iter().enumerate() {
                if i == 0 {
                    p.move_to((vertex.x as f32, -vertex.y as f32).into());
                } else {
                    p.line_to((vertex.x as f32, -vertex.y as f32).into());
                }
            }
            p.close();
        });

        frame.fill(&path, Fill::from(fill_color));
        frame.stroke(&path, Stroke::default().with_color(stroke_color));

        let center = self.center();
        frame.fill_text(Text {
            content: format!("{}", index + 1),
            position: (center.x as f32, -center.y as f32).into(),
            color: Color::BLACK,
            size: 5.0.into(),
            ..Text::default()
        });
    }
}

/// Represents a directed [`Edge`] between two [`Point`]s
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Edge {
    pub start: Point,
    pub end: Point,
}

impl Edge {
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }

    /// Returns true if this edge intersects with another edge,
    /// excluding edges that share an endpoint
    pub fn intersects(&self, other: &Edge) -> bool {
        // Skip if edges share an endpoint
        if self.start == other.start
            || self.start == other.end
            || self.end == other.start
            || self.end == other.end
        {
            return false;
        }

        // Calculate parameters for parametric equations
        // k1 = p1x - p2x  (our start.x - end.x)
        // k2 = q2y - q1y  (other.end.y - other.start.y)
        // k3 = p1y - p2y  (our start.y - end.y)
        // k4 = q2x - q1x  (other.end.x - other.start.x)
        // k5 = p1x - q1x  (our start.x - other.start.x)
        // k6 = p1y - q1y  (our start.y - other.start.y)
        let k1 = self.start.x - self.end.x;
        let k2 = other.end.y - other.start.y;
        let k3 = self.start.y - self.end.y;
        let k4 = other.end.x - other.start.x;
        let k5 = self.start.x - other.start.x;
        let k6 = self.start.y - other.start.y;

        let d = (k1 * k2) - (k3 * k4);

        // If d is 0, lines are parallel
        if d == 0 {
            // For parallel lines, check if they're collinear and overlapping
            // using our existing contains_point method
            return self.contains_point(&other.start)
                || self.contains_point(&other.end)
                || other.contains_point(&self.start)
                || other.contains_point(&self.end);
        }

        // Calculate intersection parameters
        let a = ((k2 * k5) - (k4 * k6)) as f64 / d as f64;
        let b = ((k1 * k6) - (k3 * k5)) as f64 / d as f64;

        // Lines intersect if both parameters are between 0 and 1
        (0.0..=1.0).contains(&a) && (0.0..=1.0).contains(&b)
    }

    /// Returns true if a point lies on this edge
    pub fn contains_point(&self, point: &Point) -> bool {
        // Check if point is collinear with edge endpoints
        let cross = (point.y - self.start.y) * (self.end.x - self.start.x)
            - (point.x - self.start.x) * (self.end.y - self.start.y);

        if cross != 0 {
            return false;
        }

        // Check if point lies within the bounding box of the edge
        point.x >= self.start.x.min(self.end.x)
            && point.x <= self.start.x.max(self.end.x)
            && point.y >= self.start.y.min(self.end.y)
            && point.y <= self.start.y.max(self.end.y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test fixtures
    fn create_triangle() -> Polygon {
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(100, 0),
            Point::new(50, 87), // Approximately equilateral
        ])
    }

    fn create_square() -> Polygon {
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(100, 0),
            Point::new(100, 100),
            Point::new(0, 100),
        ])
    }

    fn create_pentagon() -> Polygon {
        Polygon::new(vec![
            Point::new(50, 0),
            Point::new(97, 35),
            Point::new(80, 90),
            Point::new(20, 90),
            Point::new(3, 35),
        ])
    }

    fn create_hexagon() -> Polygon {
        Polygon::new(vec![
            Point::new(50, 0),
            Point::new(93, 25),
            Point::new(93, 75),
            Point::new(50, 100),
            Point::new(7, 75),
            Point::new(7, 25),
        ])
    }

    // Helper function to run a test against all polygon types
    fn test_all_polygons<F>(test_fn: F)
    where
        F: Fn(&Polygon),
    {
        let polygons = vec![
            create_triangle(),
            create_square(),
            create_pentagon(),
            create_hexagon(),
        ];

        for polygon in polygons {
            test_fn(&polygon);
        }
    }

    mod edge_tests {
        use super::*;

        #[test]
        fn test_edge_parallel() {
            let e1 = Edge::new(Point::new(0, 0), Point::new(10, 0));
            let e2 = Edge::new(Point::new(0, 5), Point::new(10, 5));
            assert!(!e1.intersects(&e2), "Parallel edges should not intersect");
        }

        #[test]
        fn test_edge_collinear() {
            let e1 = Edge::new(Point::new(0, 0), Point::new(10, 0));
            let e2 = Edge::new(Point::new(5, 0), Point::new(15, 0));
            assert!(
                e1.intersects(&e2),
                "Overlapping collinear edges should intersect"
            );

            let e3 = Edge::new(Point::new(0, 0), Point::new(5, 0));
            let e4 = Edge::new(Point::new(6, 0), Point::new(10, 0));
            assert!(
                !e3.intersects(&e4),
                "Non-overlapping collinear edges should not intersect"
            );
        }

        #[test]
        fn test_edge_intersection() {
            let e1 = Edge::new(Point::new(0, 0), Point::new(10, 10));
            let e2 = Edge::new(Point::new(0, 10), Point::new(10, 0));
            assert!(e1.intersects(&e2), "Crossing edges should intersect");

            let e3 = Edge::new(Point::new(0, 0), Point::new(10, 10));
            let e4 = Edge::new(Point::new(10, 10), Point::new(20, 20));
            assert!(
                !e3.intersects(&e4),
                "Edges sharing endpoint should not intersect"
            );
        }

        #[test]
        fn test_edge_contains_point() {
            let edge = Edge::new(Point::new(0, 0), Point::new(10, 10));

            assert!(
                edge.contains_point(&Point::new(5, 5)),
                "Point on edge should be contained"
            );
            assert!(
                edge.contains_point(&Point::new(0, 0)),
                "Start point should be contained"
            );
            assert!(
                edge.contains_point(&Point::new(10, 10)),
                "End point should be contained"
            );
            assert!(
                !edge.contains_point(&Point::new(5, 6)),
                "Point off edge should not be contained"
            );
        }
    }

    mod intersection_tests {
        use super::*;

        #[test]
        fn test_vertex_cases() {
            test_all_polygons(|polygon| {
                let vertices = polygon.vertices_vec();

                // Test vertex to vertex (edges)
                for i in 0..vertices.len() {
                    let j = (i + 1) % vertices.len();
                    assert!(
                        !polygon.intersects_segment(&vertices[i], &vertices[j]),
                        "Edge of polygon should not count as intersection"
                    );
                }

                // Test vertex to outside - extend directly outward from each
                // vertex, otherwise we may end up intersecting some other edge
                // of the polygon
                for i in 0..vertices.len() {
                    let vertex = vertices[i];
                    let prev = vertices[(i + vertices.len() - 1) % vertices.len()];
                    let next = vertices[(i + 1) % vertices.len()];

                    // Calculate bisector direction to ensure we go outward
                    let dx1 = vertex.x - prev.x;
                    let dy1 = vertex.y - prev.y;
                    let dx2 = vertex.x - next.x;
                    let dy2 = vertex.y - next.y;

                    // Average the directions and normalize (roughly)
                    let dx = (dx1 + dx2) / 2;
                    let dy = (dy1 + dy2) / 2;
                    let scale = 50; // Distance to extend outward

                    let outside_point = Point::new(
                        vertex.x + dx / dx.abs().max(1) * scale,
                        vertex.y + dy / dy.abs().max(1) * scale,
                    );

                    assert!(
                        !polygon.intersects_segment(&vertex, &outside_point),
                        "Line from vertex directly outward should not intersect"
                    );
                }
            });
        }

        #[test]
        fn test_crossing_cases() {
            test_all_polygons(|polygon| {
                let center = polygon.center();

                // Test lines through center
                assert!(
                    polygon.intersects_segment(
                        &Point::new(center.x - 100, center.y),
                        &Point::new(center.x + 100, center.y)
                    ),
                    "Horizontal line through center should intersect"
                );

                assert!(
                    polygon.intersects_segment(
                        &Point::new(center.x, center.y - 100),
                        &Point::new(center.x, center.y + 100)
                    ),
                    "Vertical line through center should intersect"
                );
            });
        }

        #[test]
        fn test_internal_cases() {
            test_all_polygons(|polygon| {
                let center = polygon.center();

                // Test point at center
                assert!(
                    polygon.contains_point(&center),
                    "Center point should be inside polygon"
                );

                // Test line between internal points
                let p1 = Point::new(center.x - 5, center.y - 5);
                let p2 = Point::new(center.x + 5, center.y + 5);
                assert!(
                    polygon.intersects_segment(&p1, &p2),
                    "Line between internal points should intersect"
                );
            });
        }

        #[test]
        fn test_exterior_cases() {
            test_all_polygons(|polygon| {
                let center = polygon.center();

                // Test completely external line
                let far_left = Point::new(center.x - 200, center.y);
                let far_right = Point::new(center.x + 200, center.y);
                assert!(
                    !polygon.intersects_segment(
                        &Point::new(far_left.x, far_left.y + 200),
                        &Point::new(far_right.x, far_right.y + 200)
                    ),
                    "Line completely outside should not intersect"
                );
            });
        }

        #[test]
        fn test_degenerate_cases() {
            test_all_polygons(|polygon| {
                let vertices = polygon.vertices_vec();
                let center = polygon.center();

                // Test zero-length segments
                assert!(
                    !polygon.intersects_segment(&vertices[0], &vertices[0]),
                    "Zero-length segment at vertex should not intersect"
                );

                assert!(
                    polygon.intersects_segment(&center, &center),
                    "Zero-length segment at center should intersect"
                );
            });
        }
    }

    mod geometry_tests {
        use super::*;

        #[test]
        fn test_center_calculation() {
            // For regular polygons, center should be predictable
            let square = create_square();
            assert_eq!(
                square.center(),
                Point::new(50, 50),
                "Square center should be at (50,50)"
            );

            let triangle = create_triangle();
            assert_eq!(
                triangle.center(),
                Point::new(50, 29), // Approximate due to integer division
                "Triangle center should be at (50,29)"
            );
        }

        #[test]
        fn test_edge_extraction() {
            test_all_polygons(|polygon| {
                let edges = polygon.outer_edges();
                let vertices = polygon.vertices_vec();

                assert_eq!(
                    edges.len(),
                    vertices.len(),
                    "Number of edges should equal number of vertices"
                );

                // Verify each edge connects consecutive vertices
                for (i, edge) in edges.iter().enumerate() {
                    assert_eq!(edge.start, vertices[i], "Edge should start at vertex");
                    assert_eq!(
                        edge.end,
                        vertices[(i + 1) % vertices.len()],
                        "Edge should end at next vertex"
                    );
                }
            });
        }
    }
}
