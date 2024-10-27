use iced::widget::canvas::{Fill, Frame, LineDash, Path, Stroke, Text};
use iced::Color;
use num_traits::{AsPrimitive, Signed};
use std::collections::{HashMap, HashSet};

use crate::{Board, Point};

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Heuristic {
    #[default]
    Euclidean,
    Manhattan,
}

impl std::fmt::Display for Heuristic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Heuristic::Euclidean => write!(f, "Euclidean"),
            Heuristic::Manhattan => write!(f, "Manhattan"),
        }
    }
}

impl Heuristic {
    pub const ALL: &'static [Heuristic] = &[Heuristic::Euclidean, Heuristic::Manhattan];

    pub fn distance<T>(self, p1: &Point<T>, p2: &Point<T>) -> T
    where
        T: Copy
            + Default
            + Signed
            + std::ops::Sub<Output = T>
            + std::ops::Add<Output = T>
            + std::ops::Mul<Output = T>
            + AsPrimitive<f64>,
        f64: AsPrimitive<T>,
    {
        match self {
            Heuristic::Manhattan => {
                let dx = num_traits::abs(p2.x - p1.x);
                let dy = num_traits::abs(p2.y - p1.y);
                dx + dy
            }
            Heuristic::Euclidean => {
                let dx = p2.x - p1.x;
                let dy = p2.y - p1.y;
                let squared = dx * dx + dy * dy;
                let float_result = squared.as_();
                (float_result.sqrt()).as_()
            }
        }
    }
}

#[derive(Clone)]
pub struct SearchState {
    pub open: HashSet<Point>,
    pub closed: HashSet<Point>,
    pub current_paths: HashMap<Point, Vec<Point>>,
    pub best_path: Option<Vec<Point>>,
    pub considered_edges: HashSet<(Point, Point)>,
    pub next_vertex: Option<Point>,
    pub g_scores: HashMap<Point, i32>,
    pub came_from: HashMap<Point, Point>,
}

/// Common interface for pathfinding algorithms
pub trait Pathfinder {
    /// Required methods that implementations must provide
    fn get_board(&self) -> &Board;
    fn get_state(&self) -> &SearchState;
    fn get_start(&self) -> Point;
    fn get_goal(&self) -> Point;
    fn get_heuristic(&self) -> Heuristic;

    /// Initialize a new pathfinder
    fn new(board: Board, start: Point, goal: Point, heuristic: Heuristic) -> Self
    where
        Self: Sized;

    /// Get optimal path and cost if found
    fn get_optimal_path(&self) -> Option<&(Vec<Point>, i32)>;

    /// Total steps in visualization
    fn total_steps(&self) -> usize;

    /// Current step in visualization
    fn current_step(&self) -> usize;

    /// Basic stepping operations
    fn step_forward(&mut self) -> bool;
    fn step_back(&mut self) -> bool;
    fn jump_to(&mut self, step: usize) -> bool;
    fn reset(&mut self);
    fn change_heuristic(&mut self, heuristic: Heuristic);

    /// Default implementation for checking if finished
    fn is_finished(&self) -> bool {
        self.current_step() >= self.total_steps()
    }

    /// Default implementation for path reconstruction
    fn reconstruct_path(&self, vertex: &Point) -> Vec<Point> {
        let mut path = vec![*vertex];
        let mut current = *vertex;

        while let Some(&prev) = self.get_state().came_from.get(&current) {
            path.push(prev);
            current = prev;
        }

        path.reverse();
        path
    }

    /// Default implementation for best path score
    fn best_path_score(&self) -> Option<i32> {
        self.get_state().best_path.as_ref().map(|path| {
            path.windows(2)
                .map(|window| Self::distance(&window[0], &window[1]))
                .sum()
        })
    }

    /// Default implementation for optimal path score
    fn optimal_path_score(&self) -> Option<i32> {
        self.get_optimal_path().map(|(_, score)| *score)
    }

    /// Default implementation for Euclidean distance
    fn distance(p1: &Point, p2: &Point) -> i32 {
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        ((dx * dx + dy * dy) as f64).sqrt() as i32
    }

