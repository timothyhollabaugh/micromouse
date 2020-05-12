use core::fmt::Debug;

use serde::{Deserialize, Serialize};

use super::map::MoveOptions;
use super::{MazeDirection, MazeOrientation};

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Move {
    Forward,
    Left,
    Right,
    Backward,
}

impl Default for Move {
    fn default() -> Self {
        Move::Forward
    }
}

impl Move {
    pub fn to_direction(&self, base_direction: MazeDirection) -> MazeDirection {
        match self {
            Move::Forward => base_direction,
            Move::Left => base_direction.left(),
            Move::Right => base_direction.right(),
            Move::Backward => base_direction.opposite(),
        }
    }
}

const CENTER_LEFT: [Move; 3] = [Move::Forward, Move::Left, Move::Right];
const CENTER_RIGHT: [Move; 3] = [Move::Forward, Move::Right, Move::Left];
const LEFT: [Move; 3] = [Move::Left, Move::Forward, Move::Right];
const RIGHT: [Move; 3] = [Move::Right, Move::Forward, Move::Left];

#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TwelvePartitionNavigateDebug {
    cells: [[u8; 16]; 16],
    next_move: Move,
    possibilities: [Move; 3],
}

pub struct TwelvePartitionNavigate {
    cells: [[u8; 16]; 16],
}

impl TwelvePartitionNavigate {
    pub fn new() -> TwelvePartitionNavigate {
        TwelvePartitionNavigate {
            cells: [[0; 16]; 16],
        }
    }

    fn get_cell(&self, x: i32, y: i32) -> u8 {
        if x >= 0 && x <= 15 && y >= 0 && y <= 15 {
            self.cells[x as usize][y as usize]
        } else {
            255
        }
    }

