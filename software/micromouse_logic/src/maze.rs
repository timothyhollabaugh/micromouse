use serde::Deserialize;
use serde::Serialize;

use crate::math::Orientation;
use crate::math::Vector;

pub const WIDTH: usize = 16;
pub const HEIGHT: usize = 16;

#[derive(Debug, Copy, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MazeConfig {
    pub cell_width: f32,
    pub wall_width: f32,
}

impl MazeConfig {
    pub fn wall_on_point(&self, point: Vector) -> Option<WallIndex> {
        let local_x = point.x % self.cell_width;
        let local_y = point.y % self.cell_width;
        let maze_x = (point.x / self.cell_width) as usize;
        let maze_y = (point.y / self.cell_width) as usize;

        if local_y <= self.wall_width / 2.0 {
            return Some(WallIndex {
                x: maze_x,
                y: maze_y,
                horizontal: true,
            });
        }

        if local_y >= self.cell_width - self.wall_width / 2.0 {
            return Some(WallIndex {
                x: maze_x,
                y: maze_y + 1,
                horizontal: true,
            });
        }

        if local_x <= self.wall_width / 2.0 {
            return Some(WallIndex {
                x: maze_x,
                y: maze_y,
                horizontal: false,
            });
        }

        if local_x >= self.cell_width - self.wall_width / 2.0 {
            return Some(WallIndex {
                x: maze_x + 1,
                y: maze_y,
                horizontal: false,
            });
        }

        None
    }

    pub fn wall_projection_iter(&self, from: Orientation) -> WallProjectionIterator {
        WallProjectionIterator {
            config: self,
            direction_vector: (self.wall_width / 3.0) * from.direction.into_unit_vector(),
            current_position: from.position,
        }
    }
}

/**
 *  Projects the `from` orientation onto the nearest wall, and gives the index of it
 *
 *  Loops starting at `from`, incrementing by a distance of `self.wall_width / 2.0` in the direction
 *  of `from` until a closed wall is found, then returns the index to that wall.
 *
 *  By incrementing by a distance of half the wall width, we are guaranteed to not skip over a wall.
 */
pub struct WallProjectionIterator<'a> {
    config: &'a MazeConfig,
    direction_vector: Vector,
    current_position: Vector,
}

impl<'a> Iterator for WallProjectionIterator<'a> {
    type Item = WallIndex;

    fn next(&mut self) -> Option<WallIndex> {
        // Get out of any walls we are currently in
        // Avoids returning the same wall for subsequent calls to `next`
        //while self.config.wall_on_point(self.current_position).is_some() {
        //self.current_position += self.direction_vector;
        //}

        // Keep going in the direction of `direction_vector` until an wall is found.
        loop {
            self.current_position += self.direction_vector;

            if let Some(wall_index) = self.config.wall_on_point(self.current_position) {
                break Some(wall_index);
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Wall {
    Open,
    Closed,
    Unknown,
}

impl Default for Wall {
    fn default() -> Wall {
        Wall::Unknown
    }
}

/// An index into a maze. This will uniquely identify any wall.
/// The indexes are 0-based, but do include the perimeter wall.
#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct WallIndex {
    /// The x index of the wall
    pub x: usize,

    /// The y index of the wall
    pub y: usize,

    /// Whether the wall is horizontal (true), or vertical (false)
    pub horizontal: bool,
}

/// Keeps track of all the walls in a maze
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Maze {
    horizontal_walls: [[Wall; HEIGHT - 1]; WIDTH],
    vertical_walls: [[Wall; HEIGHT]; WIDTH - 1],
}

impl Maze {
    pub fn new(wall: Wall) -> Maze {
        Maze {
            horizontal_walls: [[wall; HEIGHT - 1]; WIDTH],
            vertical_walls: [[wall; HEIGHT]; WIDTH - 1],
        }
    }

    pub fn from_walls(
        horizontal_walls: [[Wall; HEIGHT - 1]; WIDTH],
        vertical_walls: [[Wall; HEIGHT]; WIDTH - 1],
    ) -> Maze {
        Maze {
            horizontal_walls,
            vertical_walls,
        }
    }

    /**
     *  Reads files in the format described by
     *  http://www.micromouseonline.com/2018/01/31/micromouse-maze-file-collection/
     */
    pub fn from_file(bytes: [u8; WIDTH * HEIGHT]) -> Maze {
        let mut horizontal_walls = [[Wall::Unknown; HEIGHT - 1]; WIDTH];
        let mut vertical_walls = [[Wall::Unknown; HEIGHT]; WIDTH - 1];

        for (i, byte) in bytes.iter().enumerate() {
            let y = i % WIDTH;
            let x = i / WIDTH;

            let north = if byte & 0x01 == 0x01 {
                Wall::Closed
            } else {
                Wall::Open
            };
            let east = if byte & 0x02 == 0x02 {
                Wall::Closed
            } else {
                Wall::Open
            };

            if y < HEIGHT - 1 {
                horizontal_walls[x][y] = north;
            }

            if x < WIDTH - 1 {
                vertical_walls[x][y] = east;
            }
        }

        Maze {
            horizontal_walls,
            vertical_walls,
        }
    }

    pub fn get_cell(&self, x: usize, y: usize) -> (Wall, Wall, Wall, Wall) {
        let north_wall = if y >= HEIGHT - 1 {
            Wall::Closed
        } else {
            self.horizontal_walls[x][y]
        };

        let south_wall = if y <= 0 {
            Wall::Closed
        } else {
            self.horizontal_walls[x][y - 1]
        };

        let east_wall = if x >= WIDTH - 1 {
            Wall::Closed
        } else {
            self.vertical_walls[x][y]
        };

        let west_wall = if x <= 0 {
            Wall::Closed
        } else {
            self.vertical_walls[x - 1][y]
        };

        (north_wall, south_wall, east_wall, west_wall)
    }

    pub fn get_wall(&self, index: WallIndex) -> Option<&Wall> {
        if index.horizontal {
            if index.y == 0 {
                None
            } else {
                self.horizontal_walls
                    .get(index.x)
                    .and_then(|walls| walls.get(index.y - 1))
            }
        } else {
            if index.x == 0 {
                None
            } else {
                self.vertical_walls
                    .get(index.x - 1)
                    .and_then(|walls| walls.get(index.y))
            }
        }
    }
}
