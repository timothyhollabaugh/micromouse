use serde::Deserialize;
use serde::Serialize;

use libm::F32Ext;

use itertools::Itertools;

use crate::fast::{Orientation, Vector};

pub const WIDTH: usize = 16;
pub const HEIGHT: usize = 16;

#[derive(Debug, Copy, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct MazeConfig {
    pub cell_width: f32,
    pub wall_width: f32,
}

impl MazeConfig {
    /// Projects the `from` orientation onto the nearest wall or post, and gives the index of it
    pub fn wall_projection(
        &self,
        from: Orientation,
    ) -> impl Iterator<Item = MazeProjectionResult> + '_ {
        let mouse_cell_x = (from.position.x / self.cell_width) as usize;
        let mouse_cell_y = (from.position.y / self.cell_width) as usize;

        let direction_v = from.direction.into_unit_vector();

        let vertical_wall_range = if direction_v.x > 0.0 {
            itertools::Either::Left(mouse_cell_x + 1..=WIDTH)
        } else {
            itertools::Either::Right((0..=mouse_cell_x).rev())
        };

        let vertical_walls = vertical_wall_range.map(move |wall_index_x| {
            let wall_x = if wall_index_x <= mouse_cell_x {
                wall_index_x as f32 * self.cell_width + self.wall_width / 2.0
            } else {
                wall_index_x as f32 * self.cell_width - self.wall_width / 2.0
            };

            let t = (wall_x - from.position.x) / direction_v.x;

            let wall_y = t * direction_v.y + from.position.y;
            let wall_index_y = (wall_y / self.cell_width) as usize;

            // Figure out if we are looking at a wall or a post
            let local_y = wall_y % self.cell_width;
            let maze_index = if local_y < self.wall_width / 2.0 {
                MazeIndex::Post(wall_index_x, wall_index_y)
            } else if self.cell_width - local_y < self.wall_width / 2.0 {
                MazeIndex::Post(wall_index_x, wall_index_y + 1)
            } else {
                MazeIndex::Wall(WallIndex {
                    x: wall_index_x,
                    y: wall_index_y,
                    direction: WallDirection::Vertical,
                })
            };

            MazeProjectionResult {
                maze_index,
                hit_point: Vector {
                    x: wall_x,
                    y: wall_y,
                },
                distance: t,
                direction: WallDirection::Vertical,
            }
        });

        let horizontal_wall_range = if direction_v.y > 0.0 {
            itertools::Either::Left(mouse_cell_y + 1..=HEIGHT)
        } else {
            itertools::Either::Right((0..=mouse_cell_y).rev())
        };

        let horizontal_walls = horizontal_wall_range.map(move |wall_index_y| {
            let wall_y = if wall_index_y <= mouse_cell_y {
                wall_index_y as f32 * self.cell_width + self.wall_width / 2.0
            } else {
                wall_index_y as f32 * self.cell_width - self.wall_width / 2.0
            };

            let t = (wall_y - from.position.y) / direction_v.y;
            let wall_x = t * direction_v.x + from.position.x;
            let wall_index_x = (wall_x / self.cell_width) as usize;

            // Figure out if we are looking at a wall or a post
            let local_x = wall_x % self.cell_width;
            let maze_index = if local_x < self.wall_width / 2.0 {
                MazeIndex::Post(wall_index_x, wall_index_y)
            } else if self.cell_width - local_x < self.wall_width / 2.0 {
                MazeIndex::Post(wall_index_x + 1, wall_index_y)
            } else {
                MazeIndex::Wall(WallIndex {
                    x: wall_index_x,
                    y: wall_index_y,
                    direction: WallDirection::Horizontal,
                })
            };

            MazeProjectionResult {
                maze_index,
                hit_point: Vector {
                    x: wall_x,
                    y: wall_y,
                },
                distance: t,
                direction: WallDirection::Horizontal,
            }
        });

        vertical_walls
            .merge_by(horizontal_walls, |v, h| v.distance.abs() < h.distance.abs())
    }
}

