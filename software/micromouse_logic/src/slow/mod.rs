//! All of the non-realtime stuff that does not really matter how fast it runs
//!
//! Includes mapping and navigating

pub mod map;
pub mod maze;
pub mod motion_plan;
pub mod navigate;

use serde::{Deserialize, Serialize};

use crate::fast::{
    Direction, Vector, DIRECTION_0, DIRECTION_3_PI_2, DIRECTION_PI, DIRECTION_PI_2,
};

use crate::slow::map::{MapDebug, MoveOptions};
use crate::slow::navigate::TwelvePartitionNavigateDebug;
use maze::MazeConfig;

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SlowDebug {
    pub map: MapDebug,
    pub move_options: MoveOptions,
    pub navigate: TwelvePartitionNavigateDebug,
    pub next_direction: MazeDirection,
}

#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub enum MazeDirection {
    North,
    South,
    East,
    West,
}

impl MazeDirection {
    pub fn into_direction(self) -> Direction {
        match self {
            MazeDirection::North => DIRECTION_PI_2,
            MazeDirection::South => DIRECTION_3_PI_2,
            MazeDirection::East => DIRECTION_0,
            MazeDirection::West => DIRECTION_PI,
        }
    }

    pub fn opposite(self) -> MazeDirection {
        match self {
            MazeDirection::North => MazeDirection::South,
            MazeDirection::South => MazeDirection::North,
            MazeDirection::East => MazeDirection::West,
            MazeDirection::West => MazeDirection::East,
        }
    }

    pub fn left(self) -> MazeDirection {
        match self {
            MazeDirection::North => MazeDirection::West,
            MazeDirection::West => MazeDirection::South,
            MazeDirection::South => MazeDirection::East,
            MazeDirection::East => MazeDirection::North,
        }
    }

    pub fn right(self) -> MazeDirection {
        match self {
            MazeDirection::North => MazeDirection::East,
            MazeDirection::East => MazeDirection::South,
            MazeDirection::South => MazeDirection::West,
            MazeDirection::West => MazeDirection::North,
        }
    }
}

impl Default for MazeDirection {
    fn default() -> Self {
        MazeDirection::East
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct MazePosition {
    pub x: usize,
    pub y: usize,
}

impl MazePosition {
    pub fn center_position(self, config: &MazeConfig) -> Vector {
        Vector {
            x: self.x as f32 * config.cell_width + config.cell_width / 2.0,
            y: self.y as f32 * config.cell_width + config.cell_width / 2.0,
        }
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct MazeOrientation {
    pub position: MazePosition,
    pub direction: MazeDirection,
}
