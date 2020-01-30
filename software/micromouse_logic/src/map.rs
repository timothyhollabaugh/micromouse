use serde::Deserialize;
use serde::Serialize;

use crate::math::Orientation;
use crate::math::DIRECTION_PI_2;

use crate::config::MechanicalConfig;
use crate::maze::Edge;
use crate::maze::EdgeIndex;
use crate::maze::Maze;
use crate::maze::MazeConfig;

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MapConfig {
    pub maze: MazeConfig,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MapDebug {
    pub maze: Maze,
    pub front_edge: Option<EdgeIndex>,
    pub left_edge: Option<EdgeIndex>,
    pub right_edge: Option<EdgeIndex>,
}

pub struct Map {
    orientation: Orientation,
    maze: Maze,
    left_encoder: i32,
    right_encoder: i32,
}

impl Map {
    pub fn new(orientation: Orientation, left_encoder: i32, right_encoder: i32) -> Map {
        let mut horizontal_edges =
            [[Edge::Unknown; crate::maze::HEIGHT - 1]; crate::maze::WIDTH];
        let mut vertical_edges =
            [[Edge::Unknown; crate::maze::HEIGHT]; crate::maze::WIDTH - 1];

        horizontal_edges[6][8] = Edge::Closed;
        horizontal_edges[7][8] = Edge::Closed;
        horizontal_edges[8][8] = Edge::Closed;
        horizontal_edges[9][8] = Edge::Closed;

        horizontal_edges[6][7] = Edge::Open;
        horizontal_edges[7][7] = Edge::Closed;
        horizontal_edges[8][7] = Edge::Closed;
        horizontal_edges[9][7] = Edge::Open;

        horizontal_edges[6][6] = Edge::Open;
        horizontal_edges[7][6] = Edge::Closed;
        horizontal_edges[8][6] = Edge::Closed;
        horizontal_edges[9][6] = Edge::Open;

        horizontal_edges[6][5] = Edge::Closed;
        horizontal_edges[7][5] = Edge::Closed;
        horizontal_edges[8][5] = Edge::Closed;
        horizontal_edges[9][5] = Edge::Closed;

        vertical_edges[5][8] = Edge::Closed;
        vertical_edges[5][7] = Edge::Closed;
        vertical_edges[5][6] = Edge::Closed;

        vertical_edges[6][8] = Edge::Open;
        vertical_edges[6][7] = Edge::Closed;
        vertical_edges[6][6] = Edge::Open;

        vertical_edges[7][8] = Edge::Open;
        vertical_edges[7][7] = Edge::Open;
        vertical_edges[7][6] = Edge::Open;

        vertical_edges[8][8] = Edge::Open;
        vertical_edges[8][7] = Edge::Closed;
        vertical_edges[8][6] = Edge::Open;

        vertical_edges[9][8] = Edge::Closed;
        vertical_edges[9][7] = Edge::Closed;
        vertical_edges[9][6] = Edge::Closed;

        let maze = Maze::from_edges(horizontal_edges, vertical_edges);

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
            front_edge: None,
            left_edge: None,
            right_edge: None,
        };

        (self.orientation, debug)
    }
}
