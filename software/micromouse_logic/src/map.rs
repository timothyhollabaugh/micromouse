use libm::F32Ext;

use serde::Deserialize;
use serde::Serialize;

use crate::math::{
    Direction, Orientation, Vector, DIRECTION_0, DIRECTION_3_PI_2, DIRECTION_PI,
    DIRECTION_PI_2,
};

use heapless::ArrayLength;
use heapless::Vec;
use typenum::U1;
use typenum::U8;

use crate::config::MechanicalConfig;
use crate::maze::{
    Maze, MazeConfig, MazeIndex, MazeProjectionResult, Wall, WallDirection,
};
use itertools::Itertools;

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MapConfig {
    pub wall_threshold: u8,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MapDebug {
    //pub maze: Maze,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MoveOptions {
    left: bool,
    front: bool,
    right: bool,
}

/// Figures out what the maze is. For now, it will just tell you what of the three walls around are
/// open. Eventually, it will keep track of the entire maze.
pub struct Map {
    //maze: Maze,
}

impl Map {
    pub fn new() -> Map {
        Map {}
    }

    pub fn update(
        &mut self,
        mech: &MechanicalConfig,
        maze: &MazeConfig,
        config: &MapConfig,
        left_distance: u8,
        front_distance: u8,
        right_distance: u8,
    ) -> (MoveOptions, MapDebug) {
        let debug = MapDebug {
            //maze: self.maze.clone(),
        };

        (
            MoveOptions {
                left: left_distance <= config.wall_threshold,
                front: front_distance <= config.wall_threshold,
                right: right_distance <= config.wall_threshold,
            },
            debug,
        )
    }
}
