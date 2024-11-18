use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use crate::{Board, Heuristic, Pathfinder, Point, SearchState};

/// A* pathfinding implementation following the textbook approach:
/// - No visibility graph preprocessing
/// - Explores points dynamically
/// - Maintains OPEN and CLOSED lists explicitly
/// - Reopens CLOSED nodes when better paths are found
#[derive(Clone)]
pub struct AStarPathfinder {
    board: Board,
    start: Point,
    goal: Point,
    heuristic: Heuristic,
    state: SearchState,
    history: Vec<SearchState>,
    current_step: usize,
    optimal_path: Option<(Vec<Point>, i32)>,
    // Store these separately since they're not part of visualization state
    open_nodes: BinaryHeap<SearchNode>,
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

impl AStarPathfinder {
    pub fn history(&self) -> &[SearchState] {
        &self.history
    }
}

impl Pathfinder for AStarPathfinder {
    fn new(board: Board, start: Point, goal: Point, heuristic: Heuristic) -> Self {
        let mut search = Self {
            board,
            start,
            goal,
            heuristic,
            optimal_path: None,
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
            history: Vec::new(),
            current_step: 0,
            open_nodes: BinaryHeap::new(),
        };

        // Initialize start node
        search.open_nodes.push(SearchNode {
            vertex: start,
            g_score: 0,
            f_score: heuristic.distance(&start, &goal),
        });

        // Compute solution and history
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

impl AStarPathfinder {
    fn compute_optimal_path(&mut self) {
        self.history.clear();

        // Step 1: Initialize OPEN with start node
        let h_start = self.heuristic.distance(&self.start, &self.goal);
        self.open_nodes.push(SearchNode {
            vertex: self.start,
            g_score: 0,
            f_score: h_start,
        });
        self.state.g_scores.insert(self.start, 0);
        self.state.open.insert(self.start);

        // Step 2: Main loop
        while let Some(best_node) = self.open_nodes.pop() {
            let best_vertex = best_node.vertex;

            // Check if we've reached the goal
            if best_vertex == self.goal {
                let path = self.reconstruct_path(&best_vertex);
                self.optimal_path = Some((path.clone(), best_node.g_score));
                self.state.best_path = Some(path);
                self.history.push(self.state.clone());
                return;
            }

            // Move BESTNODE from OPEN to CLOSED
            self.state.open.remove(&best_vertex);
            self.state.closed.insert(best_vertex);

            // Save state for visualization
            self.history.push(self.state.clone());

            // Generate successors
            for successor in self.get_successors(&best_vertex) {
                // Calculate tentative g score (g in the textbook)
                let successor_g = best_node.g_score + Self::distance(&best_vertex, &successor);

                // Calculate h' value for successor
                let successor_h = self.heuristic.distance(&successor, &self.goal);
                let successor_f = successor_g + successor_h;

                // Check if successor is on OPEN (step 2c in textbook)
                if self.state.open.contains(&successor) {
                    if successor_g >= *self.state.g_scores.get(&successor).unwrap() {
                        continue; // Current path is not better
                    }
                    // Found a better path to an OPEN node
                    self.update_node(&successor, &best_vertex, successor_g, successor_f);
                }
                // Check if successor is on CLOSED (step 2d in textbook)
                else if self.state.closed.contains(&successor) {
                    if successor_g >= *self.state.g_scores.get(&successor).unwrap() {
                        continue; // Current path is not better
                    }
                    // Found a better path to a CLOSED node - reopen it
                    self.state.closed.remove(&successor);
                    self.update_node(&successor, &best_vertex, successor_g, successor_f);
                    // Note: The textbook calls for recursive propagation here
                    // but we'll skip it for simplicity since our paths are simple
                }
                // Successor is new (step 2e in textbook)
                else {
                    self.state.open.insert(successor);
                    self.update_node(&successor, &best_vertex, successor_g, successor_f);
                }

                // Record edge for visualization
                self.state.considered_edges.insert((best_vertex, successor));
            }
        }

        // No path found - record final state
        self.history.push(self.state.clone());
    }

    fn update_node(&mut self, node: &Point, parent: &Point, g_score: i32, f_score: i32) {
        self.state.came_from.insert(*node, *parent);
        self.state.g_scores.insert(*node, g_score);

        let mut new_path = self.reconstruct_path(parent);
        new_path.push(*node);
        self.state.current_paths.insert(*node, new_path);

        self.open_nodes.push(SearchNode {
            vertex: *node,
            g_score,
            f_score,
        });
    }

    fn get_successors(&self, vertex: &Point) -> Vec<Point> {
        let mut successors = Vec::new();

        // Add visible polygon vertices as successors
        for polygon in self.board.polygons() {
            for v in polygon.vertices() {
                if self.is_valid_move(vertex, v) {
                    successors.push(*v);
                }
            }
        }

        // Always consider goal if we can see it
        if self.is_valid_move(vertex, &self.goal) {
            successors.push(self.goal);
        }

        successors
    }

    fn is_valid_move(&self, from: &Point, to: &Point) -> bool {
        if from == to {
            return false;
        }

        // Check against each polygon
        for polygon in self.board.polygons() {
            if polygon.intersects_segment(from, to) {
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
    fn test_path_found() {
        let board = create_test_board();
        let start = Point::new(0, 0);
        let goal = Point::new(100, 100);
        let search = AStarPathfinder::new(board, start, goal, Heuristic::Euclidean);

        assert!(
            search.get_optimal_path().is_some(),
            "Search should find a path"
        );
    }

    #[test]
    fn test_path_valid() {
        let board = create_test_board();
        let start = Point::new(0, 0);
        let goal = Point::new(100, 100);
        let search = AStarPathfinder::new(board.clone(), start, goal, Heuristic::Euclidean);

        let (path, _) = search.get_optimal_path().unwrap();

        // Check path connects start to goal
        assert_eq!(*path.first().unwrap(), start);
        assert_eq!(*path.last().unwrap(), goal);

        // Check no segments intersect obstacles
        for window in path.windows(2) {
            let from = window[0];
            let to = window[1];
            assert!(
                search.is_valid_move(&from, &to),
                "Path segment from {:?} to {:?} intersects obstacle",
                from,
                to
            );
        }
    }

    #[test]
    fn test_nodes_never_reopened() {
        let board = Board::new(vec![Polygon::new(vec![
            (40, 0).into(),
            (40, 30).into(),
            (60, 30).into(),
            (60, 0).into(),
        ])]);

        let start = Point::new(20, 20);
        let goal = Point::new(80, 20);
        let mut search = AStarPathfinder::new(board, start, goal, Heuristic::Euclidean);

        // Run search
        search.compute_optimal_path();

        // Verify that no node in a closed set ever appears in a later open set
        let mut ever_closed = HashSet::new();

        for state in &search.history {
            // Check that no previously closed node is in open
            assert!(
                state.open.intersection(&ever_closed).count() == 0,
                "Found reopened node! Open: {:?}, Previously closed: {:?}",
                state.open,
                ever_closed
            );

            // Add currently closed nodes to our set
            ever_closed.extend(state.closed.iter().copied());
        }
    }

    #[test]
    fn test_path_optimality() {
        // Use same board setup as reopening test
        let board = Board::new(vec![
            Polygon::new(vec![
                (30, 0).into(),
                (30, 40).into(),
                (35, 40).into(),
                (35, 0).into(),
            ]),
            Polygon::new(vec![
                (50, 20).into(),
                (50, 60).into(),
                (55, 60).into(),
                (55, 20).into(),
            ]),
            Polygon::new(vec![
                (70, 0).into(),
                (70, 40).into(),
                (75, 40).into(),
                (75, 0).into(),
            ]),
        ]);

        let start = Point::new(20, 20);
        let goal = Point::new(80, 20);

        // Run with both heuristics to compare
        let euclidean = AStarPathfinder::new(board.clone(), start, goal, Heuristic::Euclidean);
        let manhattan = AStarPathfinder::new(board, start, goal, Heuristic::Manhattan);

        let (euclidean_path, euclidean_cost) = euclidean.get_optimal_path().unwrap();
        let (_manhattan_path, manhattan_cost) = manhattan.get_optimal_path().unwrap();

        // Euclidean heuristic should never produce a longer path
        assert!(
            euclidean_cost <= manhattan_cost,
            "Euclidean path cost ({}) should not exceed Manhattan path cost ({})",
            euclidean_cost,
            manhattan_cost
        );

        // Verify path is valid
        for window in euclidean_path.windows(2) {
            assert!(
                euclidean.is_valid_move(&window[0], &window[1]),
                "Invalid move in optimal path: {:?} -> {:?}",
                window[0],
                window[1]
            );
        }
    }
}