#[cfg(test)]
mod wall_projection_tests {
    #[allow(unused_imports)]
    use crate::test::*;

    use core::f32::consts::{FRAC_PI_8, PI};

    use crate::fast::{Orientation, Vector};
    use crate::slow::maze::{MazeIndex, WallDirection, WallIndex};

    #[test]
    fn wall_projection_positive() {
        let mouse = Orientation {
            position: Vector {
                x: 180.0 * 6.5,
                y: 180.0 * 7.5,
            },
            direction: Direction::from(FRAC_PI_8),
        };

        let mut walls = MOUSE_MAZE_MAP.maze.wall_projection(mouse);

        let result = walls.next().unwrap();
        assert_eq!(
            result.maze_index,
            MazeIndex::Wall(WallIndex {
                x: 7,
                y: 7,
                direction: WallDirection::Vertical,
            }),
        );
        assert_eq!(result.direction, WallDirection::Vertical);
        assert_close2(
            result.hit_point,
            Vector {
                x: 1254.0,
                y: 1384.794,
            },
        );
        assert_close(result.distance, 90.92095);

        let result = walls.next().unwrap();
        assert_eq!(
            result.maze_index,
            MazeIndex::Wall(WallIndex {
                x: 7,
                y: 8,
                direction: WallDirection::Horizontal,
            }),
        );
        assert_eq!(result.direction, WallDirection::Horizontal);
        assert_close2(
            result.hit_point,
            Vector {
                x: 1372.7938,
                y: 1434.0,
            },
        );
        assert_close(result.distance, 219.5025);
    }

    #[test]
    fn wall_projection_negative() {
        let mouse = Orientation {
            position: Vector {
                x: 180.0 * 6.5,
                y: 180.0 * 7.5,
            },
            direction: Direction::from(FRAC_PI_8 + PI),
        };

        let mut walls = MOUSE_MAZE_MAP.maze.wall_projection(mouse);

        let result = walls.next().unwrap();
        assert_eq!(
            result.maze_index,
            MazeIndex::Wall(WallIndex {
                x: 6,
                y: 7,
                direction: WallDirection::Vertical,
            }),
        );
        assert_eq!(result.direction, WallDirection::Vertical);
        assert_close2(
            result.hit_point,
            Vector {
                x: 1086.0,
                y: 1315.206,
            },
        );
        assert_close(result.distance, 90.92095);

        let result = walls.next().unwrap();
        assert_eq!(
            result.maze_index,
            MazeIndex::Wall(WallIndex {
                x: 5,
                y: 7,
                direction: WallDirection::Horizontal,
            }),
        );
        assert_eq!(result.direction, WallDirection::Horizontal);
        assert_close2(
            result.hit_point,
            Vector {
                x: 967.20605,
                y: 1266.0,
            },
        );
        assert_close(result.distance, 219.50258);
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WallDirection {
    Horizontal,
    Vertical,
}

impl Default for WallDirection {
    fn default() -> WallDirection {
        WallDirection::Vertical
    }
}

/// The result of projecting an orientation into the maze.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MazeProjectionResult {
    /// Thw wall or post that was projected onto
    pub maze_index: MazeIndex,

    /// The point where the projection hit the surface of the wall or post
    pub hit_point: Vector,

    /// The distance from the start orientation to the hit point
    pub distance: f32,

    /// Whether it hit a horizontal or vertical surface
    pub direction: WallDirection,
}

/// Indexes into wither a wall or a post in a maze
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MazeIndex {
    /// This index is for a wall. It can be used to get the actual wall from the maze.
    Wall(WallIndex),

    /// This index is for a post, with (x, y) maze coordinates.
    Post(usize, usize),
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
    pub direction: WallDirection,
}

/// Keeps track of all the walls in a maze
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
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
        match index.direction {
            WallDirection::Horizontal => {
                if index.y == 0 {
                    None
                } else {
                    self.horizontal_walls
                        .get(index.x)
                        .and_then(|walls| walls.get(index.y - 1))
                }
            }
            WallDirection::Vertical => {
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
}
