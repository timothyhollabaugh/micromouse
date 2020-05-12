use serde::Deserialize;
use serde::Serialize;

use crate::config::MechanicalConfig;
use crate::mouse::DistanceReading;
use crate::slow::maze::{Maze, MazeConfig, Wall, WallIndex};
use crate::slow::MazeOrientation;

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MapConfig {
    pub front_threhold: f32,
    pub left_threshold: f32,
    pub right_threshold: f32,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MapDebug {
    pub maze: Maze,
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
    maze: Maze,
    left_distance: Option<DistanceReading>,
    right_distance: Option<DistanceReading>,
    front_distance: Option<DistanceReading>,
}

impl Map {
    pub fn new() -> Map {
        Map {
            maze: Maze::new(Wall::Unknown),
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
        maze_orientation: MazeOrientation,
        left_distance: Option<DistanceReading>,
        front_distance: Option<DistanceReading>,
        right_distance: Option<DistanceReading>,
    ) -> (Option<MoveOptions>, MapDebug) {
        let debug = MapDebug {
            maze: self.maze.clone(),
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
                self.left_distance = None;
                self.right_distance = None;
                self.front_distance = None;

                let options = MoveOptions {
                    left: left_distance >= config.left_threshold,
                    front: front_distance >= config.front_threhold,
                    right: right_distance >= config.right_threshold,
                };

                let front_index = WallIndex::from_maze_orientation(maze_orientation);
                let front_wall = if options.front {
                    Wall::Open
                } else {
                    Wall::Closed
                };
                self.maze.set_wall(front_index, front_wall);

                let left_index = WallIndex::from_maze_orientation(MazeOrientation {
                    direction: maze_orientation.direction.left(),
                    ..maze_orientation
                });
                let left_wall = if options.left {
                    Wall::Open
                } else {
                    Wall::Closed
                };
                self.maze.set_wall(left_index, left_wall);

                let right_index = WallIndex::from_maze_orientation(MazeOrientation {
                    direction: maze_orientation.direction.right(),
                    ..maze_orientation
                });
                let right_wall = if options.right {
                    Wall::Open
                } else {
                    Wall::Closed
                };
                self.maze.set_wall(right_index, right_wall);

                Some(options)
            } else {
                None
            };

        (move_options, debug)
    }
}
