use crate::{Board, Point};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Represents a path-finding problem from start to goal on a board
#[derive(Debug)]
pub struct Search {
    /// The board containing obstacles
    board: Board,
    /// Starting point
    start: Point,
    /// Goal point
    goal: Point,
}

/// Represents a node in the A* search
#[derive(Clone, Eq, PartialEq)]
struct SearchNode {
    vertex: Point<i32>,
    g_score: i32, // Cost from start to this node
    f_score: i32, // Estimated total cost (g_score + heuristic)
    came_from: Option<Point<i32>>,
}

impl Ord for SearchNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering because BinaryHeap is a max-heap
        other.f_score.cmp(&self.f_score)
    }
}

impl PartialOrd for SearchNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Search {
    /// Creates a new search problem with the given board and points
    pub fn new(board: Board, start: Point, goal: Point) -> Self {
        Self { board, start, goal }
    }

    /// Builds a visibility graph representing which vertices can see each other
    pub fn build_visibility_graph(&self) -> HashMap<Point, HashSet<Point>> {
        let mut graph: HashMap<Point, HashSet<Point>> = HashMap::new();
        let mut vertices = self.board.all_vertices();
        vertices.insert(self.start);
        vertices.insert(self.goal);

        for &from_vertex in &vertices {
            let mut visible = HashSet::new();

            'outer: for &to_vertex in &vertices {
                if from_vertex == to_vertex {
                    continue;
                }

                for polygon in self.board.polygons() {
                    if polygon.intersects_segment(&from_vertex, &to_vertex) {
                        continue 'outer;
                    }
                }

                visible.insert(to_vertex);
            }

            graph.insert(from_vertex, visible);
        }

        graph
    }

    /// Counts all possible simple paths (no repeated vertices) from start to goal using DFS
    pub fn count_paths(&self) -> usize {
        let visibility_graph = self.build_visibility_graph();
        let mut visited = HashSet::new();
        visited.insert(self.start); // Mark start as visited immediately

        self.count_paths_recursive(&self.start, &mut visited, &visibility_graph)
    }

    fn count_paths_recursive(
        &self,
        current: &Point<i32>,
        visited: &mut HashSet<Point<i32>>,
        graph: &HashMap<Point<i32>, HashSet<Point<i32>>>,
    ) -> usize {
        if current == &self.goal {
            return 1;
        }

        let mut count = 0;

        if let Some(neighbors) = graph.get(current) {
            for next in neighbors {
                if !visited.contains(next) {
                    visited.insert(*next);
                    count += self.count_paths_recursive(next, visited, graph);
                    visited.remove(next);
                }
            }
        }

        count
    }

    /// Calculate Manhattan distance heuristic
    fn heuristic(p1: &Point<i32>, p2: &Point<i32>) -> i32 {
        (p2.x - p1.x).abs() + (p2.y - p1.y).abs()
    }

    /// Calculate actual distance between two points
    fn distance(p1: &Point<i32>, p2: &Point<i32>) -> i32 {
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        ((dx * dx + dy * dy) as f64).sqrt() as i32
    }

    /// Find the shortest path using A* algorithm
    pub fn find_shortest_path(&self) -> Option<(Vec<Point<i32>>, i32)> {
        let visibility_graph = self.build_visibility_graph();
        let mut open_set = BinaryHeap::new();
        let mut came_from = HashMap::new();
        let mut g_scores = HashMap::new();

        // Initialize start node
        open_set.push(SearchNode {
            vertex: self.start,
            g_score: 0,
            f_score: Self::heuristic(&self.start, &self.goal),
            came_from: None,
        });
        g_scores.insert(self.start, 0);

        while let Some(current) = open_set.pop() {
            if current.vertex == self.goal {
                // Reconstruct path
                let mut path = vec![self.goal];
                let mut current_vertex = self.goal;
                while let Some(prev) = came_from.get(&current_vertex) {
                    path.push(*prev);
                    current_vertex = *prev;
                }
                path.reverse();
                return Some((path, current.g_score));
            }

            if let Some(neighbors) = visibility_graph.get(&current.vertex) {
                for &neighbor in neighbors {
                    let tentative_g_score =
                        current.g_score + Self::distance(&current.vertex, &neighbor);

                    if !g_scores.contains_key(&neighbor)
                        || tentative_g_score < *g_scores.get(&neighbor).unwrap()
                    {
                        came_from.insert(neighbor, current.vertex);
                        g_scores.insert(neighbor, tentative_g_score);

                        open_set.push(SearchNode {
                            vertex: neighbor,
                            g_score: tentative_g_score,
                            f_score: tentative_g_score + Self::heuristic(&neighbor, &self.goal),
                            came_from: Some(current.vertex),
                        });
                    }
                }
            }
        }

        None // No path found
    }

    /// Returns the total number of points in the state space
    pub fn state_space_size(&self) -> usize {
        // All polygon vertices plus start and goal points
        self.board.vertex_count() + 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Polygon;

    #[test]
    fn test_simple_board() {
        // Create a simple board with just one polygon
        let board = Board::new(vec![Polygon::new(vec![
            Point::new(200, 400),
            Point::new(200, 600),
            Point::new(300, 600),
            Point::new(300, 400),
        ])]);

        let search = Search::new(
            board,
            Point::new(100, 500), // start: left of polygon
            Point::new(400, 500), // goal: right of polygon
        );

        let result = search.find_shortest_path();
        assert!(result.is_some());
        let (path, distance) = result.unwrap();
        assert!(path.len() >= 2); // Should at least contain start and end
        println!("Path length: {}, Distance: {}", path.len(), distance);
    }

    #[test]
    fn test_empty_board() {
        let board = Board::new(vec![]);
        let search = Search::new(board, Point::new(0, 0), Point::new(100, 100));

        let result = search.find_shortest_path();
        assert!(result.is_some());
        let (path, _) = result.unwrap();
        println!("Empty board path: {:?}", path);
        assert_eq!(path.len(), 2);
    }

    #[test]
    fn test_default_board() {
        let board = Board::default();
        let search = Search::new(board, Point::new(100, 500), Point::new(400, 500));

        let result = search.find_shortest_path();
        assert!(result.is_some());
        let (path, distance) = result.unwrap();
        println!("Default board path: {:?}, distance: {}", path, distance);
    }

    #[test]
    fn test_state_space_size() {
        let board = Board::default();
        let vertices_per_polygon = board.vertices_per_polygon();

        // Verify each polygon's vertex count
        assert_eq!(vertices_per_polygon, vec![4, 4, 6, 5, 4, 4, 3, 3]);

        // Verify total vertex count (33 vertices)
        let count = board.vertex_count();
        assert_eq!(count, 33);

        // Create a search to verify total state space (33 + 2 points)
        let search = Search::new(board, Point::new(100, 500), Point::new(400, 500));

        assert_eq!(search.state_space_size(), 35);

        // Print breakdown for clarity
        println!("Vertices per polygon: {:?}", vertices_per_polygon);
        println!("Total polygon vertices: {}", count);
        println!(
            "Total state space (with start/goal): {}",
            search.state_space_size()
        );
    }
}
