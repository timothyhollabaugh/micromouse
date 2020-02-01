use serde::Deserialize;
use serde::Serialize;

use crate::math::Orientation;

use crate::config::MechanicalConfig;
use crate::maze::Maze;
use crate::maze::MazeConfig;
use crate::maze::Wall;
use crate::maze::WallIndex;

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MapConfig {
    pub maze: MazeConfig,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MapDebug {
    pub maze: Maze,
    pub front_wall: Option<WallIndex>,
    pub left_wall: Option<WallIndex>,
    pub right_wall: Option<WallIndex>,
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
        mech_config: &MechanicalConfig,
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
            .update_from_encoders(&mech_config, delta_left, delta_right);

        self.left_encoder = left_encoder;
        self.right_encoder = right_encoder;

        /*
        let front_edge = maze_config
            .edge_projection_iter(self.orientation)
            .find(|edge_index| {
                *self.maze.get_edge(*edge_index).unwrap_or(&Edge::Closed)
                    == Edge::Closed
            });

        let left_distance_orientation = Orientation {
            position: self.orientation.position,
            direction: self.orientation.direction + DIRECTION_PI_2,
        };

        let left_edge = maze_config
            .edge_projection_iter(left_distance_orientation)
            .find(|edge_index| {
                *self.maze.get_edge(*edge_index).unwrap_or(&Edge::Closed)
                    == Edge::Closed
            });

        let right_distance_orientation = Orientation {
            position: self.orientation.position,
            direction: self.orientation.direction - DIRECTION_PI_2,
        };

        let right_edge = maze_config
            .edge_projection_iter(right_distance_orientation)
            .find(|edge_index| {
                *self.maze.get_edge(*edge_index).unwrap_or(&Edge::Closed)
                    == Edge::Closed
            });
            */

        let debug = MapDebug {
            maze: self.maze.clone(),
            front_wall: None,
            left_wall: None,
            right_wall: None,
        };

        (self.orientation, debug)
    }
}
