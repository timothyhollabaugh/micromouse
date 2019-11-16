use crate::map::MapConfig;
use crate::map::Orientation;

pub const WIDTH: usize = 16;
pub const HEIGHT: usize = 16;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Edge {
    Open,
    Closed,
    Unknown,
}

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

    pub fn get_edge(&self, x: usize, y: usize, horizontal: bool) -> Edge {
        if horizontal {
            *self
                .horizontal_edges
                .get(x)
                .and_then(|walls| walls.get(y))
                .unwrap_or(&Edge::Closed)
        } else {
            *self
                .vertical_edges
                .get(y)
                .and_then(|walls| walls.get(x))
                .unwrap_or(&Edge::Closed)
        }
    }

    /**
     *  Projects the `from` orientation onto the nearest wall, and gives the index of it
     */
    pub fn project_wall(&self, config: MapConfig, from: Orientation) -> (usize, usize, bool) {
        let direction_vector = from.clone().into_unit_vector();
        let mut current_position = from.position;
        loop {
            let local_x = direction_vector.x % config.cell_width;
            if local_x <= config.wall_width / 2
                || local_x >= config.cell_width - config.wall_width / 2
            {}
        }
    }
}
