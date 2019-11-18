use crate::map::MapConfig;
use crate::map::Orientation;
use core::ops::Index;
use core::ops::IndexMut;

pub const WIDTH: usize = 16;
pub const HEIGHT: usize = 16;

pub struct MazeConfig {
    pub cell_width: f32,
    pub wall_width: f32,
}

impl MazeConfig {
    /**
     *  Projects the `from` orientation onto the nearest wall, and gives the index of it
     *
     *  Loops starting at `from`, incrementing by a distance of `self.wall_width / 2.0` in the direction
     *  of `from` until a closed wall is found, then returns the index to that wall.
     *
     *  By incrementing by a distance of half the wall width, we are guaranteed to not skip over a wall.
     */
    pub fn project_wall(&self, from: Orientation) -> EdgeIndex {
        let direction_vector = (self.wall_width / 2.0) * from.direction.into_unit_vector();

        let mut current_position = from.position;

        loop {
            let local_x = current_position.x % self.cell_width;
            let local_y = current_position.y % self.cell_width;
            let maze_x = (current_position.x / self.cell_width) as usize;
            let maze_y = (current_position.y / self.cell_width) as usize;

            if local_y <= self.wall_width / 2.0 {
                break EdgeIndex {
                    x: maze_x,
                    y: maze_y,
                    horizontal: true,
                };
            }

            if local_y >= self.cell_width - self.wall_width / 2.0 {
                break EdgeIndex {
                    x: maze_x,
                    y: maze_y + 1,
                    horizontal: true,
                };
            }

            if local_x <= self.wall_width / 2.0 {
                break EdgeIndex {
                    x: maze_x,
                    y: maze_y,
                    horizontal: false,
                };
            }

            if local_x >= self.cell_width - self.wall_width / 2.0 {
                break EdgeIndex {
                    x: maze_x + 1,
                    y: maze_y,
                    horizontal: false,
                };
            }

            current_position += direction_vector;
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Edge {
    Open,
    Closed,
    Unknown,
}

/// An index into a maze. This will uniquely identify any edge.
/// The indexes are 0-based, but do include the perimeter edges.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct EdgeIndex {
    /// The x index of the edge
    pub x: usize,

    /// The y index of the edge
    pub y: usize,

    /// Whether the edge is horizontal (true), or vertical (false)
    pub horizontal: bool,
}

/// Keeps track of all the walls in a maze
#[derive(Clone, Debug)]
pub struct Maze {
    horizontal_edges: [[Edge; HEIGHT - 1]; WIDTH],
    vertical_edges: [[Edge; HEIGHT]; WIDTH - 1],
}

impl Maze {
    pub fn new(edge: Edge) -> Maze {
        Maze {
            horizontal_edges: [[edge; HEIGHT - 1]; WIDTH],
            vertical_edges: [[edge; HEIGHT]; WIDTH - 1],
        }
    }

    pub fn from_edges(
        horizontal_edges: [[Edge; HEIGHT - 1]; WIDTH],
        vertical_edges: [[Edge; HEIGHT]; WIDTH - 1],
    ) -> Maze {
        Maze {
            horizontal_edges,
            vertical_edges,
        }
    }

    /**
     *  Reads files in the format described by
     *  http://www.micromouseonline.com/2018/01/31/micromouse-maze-file-collection/
     */
    pub fn from_file(bytes: [u8; WIDTH * HEIGHT]) -> Maze {
        let mut horizontal_edges = [[Edge::Unknown; HEIGHT - 1]; WIDTH];
        let mut vertical_edges = [[Edge::Unknown; HEIGHT]; WIDTH - 1];

        for (i, byte) in bytes.iter().enumerate() {
            let y = i % WIDTH;
            let x = i / WIDTH;

            let north = if byte & 0x01 == 0x01 {
                Edge::Closed
            } else {
                Edge::Open
            };
            let east = if byte & 0x02 == 0x02 {
                Edge::Closed
            } else {
                Edge::Open
            };

            if y < HEIGHT - 1 {
                horizontal_edges[x][y] = north;
            }

            if x < WIDTH - 1 {
                vertical_edges[x][y] = east;
            }
        }

        Maze {
            horizontal_edges,
            vertical_edges,
        }
    }

    pub fn get_cell(&self, x: usize, y: usize) -> (Edge, Edge, Edge, Edge) {
        let north_edge = if y >= HEIGHT - 1 {
            Edge::Closed
        } else {
            self.horizontal_edges[x][y]
        };

        let south_edge = if y <= 0 {
            Edge::Closed
        } else {
            self.horizontal_edges[x][y - 1]
        };

        let east_edge = if x >= WIDTH - 1 {
            Edge::Closed
        } else {
            self.vertical_edges[x][y]
        };

        let west_edge = if x <= 0 {
            Edge::Closed
        } else {
            self.vertical_edges[x - 1][y]
        };

        (north_edge, south_edge, east_edge, west_edge)
    }

    pub fn get_edge(&self, index: EdgeIndex) -> Option<&Edge> {
        if index.horizontal {
            self.horizontal_edges
                .get(index.x)
                .and_then(|walls| walls.get(index.y - 1))
        } else {
            self.vertical_edges
                .get(index.x - 1)
                .and_then(|walls| walls.get(index.y))
        }
    }

    pub fn get_edge_mut(&mut self, index: EdgeIndex) -> Option<&mut Edge> {
        if index.horizontal {
            self.horizontal_edges
                .get_mut(index.x)
                .and_then(|walls| walls.get_mut(index.y))
        } else {
            self.vertical_edges
                .get_mut(index.x)
                .and_then(|walls| walls.get_mut(index.y))
        }
    }
}
