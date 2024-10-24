use crate::{Board, Point};
use iced::widget::canvas::{Fill, Frame, LineDash, Path, Stroke, Text};
use iced::Color;
use num_traits::{AsPrimitive, Signed};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

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

#[derive(Clone, Debug)]
struct State {
    open: HashSet<Point>,
    closed: HashSet<Point>,
    current_paths: HashMap<Point, Vec<Point>>,
    best_path: Option<Vec<Point>>,
    considered_edges: HashSet<(Point, Point)>,
    next_vertex: Option<Point>,
    g_scores: HashMap<Point, i32>,
    came_from: HashMap<Point, Point>,
}

#[derive(Clone)]
pub struct Search {
    board: Board,
    start: Point,
    goal: Point,
    heuristic: Heuristic,
    optimal_path: Option<(Vec<Point>, i32)>,
    state: State,
    current_step: usize,
    history: Vec<State>,
    visibility_graph: HashMap<Point, HashSet<Point>>,
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

impl Search {
    pub fn new(board: Board, start: Point, goal: Point, heuristic: Heuristic) -> Self {
        let mut search = Self {
            board,
            start,
            goal,
            heuristic,
            optimal_path: None,
            visibility_graph: HashMap::new(),
            state: State {
                open: HashSet::from([start]),
                closed: HashSet::new(),
                current_paths: HashMap::from([(start, vec![start])]),
                best_path: None,
                considered_edges: HashSet::new(),
                next_vertex: Some(start),
                g_scores: HashMap::from([(start, 0)]),
                came_from: HashMap::new(),
            },
            current_step: 0,
            history: Vec::new(),
        };

        // Build visibility graph for this board, compute solution and history
        search.visibility_graph = search.build_visibility_graph();
        search.compute_optimal_path();
        search.history.push(search.state.clone());
        search.reset(); // Return to step 0

        search
    }

    pub fn total_steps(&self) -> usize {
        self.history.len() - 1
    }

    pub fn is_finished(&self) -> bool {
        self.current_step >= self.total_steps()
    }

    pub fn current_step(&self) -> usize {
        self.current_step
    }

    pub fn step_forward(&mut self) -> bool {
        if self.is_finished() {
            return false;
        }
        self.current_step += 1;
        self.state = self.history[self.current_step].clone();
        true
    }

    pub fn step_back(&mut self) -> bool {
        if self.current_step == 0 {
            return false;
        }
        self.current_step -= 1;
        self.state = self.history[self.current_step].clone();
        true
    }

    pub fn jump_to(&mut self, step: usize) -> bool {
        if step > self.total_steps() {
            return false;
        }
        self.current_step = step;
        self.state = self.history[self.current_step].clone();
        true
    }

    pub fn reset(&mut self) {
        self.current_step = 0;
        self.state = self.history[0].clone();
    }

    pub fn change_heuristic(&mut self, heuristic: Heuristic) {
        self.heuristic = heuristic;
        self.reset();
        self.compute_optimal_path();
    }

    pub fn get_optimal_path(&self) -> Option<&(Vec<Point>, i32)> {
        self.optimal_path.as_ref()
    }

    pub fn best_path_score(&self) -> Option<i32> {
        self.state.best_path.as_ref().map(|path| {
            path.windows(2)
                .map(|window| Self::distance(&window[0], &window[1]))
                .sum()
        })
    }

    pub fn optimal_path_score(&self) -> Option<i32> {
        self.optimal_path.as_ref().map(|(_, score)| *score)
    }

