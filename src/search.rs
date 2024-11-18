// search.rs
mod simple;
mod visibility;

pub use simple::AStarPathfinder;
pub use visibility::VisibilityGraphPathfinder;

use crate::{Board, Heuristic, Pathfinder, Point, SearchState};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SearchVariant {
    VisibilityGraph,
    AStar,
}

impl SearchVariant {
    pub const ALL: &'static [SearchVariant] =
        &[SearchVariant::VisibilityGraph, SearchVariant::AStar];
}

impl std::fmt::Display for SearchVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchVariant::VisibilityGraph => write!(f, "Visibility Graph"),
            SearchVariant::AStar => write!(f, "A*"),
        }
    }
}

#[derive(Clone)]
pub enum Search {
    Visibility(VisibilityGraphPathfinder),
    AStar(AStarPathfinder),
}

impl std::fmt::Display for Search {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Search::Visibility(_) => write!(f, "Visibility Graph"),
            Search::AStar(_) => write!(f, "A*"),
        }
    }
}

impl Search {
    pub fn variant(&self) -> SearchVariant {
        match self {
            Search::Visibility(_) => SearchVariant::VisibilityGraph,
            Search::AStar(_) => SearchVariant::AStar,
        }
    }

    pub fn history(&self) -> &[SearchState] {
        match self {
            Search::Visibility(p) => p.history(),
            Search::AStar(p) => p.history(),
        }
    }

    pub fn new_for_variant(
        board: Board,
        start: Point,
        goal: Point,
        heuristic: Heuristic,
        variant: SearchVariant,
    ) -> Self {
        match variant {
            SearchVariant::VisibilityGraph => Self::Visibility(VisibilityGraphPathfinder::new(
                board, start, goal, heuristic,
            )),
            SearchVariant::AStar => {
                Self::AStar(AStarPathfinder::new(board, start, goal, heuristic))
            }
        }
    }
}

// Delegate all trait methods to the contained implementation
impl Pathfinder for Search {
    fn get_board(&self) -> &Board {
        match self {
            Self::Visibility(p) => p.get_board(),
            Self::AStar(p) => p.get_board(),
        }
    }

    fn get_state(&self) -> &SearchState {
        match self {
            Self::Visibility(p) => p.get_state(),
            Self::AStar(p) => p.get_state(),
        }
    }

    fn get_start(&self) -> Point {
        match self {
            Self::Visibility(p) => p.get_start(),
            Self::AStar(p) => p.get_start(),
        }
    }

    fn get_heuristic(&self) -> Heuristic {
        match self {
            Self::Visibility(p) => p.get_heuristic(),
            Self::AStar(p) => p.get_heuristic(),
        }
    }

    fn new(board: Board, start: Point, goal: Point, heuristic: Heuristic) -> Self {
        Self::Visibility(VisibilityGraphPathfinder::new(
            board, start, goal, heuristic,
        ))
    }

    fn get_goal(&self) -> Point {
        match self {
            Self::Visibility(p) => p.get_goal(),
            Self::AStar(p) => p.get_goal(),
        }
    }

    fn get_optimal_path(&self) -> Option<&(Vec<Point>, i32)> {
        match self {
            Self::Visibility(p) => p.get_optimal_path(),
            Self::AStar(p) => p.get_optimal_path(),
        }
    }

    fn current_step(&self) -> usize {
        match self {
            Self::Visibility(p) => p.current_step(),
            Self::AStar(p) => p.current_step(),
        }
    }

    fn total_steps(&self) -> usize {
        match self {
            Self::Visibility(p) => p.total_steps(),
            Self::AStar(p) => p.total_steps(),
        }
    }

    fn step_forward(&mut self) -> bool {
        match self {
            Self::Visibility(p) => p.step_forward(),
            Self::AStar(p) => p.step_forward(),
        }
    }

    fn step_back(&mut self) -> bool {
        match self {
            Self::Visibility(p) => p.step_back(),
            Self::AStar(p) => p.step_back(),
        }
    }

    fn jump_to(&mut self, step: usize) -> bool {
        match self {
            Self::Visibility(p) => p.jump_to(step),
            Self::AStar(p) => p.jump_to(step),
        }
    }

    fn reset(&mut self) {
        match self {
            Self::Visibility(p) => p.reset(),
            Self::AStar(p) => p.reset(),
        }
    }

    fn change_heuristic(&mut self, heuristic: Heuristic) {
        match self {
            Self::Visibility(p) => p.change_heuristic(heuristic),
            Self::AStar(p) => p.change_heuristic(heuristic),
        }
    }
}
