use serde::Deserialize;
use serde::Serialize;

use crate::math::Orientation;

use crate::config::MechanicalConfig;
use crate::maze::Wall;
use crate::maze::{Maze, MazeProjectionResult};
use crate::maze::{MazeConfig, MazeIndex};

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MapConfig {
    pub maze: MazeConfig,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MapDebug {
    pub maze: Maze,
    pub front_wall: Option<MazeProjectionResult>,
    pub left_wall: Option<MazeProjectionResult>,
    pub right_wall: Option<MazeProjectionResult>,
}

/// Find the closest closed wall
fn find_closed_wall(
    config: &MazeConfig,
    maze: &Maze,
    from: Orientation,
) -> Option<MazeProjectionResult> {
    config.wall_projection(from).find(|maze_projection_result| {
        if let MazeIndex::Wall(wall_index) = maze_projection_result.maze_index {
            maze.get_wall(wall_index).unwrap_or(&Wall::Closed) == &Wall::Closed
        } else {
            true
        }
    })
}

pub struct Map {
    orientation: Orientation,
    maze: Maze,
    left_encoder: i32,
    right_encoder: i32,
}

impl Map {
    pub fn new(orientation: Orientation, left_encoder: i32, right_encoder: i32) -> Map {
        let mut horizontal_walls =
            [[Wall::Unknown; crate::maze::HEIGHT - 1]; crate::maze::WIDTH];
        let mut vertical_walls =
            [[Wall::Unknown; crate::maze::HEIGHT]; crate::maze::WIDTH - 1];

        horizontal_walls[6][8] = Wall::Closed;
        horizontal_walls[7][8] = Wall::Closed;
        horizontal_walls[8][8] = Wall::Closed;
        horizontal_walls[9][8] = Wall::Closed;

        horizontal_walls[6][7] = Wall::Open;
        horizontal_walls[7][7] = Wall::Closed;
        horizontal_walls[8][7] = Wall::Closed;
        horizontal_walls[9][7] = Wall::Open;

        horizontal_walls[6][6] = Wall::Open;
        horizontal_walls[7][6] = Wall::Closed;
        horizontal_walls[8][6] = Wall::Closed;
        horizontal_walls[9][6] = Wall::Open;

        horizontal_walls[6][5] = Wall::Closed;
        horizontal_walls[7][5] = Wall::Closed;
        horizontal_walls[8][5] = Wall::Closed;
        horizontal_walls[9][5] = Wall::Closed;

        vertical_walls[5][8] = Wall::Closed;
        vertical_walls[5][7] = Wall::Closed;
        vertical_walls[5][6] = Wall::Closed;

        vertical_walls[6][8] = Wall::Open;
        vertical_walls[6][7] = Wall::Closed;
        vertical_walls[6][6] = Wall::Open;

        vertical_walls[7][8] = Wall::Open;
        vertical_walls[7][7] = Wall::Open;
        vertical_walls[7][6] = Wall::Open;

        vertical_walls[8][8] = Wall::Open;
        vertical_walls[8][7] = Wall::Closed;
        vertical_walls[8][6] = Wall::Open;

        vertical_walls[9][8] = Wall::Closed;
        vertical_walls[9][7] = Wall::Closed;
        vertical_walls[9][6] = Wall::Closed;

        let maze = Maze::from_walls(horizontal_walls, vertical_walls);

        Map {
            orientation,
            left_encoder,
            right_encoder,
            maze,
        }
    }

    pub fn update(
        &mut self,
        mech: &MechanicalConfig,
        maze_config: &MazeConfig,
        left_encoder: i32,
        right_encoder: i32,
        _left_distance: u8,
        _front_distance: u8,
        _right_distance: u8,
    ) -> (Orientation, MapDebug) {
        let delta_left = left_encoder - self.left_encoder;
        let delta_right = right_encoder - self.right_encoder;

        self.orientation
            .update_from_encoders(&mech, delta_left, delta_right);

        self.left_encoder = left_encoder;
        self.right_encoder = right_encoder;

        let front_wall = find_closed_wall(
            maze_config,
            &self.maze,
            self.orientation.offset(mech.front_sensor_orientation),
        );

        let left_wall = find_closed_wall(
            maze_config,
            &self.maze,
            self.orientation.offset(mech.left_sensor_orientation),
        );

        let right_wall = find_closed_wall(
            maze_config,
            &self.maze,
            self.orientation.offset(mech.right_sensor_orientation),
        );

        let debug = MapDebug {
            maze: self.maze.clone(),
            front_wall,
            left_wall,
            right_wall,
        };

        (self.orientation, debug)
    }
}

fn update_orientation_from_sensors(
    approx_orientation: Orientation,
    front_wall: Option<MazeProjectionResult>,
    front_distance: u8,
    left_wall: Option<MazeProjectionResult>,
    left_distance: u8,
    right_wall: Option<MazeProjectionResult>,
    right_distance: u8,
) -> Orientation {
    unimplemented!()
}
