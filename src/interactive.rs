use iced::widget::canvas::{Fill, Frame, Path, Stroke, Text};
use iced::Color;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::{Board, Point, Search};

/// Available heuristic functions for the A* search
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
    /// Returns a slice with all available heuristics
    pub const ALL: &'static [Heuristic] = &[Heuristic::Euclidean, Heuristic::Manhattan];

    /// Converts the heuristic enum into a function that can be used by the search
    pub fn to_function(&self) -> Box<dyn Fn(&Point, &Point) -> i32> {
        match self {
            Heuristic::Manhattan => {
                Box::new(|p1: &Point, p2: &Point| (p2.x - p1.x).abs() + (p2.y - p1.y).abs())
            }
            Heuristic::Euclidean => Box::new(|p1: &Point, p2: &Point| {
                let dx = p2.x - p1.x;
                let dy = p2.y - p1.y;
                ((dx * dx + dy * dy) as f64).sqrt() as i32
            }),
        }
    }
}

/// Represents the current state of the interactive search
#[derive(Debug)]
pub struct SearchState {
    /// All vertices currently in the open set
    pub open: HashSet<Point>,
    /// All vertices that have been processed
    pub closed: HashSet<Point>,
    /// The current best path from start to any point
    pub current_paths: HashMap<Point, Vec<Point>>,
    /// The current best known path to the goal (if any)
    pub best_path: Option<Vec<Point>>,
    /// All edges considered so far (for visualization)
    pub considered_edges: HashSet<(Point, Point)>,
    /// The next vertex to be processed
    pub next_vertex: Option<Point>,
}

/// Controls the interactive search process
pub struct InteractiveSearch {
    /// The underlying board and points
    board: Board,
    start: Point,
    goal: Point,
    /// The visibility graph (pre-computed)
    visibility_graph: HashMap<Point, HashSet<Point>>,
    /// Priority queue for A* search
    open_set: BinaryHeap<SearchNode>,
    /// Track g-scores for A* search
    g_scores: HashMap<Point, i32>,
    /// Track where each vertex came from
    came_from: HashMap<Point, Point>,
    /// Current state for visualization
    pub state: SearchState,
    /// Whether the search has completed
    pub completed: bool,
    /// The heuristic function to use
    heuristic: Box<dyn Fn(&Point, &Point) -> i32>,
    /// The optimal path (pre-computed)
    pub optimal_path: Option<(Vec<Point>, i32)>,
}

