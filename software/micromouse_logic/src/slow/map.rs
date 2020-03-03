use serde::Deserialize;
use serde::Serialize;

use crate::config::MechanicalConfig;
use crate::fast::Orientation;
use crate::slow::maze::MazeConfig;

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MapConfig {
    pub front_threhold: u8,
    pub left_threshold: u8,
    pub right_threshold: u8,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MapDebug {
    //pub maze: Maze,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MoveOptions {
    pub left: bool,
    pub front: bool,
    pub right: bool,
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
        _mech: &MechanicalConfig,
        _maze: &MazeConfig,
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
                left: left_distance >= config.left_threshold,
                front: front_distance >= config.front_threhold,
                right: right_distance >= config.right_threshold,
            },
            debug,
        )
    }
}
