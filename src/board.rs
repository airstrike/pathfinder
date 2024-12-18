use iced::widget::canvas::{Fill, Frame, Path, Stroke, Text};
use iced::Color;
use std::collections::HashSet;

use crate::{Edge, Point, Polygon};

/// Represents the game board containing polygonal obstacles
#[derive(Clone, Debug)]
pub struct Board {
    /// The collection of polygon obstacles
    polygons: Vec<Polygon>,
}

impl Default for Board {
    fn default() -> Self {
        sample_board()
    }
}

impl Board {
    /// Creates a new board with the given polygons, start point, and goal point
    pub fn new(polygons: Vec<Polygon>) -> Self {
        Self { polygons }
    }

    /// Returns an iterator over the polygons on the board
    pub fn polygons(&self) -> impl Iterator<Item = &Polygon> {
        self.polygons.iter()
    }

    /// Returns all vertices from all polygons
    pub fn vertices(&self) -> HashSet<Point<i32>> {
        let mut vertices = HashSet::new();
        for polygon in &self.polygons {
            vertices.extend(polygon.vertices_vec());
        }
        vertices
    }

    /// Returns all outer edges from all polygons
    pub fn outer_edges(&self) -> Vec<Edge> {
        self.polygons().flat_map(|p| p.outer_edges()).collect()
    }

    /// Draw the board on the given frame. Note that y-coordinates are flipped
    /// to match mathematical coordinates.
    pub fn draw(&self, frame: &mut Frame) {
        // Determine the bounds of the board by finding min/max coordinates of polygons
        let (min_x, min_y, max_x, max_y) = self.bounds();

        // Draw the white background
        let background = Path::rectangle(
            (min_x as f32, -max_y as f32).into(), // Flip y-coordinate
            (max_x as f32 - min_x as f32, (max_y - min_y) as f32).into(),
        );
        frame.fill(&background, Fill::from(Color::WHITE));

        // Draw the boundary square around the board
        let boundary = Path::rectangle(
            (min_x as f32, -max_y as f32).into(), // Flip y-coordinate
            (max_x as f32 - min_x as f32, (max_y - min_y) as f32).into(),
        );
        frame.stroke(
            &boundary,
            Stroke::default().with_color(Color::BLACK).with_width(2.0),
        );

        // Draw x-axis tick marks every 50 units
        let tick_stroke = Stroke::default().with_color(Color::BLACK).with_width(1.0);
        for x in (min_x..=max_x).step_by(50) {
            let min_tick = Path::line(
                (x as f32, -min_y as f32).into(),
                (x as f32, -(min_y as f32 + 2.5)).into(),
            );
            let max_tick = Path::line(
                (x as f32, -max_y as f32).into(),
                (x as f32, -(max_y as f32 - 2.5)).into(),
            );
            frame.stroke(&min_tick, tick_stroke);
            frame.stroke(&max_tick, tick_stroke);
            frame.fill_text(Text {
                content: x.to_string(),
                position: (x as f32, -(min_y as f32 - 2.5)).into(),
                color: Color::BLACK,
                size: 4.0.into(),
                horizontal_alignment: iced::alignment::Horizontal::Center,
                ..Text::default()
            });
        }

        // Draw y-axis tick marks every 50 units and flip y-coords throughout
        for y in (min_y..=max_y).step_by(50) {
            let min_tick = Path::line(
                (min_x as f32, -y as f32).into(),
                (min_x as f32 + 2.5, -y as f32).into(),
            );
            let max_tick = Path::line(
                (max_x as f32, -y as f32).into(),
                (max_x as f32 - 2.5, -y as f32).into(),
            );
            frame.stroke(&min_tick, tick_stroke);
            frame.stroke(&max_tick, tick_stroke);
            frame.fill_text(Text {
                content: y.to_string(),
                position: (min_x as f32 - 2.5, -y as f32 - 2.5).into(),
                color: Color::BLACK,
                size: 4.0.into(),
                horizontal_alignment: iced::alignment::Horizontal::Right,
                ..Text::default()
            });
        }

        for (i, polygon) in self.polygons().enumerate() {
            polygon.draw(i, frame);
        }
    }

    /// Finds the board's bounding box by getting the min/max x and y coords
    pub fn bounds(&self) -> (i32, i32, i32, i32) {
        let mut min_x = i32::MAX;
        let mut max_x = i32::MIN;
        let mut min_y = i32::MAX;
        let mut max_y = i32::MIN;

        for polygon in &self.polygons {
            for vertex in polygon.vertices() {
                min_x = min_x.min(vertex.x);
                max_x = max_x.max(vertex.x);
                min_y = min_y.min(vertex.y);
                max_y = max_y.max(vertex.y);
            }
        }

        // Round down/up to the nearest 100 to make it look nicer
        min_x = (min_x / 100) * 100;
        min_y = (min_y / 100) * 100;
        max_x = ((max_x + 99) / 100) * 100;
        max_y = ((max_y + 99) / 100) * 100;

        (min_x, min_y, max_x, max_y)
    }

    /// Returns the total number of vertices across all polygons
    pub fn vertex_count(&self) -> usize {
        self.polygons.iter().map(|p| p.vertices_vec().len()).sum()
    }

    /// Returns the number of vertices for each polygon
    pub fn vertices_per_polygon(&self) -> Vec<usize> {
        self.polygons
            .iter()
            .map(|p| p.vertices_vec().len())
            .collect()
    }
}

/// Create a sample board with some polygons
pub fn sample_board() -> Board {
    let polygons = vec![
        Polygon::new(vec![
            (220, 616).into(),
            (220, 666).into(),
            (251, 670).into(),
            (272, 647).into(),
        ]),
        Polygon::new(vec![
            (341, 655).into(),
            (359, 667).into(),
            (374, 651).into(),
            (366, 577).into(),
        ]),
        Polygon::new(vec![
            (311, 530).into(),
            (311, 559).into(),
            (339, 578).into(),
            (361, 560).into(),
            (361, 528).into(),
            (336, 516).into(),
        ]),
        Polygon::new(vec![
            (105, 628).into(),
            (151, 670).into(),
            (180, 629).into(),
            (156, 577).into(),
            (113, 587).into(),
        ]),
        Polygon::new(vec![
            (118, 517).into(),
            (245, 517).into(),
            (245, 577).into(),
            (118, 557).into(),
        ]),
        Polygon::new(vec![
            (280, 583).into(),
            (333, 583).into(),
            (333, 665).into(),
            (280, 665).into(),
        ]),
        Polygon::new(vec![
            (252, 594).into(),
            (290, 562).into(),
            (264, 538).into(),
        ]),
        Polygon::new(vec![
            (198, 635).into(),
            (217, 574).into(),
            (182, 574).into(),
        ]),
    ];

    Board::new(polygons)
}