#[derive(Clone, Eq, PartialEq)]
struct SearchNode {
    vertex: Point,
    g_score: i32,
    f_score: i32,
}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f_score.cmp(&self.f_score)
    }
}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl InteractiveSearch {
    /// Creates a new interactive search with the given board, points, and heuristic
    pub fn new(board: Board, start: Point, goal: Point, heuristic: Heuristic) -> Self {
        // Get heuristic function
        let h = heuristic.to_function();

        // Pre-compute visibility graph and optimal path
        let search = Search::new(board.clone(), start, goal);
        let visibility_graph = search.build_visibility_graph();
        let optimal_path = search.find_shortest_path();

        // Initialize open set with start node
        let mut open_set = BinaryHeap::new();
        open_set.push(SearchNode {
            vertex: start,
            g_score: 0,
            f_score: h(&start, &goal),
        });

        // Initialize state tracking
        let mut g_scores = HashMap::new();
        g_scores.insert(start, 0);

        let mut open = HashSet::new();
        open.insert(start);

        let mut current_paths = HashMap::new();
        current_paths.insert(start, vec![start]);

        Self {
            board,
            start,
            goal,
            visibility_graph,
            open_set,
            g_scores,
            came_from: HashMap::new(),
            state: SearchState {
                open,
                closed: HashSet::new(),
                current_paths,
                best_path: None,
                considered_edges: HashSet::new(),
                next_vertex: Some(start),
            },
            completed: false,
            heuristic: h,
            optimal_path,
        }
    }

    /// Changes the heuristic function and resets the search
    pub fn change_heuristic(&mut self, heuristic: Heuristic) {
        self.heuristic = heuristic.to_function();
        self.reset();
    }

    /// Returns the score of the current best path
    pub fn best_path_score(&self) -> Option<i32> {
        self.state.best_path.as_ref().map(|path| {
            path.windows(2)
                .map(|window| Self::distance(&window[0], &window[1]))
                .sum()
        })
    }

    /// Returns the score of the optimal path
    pub fn optimal_path_score(&self) -> Option<i32> {
        self.optimal_path.as_ref().map(|(_, score)| *score)
    }

    /// Performs one step of the A* search algorithm
    pub fn step(&mut self) -> bool {
        if self.completed {
            return false;
        }

        if let Some(current) = self.open_set.pop() {
            // Update state for visualization
            self.state.open.remove(&current.vertex);
            self.state.closed.insert(current.vertex);
            self.state.next_vertex = None;

            // Check if we've reached the goal
            if current.vertex == self.goal {
                // Reconstruct the final path
                let final_path = self.reconstruct_path(&current.vertex);
                self.state.best_path = Some(final_path);
                self.completed = true;
                return true;
            }

            // Process neighbors
            if let Some(neighbors) = self.visibility_graph.get(&current.vertex) {
                for &neighbor in neighbors {
                    // Record this edge as considered
                    self.state
                        .considered_edges
                        .insert((current.vertex, neighbor));

                    let tentative_g_score =
                        current.g_score + Self::distance(&current.vertex, &neighbor);

                    if !self.g_scores.contains_key(&neighbor)
                        || tentative_g_score < *self.g_scores.get(&neighbor).unwrap()
                    {
                        // This path is better - record it
                        self.came_from.insert(neighbor, current.vertex);
                        self.g_scores.insert(neighbor, tentative_g_score);

                        // Update the current best path to this neighbor
                        let mut new_path = self.reconstruct_path(&current.vertex);
                        new_path.push(neighbor);
                        self.state.current_paths.insert(neighbor, new_path);

                        let next = SearchNode {
                            vertex: neighbor,
                            g_score: tentative_g_score,
                            f_score: tentative_g_score + (self.heuristic)(&neighbor, &self.goal),
                        };

                        self.open_set.push(next);
                        self.state.open.insert(neighbor);

                        // Update next vertex for visualization
                        if self.state.next_vertex.is_none() {
                            self.state.next_vertex = Some(neighbor);
                        }
                    }
                }
            }

            true
        } else {
            self.completed = true;
            false
        }
    }

    /// Reset the search to its initial state
    pub fn reset(&mut self) {
        let start = self.start;
        let goal = self.goal;

        self.open_set.clear();
        self.open_set.push(SearchNode {
            vertex: start,
            g_score: 0,
            f_score: (self.heuristic)(&start, &goal),
        });

        self.g_scores.clear();
        self.g_scores.insert(start, 0);

        self.came_from.clear();

        self.state = SearchState {
            open: HashSet::from([start]),
            closed: HashSet::new(),
            current_paths: HashMap::from([(start, vec![start])]),
            best_path: None,
            considered_edges: HashSet::new(),
            next_vertex: Some(start),
        };

        self.completed = false;
    }

    /// Euclidean distance between points (for actual distance calculation)
    fn distance(p1: &Point, p2: &Point) -> i32 {
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        ((dx * dx + dy * dy) as f64).sqrt() as i32
    }

    /// Reconstructs the path to a given vertex
    fn reconstruct_path(&self, vertex: &Point) -> Vec<Point> {
        let mut path = vec![*vertex];
        let mut current = *vertex;

        while let Some(&prev) = self.came_from.get(&current) {
            path.push(prev);
            current = prev;
        }

        path.reverse();
        path
    }

    /// Draws the current state of the search on the canvas
    pub fn draw(&self, frame: &mut Frame, show_solution: bool) {
        // First draw the board
        self.board.draw(frame);

        // Draw considered edges as thin gray lines
        let edge_stroke = Stroke::default()
            .with_color(Color::from_rgba8(128, 128, 128, 0.3))
            .with_width(1.0);

        for (from, to) in &self.state.considered_edges {
            let path = Path::line(
                (from.x as f32, from.y as f32).into(),
                (to.x as f32, to.y as f32).into(),
            );
            frame.stroke(&path, edge_stroke.clone());
        }

        // Draw current paths in progress as medium-thickness blue lines
        if !show_solution {
            let current_stroke = Stroke::default()
                .with_color(Color::from_rgba8(0, 100, 255, 0.5))
                .with_width(1.0);

            for path in self.state.current_paths.values() {
                if path.len() > 1 {
                    for window in path.windows(2) {
                        let from = window[0];
                        let to = window[1];
                        let path = Path::line(
                            (from.x as f32, from.y as f32).into(),
                            (to.x as f32, to.y as f32).into(),
                        );
                        frame.stroke(&path, current_stroke.clone());
                    }
                }
            }

            // Set the label position to a (5, 5) offset relative to the current best path's end
            let label_position = self
                .state
                .best_path
                .as_ref()
                .and_then(|path| path.last())
                .map(|p| (p.x as f32 + 5.0, p.y as f32 + 5.0));

            if let Some(position) = label_position {
                frame.fill_text(Text {
                    content: format!("Best path: {}", self.best_path_score().unwrap_or(0)),
                    position: position.into(),
                    color: Color::BLACK,
                    size: 4.0.into(),
                    ..Text::default()
                });
            }
        }

        // Draw optimal path if requested and available
        if show_solution {
            if let Some((path, _)) = &self.optimal_path {
                let solution_stroke = Stroke::default()
                    .with_color(Color::from_rgb8(50, 205, 50))
                    .with_width(3.0);

                for window in path.windows(2) {
                    let from = window[0];
                    let to = window[1];
                    let path = Path::line(
                        (from.x as f32, from.y as f32).into(),
                        (to.x as f32, to.y as f32).into(),
                    );
                    frame.stroke(&path, solution_stroke.clone());
                }
            }

            // Set the label position to a (5, 5) offset relative to the current best path's end
            let label_position = self
                .state
                .best_path
                .as_ref()
                .and_then(|path| path.last())
                .map(|p| (p.x as f32 + 5.0, p.y as f32 + 5.0));

            if let Some(position) = label_position {
                frame.fill_text(Text {
                    content: format!("Best path: {}", self.optimal_path_score().unwrap_or(0)),
                    position: position.into(),
                    color: Color::BLACK,
                    size: 4.0.into(),
                    ..Text::default()
                });
            }
        }

        // Draw best path found so far if we're not showing the solution
        if !show_solution {
            if let Some(path) = &self.state.best_path {
                let best_stroke = Stroke::default()
                    .with_color(Color::from_rgb8(50, 205, 50))
                    .with_width(3.0);

                for window in path.windows(2) {
                    let from = window[0];
                    let to = window[1];
                    let path = Path::line(
                        (from.x as f32, from.y as f32).into(),
                        (to.x as f32, to.y as f32).into(),
                    );
                    frame.stroke(&path, best_stroke.clone());
                }
            }
        }

        // Draw open set vertices as blue circles
        for vertex in &self.state.open {
            let circle = Path::circle((vertex.x as f32, vertex.y as f32).into(), 1.0);
            frame.fill(&circle, Fill::from(Color::from_rgb8(0, 100, 255)));
        }

        // Draw closed set vertices as red circles
        for vertex in &self.state.closed {
            let circle = Path::circle((vertex.x as f32, vertex.y as f32).into(), 1.0);
            frame.fill(&circle, Fill::from(Color::from_rgb8(255, 100, 100)));
        }

        // Draw next vertex (if any) as a larger green circle
        if let Some(next) = self.state.next_vertex {
            let circle = Path::circle((next.x as f32, next.y as f32).into(), 1.5);
            frame.fill(&circle, Fill::from(Color::from_rgb8(50, 205, 50)));
        }

        // Draw start point as a large blue circle
        let start_circle = Path::circle((self.start.x as f32, self.start.y as f32).into(), 2.0);
        frame.fill(&start_circle, Fill::from(Color::from_rgb8(0, 0, 255)));

        // Draw goal point as a large red circle
        let goal_circle = Path::circle((self.goal.x as f32, self.goal.y as f32).into(), 2.0);
        frame.fill(&goal_circle, Fill::from(Color::from_rgb8(255, 0, 0)));
    }
}
