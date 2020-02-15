use libm::F32Ext;

use serde::Deserialize;
use serde::Serialize;

use crate::math::{
    Direction, Orientation, Vector, DIRECTION_0, DIRECTION_3_PI_2, DIRECTION_PI,
    DIRECTION_PI_2,
};

use crate::config::MechanicalConfig;
use crate::maze::{
    Maze, MazeConfig, MazeIndex, MazeProjectionResult, Wall, WallDirection,
};

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MapConfig {
    pub maze: MazeConfig,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MapDebug {
    pub maze: Maze,
    pub front_result: Option<MazeProjectionResult>,
    pub left_result: Option<MazeProjectionResult>,
    pub right_result: Option<MazeProjectionResult>,
    pub delta_position: Vector,
    pub approx_sensor_orientation: Orientation,
    pub left_distance: Option<f32>,
    pub front_distance: Option<f32>,
    pub right_distance: Option<f32>,
    pub adjusted_orientation: Orientation,
}

/// Find the closest closed wall
pub fn find_closed_wall(
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

/// Makes sure the distance reading is in range and not too far from the expected result
fn cleanup_distance_reading(
    offset: f32,
    limit: f32,
    tolerance: f32,
    distance: u8,
    result: Option<MazeProjectionResult>,
) -> Option<f32> {
    let distance = distance as f32;

    let distance = if distance <= limit {
        Some(distance + offset)
    } else {
        None
    };

    let distance = if let (Some(distance), Some(result)) = (distance, result) {
        if (distance - result.distance).abs() < tolerance {
            Some(distance)
        } else {
            None
        }
    } else {
        None
    };

    distance
}

pub struct Map {
    orientation: Orientation,
    delta_position: Vector,
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
            delta_position: Vector { x: 0.0, y: 0.0 },
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

        let encoder_orientation =
            self.orientation
                .update_from_encoders(&mech, delta_left, delta_right);

        self.left_encoder = left_encoder;
        self.right_encoder = right_encoder;

        let front_result = find_closed_wall(
            maze_config,
            &self.maze,
            encoder_orientation.offset(mech.front_sensor_orientation()),
        )
        .map(|front_result| MazeProjectionResult {
            distance: front_result.distance + mech.front_sensor_offset,
            ..front_result
        });

        let left_result = find_closed_wall(
            maze_config,
            &self.maze,
            encoder_orientation.offset(mech.left_sensor_orientation()),
        )
        .map(|left_result| MazeProjectionResult {
            distance: left_result.distance + mech.left_sensor_offset,
            ..left_result
        });

        let right_result = find_closed_wall(
            maze_config,
            &self.maze,
            encoder_orientation.offset(mech.right_sensor_orientation()),
        )
        .map(|right_result| MazeProjectionResult {
            distance: right_result.distance + mech.right_sensor_offset,
            ..right_result
        });

        let front_distance = cleanup_distance_reading(
            mech.front_sensor_offset,
            mech.front_sensor_limit as f32,
            maze_config.cell_width / 2.0,
            front_distance,
            front_result,
        );

        let left_distance = cleanup_distance_reading(
            mech.left_sensor_offset,
            mech.left_sensor_limit as f32,
            maze_config.cell_width / 2.0,
            left_distance,
            left_result,
        );

        let right_distance = cleanup_distance_reading(
            mech.right_sensor_offset,
            mech.right_sensor_limit as f32,
            maze_config.cell_width / 2.0,
            right_distance,
            right_result,
        );

        let approx_sensor_orientation = encoder_orientation.offset(Orientation {
            position: Vector {
                x: mech.sensor_center_offset,
                y: 0.0,
            },
            direction: DIRECTION_0,
        });

        let adjusted_orientation = update_orientation_from_distances(
            self.delta_position,
            approx_sensor_orientation,
            front_result,
            front_distance,
            left_result,
            left_distance,
            right_result,
            right_distance,
        );

        let orientation = adjusted_orientation.offset(Orientation {
            position: Vector {
                x: -mech.sensor_center_offset,
                y: 0.0,
            },
            direction: DIRECTION_0,
        });

        let debug = MapDebug {
            maze: self.maze.clone(),
            front_result,
            left_result,
            right_result,
            delta_position: self.delta_position,
            approx_sensor_orientation,
            adjusted_orientation,
            left_distance,
            front_distance,
            right_distance,
        };

        self.delta_position = orientation.position - self.orientation.position;
        self.orientation = orientation;

        (self.orientation, debug)
    }
}

fn h_h_direction(
    delta_position: Vector,
    approx_orientation: Orientation,
    left_wall: f32,
    right_wall: f32,
    left_distance: f32,
    right_distance: f32,
) -> (f32, Direction) {
    let mut cos_theta = (left_wall - right_wall) / (left_distance + right_distance);

    if cos_theta >= 1.0 {
        cos_theta = 1.0
    } else if cos_theta <= -1.0 {
        cos_theta = -1.0
    }

    let direction = Direction::from(F32Ext::acos(cos_theta));

    let direction = if delta_position.y > 0.0 {
        direction
    } else if delta_position.y < 0.0 {
        -direction
    } else {
        if approx_orientation.direction.close(&DIRECTION_0) {
            DIRECTION_0
        } else if approx_orientation.direction.close(&DIRECTION_PI) {
            DIRECTION_PI
        } else if approx_orientation.direction > DIRECTION_PI {
            -direction
        } else {
            direction
        }
    };

    (cos_theta, direction)
}

#[cfg(test)]
mod test_h_h_direction {
    #[allow(unused_imports)]
    use crate::test::*;

    use crate::map::h_h_direction;
    use crate::math::{Direction, Orientation, Vector, DIRECTION_0, DIRECTION_PI};
    use core::f32::consts::FRAC_PI_8;

    #[test]
    fn positive_approx_direction() {
        let approx_orientation = Orientation {
            position: Vector { x: 90.0, y: 90.0 },
            direction: Direction::from(FRAC_PI_8),
        };

        let (cos_theta, direction) = h_h_direction(
            Vector { x: 0.0, y: 0.0 },
            approx_orientation,
            174.0,
            6.0,
            90.92,
            90.92,
        );

        assert_close(f32::from(direction), f32::from(0.39267397));
        assert_close(cos_theta, 0.92388);
    }

    #[test]
    fn negative_approx_direction() {
        let approx_orientation = Orientation {
            position: Vector { x: 90.0, y: 90.0 },
            direction: -Direction::from(FRAC_PI_8),
        };

        let (cos_theta, direction) = h_h_direction(
            Vector { x: 0.0, y: 0.0 },
            approx_orientation,
            174.0,
            6.0,
            90.92,
            90.92,
        );

        assert_close(f32::from(direction), f32::from(5.890511));
        assert_close(cos_theta, 0.92388);
    }

    #[test]
    fn zero_approx_direction() {
        let delta_position = Vector { x: 5.0, y: 1.0 };

        let approx_orientation = Orientation {
            position: Vector { x: 90.0, y: 90.0 },
            direction: DIRECTION_0,
        };

        let (cos_theta, direction) =
            h_h_direction(delta_position, approx_orientation, 174.0, 6.0, 90.92, 90.92);

        assert_close(f32::from(direction), f32::from(0.39267397));
        assert_close(cos_theta, 0.92388);
    }

    #[test]
    fn pi_approx_direction() {
        let delta_position = Vector { x: 5.0, y: 1.0 };

        let approx_orientation = Orientation {
            position: Vector { x: 90.0, y: 90.0 },
            direction: DIRECTION_PI,
        };

        let (cos_theta, direction) =
            h_h_direction(delta_position, approx_orientation, 174.0, 6.0, 90.92, 90.92);

        assert_close(f32::from(direction), f32::from(0.39267397));
        assert_close(cos_theta, 0.92388);
    }
}

fn v_v_direction(
    delta_position: Vector,
    approx_orientation: Orientation,
    left_wall: f32,
    right_wall: f32,
    left_distance: f32,
    right_distance: f32,
) -> (f32, Direction) {
    let mut sin_theta = (right_wall - left_wall) / (left_distance + right_distance);

    if sin_theta >= 1.0 {
        sin_theta = 1.0
    } else if sin_theta <= -1.0 {
        sin_theta = -1.0
    }

    let direction = Direction::from(F32Ext::asin(sin_theta));

    let direction = if delta_position.x > 0.0 {
        direction
    } else if delta_position.x < 0.0 {
        DIRECTION_PI - direction
    } else {
        if approx_orientation.direction.close(&DIRECTION_PI_2) {
            DIRECTION_PI_2
        } else if approx_orientation.direction.close(&DIRECTION_3_PI_2) {
            DIRECTION_3_PI_2
        } else if approx_orientation.direction > DIRECTION_PI_2
            && approx_orientation.direction < DIRECTION_3_PI_2
        {
            DIRECTION_PI - direction
        } else {
            direction
        }
    };

    (sin_theta, direction)
}

#[cfg(test)]
mod test_v_v_direction {
    #[allow(unused_imports)]
    use crate::test::*;

    use crate::map::v_v_direction;
    use crate::math::{
        Direction, Orientation, Vector, DIRECTION_0, DIRECTION_3_PI_2, DIRECTION_PI,
        DIRECTION_PI_2,
    };
    use core::f32::consts::{FRAC_PI_2, FRAC_PI_8};

    #[test]
    fn right_half_approx_direction() {
        let approx_orientation = Orientation {
            position: Vector { x: 90.0, y: 90.0 },
            direction: Direction::from(FRAC_PI_2 + FRAC_PI_8),
        };

        let (sin_theta, direction) = v_v_direction(
            Vector { x: 0.0, y: 0.0 },
            approx_orientation,
            6.0,
            174.0,
            90.92,
            90.92,
        );

        assert_close(f32::from(direction), f32::from(1.96347));
        assert_close(sin_theta, 0.923889);
    }

    #[test]
    fn left_half_approx_direction() {
        let approx_orientation = Orientation {
            position: Vector { x: 90.0, y: 90.0 },
            direction: Direction::from(FRAC_PI_2 - FRAC_PI_8),
        };

        let (sin_theta, direction) = v_v_direction(
            Vector { x: 0.0, y: 0.0 },
            approx_orientation,
            6.0,
            174.0,
            90.92,
            90.92,
        );

        assert_close(f32::from(direction), f32::from(1.1781225));
        assert_close(sin_theta, 0.923889);
    }

    #[test]
    fn pi_over_two_approx_direction() {
        let delta_position = Vector { x: 1.0, y: 5.0 };

        let approx_orientation = Orientation {
            position: Vector { x: 90.0, y: 90.0 },
            direction: DIRECTION_PI_2,
        };

        let (sin_theta, direction) =
            v_v_direction(delta_position, approx_orientation, 6.0, 174.0, 90.92, 90.92);

        assert_close(f32::from(direction), f32::from(1.1781225));
        assert_close(sin_theta, 0.923889);
    }

    #[test]
    fn three_pi_over_two_approx_direction() {
        let delta_position = Vector { x: 1.0, y: 5.0 };

        let approx_orientation = Orientation {
            position: Vector { x: 90.0, y: 90.0 },
            direction: DIRECTION_3_PI_2,
        };

        let (sin_theta, direction) =
            v_v_direction(delta_position, approx_orientation, 6.0, 174.0, 90.92, 90.92);

        assert_close(f32::from(direction), f32::from(1.1781225));
        assert_close(sin_theta, 0.923889);
    }
}

fn update_orientation_from_distances(
    delta_position: Vector,
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
            let (cos_theta, direction) = h_h_direction(
                delta_position,
                approx_orientation,
                left_result.hit_point.y,
                right_result.hit_point.y,
                left_distance,
                right_distance,
            );

            Orientation {
                position: Vector {
                    x: front_result.hit_point.x - front_distance * cos_theta,
                    y: left_result.hit_point.y - left_distance * cos_theta,
                },
                direction,
            }
        }

        // H_H
        (
            (Some(left_result), Some(left_distance)),
            _,
            (Some(right_result), Some(right_distance)),
        ) if left_result.direction == WallDirection::Horizontal
            && right_result.direction == WallDirection::Horizontal =>
        {
            let (cos_theta, direction) = h_h_direction(
                delta_position,
                approx_orientation,
                left_result.hit_point.y,
                right_result.hit_point.y,
                left_distance,
                right_distance,
            );

            Orientation {
                position: Vector {
                    x: approx_orientation.position.x,
                    y: left_result.hit_point.y - left_distance * cos_theta,
                },
                direction,
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
            let (sin_theta, direction) = v_v_direction(
                delta_position,
                approx_orientation,
                left_result.hit_point.x,
                right_result.hit_point.x,
                left_distance,
                right_distance,
            );

            Orientation {
                position: Vector {
                    x: left_result.hit_point.x + left_distance * sin_theta,
                    y: front_result.hit_point.y - front_distance * sin_theta,
                },
                direction,
            }
        }

        // V_V
        (
            (Some(left_result), Some(left_distance)),
            _,
            (Some(right_result), Some(right_distance)),
        ) if left_result.direction == WallDirection::Vertical
            && right_result.direction == WallDirection::Vertical =>
        {
            let (sin_theta, direction) = v_v_direction(
                delta_position,
                approx_orientation,
                left_result.hit_point.x,
                right_result.hit_point.x,
                left_distance,
                right_distance,
            );

            Orientation {
                position: Vector {
                    x: left_result.hit_point.x + left_distance * sin_theta,
                    y: approx_orientation.position.y,
                },
                direction,
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
    use crate::math::{
        Direction, Orientation, Vector, DIRECTION_0, DIRECTION_PI, DIRECTION_PI_2,
    };
    use crate::maze::{MazeIndex, MazeProjectionResult, WallDirection, WallIndex};
    use crate::test::{assert_close, assert_close2};
    use core::f32::consts::FRAC_PI_8;

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
            Vector { x: 0.0, y: 0.0 },
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
    fn hvh2() {
        let approx_orientation = Orientation {
            position: Vector {
                x: 1610.0,
                y: 1170.0,
            },
            direction: DIRECTION_PI / 16.0,
        };

        let actual_mouse = Orientation {
            position: Vector {
                x: 1600.0105,
                y: 1170.0,
            },
            direction: Direction::from(0.19660425),
        };

        let left_hit_point = Vector {
            x: 1583.29,
            y: 1254.0,
        };
        let left_distance = 85.65;

        let front_hit_point = Vector {
            x: 1794.0,
            y: 1208.59,
        };
        let front_distance = 197.8;

        let right_hit_point = Vector {
            x: 1616.71,
            y: 1086.0,
        };
        let right_distance = 85.65;

        let left_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 8,
                y: 7,
                direction: WallDirection::Horizontal,
            }),
            hit_point: left_hit_point,
            distance: left_distance,
            direction: WallDirection::Horizontal,
        };

        let front_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 10,
                y: 6,
                direction: WallDirection::Vertical,
            }),
            hit_point: front_hit_point,
            distance: front_distance,
            direction: WallDirection::Vertical,
        };

        let right_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 8,
                y: 6,
                direction: WallDirection::Horizontal,
            }),
            hit_point: right_hit_point,
            distance: right_distance,
            direction: WallDirection::Horizontal,
        };

        let result_orientation = update_orientation_from_distances(
            Vector { x: 0.0, y: 0.0 },
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
    fn hvh3() {
        let approx_orientation = Orientation {
            position: Vector {
                x: 1610.0,
                y: 1170.0,
            },
            direction: DIRECTION_0 - (DIRECTION_PI / 16.0),
        };

        let actual_mouse = Orientation {
            position: Vector {
                x: 1600.0105,
                y: 1170.0,
            },
            direction: DIRECTION_0 - Direction::from(0.19660425),
        };

        let left_hit_point = Vector {
            x: 1583.29,
            y: 1254.0,
        };
        let left_distance = 85.65;

        let front_hit_point = Vector {
            x: 1794.0,
            y: 1208.59,
        };
        let front_distance = 197.8;

        let right_hit_point = Vector {
            x: 1616.71,
            y: 1086.0,
        };
        let right_distance = 85.65;

        let left_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 8,
                y: 7,
                direction: WallDirection::Horizontal,
            }),
            hit_point: left_hit_point,
            distance: left_distance,
            direction: WallDirection::Horizontal,
        };

        let front_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 10,
                y: 6,
                direction: WallDirection::Vertical,
            }),
            hit_point: front_hit_point,
            distance: front_distance,
            direction: WallDirection::Vertical,
        };

        let right_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 8,
                y: 6,
                direction: WallDirection::Horizontal,
            }),
            hit_point: right_hit_point,
            distance: right_distance,
            direction: WallDirection::Horizontal,
        };

        let result_orientation = update_orientation_from_distances(
            Vector { x: 0.0, y: 0.0 },
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
    fn hvh4() {
        let delta_position = Vector { x: 5.0, y: 1.0 };

        let approx_orientation = Orientation {
            position: Vector { x: 90.0, y: 90.0 },
            direction: DIRECTION_0,
        };

        let actual_mouse = Orientation {
            position: Vector { x: 90.0, y: 80.0 },
            direction: Direction::from(FRAC_PI_8),
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
            delta_position,
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
    fn h_h() {
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
                direction: WallDirection::Horizontal,
            }),
            hit_point: right_hit_point,
            distance: right_distance,
            direction: WallDirection::Horizontal,
        };

        let result_orientation = update_orientation_from_distances(
            Vector { x: 0.0, y: 0.0 },
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
    fn h_h2() {
        let approx_orientation = Orientation {
            position: Vector {
                x: 1600.0105,
                y: 1150.0,
            },
            direction: DIRECTION_PI / 16.0,
        };

        let actual_mouse = Orientation {
            position: Vector {
                x: 1600.0105,
                y: 1170.0,
            },
            direction: Direction::from(0.19660425),
        };

        let left_hit_point = Vector {
            x: 1583.29,
            y: 1254.0,
        };
        let left_distance = 85.65;

        let front_hit_point = Vector {
            x: 1794.0,
            y: 1208.59,
        };
        let front_distance = 197.8;

        let right_hit_point = Vector {
            x: 1616.71,
            y: 1086.0,
        };
        let right_distance = 85.65;

        let left_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 8,
                y: 7,
                direction: WallDirection::Horizontal,
            }),
            hit_point: left_hit_point,
            distance: left_distance,
            direction: WallDirection::Horizontal,
        };

        let front_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 10,
                y: 6,
                direction: WallDirection::Horizontal,
            }),
            hit_point: front_hit_point,
            distance: front_distance,
            direction: WallDirection::Horizontal,
        };

        let right_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 8,
                y: 6,
                direction: WallDirection::Horizontal,
            }),
            hit_point: right_hit_point,
            distance: right_distance,
            direction: WallDirection::Horizontal,
        };

        let result_orientation = update_orientation_from_distances(
            Vector { x: 0.0, y: 0.0 },
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
    fn h_h3() {
        let approx_orientation = Orientation {
            position: Vector {
                x: 1600.0105,
                y: 1150.0,
            },
            direction: DIRECTION_0 - (DIRECTION_PI / 16.0),
        };

        let actual_mouse = Orientation {
            position: Vector {
                x: 1600.0105,
                y: 1170.0,
            },
            direction: DIRECTION_0 - Direction::from(0.19660425),
        };

        let left_hit_point = Vector {
            x: 1583.29,
            y: 1254.0,
        };
        let left_distance = 85.65;

        let front_hit_point = Vector {
            x: 1794.0,
            y: 1208.59,
        };
        let front_distance = 197.8;

        let right_hit_point = Vector {
            x: 1616.71,
            y: 1086.0,
        };
        let right_distance = 85.65;

        let left_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 8,
                y: 7,
                direction: WallDirection::Horizontal,
            }),
            hit_point: left_hit_point,
            distance: left_distance,
            direction: WallDirection::Horizontal,
        };

        let front_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 10,
                y: 6,
                direction: WallDirection::Horizontal,
            }),
            hit_point: front_hit_point,
            distance: front_distance,
            direction: WallDirection::Horizontal,
        };

        let right_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 8,
                y: 6,
                direction: WallDirection::Horizontal,
            }),
            hit_point: right_hit_point,
            distance: right_distance,
            direction: WallDirection::Horizontal,
        };

        let result_orientation = update_orientation_from_distances(
            Vector { x: 0.0, y: 0.0 },
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
            Vector { x: 0.0, y: 0.0 },
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
    fn vhv2() {
        let approx_orientation = Orientation {
            position: Vector {
                x: 1150.0,
                y: 1506.0,
            },
            direction: Direction::from(5.49),
        };

        let actual_mouse = Orientation {
            position: Vector {
                x: 1156.9972,
                y: 1505.997,
            },
            direction: Direction::from(5.490993),
        };

        let left_hit_point = Vector {
            x: 1254.0,
            y: 1601.69,
        };
        let left_distance = 136.26;

        let front_hit_point = Vector {
            x: 1254.0,
            y: 1407.67,
        };
        let front_distance = 138.12;

        let right_hit_point = Vector {
            x: 1086.0,
            y: 1435.96,
        };
        let right_distance = 99.73;

        let left_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 7,
                y: 8,
                direction: WallDirection::Vertical,
            }),
            hit_point: left_hit_point,
            distance: left_distance,
            direction: WallDirection::Vertical,
        };

        let front_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 7,
                y: 7,
                direction: WallDirection::Horizontal,
            }),
            hit_point: front_hit_point,
            distance: front_distance,
            direction: WallDirection::Horizontal,
        };

        let right_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 6,
                y: 7,
                direction: WallDirection::Vertical,
            }),
            hit_point: right_hit_point,
            distance: right_distance,
            direction: WallDirection::Vertical,
        };

        let result_orientation = update_orientation_from_distances(
            Vector { x: 0.0, y: 0.0 },
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
    fn vhv3() {
        let approx_orientation = Orientation {
            position: Vector {
                x: 1150.0,
                y: 1506.0,
            },
            direction: DIRECTION_PI - Direction::from(5.49),
        };

        let actual_mouse = Orientation {
            position: Vector {
                x: 1156.9972,
                y: 1505.997,
            },
            direction: DIRECTION_PI - Direction::from(5.490993),
        };

        let left_hit_point = Vector {
            x: 1254.0,
            y: 1601.69,
        };
        let left_distance = 136.26;

        let front_hit_point = Vector {
            x: 1254.0,
            y: 1407.67,
        };
        let front_distance = 138.12;

        let right_hit_point = Vector {
            x: 1086.0,
            y: 1435.96,
        };
        let right_distance = 99.73;

        let left_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 7,
                y: 8,
                direction: WallDirection::Vertical,
            }),
            hit_point: left_hit_point,
            distance: left_distance,
            direction: WallDirection::Vertical,
        };

        let front_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 7,
                y: 7,
                direction: WallDirection::Horizontal,
            }),
            hit_point: front_hit_point,
            distance: front_distance,
            direction: WallDirection::Horizontal,
        };

        let right_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 6,
                y: 7,
                direction: WallDirection::Vertical,
            }),
            hit_point: right_hit_point,
            distance: right_distance,
            direction: WallDirection::Vertical,
        };

        let result_orientation = update_orientation_from_distances(
            Vector { x: 0.0, y: 0.0 },
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
    fn v_v() {
        let approx_orientation = Orientation {
            position: Vector {
                x: 1175.4,
                y: 1485.0,
            },
            direction: Direction::from(3.867),
        };

        let actual_mouse = Orientation {
            position: Vector {
                x: 1175.4408,
                y: 1485.0,
            },
            direction: Direction::from(3.8670895),
        };

        let left_hit_point = Vector {
            x: 1254.0,
            y: 1396.0,
        };
        let left_distance = 118.4;

        let front_hit_point = Vector {
            x: 1086.0,
            y: 1405.0,
        };
        let front_distance = 119.6;

        let right_hit_point = Vector {
            x: 1086.0,
            y: 1585.0,
        };
        let right_distance = 134.8;

        let left_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 7,
                y: 7,
                direction: WallDirection::Vertical,
            }),
            hit_point: left_hit_point,
            distance: left_distance,
            direction: WallDirection::Vertical,
        };

        let front_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 6,
                y: 7,
                direction: WallDirection::Horizontal,
            }),
            hit_point: front_hit_point,
            distance: front_distance,
            direction: WallDirection::Vertical,
        };

        let right_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 6,
                y: 8,
                direction: WallDirection::Vertical,
            }),
            hit_point: right_hit_point,
            distance: right_distance,
            direction: WallDirection::Vertical,
        };

        let result_orientation = update_orientation_from_distances(
            Vector { x: 0.0, y: 0.0 },
            approx_orientation,
            Some(front_result),
            Some(front_distance),
            Some(left_result),
            Some(left_distance),
            Some(right_result),
            Some(right_distance),
        );

        assert_close(
            f32::from(result_orientation.direction),
            f32::from(actual_mouse.direction),
        );
        assert_close2(result_orientation.position, actual_mouse.position);
    }

    #[test]
    fn v_v2() {
        let approx_orientation = Orientation {
            position: Vector {
                x: 1150.0,
                y: 1506.0,
            },
            direction: Direction::from(5.49),
        };

        let actual_mouse = Orientation {
            position: Vector {
                x: 1156.9972,
                y: 1506.0,
            },
            direction: Direction::from(5.490993),
        };

        let left_hit_point = Vector {
            x: 1254.0,
            y: 1601.69,
        };
        let left_distance = 136.26;

        let front_hit_point = Vector {
            x: 1254.0,
            y: 1407.67,
        };
        let front_distance = 138.12;

        let right_hit_point = Vector {
            x: 1086.0,
            y: 1435.96,
        };
        let right_distance = 99.73;

        let left_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 7,
                y: 8,
                direction: WallDirection::Vertical,
            }),
            hit_point: left_hit_point,
            distance: left_distance,
            direction: WallDirection::Vertical,
        };

        let front_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 7,
                y: 7,
                direction: WallDirection::Vertical,
            }),
            hit_point: front_hit_point,
            distance: front_distance,
            direction: WallDirection::Vertical,
        };

        let right_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 6,
                y: 7,
                direction: WallDirection::Vertical,
            }),
            hit_point: right_hit_point,
            distance: right_distance,
            direction: WallDirection::Vertical,
        };

        let result_orientation = update_orientation_from_distances(
            Vector { x: 0.0, y: 0.0 },
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
    fn v_v3() {
        let approx_orientation = Orientation {
            position: Vector {
                x: 1150.0,
                y: 1506.0,
            },
            direction: DIRECTION_PI - Direction::from(5.49),
        };

        let actual_mouse = Orientation {
            position: Vector {
                x: 1156.9972,
                y: 1506.0,
            },
            direction: DIRECTION_PI - Direction::from(5.490993),
        };

        let left_hit_point = Vector {
            x: 1254.0,
            y: 1601.69,
        };
        let left_distance = 136.26;

        let front_hit_point = Vector {
            x: 1254.0,
            y: 1407.67,
        };
        let front_distance = 138.12;

        let right_hit_point = Vector {
            x: 1086.0,
            y: 1435.96,
        };
        let right_distance = 99.73;

        let left_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 7,
                y: 8,
                direction: WallDirection::Vertical,
            }),
            hit_point: left_hit_point,
            distance: left_distance,
            direction: WallDirection::Vertical,
        };

        let front_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 7,
                y: 7,
                direction: WallDirection::Vertical,
            }),
            hit_point: front_hit_point,
            distance: front_distance,
            direction: WallDirection::Vertical,
        };

        let right_result = MazeProjectionResult {
            maze_index: MazeIndex::Wall(WallIndex {
                x: 6,
                y: 7,
                direction: WallDirection::Vertical,
            }),
            hit_point: right_hit_point,
            distance: right_distance,
            direction: WallDirection::Vertical,
        };

        let result_orientation = update_orientation_from_distances(
            Vector { x: 0.0, y: 0.0 },
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
