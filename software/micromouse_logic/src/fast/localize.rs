use core::f32::consts::FRAC_PI_8;

use itertools::Itertools;

use libm::F32Ext;

use serde::Deserialize;
use serde::Serialize;

use heapless::{ArrayLength, Vec};

use typenum::{U1, U8};

use crate::config::MechanicalConfig;
use crate::slow::maze::MazeConfig;

use super::{
    Direction, Orientation, Vector, DIRECTION_0, DIRECTION_3_PI_2, DIRECTION_PI,
    DIRECTION_PI_2,
};
use crate::slow::MazeDirection;

pub struct AverageFilter<N: ArrayLength<f32>> {
    values: Vec<f32, N>,
}

impl<N: ArrayLength<f32>> AverageFilter<N> {
    pub fn new() -> AverageFilter<N> {
        AverageFilter { values: Vec::new() }
    }

    pub fn filter(&mut self, value: f32) -> f32 {
        let len = self.values.len();
        if len >= self.values.capacity() {
            self.values.rotate_left(1);
            self.values[len - 1] = value;
        } else {
            self.values.push(value).ok();
        }

        if let Some(sum) = self.values.iter().sum1::<f32>() {
            sum / self.values.len() as f32
        } else {
            value
        }
    }
}

#[cfg(test)]
mod test_filter {
    #[allow(unused_imports)]
    use crate::test::*;

    use super::AverageFilter;
    use typenum::U8;

    #[test]
    fn unfilled() {
        let mut filter = AverageFilter::<U8>::new();

        assert_close(filter.filter(1.0), 1.0);
        assert_close(filter.filter(2.0), (1.0 + 2.0) / 2.0);
        assert_close(filter.filter(3.0), (1.0 + 2.0 + 3.0) / 3.0);
    }