    pub fn navigate(
        &mut self,
        orientation: MazeOrientation,
        move_options: MoveOptions,
    ) -> (MazeDirection, TwelvePartitionNavigateDebug) {
        let x = orientation.position.x as i32;
        let y = orientation.position.y as i32;
        let ux = if x < 0 {
            0
        } else if x > 15 {
            15
        } else {
            x
        } as usize;
        let uy = if y < 0 {
            0
        } else if y > 15 {
            15
        } else {
            y
        } as usize;

        if self.cells[ux][uy] < 255 {
            self.cells[ux][uy] += 1;
        }

        // win condition
        //if x >= 7 && x <= 8 && y >= 7 && y <= 8 {
        //[Some(Move::TurnLeft), Some(Move::TurnLeft)]
        //} else {
        let left_cell = match orientation.direction {
            MazeDirection::North => self.get_cell(x - 1, y),
            MazeDirection::South => self.get_cell(x + 1, y),
            MazeDirection::East => self.get_cell(x, y + 1),
            MazeDirection::West => self.get_cell(x, y - 1),
        };

        let front_cell = match orientation.direction {
            MazeDirection::North => self.get_cell(x, y + 1),
            MazeDirection::South => self.get_cell(x, y - 1),
            MazeDirection::East => self.get_cell(x + 1, y),
            MazeDirection::West => self.get_cell(x - 1, y),
        };

        let right_cell = match orientation.direction {
            MazeDirection::North => self.get_cell(x + 1, y),
            MazeDirection::South => self.get_cell(x - 1, y),
            MazeDirection::East => self.get_cell(x, y - 1),
            MazeDirection::West => self.get_cell(x, y + 1),
        };

        /*
        let rear_cell = match orientation.direction {
            MazeDirection::North => self.get_cell(x, y - 1),
            MazeDirection::South => self.get_cell(x, y + 1),
            MazeDirection::East => self.get_cell(x - 1, y),
            MazeDirection::West => self.get_cell(x + 1, y),
        };
         */

        /*
        let front_blocked = front_cell == 255 || !move_options.front;
        let left_blocked = left_cell == 255 || !move_options.left;
        let right_blocked = right_cell == 255 || !move_options.right;
        let rear_blocked = rear_cell == 255;
         */

        /*
        let num_blocked = [front_blocked, left_blocked, right_blocked, rear_blocked]
            .iter()
            .filter(|&c| *c)
            .count();

        if num_blocked == 3 {
            self.cells[ux][uy] = 255;
        }
        */

        let possibilities = match (x, y) {
            (x, y) if x < 7 && y < 7 => match orientation.direction {
                MazeDirection::North => CENTER_RIGHT,
                MazeDirection::South => LEFT,
                MazeDirection::East => CENTER_LEFT,
                MazeDirection::West => RIGHT,
            },

            (x, y) if x > 8 && y < 7 => match orientation.direction {
                MazeDirection::North => CENTER_LEFT,
                MazeDirection::South => RIGHT,
                MazeDirection::East => LEFT,
                MazeDirection::West => CENTER_RIGHT,
            },

            (x, y) if x > 8 && y > 8 => match orientation.direction {
                MazeDirection::North => LEFT,
                MazeDirection::South => CENTER_RIGHT,
                MazeDirection::East => RIGHT,
                MazeDirection::West => CENTER_LEFT,
            },

            (x, y) if x < 7 && y > 8 => match orientation.direction {
                MazeDirection::North => RIGHT,
                MazeDirection::South => CENTER_LEFT,
                MazeDirection::East => CENTER_RIGHT,
                MazeDirection::West => LEFT,
            },

            (7, y) if y < 7 => match orientation.direction {
                MazeDirection::North => CENTER_RIGHT,
                MazeDirection::South => LEFT,
                MazeDirection::East => LEFT,
                MazeDirection::West => RIGHT,
            },

            (8, y) if y < 7 => match orientation.direction {
                MazeDirection::North => CENTER_LEFT,
                MazeDirection::South => RIGHT,
                MazeDirection::East => LEFT,
                MazeDirection::West => RIGHT,
            },

            (x, 7) if x > 8 => match orientation.direction {
                MazeDirection::North => LEFT,
                MazeDirection::South => RIGHT,
                MazeDirection::East => CENTER_RIGHT,
                MazeDirection::West => LEFT,
            },

            (x, 8) if x > 8 => match orientation.direction {
                MazeDirection::North => LEFT,
                MazeDirection::South => RIGHT,
                MazeDirection::East => CENTER_LEFT,
                MazeDirection::West => RIGHT,
            },

            (8, y) if y > 8 => match orientation.direction {
                MazeDirection::North => LEFT,
                MazeDirection::South => CENTER_RIGHT,
                MazeDirection::East => RIGHT,
                MazeDirection::West => LEFT,
            },

            (7, y) if y > 8 => match orientation.direction {
                MazeDirection::North => RIGHT,
                MazeDirection::South => CENTER_LEFT,
                MazeDirection::East => RIGHT,
                MazeDirection::West => LEFT,
            },

            (x, 8) if x < 7 => match orientation.direction {
                MazeDirection::North => RIGHT,
                MazeDirection::South => LEFT,
                MazeDirection::East => CENTER_RIGHT,
                MazeDirection::West => LEFT,
            },

            (x, 7) if x < 7 => match orientation.direction {
                MazeDirection::North => RIGHT,
                MazeDirection::South => LEFT,
                MazeDirection::East => CENTER_LEFT,
                MazeDirection::West => RIGHT,
            },

            (x, y) if x >= 7 && x <= 8 && y >= 7 && y <= 8 => CENTER_RIGHT,

            (_, _) => panic!("Invalid location!"),
        };

        let mut next_move = Move::Backward;

        // filter by walls
        let possibilities_iter = possibilities.iter().filter(|&moves| match moves {
            &Move::Forward => move_options.front,
            &Move::Left => move_options.left,
            &Move::Right => move_options.right,
            _ => true,
        });

        let &min = [
            if move_options.front { front_cell } else { 255 },
            if move_options.left { left_cell } else { 255 },
            if move_options.right { right_cell } else { 255 },
        ]
        .iter()
        .min()
        .unwrap();

        for &possible_move in possibilities_iter {
            let value = match possible_move {
                Move::Forward => front_cell,
                Move::Left => left_cell,
                Move::Right => right_cell,
                _ => 0,
            };

            if value == min {
                next_move = possible_move;
                break;
            }
        }

        let direction = next_move.to_direction(orientation.direction);

        (
            direction,
            TwelvePartitionNavigateDebug {
                cells: self.cells,
                next_move,
                possibilities,
            },
        )
        //}
    }
}
