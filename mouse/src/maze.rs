use crate::map::MapConfig;
use crate::map::Orientation;
use core::ops::Index;
use core::ops::IndexMut;

pub const WIDTH: usize = 16;
pub const HEIGHT: usize = 16;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Edge {
    Open,
    Closed,
    Unknown,
}

/// An index into a maze. This will uniquely identify any edge.
/// The indexes are 0-based, but do not include any of the perimiter edges.
pub struct EdgeIndex {
    /// The x index of the edge
    x: usize,

    /// The y index of the edge
    y: usize,

    /// Whether the edge is horizontal (true), or vertical (false)
    horizontal: bool,
} /// Keeps track of all the walls in a maze pub struct Maze {
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

}

impl Index<EdgeIndex> for Maze {
    type Output = Edge;

    fn index(&self, index: EdgeIndex) -> &Self::Output {
        if horizontal {
            self.horizontal_edges
                .get(x)
                .and_then(|walls| walls.get(y))
                .unwrap_or(&Edge::Closed)
        } else {
            self.vertical_edges
                .get(y)
                .and_then(|walls| walls.get(x))
                .unwrap_or(&Edge::Closed)
        }
    }
}

impl IndexMut<EdgeIndex> for Maze {
    fn index_mut(&mut self, index: EdgeIndex) -> &mut Self::Output {
        if horizontal {
            self.horizontal_edges
                .get_mut(x)
                .and_then(|walls| walls.get_mut(y))
                .unwrap_or(&mut Edge::Closed)
        } else {
            self.vertical_edges
                .get_mut(y)
                .and_then(|walls| walls.get_mut(x))
                .unwrap_or(&mut Edge::Closed)
        }
    }
}