    #[test]
    fn filled() {
        let mut filter = AverageFilter::<U8>::new();

        assert_close(filter.filter(1.0), 1.0);
        assert_close(filter.filter(2.0), (1.0 + 2.0) / 2.0);
        assert_close(filter.filter(3.0), (1.0 + 2.0 + 3.0) / 3.0);
        assert_close(filter.filter(4.0), (1.0 + 2.0 + 3.0 + 4.0) / 4.0);
        assert_close(filter.filter(5.0), (1.0 + 2.0 + 3.0 + 4.0 + 5.0) / 5.0);
        assert_close(
            filter.filter(6.0),
            (1.0 + 2.0 + 3.0 + 4.0 + 5.0 + 6.0) / 6.0,
        );
        assert_close(
            filter.filter(7.0),
            (1.0 + 2.0 + 3.0 + 4.0 + 5.0 + 6.0 + 7.0) / 7.0,
        );
        assert_close(
            filter.filter(8.0),
            (1.0 + 2.0 + 3.0 + 4.0 + 5.0 + 6.0 + 7.0 + 8.0) / 8.0,
        );
        assert_close(
            filter.filter(9.0),
            (2.0 + 3.0 + 4.0 + 5.0 + 6.0 + 7.0 + 8.0 + 9.0) / 8.0,
        );
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LocalizeConfig {
    pub use_sensors: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LocalizeDebug {
    //pub maze: Maze,
    pub encoder_orientation: Orientation,
    pub sensor: Option<SensorDebug>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SensorDebug {
    pub left_distance: f32,
    pub front_distance: f32,
    pub right_distance: f32,
    pub cell_center: Vector,
    pub sensor_width: f32,
    pub center_offset: f32,
    pub maybe_x: Option<f32>,
    pub maybe_y: Option<f32>,
}

pub struct Localize {
    orientation: Orientation,
    left_encoder: i32,
    right_encoder: i32,
    left_filter: AverageFilter<U8>,
    front_filter: AverageFilter<U1>,
    right_filter: AverageFilter<U8>,
    last_buffer_len: usize,
}

impl Localize {
    pub fn new(
        orientation: Orientation,
        left_encoder: i32,
        right_encoder: i32,
    ) -> Localize {
        Localize {
            orientation,
            left_encoder,
            right_encoder,
            left_filter: AverageFilter::new(),
            front_filter: AverageFilter::new(),
            right_filter: AverageFilter::new(),
            last_buffer_len: 0,
        }
    }

    pub fn update(
        &mut self,
        mech: &MechanicalConfig,
        maze: &MazeConfig,
        config: &LocalizeConfig,
        left_encoder: i32,
        right_encoder: i32,
        left_distance: u8,
        front_distance: u8,
        right_distance: u8,
        path_direction: Direction,
        buffer_len: usize,
    ) -> (Orientation, LocalizeDebug) {
        let delta_left = left_encoder - self.left_encoder;
        let delta_right = right_encoder - self.right_encoder;

        let encoder_orientation =
            self.orientation
                .update_from_encoders(&mech, delta_left, delta_right);

        let encoder_maze_orientation = encoder_orientation.to_maze_orientation(maze);
        let encoder_cell_center = encoder_maze_orientation.position.center_position(maze);
        let tolerance = maze.cell_width / 2.0 - maze.wall_width;

        let in_center = match encoder_maze_orientation.direction {
            MazeDirection::North | MazeDirection::South => {
                encoder_orientation.position.y > encoder_cell_center.y - tolerance
                    && encoder_orientation.position.y < encoder_cell_center.y + tolerance
            }
            MazeDirection::East | MazeDirection::West => {
                encoder_orientation.position.x > encoder_cell_center.x - tolerance
                    && encoder_orientation.position.x < encoder_cell_center.x + tolerance
            }
        };

        let (orientation, sensor_debug) = if config.use_sensors && in_center {
            const DIRECTION_WITHIN: f32 = FRAC_PI_8 / 2.0;
            const FRONT_TOLERANCE: f32 = 20.0;

            let left_distance = if left_distance <= mech.left_sensor_limit {
                self.left_filter
                    .filter(left_distance as f32 + mech.left_sensor_offset_y)
            } else {
                self.left_filter = AverageFilter::new();
                left_distance as f32 + mech.left_sensor_offset_y
            };

            let right_distance = if right_distance <= mech.right_sensor_limit {
                self.right_filter
                    .filter(right_distance as f32 + mech.right_sensor_offset_y)
            } else {
                self.right_filter = AverageFilter::new();
                right_distance as f32 + mech.right_sensor_offset_y
            };

            let front_distance = if front_distance <= mech.front_sensor_limit {
                self.front_filter
                    .filter(front_distance as f32 + mech.front_sensor_offset_x)
            } else {
                self.front_filter = AverageFilter::new();
                front_distance as f32 + mech.front_sensor_offset_x
            };

            let cell_center_x = (encoder_orientation.position.x / maze.cell_width)
                .floor()
                * maze.cell_width
                + maze.cell_width / 2.0;

            let cell_center_y = (encoder_orientation.position.y / maze.cell_width)
                .floor()
                * maze.cell_width
                + maze.cell_width / 2.0;

            let sensor_width = left_distance + right_distance;

            let center_to_wall = maze.cell_width / 2.0 - maze.wall_width / 2.0;

            let center_offset = if sensor_width <= maze.cell_width {
                (right_distance - left_distance) / 2.0
            } else if left_distance > right_distance {
                right_distance - center_to_wall
            } else if right_distance > left_distance {
                center_to_wall - left_distance
            } else {
                0.0
            };

            let (maybe_x, maybe_y) =
                if path_direction.within(DIRECTION_0, DIRECTION_WITHIN) {
                    let y = Some(cell_center_y + center_offset);
                    let x = if front_distance
                        < maze.cell_width - maze.wall_width / 2.0 - FRONT_TOLERANCE
                    {
                        Some(cell_center_x + center_to_wall - front_distance)
                    } else {
                        None
                    };

                    (x, y)
                } else if path_direction.within(DIRECTION_PI, DIRECTION_WITHIN) {
                    let y = Some(cell_center_y - center_offset);
                    let x = if front_distance
                        < maze.cell_width - maze.wall_width / 2.0 - FRONT_TOLERANCE
                    {
                        Some(cell_center_x - center_to_wall + front_distance)
                    } else {
                        None
                    };

                    (x, y)
                } else if path_direction.within(DIRECTION_PI_2, DIRECTION_WITHIN) {
                    let x = Some(cell_center_x - center_offset);
                    let y = if front_distance
                        < maze.cell_width - maze.wall_width / 2.0 - FRONT_TOLERANCE
                    {
                        Some(cell_center_y + center_to_wall - front_distance)
                    } else {
                        None
                    };

                    (x, y)
                } else if path_direction.within(DIRECTION_3_PI_2, DIRECTION_WITHIN) {
                    let x = Some(cell_center_x + center_offset);
                    let y = if front_distance
                        < maze.cell_width - maze.wall_width / 2.0 - FRONT_TOLERANCE
                    {
                        Some(cell_center_y - center_to_wall + front_distance)
                    } else {
                        None
                    };

                    (x, y)
                } else {
                    (None, None)
                };

            let direction = if buffer_len < self.last_buffer_len {
                path_direction
            //encoder_orientation.direction
            } else {
                encoder_orientation.direction
            };

            let orienation = Orientation {
                position: Vector {
                    x: maybe_x.unwrap_or(encoder_orientation.position.x),
                    y: maybe_y.unwrap_or(encoder_orientation.position.y),
                },
                direction,
            };

            let sensor_debug = SensorDebug {
                left_distance,
                front_distance,
                right_distance,
                cell_center: Vector {
                    x: cell_center_x,
                    y: cell_center_y,
                },
                sensor_width,
                center_offset,
                maybe_x,
                maybe_y,
            };

            (orienation, Some(sensor_debug))
        } else {
            (encoder_orientation, None)
        };

        let debug = LocalizeDebug {
            //maze: self.maze.clone(),
            encoder_orientation,
            sensor: sensor_debug,
        };

        self.left_encoder = left_encoder;
        self.right_encoder = right_encoder;
        self.orientation = orientation;
        self.last_buffer_len = buffer_len;

        (self.orientation, debug)
    }
}