    fn compute_optimal_path(&mut self) {
        self.history.clear();
        let mut open_set = BinaryHeap::new();

        open_set.push(SearchNode {
            vertex: self.start,
            g_score: 0,
            f_score: self.heuristic.distance(&self.start, &self.goal),
        });
        self.state.g_scores.insert(self.start, 0);

        while let Some(current) = open_set.pop() {
            // Found the goal
            if current.vertex == self.goal {
                let path = self.reconstruct_path(&current.vertex);
                self.optimal_path = Some((path.clone(), current.g_score));
                self.state.best_path = Some(path);
                return; // Don't push any state when we find the goal
            }

            // Save state for visualization before processing current node
            self.history.push(self.state.clone());

            self.state.closed.insert(current.vertex);

            if let Some(neighbors) = self.visibility_graph.get(&current.vertex) {
                for &neighbor in neighbors {
                    let tentative_g_score =
                        current.g_score + Self::distance(&current.vertex, &neighbor);

                    if !self.state.g_scores.contains_key(&neighbor)
                        || tentative_g_score < *self.state.g_scores.get(&neighbor).unwrap()
                    {
                        self.state.came_from.insert(neighbor, current.vertex);
                        self.state.g_scores.insert(neighbor, tentative_g_score);

                        let mut new_path = self.reconstruct_path(&current.vertex);
                        new_path.push(neighbor);
                        self.state.current_paths.insert(neighbor, new_path);

                        self.state
                            .considered_edges
                            .insert((current.vertex, neighbor));

                        open_set.push(SearchNode {
                            vertex: neighbor,
                            g_score: tentative_g_score,
                            f_score: tentative_g_score
                                + self.heuristic.distance(&neighbor, &self.goal),
                        });
                        self.state.open.insert(neighbor);
                    }
                }
            }
        }
    }

    fn reconstruct_path(&self, vertex: &Point) -> Vec<Point> {
        let mut path = vec![*vertex];
        let mut current = *vertex;

        while let Some(&prev) = self.state.came_from.get(&current) {
            path.push(prev);
            current = prev;
        }

        path.reverse();
        path
    }

