use libm::F32Ext;

use serde::Deserialize;
use serde::Serialize;

use crate::math::{Direction, Orientation, Vector};

use crate::config::MechanicalConfig;
use crate::maze::{Maze, MazeProjectionResult};
use crate::maze::{MazeConfig, MazeIndex};
use crate::maze::{Wall, WallDirection};

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
        left_distance: u8,
        front_distance: u8,
        right_distance: u8,
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

fn update_orientation_from_distances(
    approx_orientation: Orientation,
    front_result: Option<MazeProjectionResult>,
    front_distance: Option<f32>,
    left_result: Option<MazeProjectionResult>,
    left_distance: Option<f32>,
    right_result: Option<MazeProjectionResult>,
    right_distance: Option<f32>,
) -> Orientation {
    match (
        (left_result, left_distance),
        (front_result, front_distance),
        (right_result, right_distance),
    ) {
        // HVH
        (
            (Some(left_result), Some(left_distance)),
            (Some(front_result), Some(front_distance)),
            (Some(right_result), Some(right_distance)),
        ) if left_result.direction == WallDirection::Horizontal
            && front_result.direction == WallDirection::Vertical
            && right_result.direction == WallDirection::Horizontal =>
        {
            let cos_theta = (left_result.hit_point.y - right_result.hit_point.y)
                / (left_distance + right_distance);

            Orientation {
                position: Vector {
                    x: front_result.hit_point.x - front_distance * cos_theta,
                    y: left_result.hit_point.y - left_distance * cos_theta,
                },
                direction: Direction::from(F32Ext::acos(cos_theta)),
            }
        }

        // VHV
        (
            (Some(left_result), Some(left_distance)),
            (Some(front_result), Some(front_distance)),
            (Some(right_result), Some(right_distance)),
        ) if left_result.direction == WallDirection::Vertical
            && front_result.direction == WallDirection::Horizontal
            && right_result.direction == WallDirection::Vertical =>
        {
            let sin_theta = -(left_result.hit_point.x - right_result.hit_point.x)
                / (left_distance + right_distance);

            Orientation {
                position: Vector {
                    x: left_result.hit_point.x + left_distance * sin_theta,
                    y: front_result.hit_point.y - front_distance * sin_theta,
                },
                direction: Direction::from(F32Ext::asin(sin_theta)),
            }
        }

        _ => approx_orientation,
    }
}

#[cfg(test)]
mod test_update_orientation_from_distance {
    #[allow(unused_imports)]
    use crate::test;

    use crate::map::update_orientation_from_distances;
    use crate::math::{Orientation, Vector, DIRECTION_PI_2};
    use crate::maze::{MazeIndex, MazeProjectionResult, WallDirection, WallIndex};
    use crate::test::{assert_close, assert_close2};

    #[test]
    fn hvh() {
        let approx_orientation = Orientation {
            position: Vector { x: 90.0, y: 90.0 },
            direction: DIRECTION_PI_2 / 4.0,
        };

        let actual_mouse = Orientation {
            position: Vector { x: 90.0, y: 80.0 },
            direction: DIRECTION_PI_2 / 4.0,
        };

        let left_hit_point = Vector {
            x: 51.0639251369291,
            y: 174.0,
        };
        let left_distance = (left_hit_point - actual_mouse.position).magnitude();

        let front_hit_point = Vector {
            x: 174.0,
            y: 114.79393923934,
        };
        let front_distance = (front_hit_point - actual_mouse.position).magnitude();

        let right_hit_point = Vector {
            x: 120.651803615609,
            y: 6.0,
        };
        let right_distance = (right_hit_point - actual_mouse.position).magnitude();

        let left_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 0,
                y: 1,
                direction: WallDirection::Horizontal,
            }),
            hit_point: left_hit_point,
            distance: left_distance,
            direction: WallDirection::Horizontal,
        };

        let front_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 1,
                y: 0,
                direction: WallDirection::Vertical,
            }),
            hit_point: front_hit_point,
            distance: front_distance,
            direction: WallDirection::Vertical,
        };

        let right_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 0,
                y: 0,
                direction: WallDirection::Horizontal,
            }),
            hit_point: right_hit_point,
            distance: right_distance,
            direction: WallDirection::Horizontal,
        };

        let result_orientation = update_orientation_from_distances(
            approx_orientation,
            Some(front_result),
            Some(front_distance),
            Some(left_result),
            Some(left_distance),
            Some(right_result),
            Some(right_distance),
        );

        assert_close2(result_orientation.position, actual_mouse.position);
        assert_close(
            f32::from(result_orientation.direction),
            f32::from(actual_mouse.direction),
        );
    }

    #[test]
    fn vhv() {
        let approx_orientation = Orientation {
            position: Vector { x: 90.0, y: 90.0 },
            direction: DIRECTION_PI_2 - DIRECTION_PI_2 / 4.0,
        };

        let actual_mouse = Orientation {
            position: Vector { x: 90.0, y: 80.0 },
            direction: DIRECTION_PI_2 - DIRECTION_PI_2 / 4.0,
        };

        let left_hit_point = Vector {
            x: 6.0,
            y: 114.79393923934,
        };
        let left_distance = (left_hit_point - actual_mouse.position).magnitude();

        let front_hit_point = Vector {
            x: 128.936074863071,
            y: 174.0,
        };
        let front_distance = (front_hit_point - actual_mouse.position).magnitude();

        let right_hit_point = Vector {
            x: 174.0,
            y: 45.20606076066,
        };
        let right_distance = (right_hit_point - actual_mouse.position).magnitude();

        let left_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 0,
                y: 1,
                direction: WallDirection::Vertical,
            }),
            hit_point: left_hit_point,
            distance: left_distance,
            direction: WallDirection::Vertical,
        };

        let front_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 1,
                y: 0,
                direction: WallDirection::Horizontal,
            }),
            hit_point: front_hit_point,
            distance: front_distance,
            direction: WallDirection::Horizontal,
        };

        let right_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 0,
                y: 0,
                direction: WallDirection::Vertical,
            }),
            hit_point: right_hit_point,
            distance: right_distance,
            direction: WallDirection::Vertical,
        };

        let result_orientation = update_orientation_from_distances(
            approx_orientation,
            Some(front_result),
            Some(front_distance),
            Some(left_result),
            Some(left_distance),
            Some(right_result),
            Some(right_distance),
        );

        assert_close2(result_orientation.position, actual_mouse.position);
        assert_close(
            f32::from(result_orientation.direction),
            f32::from(actual_mouse.direction),
        );
    }
}
