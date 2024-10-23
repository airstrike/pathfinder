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
#[derive(Debug, Clone)]
pub struct Polygon {
    /// The vertices that make up the polygon, stored in order
    vertices: Vec<Point>,
}

impl Polygon {
    /// Creates a new polygon from a vector of points
    pub fn new(vertices: Vec<Point>) -> Self {
        Self { vertices }
    }

    pub fn center(&self) -> Point {
        let n = self.vertices.len();
        let mut x = 0;
        let mut y = 0;

        for vertex in &self.vertices {
            x += vertex.x;
            y += vertex.y;
        }

        Point::new(x / n as i32, y / n as i32)
    }

    /// Returns an iterator over the vertices of the polygon
    pub fn vertices(&self) -> impl Iterator<Item = &Point> {
        self.vertices.iter()
    }

    /// Returns all vertices as a vector of points
    pub fn vertices_vec(&self) -> Vec<Point> {
        self.vertices.clone()
    }

    pub fn draw(&self, index: usize, frame: &mut Frame) {
        // Get the current color index, and wrap it within the bounds of the COLORS array

        // Get the current fill color and calculate the stroke color
        let fill_color = COLORS[index];
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

impl Polygon {
    /// Checks if a point lies on a line segment between two points
    fn point_on_segment(&self, point: &Point<i32>, start: &Point<i32>, end: &Point<i32>) -> bool {
        let cross =
            (point.y - start.y) * (end.x - start.x) - (point.x - start.x) * (end.y - start.y);

        if cross != 0 {
            return false;
        }

        if point.x >= start.x.min(end.x)
            && point.x <= start.x.max(end.x)
            && point.y >= start.y.min(end.y)
            && point.y <= start.y.max(end.y)
        {
            return true;
        }

        false
    }

    /// Determines if a line segment intersects with any edge of the polygon
    pub fn intersects_segment(&self, start: &Point<i32>, end: &Point<i32>) -> bool {
        let n = self.vertices.len();

        for i in 0..n {
            let next = (i + 1) % n;
            let curr_vertex = &self.vertices[i];
            let next_vertex = &self.vertices[next];

            // Check if either endpoint lies on the polygon edge
            if self.point_on_segment(start, curr_vertex, next_vertex)
                || self.point_on_segment(end, curr_vertex, next_vertex)
            {
                continue;
            }

            let o1 = orientation(curr_vertex, next_vertex, start);
            let o2 = orientation(curr_vertex, next_vertex, end);
            let o3 = orientation(start, end, curr_vertex);
            let o4 = orientation(start, end, next_vertex);

            // If orientations are different, segments intersect
            if o1 != o2 && o3 != o4 {
                return true;
            }
        }

        false
    }
}

/// Represents the orientation of three points in 2D space
#[derive(Debug, PartialEq, Eq)]
enum Orientation {
    Collinear,
    Clockwise,
    Counterclockwise,
}

// Helper function to determine orientation of three points
fn orientation(p: &Point<i32>, q: &Point<i32>, r: &Point<i32>) -> Orientation {
    let val = (q.y - p.y) * (r.x - q.x) - (q.x - p.x) * (r.y - q.y);

    match val.cmp(&0) {
        std::cmp::Ordering::Equal => Orientation::Collinear,
        std::cmp::Ordering::Greater => Orientation::Clockwise,
        std::cmp::Ordering::Less => Orientation::Counterclockwise,
    }
}