    pub fn draw(&self, frame: &mut Frame, show_solution: bool) {
        // First draw the board
        self.board.draw(frame);

        // Draw historical considered edges as thin gray lines
        let historical_stroke = Stroke::default()
            .with_color(Color::from_rgba8(128, 128, 128, 0.3))
            .with_width(1.0);

        for (from, to) in &self.state.considered_edges {
            let path = Path::line(
                (from.x as f32, -from.y as f32).into(), // Flip y-coordinate
                (to.x as f32, -to.y as f32).into(),     // Flip y-coordinate
            );
            frame.stroke(&path, historical_stroke);
        }

        // Draw current active paths
        let current_stroke = Stroke::default()
            .with_color(Color::from_rgba8(0, 100, 255, 0.5))
            .with_width(2.0);

        // Find the path that gets closest to the goal
        let mut best_current_path = None;
        let mut best_distance_to_goal = i32::MAX;

        for (target, path) in &self.state.current_paths {
            if path.len() > 1 {
                let distance_to_goal = Self::distance(target, &self.goal);

                if distance_to_goal < best_distance_to_goal {
                    best_distance_to_goal = distance_to_goal;
                    best_current_path = Some(path.clone());
                }

                for window in path.windows(2) {
                    let from = window[0];
                    let to = window[1];
                    let path = Path::line(
                        (from.x as f32, -from.y as f32).into(), // Flip y-coordinate
                        (to.x as f32, -to.y as f32).into(),     // Flip y-coordinate
                    );
                    frame.stroke(&path, current_stroke);
                }
            }
        }

        // Draw the best current path in green
        if let Some(path) = best_current_path {
            let best_stroke = Stroke::default()
                .with_color(Color::from_rgb8(50, 205, 50))
                .with_width(3.0);

            for window in path.windows(2) {
                let from = window[0];
                let to = window[1];
                let path = Path::line(
                    (from.x as f32, -from.y as f32).into(), // Flip y-coordinate
                    (to.x as f32, -to.y as f32).into(),     // Flip y-coordinate
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

        // Overlay the optimal solution if requested
        if show_solution {
            if let Some((path, score)) = &self.optimal_path {
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
                        (from.x as f32, -from.y as f32).into(), // Flip y-coordinate
                        (to.x as f32, -to.y as f32).into(),     // Flip y-coordinate
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
        for vertex in &self.state.open {
            let circle = Path::circle((vertex.x as f32, -vertex.y as f32).into(), 1.0);
            frame.fill(&circle, Fill::from(Color::from_rgb8(0, 100, 255)));
        }

        for vertex in &self.state.closed {
            let circle = Path::circle((vertex.x as f32, -vertex.y as f32).into(), 1.0);
            frame.fill(&circle, Fill::from(Color::from_rgb8(255, 100, 100)));
        }

        if let Some(next) = self.state.next_vertex {
            let circle = Path::circle((next.x as f32, -next.y as f32).into(), 1.5);
            frame.fill(&circle, Fill::from(Color::from_rgb8(50, 205, 50)));
        }

        // Draw start and goal
        let start_circle = Path::circle((self.start.x as f32, -self.start.y as f32).into(), 2.0);
        frame.fill(&start_circle, Fill::from(Color::from_rgb8(0, 0, 255)));
        frame.fill_text(Text {
            content: format!("({}, {})", self.start.x, self.start.y),
            position: (self.start.x as f32, -self.start.y as f32 - 6.5).into(),
            color: Color::BLACK,
            size: 4.0.into(),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            ..Text::default()
        });

        let goal_circle = Path::circle((self.goal.x as f32, -self.goal.y as f32).into(), 2.0);
        frame.fill(&goal_circle, Fill::from(Color::from_rgb8(255, 0, 0)));
        frame.fill_text(Text {
            content: format!("({}, {})", self.goal.x, self.goal.y),
            position: (self.goal.x as f32 - 2.5, -self.goal.y as f32 - 6.5).into(),
            color: Color::BLACK,
            size: 4.0.into(),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            ..Text::default()
        });
    }

    /// Calculate actual distance between two points
    fn distance(p1: &Point<i32>, p2: &Point<i32>) -> i32 {
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        ((dx * dx + dy * dy) as f64).sqrt() as i32
    }

    /// Builds a visibility graph based on inter-visible vertices
    pub fn build_visibility_graph(&self) -> HashMap<Point, HashSet<Point>> {
        let mut graph: HashMap<Point, HashSet<Point>> = HashMap::new();
        let mut vertices = self.board.vertices();

        // Add start and goal to vertices
        vertices.insert(self.start);
        vertices.insert(self.goal);
        let vertices: Vec<_> = vertices.into_iter().collect();

        for (i, &v1) in vertices.iter().enumerate() {
            for (j, &v2) in vertices.iter().enumerate() {
                if i == j {
                    continue;
                }

                // Check if vertices are visible to each other
                if self.are_vertices_visible(v1, v2) {
                    graph.entry(v1).or_default().insert(v2);
                    graph.entry(v2).or_default().insert(v1);
                }
            }
        }

        graph
    }

    /// Determines if two vertices can see each other
    fn are_vertices_visible(&self, v1: Point, v2: Point) -> bool {
        // If points are the same, they're not visible to each other
        if v1 == v2 {
            return false;
        }

        // For each polygon
        for polygon in self.board.polygons() {
            // Skip intersection check if both points are vertices of this polygon
            let v1_in_polygon = polygon.vertices_vec().contains(&v1);
            let v2_in_polygon = polygon.vertices_vec().contains(&v2);

            if v1_in_polygon && v2_in_polygon {
                // If they're adjacent vertices in the polygon, they're visible
                let vertices = polygon.vertices_vec();
                let n = vertices.len();
                for i in 0..n {
                    let j = (i + 1) % n;
                    if (vertices[i] == v1 && vertices[j] == v2)
                        || (vertices[i] == v2 && vertices[j] == v1)
                    {
                        return true;
                    }
                }
                // Non-adjacent vertices of the same polygon can't see each other
                return false;
            }

            // Check if the line segment intersects this polygon
            if polygon.intersects_segment(&v1, &v2) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Polygon;

    // Helper function to create a simple test board with one obstacle
    fn create_test_board() -> Board {
        let polygons = vec![Polygon::new(vec![
            (40, 40).into(),
            (40, 60).into(),
            (60, 60).into(),
            (60, 40).into(),
        ])];
        Board::new(polygons)
    }

    #[test]
    fn test_search_completes() {
        let board = create_test_board();
        let start = Point::new(0, 0);
        let goal = Point::new(100, 100);
        let search = Search::new(board, start, goal, Heuristic::Euclidean);

        assert!(
            search.get_optimal_path().is_some(),
            "Search should find a path"
        );
    }

    #[test]
    fn test_path_connects_start_to_goal() {
        let board = create_test_board();
        let start = Point::new(0, 0);
        let goal = Point::new(100, 100);
        let search = Search::new(board, start, goal, Heuristic::Euclidean);

        let (path, _) = search.get_optimal_path().unwrap();
        assert_eq!(
            *path.first().unwrap(),
            start,
            "Path should start at start point"
        );
        assert_eq!(*path.last().unwrap(), goal, "Path should end at goal point");
    }

    #[test]
    fn test_consecutive_states_are_different() {
        let board = create_test_board();
        let start = Point::new(0, 0);
        let goal = Point::new(100, 100);
        let mut search = Search::new(board, start, goal, Heuristic::Euclidean);

        let total_steps = search.total_steps();
        let mut previous_state = None;

        for step in 0..=total_steps {
            search.jump_to(step);
            let current_state = (
                search.state.open.clone(),
                search.state.closed.clone(),
                search.state.considered_edges.clone(),
            );

            if let Some(prev_state) = previous_state {
                assert_ne!(
                    current_state, prev_state,
                    "State at step {} should be different from previous state",
                    step
                );
            }

            previous_state = Some(current_state);
        }
    }

    #[test]
    fn test_path_avoids_obstacles() {
        let board = create_test_board();
        let start = Point::new(0, 0);
        let goal = Point::new(100, 100);
        let search = Search::new(board.clone(), start, goal, Heuristic::Euclidean);

        let (path, _) = search.get_optimal_path().unwrap();

        // Check that no line segment in the path intersects with any polygon
        for window in path.windows(2) {
            let from = window[0];
            let to = window[1];

            for polygon in board.polygons() {
                assert!(
                    !polygon.intersects_segment(&from, &to),
                    "Path segment from {:?} to {:?} intersects with polygon",
                    from,
                    to
                );
            }
        }
    }

    #[test]
    fn test_heuristic_consistency() {
        let board = create_test_board();
        let start = Point::new(0, 0);
        let goal = Point::new(100, 100);

        // Run search with both heuristics
        let euclidean = Search::new(board.clone(), start, goal, Heuristic::Euclidean);
        let manhattan = Search::new(board, start, goal, Heuristic::Manhattan);

        // Both should find a path
        assert!(euclidean.get_optimal_path().is_some());
        assert!(manhattan.get_optimal_path().is_some());

        let (_, euclidean_score) = euclidean.get_optimal_path().unwrap();
        let (_, manhattan_score) = manhattan.get_optimal_path().unwrap();

        // Euclidean distance is always shortest possible, so its path should never be longer
        assert!(
            euclidean_score <= manhattan_score,
            "Euclidean path length ({}) should not exceed Manhattan path length ({})",
            euclidean_score,
            manhattan_score
        );
    }

    #[test]
    fn test_visibility_graph_properties() {
        let board = create_test_board();
        let start = Point::new(0, 0);
        let goal = Point::new(100, 100);
        let search = Search::new(board.clone(), start, goal, Heuristic::Euclidean);

        let graph = search.build_visibility_graph();

        // Check that start and goal are in the graph
        assert!(
            graph.contains_key(&start),
            "Start point should be in visibility graph"
        );
        assert!(
            graph.contains_key(&goal),
            "Goal point should be in visibility graph"
        );

        // Check symmetry property: if A can see B, B can see A
        for (vertex, visible) in &graph {
            for neighbor in visible {
                assert!(
                    graph.get(neighbor).unwrap().contains(vertex),
                    "Visibility graph should be symmetric: if {:?} sees {:?}, {:?} should see {:?}",
                    vertex,
                    neighbor,
                    neighbor,
                    vertex
                );
            }
        }
    }

    #[test]
    fn test_state_history_ends_at_goal() {
        let board = create_test_board();
        let start = Point::new(0, 0);
        let goal = Point::new(100, 100);
        let search = Search::new(board, start, goal, Heuristic::Euclidean);

        // Get the final state
        let mut final_search = search.clone();
        final_search.jump_to(final_search.total_steps());

        // The goal should be in either open or closed set
        assert!(
            final_search.state.open.contains(&goal) || final_search.state.closed.contains(&goal),
            "Final state should contain goal point"
        );

        // The best path should reach the goal
        assert_eq!(
            final_search.state.best_path.unwrap().last().unwrap(),
            &goal,
            "Best path should reach goal in final state"
        );
    }
}
