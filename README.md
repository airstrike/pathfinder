<div align="center">

# Pathfinder

Interactive pathfinding visualization demonstrating optimal path discovery around polygonal obstacles.

<img src="./assets/logo.png" alt="Pathfinder Logo" width="100"/>

[![Made with iced](https://iced.rs/badge.svg)](https://github.com/iced-rs/iced)

  <img src="./assets/demo.gif" alt="Pathfinder Demo" width="600"/>

</div>

## Installation

> Note: You will need the Rust toolchain installed. You can install it by following the instructions at [rustup.rs](https://rustup.rs/).

```bash
# Clone the repository, build and run
git clone https://github.com/airstrike/pathfinder
cd pathfinder
cargo run --release
```

## Overview

Pathfinder is built in Rust using the [`iced`](https://iced.rs) GUI framework.
The application provides an extensible framework for implementing and
visualizing different pathfinding strategies between start and goal points
while avoiding polygonal obstacles.

### Features

- Interactive visualization with play/pause and step-by-step controls
- Click to place start/goal points
- Multiple pathfinding strategies (A* and Visibility Graph)
- Choice of distance heuristics (Euclidean, Manhattan)
- Real-time visualization of search progress
- Polygon-based obstacles with robust intersection testing
- Pastel color scheme for clear obstacle identification

### Code Structure

The codebase is organized into several key modules:

- `main.rs`: Entry point and GUI implementation using iced. Handles user
  interaction, visualization loop, and keyboard/mouse controls.

- `board.rs`: Defines the game board and its polygonal obstacles. Handles
  drawing the board and provides interfaces to query board state.

- `polygon.rs`: Sophisticated polygon representation with robust geometric
  operations:
  - Intersection detection using orientation predicates
  - Point-in-polygon testing via ray casting
  - Special case handling for collinear points and edge cases
  - Colored visualization with pastel shades

- `pathfinder.rs`: Defines the core `Pathfinder` trait that all pathfinding
  implementations must satisfy. Provides default implementations for:
  - Path reconstruction
  - Distance calculations
  - State management
  - Visualization
  - Step controls (forward/back/reset)

- `search/`: Contains concrete pathfinding implementations:
  - `simple.rs`: Classic A* implementation that explores points dynamically
  - `visibility.rs`: Visibility graph-based implementation that pre-computes
    valid paths between visible vertices

### Pathfinding Implementations

#### Common Interface (`Pathfinder` trait)

The core `Pathfinder` trait defines a common interface that all pathfinding strategies must implement. This includes:
- Board and state access (current board configuration, search state)
- Path management (reconstruction, scoring, validation)
- Algorithm control (initialization, stepping, reset)
- Visualization support (drawing current state, history)
- Heuristic configuration
- Solution access and validation

The trait provides default implementations for visualization, path reconstruction, scoring, and state management, allowing implementations to focus on their core pathfinding logic.

#### A* Implementation (`AStarPathfinder`)

- Follows the textbook approach with OPEN/CLOSED lists
- Dynamically explores points without preprocessing
- Reopens CLOSED nodes when better paths are found
- Maintains comprehensive path history for visualization

#### Visibility Graph Implementation (`VisibilityGraphPathfinder`)

- Pre-computes a visibility graph connecting mutually visible vertices
- Uses a geometric approach to determine vertex visibility
- Performs A* search on the reduced graph
- More efficient for static environments
- Guarantees optimal paths through vertex-vertex movements

### Visualization

The visualization system leverages iced's `Canvas` widget to provide:

- Real-time rendering of the search process
- Color-coded elements:
  - Open nodes (blue)
  - Closed nodes (red)
  - Current best path (green)
  - Historical paths (gray)
  - Polygonal obstacles (pastel colors)
- Interactive controls:
  - Play/pause/step buttons
  - Navigation slider
  - Algorithm selection
  - Heuristic selection
  - Solution overlay toggle

## TODOs

If I had infinite free time, I'd implement some or all of the below:

- [ ] Add more pathfinding implementations:
  - [ ] Dijkstra's algorithm
  - [ ] RRT (Rapidly-exploring Random Trees)
  - [ ] Potential fields
- [ ] Support custom boards and obstacle placement
- [ ] Add more visualization modes (heatmaps, path costs)
- [ ] Implement dynamic obstacle avoidance
- [ ] Add comparative performance metrics
- [ ] Support for weighted edges and terrain costs
- [ ] Path smoothing and optimization

## Contributing

New pathfinding implementations can be added by:
1. Creating a new implementation of the `Pathfinder` trait
2. Adding the implementation to the `Search` enum
3. Including comprehensive test cases
4. Ensuring proper visualization support

The project emphasizes intuitive, interactive visualizations that help users
understand how different pathfinding algorithms work. The modular architecture,
built around the `Pathfinder` trait, makes it straightforward to add new
algorithms while maintaining a consistent visualization and interaction model.

## Acknowledgements

- [Rust Programming Language](https://www.rust-lang.org/)
- [iced](https://iced.rs)
