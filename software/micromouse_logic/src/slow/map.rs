use serde::Deserialize;
use serde::Serialize;

use crate::config::MechanicalConfig;
use crate::mouse::DistanceReading;
use crate::slow::maze::MazeConfig;

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MapConfig {
    pub front_threhold: f32,
    pub left_threshold: f32,
    pub right_threshold: f32,
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
    left_distance: Option<DistanceReading>,
    right_distance: Option<DistanceReading>,
    front_distance: Option<DistanceReading>,
}

impl Map {
    pub fn new() -> Map {
        Map {
            left_distance: None,
            right_distance: None,
            front_distance: None,
        }
    }

    pub fn update(
        &mut self,
        _mech: &MechanicalConfig,
        _maze: &MazeConfig,
        config: &MapConfig,
        left_distance: Option<DistanceReading>,
        front_distance: Option<DistanceReading>,
        right_distance: Option<DistanceReading>,
    ) -> (Option<MoveOptions>, MapDebug) {
        let debug = MapDebug {
            //maze: self.maze.clone(),
        };

        if left_distance != None {
            self.left_distance = left_distance
        }

        if right_distance != None {
            self.right_distance = right_distance
        }

        if front_distance != None {
            self.front_distance = front_distance
        }

        let move_options =
            if let (Some(left_distance), Some(right_distance), Some(front_distance)) =
                (left_distance, right_distance, front_distance)
            {
                Some(MoveOptions {
                    left: left_distance >= config.left_threshold,
                    front: front_distance >= config.front_threhold,
                    right: right_distance >= config.right_threshold,
                })
            } else {
                None
            };

        (move_options, debug)
    }
}
