use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::{Board, Heuristic, Pathfinder, Point, SearchState};

#[derive(Debug, Clone)]
/// A* pathfinding implementation using pre-computed visibility graph
pub struct VisibilityGraphPathfinder {
    board: Board,
    start: Point,
    goal: Point,
    heuristic: Heuristic,
    visibility_graph: HashMap<Point, HashSet<Point>>,
    state: SearchState,
    history: Vec<SearchState>,
    current_step: usize,
    optimal_path: Option<(Vec<Point>, i32)>,
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

impl VisibilityGraphPathfinder {
    pub fn history(&self) -> &[SearchState] {
        &self.history
    }
}

impl Pathfinder for VisibilityGraphPathfinder {
    fn new(board: Board, start: Point, goal: Point, heuristic: Heuristic) -> Self {
        let mut search = Self {
            board,
            start,
            goal,
            heuristic,
            optimal_path: None,
            visibility_graph: HashMap::new(),
            state: SearchState {
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

        // Build visibility graph and compute solution
        search.visibility_graph = search.build_visibility_graph();
        search.compute_optimal_path();
        search.history.push(search.state.clone());
        search.reset();

        search
    }

    fn get_board(&self) -> &Board {
        &self.board
    }
    fn get_state(&self) -> &SearchState {
        &self.state
    }
    fn get_start(&self) -> Point {
        self.start
    }
    fn get_goal(&self) -> Point {
        self.goal
    }
    fn get_heuristic(&self) -> Heuristic {
        self.heuristic
    }

    fn get_optimal_path(&self) -> Option<&(Vec<Point>, i32)> {
        self.optimal_path.as_ref()
    }

    fn total_steps(&self) -> usize {
        self.history.len() - 1
    }

    fn current_step(&self) -> usize {
        self.current_step
    }

    fn step_forward(&mut self) -> bool {
        if self.is_finished() {
            return false;
        }
        self.current_step += 1;
        self.state = self.history[self.current_step].clone();
        true
    }

    fn step_back(&mut self) -> bool {
        if self.current_step == 0 {
            return false;
        }
        self.current_step -= 1;
        self.state = self.history[self.current_step].clone();
        true
    }

    fn jump_to(&mut self, step: usize) -> bool {
        if step > self.total_steps() {
            return false;
        }
        self.current_step = step;
        self.state = self.history[self.current_step].clone();
        true
    }

    fn reset(&mut self) {
        self.current_step = 0;
        self.state = self.history[0].clone();
    }

    fn change_heuristic(&mut self, heuristic: Heuristic) {
        self.heuristic = heuristic;
        self.reset();
        self.compute_optimal_path();
    }
}

impl VisibilityGraphPathfinder {
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
            if current.vertex == self.goal {
                let path = self.reconstruct_path(&current.vertex);
                self.optimal_path = Some((path.clone(), current.g_score));
                self.state.best_path = Some(path);
                return;
            }

            // Save state for visualization
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

    /// Builds visibility graph based on inter-visible vertices
    fn build_visibility_graph(&self) -> HashMap<Point, HashSet<Point>> {
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
        if v1 == v2 {
            return false;
        }

        for polygon in self.board.polygons() {
            // Special case: if both points are vertices of same polygon
            let v1_in_polygon = polygon.vertices_vec().contains(&v1);
            let v2_in_polygon = polygon.vertices_vec().contains(&v2);

            if v1_in_polygon && v2_in_polygon {
                // Visible if they're adjacent vertices
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
                // Non-adjacent vertices of same polygon can't see each other
                return false;
            }

            // Check if line segment intersects this polygon
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
        let search = VisibilityGraphPathfinder::new(board, start, goal, Heuristic::Euclidean);

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
        let search = VisibilityGraphPathfinder::new(board, start, goal, Heuristic::Euclidean);

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
        let mut search = VisibilityGraphPathfinder::new(board, start, goal, Heuristic::Euclidean);

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
        let search =
            VisibilityGraphPathfinder::new(board.clone(), start, goal, Heuristic::Euclidean);

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
        let euclidean =
            VisibilityGraphPathfinder::new(board.clone(), start, goal, Heuristic::Euclidean);
        let manhattan = VisibilityGraphPathfinder::new(board, start, goal, Heuristic::Manhattan);

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
        let search =
            VisibilityGraphPathfinder::new(board.clone(), start, goal, Heuristic::Euclidean);

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
        let search = VisibilityGraphPathfinder::new(board, start, goal, Heuristic::Euclidean);

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
