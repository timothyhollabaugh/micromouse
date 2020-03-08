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
use crate::fast::motion_queue::Motion;
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
    pub left_distance: Option<f32>,
    pub front_distance: Option<f32>,
    pub right_distance: Option<f32>,
    pub cell_center: Vector,
    pub center_offset: Option<f32>,
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
    last_left_distance: u8,
    last_left_distance_delta: i16,
    last_right_distance: u8,
    last_right_distance_delta: i16,
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
            last_left_distance: 0,
            last_left_distance_delta: 0,
            last_right_distance: 0,
            last_right_distance_delta: 0,
        }
    }

    pub fn update(
        &mut self,
        mech: &MechanicalConfig,
        maze: &MazeConfig,
        config: &LocalizeConfig,
        left_encoder: i32,
        right_encoder: i32,
        raw_left_distance: u8,
        raw_front_distance: u8,
        raw_right_distance: u8,
        motion: Option<Motion>,
        moves_completed: usize,
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

        let (orientation, sensor_debug) = if let Some(Motion::Path(motion)) = motion {
            let (t, p) = motion.closest_point(encoder_orientation.position);
            let path_direction = motion.derivative(t).direction();

            if config.use_sensors
            /* && in_center */
            {
                const DIRECTION_WITHIN: f32 = FRAC_PI_8 / 2.0;
                const FRONT_TOLERANCE: f32 = 45.0;

                let left_distance_delta =
                    raw_left_distance as i16 - self.last_left_distance as i16;
                let right_distance_delta =
                    raw_right_distance as i16 - self.last_right_distance as i16;

                let left_distance_delta2 =
                    left_distance_delta - self.last_left_distance_delta;
                let right_distance_delta2 =
                    right_distance_delta - self.last_right_distance_delta;

                let left_stabilized =
                    left_distance_delta.abs() <= 10 && left_distance_delta2.abs() <= 10;
                let right_stabilized =
                    right_distance_delta.abs() <= 10 && right_distance_delta2.abs() <= 10;

                let stabilized = left_stabilized && right_stabilized;

                self.last_left_distance = raw_left_distance;
                self.last_right_distance = raw_right_distance;
                self.last_left_distance_delta = left_distance_delta;
                self.last_right_distance_delta = right_distance_delta;

                let left_distance = if raw_left_distance <= mech.left_sensor_limit
                    && stabilized
                {
                    Some(
                        self.left_filter
                            .filter(raw_left_distance as f32 + mech.left_sensor_offset_y),
                    )
                } else {
                    self.left_filter = AverageFilter::new();
                    None
                };

                let right_distance =
                    if raw_right_distance <= mech.right_sensor_limit && stabilized {
                        Some(self.right_filter.filter(
                            raw_right_distance as f32 + mech.right_sensor_offset_y,
                        ))
                    } else {
                        self.right_filter = AverageFilter::new();
                        None
                    };

                let front_distance =
                    if raw_front_distance <= mech.front_sensor_limit {
                        Some(self.front_filter.filter(
                            raw_front_distance as f32 + mech.front_sensor_offset_x,
                        ))
                    } else {
                        self.front_filter = AverageFilter::new();
                        None
                    };

                let cell_center_x = (encoder_orientation.position.x / maze.cell_width)
                    .floor()
                    * maze.cell_width
                    + maze.cell_width / 2.0;

                let cell_center_y = (encoder_orientation.position.y / maze.cell_width)
                    .floor()
                    * maze.cell_width
                    + maze.cell_width / 2.0;

                let center_to_wall = maze.cell_width / 2.0 - maze.wall_width / 2.0;

                let center_offset = match (left_distance, right_distance) {
                    (Some(left), Some(right)) => {
                        if left + right <= maze.cell_width {
                            Some((right - left) / 2.0)
                        } else if left < right {
                            Some(center_to_wall - left)
                        } else {
                            Some(right - center_to_wall)
                        }
                    }
                    (None, Some(right)) => Some(right - center_to_wall),
                    (Some(left), None) => Some(center_to_wall - left),
                    _ => None,
                };

                let (maybe_x, maybe_y) = if path_direction
                    .within(DIRECTION_0, DIRECTION_WITHIN)
                {
                    let y =
                        center_offset.map(|center_offset| cell_center_y + center_offset);

                    let x = front_distance.and_then(|front_distance| {
                        if front_distance
                            < maze.cell_width - maze.wall_width / 2.0 - FRONT_TOLERANCE
                        {
                            Some(cell_center_x + center_to_wall - front_distance)
                        } else {
                            None
                        }
                    });

                    (x, y)
                } else if path_direction.within(DIRECTION_PI, DIRECTION_WITHIN) {
                    let y =
                        center_offset.map(|center_offset| cell_center_y - center_offset);
                    let x = front_distance.and_then(|front_distance| {
                        if front_distance
                            < maze.cell_width - maze.wall_width / 2.0 - FRONT_TOLERANCE
                        {
                            Some(cell_center_x - center_to_wall + front_distance)
                        } else {
                            None
                        }
                    });

                    (x, y)
                } else if path_direction.within(DIRECTION_PI_2, DIRECTION_WITHIN) {
                    let x =
                        center_offset.map(|center_offset| cell_center_x - center_offset);
                    let y = front_distance.and_then(|front_distance| {
                        if front_distance
                            < maze.cell_width - maze.wall_width / 2.0 - FRONT_TOLERANCE
                        {
                            Some(cell_center_y + center_to_wall - front_distance)
                        } else {
                            None
                        }
                    });

                    (x, y)
                } else if path_direction.within(DIRECTION_3_PI_2, DIRECTION_WITHIN) {
                    let x =
                        center_offset.map(|center_offset| cell_center_x + center_offset);
                    let y = front_distance.and_then(|front_distance| {
                        if front_distance
                            < maze.cell_width - maze.wall_width / 2.0 - FRONT_TOLERANCE
                        {
                            Some(cell_center_y - center_to_wall + front_distance)
                        } else {
                            None
                        }
                    });

                    (x, y)
                } else {
                    (None, None)
                };

                let direction = if moves_completed > 0
                    || left_distance.map(|left| left < 20.0).unwrap_or(false)
                    || right_distance.map(|right| right < 20.0).unwrap_or(false)
                {
                    path_direction
                } else {
                    encoder_orientation.direction
                };

                let orientation = Orientation {
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
                    center_offset,
                    maybe_x,
                    maybe_y,
                };

                (orientation, Some(sensor_debug))
            } else {
                self.front_filter = AverageFilter::new();
                self.left_filter = AverageFilter::new();
                self.right_filter = AverageFilter::new();
                (encoder_orientation, None)
            }
        } else {
            self.front_filter = AverageFilter::new();
            self.left_filter = AverageFilter::new();
            self.right_filter = AverageFilter::new();
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

        (self.orientation, debug)
    }
}
