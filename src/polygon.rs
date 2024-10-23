use iced::widget::canvas::{Fill, Frame, Path, Stroke, Text};
use iced::{color, Color};
use palette::{Darken, Srgba};

use crate::Point;

/// Static slice of pastelish colors for drawing polygons
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

/// Darkens a given iced::Color by a percentage using the palette crate
fn darken(color: Color, factor: f32) -> Color {
    let srgba: Srgba = color.into();
    let darkened = srgba.darken(factor);
    Color::from(darkened)
}

/// Represents a convex polygon obstacle on the board.
/// Vertices are stored in clockwise or counter-clockwise order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Polygon {
    /// The vertices that make up the polygon, stored in order
    vertices: Vec<Point>,
}

/// Represents the orientation of three points in 2D space
#[derive(Debug, PartialEq, Eq)]
enum Orientation {
    Collinear,
    Clockwise,
    Counterclockwise,
}

// Helper function to determine orientation of three points
fn orientation(p: &Point, q: &Point, r: &Point) -> Orientation {
    let val = (q.y - p.y) * (r.x - q.x) - (q.x - p.x) * (r.y - q.y);

    match val.cmp(&0) {
        std::cmp::Ordering::Equal => Orientation::Collinear,
        std::cmp::Ordering::Greater => Orientation::Clockwise,
        std::cmp::Ordering::Less => Orientation::Counterclockwise,
    }
}

impl Polygon {
    /// Creates a new polygon from a vector of points
    pub fn new(vertices: Vec<Point>) -> Self {
        Self { vertices }
    }

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

    /// Returns an iterator over the vertices of the polygon
    pub fn vertices(&self) -> impl Iterator<Item = &Point> {
        self.vertices.iter()
    }

    /// Returns all vertices as a vector of points
    pub fn vertices_vec(&self) -> Vec<Point> {
        self.vertices.clone()
    }

    /// Returns the outer edges of the polygon as directed edges
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

    /// Determines if a line segment intersects with the polygon
    pub fn intersects_segment(&self, start: &Point, end: &Point) -> bool {
        let n = self.vertices.len();

        // If either point is inside the polygon (and not a vertex), it intersects
        if !self.vertices.contains(start) && self.contains_point(start) {
            return true;
        }
        if !self.vertices.contains(end) && self.contains_point(end) {
            return true;
        }

        // Check each edge for intersection
        for i in 0..n {
            let j = (i + 1) % n;
            let edge_start = &self.vertices[i];
            let edge_end = &self.vertices[j];

            // Skip if the segment starts or ends at this edge's endpoints
            if start == edge_start || start == edge_end || end == edge_start || end == edge_end {
                continue;
            }

            // Check for actual intersection
            let o1 = orientation(edge_start, edge_end, start);
            let o2 = orientation(edge_start, edge_end, end);
            let o3 = orientation(start, end, edge_start);
            let o4 = orientation(start, end, edge_end);

            if o1 != o2 && o3 != o4 {
                return true;
            }
        }

        // Check midpoint
        let mid = Point::new((start.x + end.x) / 2, (start.y + end.y) / 2);
        if !self.vertices.contains(&mid) && self.contains_point(&mid) {
            return true;
        }

        false
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

    /// Draws the polygon on the canvas
    pub fn draw(&self, index: usize, frame: &mut Frame) {
        // Get the current fill color and calculate the stroke color
        let fill_color = COLORS[index % COLORS.len()];
        let stroke_color = darken(fill_color, 0.5); // Darken the fill color by 50%

        let path = Path::new(|p| {
            for (i, vertex) in self.vertices.iter().enumerate() {
                if i == 0 {
                    p.move_to((vertex.x as f32, vertex.y as f32).into());
                } else {
                    p.line_to((vertex.x as f32, vertex.y as f32).into());
                }
            }
            p.close();
        });

        frame.fill(&path, Fill::from(fill_color));
        frame.stroke(&path, Stroke::default().with_color(stroke_color));
        frame.fill_text(Text {
            content: format!("{}", index + 1),
            position: (self.center().x as f32, self.center().y as f32).into(),
            color: Color::BLACK,
            size: 5.0.into(),
            ..Text::default()
        })
    }
}

/// Represents a directed edge between two points
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
    pub fn intersects_edge(&self, other: &Edge) -> bool {
        // Skip if edges share an endpoint
        if self.start == other.start
            || self.start == other.end
            || self.end == other.start
            || self.end == other.end
        {
            return false;
        }

        let o1 = orientation(&self.start, &self.end, &other.start);
        let o2 = orientation(&self.start, &self.end, &other.end);
        let o3 = orientation(&other.start, &other.end, &self.start);
        let o4 = orientation(&other.start, &other.end, &self.end);

        o1 != o2 && o3 != o4
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

    #[test]
    fn test_polygon_intersection() {
        let polygon = Polygon::new(vec![
            Point::new(0, 0),
            Point::new(100, 0),
            Point::new(100, 100),
            Point::new(0, 100),
        ]);

        // Test cases that should NOT intersect
        assert!(!polygon.intersects_segment(
            &Point::new(0, 0),     // vertex
            &Point::new(-50, -50)  // outside
        ));
        assert!(!polygon.intersects_segment(
            &Point::new(0, 0),   // vertex
            &Point::new(100, 0)  // another vertex
        ));

        // Test cases that SHOULD intersect
        assert!(polygon.intersects_segment(
            &Point::new(-50, 50), // outside
            &Point::new(50, 50)   // inside
        ));
        assert!(polygon.intersects_segment(
            &Point::new(-50, -50), // outside
            &Point::new(150, 150)  // outside, but crosses
        ));
    }
}