    /// Default implementation for drawing current state
    fn draw(&self, frame: &mut Frame, show_solution: bool) {
        // First draw the board
        self.get_board().draw(frame);

        // Draw historical considered edges
        let historical_stroke = Stroke::default()
            .with_color(Color::from_rgba8(128, 128, 128, 0.3))
            .with_width(1.0);

        for (from, to) in &self.get_state().considered_edges {
            let path = Path::line(
                (from.x as f32, -from.y as f32).into(),
                (to.x as f32, -to.y as f32).into(),
            );
            frame.stroke(&path, historical_stroke);
        }

        // Draw current active paths
        let current_stroke = Stroke::default()
            .with_color(Color::from_rgba8(0, 100, 255, 0.5))
            .with_width(2.0);

        // Find path closest to goal
        let mut best_current_path = None;
        let mut best_distance_to_goal = i32::MAX;

        for (target, path) in &self.get_state().current_paths {
            if path.len() > 1 {
                let distance_to_goal = Self::distance(target, &self.get_goal());

                if distance_to_goal < best_distance_to_goal {
                    best_distance_to_goal = distance_to_goal;
                    best_current_path = Some(path.clone());
                }

                for window in path.windows(2) {
                    let from = window[0];
                    let to = window[1];
                    let path = Path::line(
                        (from.x as f32, -from.y as f32).into(),
                        (to.x as f32, -to.y as f32).into(),
                    );
                    frame.stroke(&path, current_stroke);
                }
            }
        }

        // Draw best current path
        if let Some(path) = best_current_path {
            let best_stroke = Stroke::default()
                .with_color(Color::from_rgb8(50, 205, 50))
                .with_width(3.0);

            for window in path.windows(2) {
                let from = window[0];
                let to = window[1];
                let path = Path::line(
                    (from.x as f32, -from.y as f32).into(),
                    (to.x as f32, -to.y as f32).into(),
                );
                frame.stroke(&path, best_stroke);
            }

            if let Some(last) = path.last() {
                let current_path_score: i32 = path
                    .windows(2)
                    .map(|window| Self::distance(&window[0], &window[1]))
                    .sum();

                let content = match best_distance_to_goal {
                    0 => format!("Goal: {current_path_score}"),
                    _ => format!(
                        "Current best: {current_path_score}\nTo goal: {best_distance_to_goal}"
                    ),
                };
                frame.fill_text(Text {
                    content,
                    position: (last.x as f32 + 2.5, -last.y as f32 + 2.5).into(),
                    color: Color::BLACK,
                    size: 4.0.into(),
                    ..Text::default()
                });
            }
        }

        // Draw optimal solution if requested
        if show_solution {
            if let Some((path, score)) = self.get_optimal_path() {
                let solution_stroke = Stroke {
                    line_dash: LineDash {
                        segments: &[5.0, 5.0],
                        offset: 2,
                    },
                    ..Default::default()
                }
                .with_color(Color::from_rgb8(50, 205, 50))
                .with_width(3.0);

                for window in path.windows(2) {
                    let from = window[0];
                    let to = window[1];
                    let path = Path::line(
                        (from.x as f32, -from.y as f32).into(),
                        (to.x as f32, -to.y as f32).into(),
                    );
                    frame.stroke(&path, solution_stroke);
                }

                if let Some(last) = path.last() {
                    frame.fill_text(Text {
                        content: format!("Optimal: {}", score),
                        position: (last.x as f32 + 5.0, -last.y as f32 - 5.0).into(),
                        color: Color::BLACK,
                        size: 4.0.into(),
                        ..Text::default()
                    });
                }
            }
        }

        // Draw vertices
        for vertex in &self.get_state().open {
            let circle = Path::circle((vertex.x as f32, -vertex.y as f32).into(), 1.0);
            frame.fill(&circle, Fill::from(Color::from_rgb8(0, 100, 255)));
        }

        for vertex in &self.get_state().closed {
            let circle = Path::circle((vertex.x as f32, -vertex.y as f32).into(), 1.0);
            frame.fill(&circle, Fill::from(Color::from_rgb8(255, 100, 100)));
        }

        if let Some(next) = self.get_state().next_vertex {
            let circle = Path::circle((next.x as f32, -next.y as f32).into(), 1.5);
            frame.fill(&circle, Fill::from(Color::from_rgb8(50, 205, 50)));
        }

        // Draw start and goal
        let start = self.get_start();
        let goal = self.get_goal();

        let start_circle = Path::circle((start.x as f32, -start.y as f32).into(), 2.0);
        frame.fill(&start_circle, Fill::from(Color::from_rgb8(0, 0, 255)));
        frame.fill_text(Text {
            content: format!("({}, {})", start.x, start.y),
            position: (start.x as f32, -start.y as f32 - 6.5).into(),
            color: Color::BLACK,
            size: 4.0.into(),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            ..Text::default()
        });

        let goal_circle = Path::circle((goal.x as f32, -goal.y as f32).into(), 2.0);
        frame.fill(&goal_circle, Fill::from(Color::from_rgb8(255, 0, 0)));
        frame.fill_text(Text {
            content: format!("({}, {})", goal.x, goal.y),
            position: (goal.x as f32 - 2.5, -goal.y as f32 - 6.5).into(),
            color: Color::BLACK,
            size: 4.0.into(),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            ..Text::default()
        });
    }
}
