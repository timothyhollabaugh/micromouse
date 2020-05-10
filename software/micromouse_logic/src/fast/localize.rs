use core::f32::consts::FRAC_PI_8;

use itertools::Itertools;

use libm::F32Ext;

use serde::Deserialize;
use serde::Serialize;

use heapless::{ArrayLength, Vec};

use typenum::U8;

use crate::config::MechanicalConfig;
use crate::mouse::ContainsDistanceReading;
use crate::mouse::DistanceReading;
use crate::slow::maze::MazeConfig;

use super::{
    Direction, Orientation, Vector, DIRECTION_0, DIRECTION_3_PI_2, DIRECTION_PI,
    DIRECTION_PI_2,
};
use crate::fast::motion_queue::Motion;

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
mod test_average_filter {
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

/// Configuration for a [SideDistanceFilter]
#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SideDistanceFilterConfig {
    /// The max allowed change between readings
    pub max_delta: f32,

    /// The max allowed change between the change in readings
    pub max_delta2: f32,
}

/// Filters a raw distance reading into something that makes sense
///
///  - Makes sure that the readings are within the max delta and second delta
///  - Feeds through an averaging filter
///  - Offsets from the mechanical location of the sensor to the center of the mouse
struct SideDistanceFilter {
    average_filter: AverageFilter<U8>,
    last_raw: Option<f32>,
    last_delta: Option<f32>,
}

impl SideDistanceFilter {
    pub fn new() -> SideDistanceFilter {
        SideDistanceFilter {
            average_filter: AverageFilter::new(),
            last_raw: None,
            last_delta: None,
        }
    }

    /// Filter the distance reading
    pub fn filter(
        &mut self,
        config: &SideDistanceFilterConfig,
        raw: Option<DistanceReading>,
    ) -> Option<f32> {
        match raw {
            Some(DistanceReading::InRange(raw)) => {
                let delta = self.last_raw.map(|last_raw| raw - last_raw);

                let delta2 = self
                    .last_delta
                    .iter()
                    .zip(delta.iter())
                    .map(|(&last_delta, &delta)| delta - last_delta)
                    .next();

                let stabilized = match (delta, delta2) {
                    (None, None) => true,
                    (Some(delta), None) => delta.abs() <= config.max_delta,
                    (None, Some(delta2)) => delta2.abs() <= config.max_delta2,
                    (Some(delta), Some(delta2)) => {
                        delta.abs() <= config.max_delta
                            && delta2.abs() < config.max_delta2
                    }
                };

                self.last_raw = Some(raw);
                self.last_delta = delta;

                if stabilized {
                    Some(self.average_filter.filter(raw))
                } else {
                    self.last_raw = None;
                    self.last_delta = None;
                    self.average_filter = AverageFilter::new();
                    None
                }
            }

            Some(DistanceReading::OutOfRange) => {
                self.last_raw = None;
                self.last_delta = None;
                self.average_filter = AverageFilter::new();
                None
            }

            None => None,
        }
    }
}

#[cfg(test)]
mod side_distance_filter_test {
    #[allow(unused_imports)]
    use crate::test::*;

    use super::SideDistanceFilter;
    use super::SideDistanceFilterConfig;
    use crate::mouse::DistanceReading;

    const CONFIG: SideDistanceFilterConfig = SideDistanceFilterConfig {
        max_delta: 10.0,
        max_delta2: 5.0,
    };

    #[test]
    fn single_in_range() {
        let mut filter = SideDistanceFilter::new();
        assert_eq!(
            filter.filter(&CONFIG, Some(DistanceReading::InRange(1.0))),
            Some(1.0)
        );
    }

    #[test]
    fn single_out_of_range() {
        let mut filter = SideDistanceFilter::new();
        assert_eq!(
            filter.filter(&CONFIG, Some(DistanceReading::OutOfRange)),
            None
        )
    }

    #[test]
    fn single_none() {
        let mut filter = SideDistanceFilter::new();
        assert_eq!(filter.filter(&CONFIG, None), None)
    }

    #[test]
    fn two_in_range_out_of_range() {
        let mut filter = SideDistanceFilter::new();
        assert_eq!(
            filter.filter(&CONFIG, Some(DistanceReading::InRange(1.0))),
            Some(1.0)
        );
        assert_eq!(
            filter.filter(&CONFIG, Some(DistanceReading::OutOfRange)),
            None
        );
    }

    #[test]
    fn out_of_range_is_none_and_clears_average_filter() {
        let mut filter = SideDistanceFilter::new();
        assert_eq!(
            filter.filter(&CONFIG, Some(DistanceReading::InRange(1.0))),
            Some(1.0)
        );
        assert_eq!(
            filter.filter(&CONFIG, Some(DistanceReading::OutOfRange)),
            None
        );
        assert_eq!(
            filter.filter(&CONFIG, Some(DistanceReading::InRange(3.0))),
            Some(3.0)
        );
    }

    #[test]
    fn delta_too_high_is_none_and_clears_average_filter() {
        let mut filter = SideDistanceFilter::new();
        assert_eq!(
            filter.filter(&CONFIG, Some(DistanceReading::InRange(1.0))),
            Some(1.0)
        );
        assert_eq!(
            filter.filter(&CONFIG, Some(DistanceReading::InRange(20.0))),
            None
        );
        assert_eq!(
            filter.filter(&CONFIG, Some(DistanceReading::InRange(3.0))),
            Some(3.0)
        );
    }

    #[test]
    fn delta2_too_high_is_none_and_clears_average_filter() {
        let mut filter = SideDistanceFilter::new();
        assert_eq!(
            filter.filter(&CONFIG, Some(DistanceReading::InRange(1.0))),
            Some(1.0)
        );
        assert_eq!(
            filter.filter(&CONFIG, Some(DistanceReading::InRange(1.0))),
            Some(1.0)
        );
        assert_eq!(
            filter.filter(&CONFIG, Some(DistanceReading::InRange(11.0))),
            None
        );
        assert_eq!(
            filter.filter(&CONFIG, Some(DistanceReading::InRange(3.0))),
            Some(3.0)
        );
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct LocalizeConfig {
    pub use_sensors: bool,
    pub left_side_filter: SideDistanceFilterConfig,
    pub right_side_filter: SideDistanceFilterConfig,
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
    left_filter: SideDistanceFilter,
    right_filter: SideDistanceFilter,
    last_direction_moved: Direction,
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
            left_filter: SideDistanceFilter::new(),
            right_filter: SideDistanceFilter::new(),
            last_direction_moved: orientation.direction,
        }
    }

    pub fn update(
        &mut self,
        mech: &MechanicalConfig,
        maze: &MazeConfig,
        config: &LocalizeConfig,
        left_encoder: i32,
        right_encoder: i32,
        raw_left_distance: Option<DistanceReading>,
        raw_front_distance: Option<DistanceReading>,
        raw_right_distance: Option<DistanceReading>,
        motion: Option<Motion>,
        moves_completed: usize,
    ) -> (Orientation, LocalizeDebug) {
        let delta_left = left_encoder - self.left_encoder;
        let delta_right = right_encoder - self.right_encoder;

        let encoder_orientation =
            self.orientation
                .update_from_encoders(&mech, delta_left, delta_right);

        let (orientation, sensor_debug) = if let Some(Motion::Path(motion)) = motion {
            let (t, _) = motion.closest_point(encoder_orientation.position);
            let path_direction = motion.derivative(t).direction();

            const DIRECTION_WITHIN: f32 = FRAC_PI_8 / 2.0;
            const FRONT_TOLERANCE: f32 = 45.0;

            let within_east = path_direction.within(DIRECTION_0, DIRECTION_WITHIN);
            let within_west = path_direction.within(DIRECTION_PI, DIRECTION_WITHIN);
            let within_north = path_direction.within(DIRECTION_PI_2, DIRECTION_WITHIN);
            let within_south = path_direction.within(DIRECTION_3_PI_2, DIRECTION_WITHIN);

            if config.use_sensors
                && (within_east || within_west || within_north || within_south)
            {
                // Calculate maze 'constants' for this location
                let cell_center_x = (encoder_orientation.position.x / maze.cell_width)
                    .floor()
                    * maze.cell_width
                    + maze.cell_width / 2.0;

                let cell_center_y = (encoder_orientation.position.y / maze.cell_width)
                    .floor()
                    * maze.cell_width
                    + maze.cell_width / 2.0;

                // Filter distance values
                let left_distance = self
                    .left_filter
                    .filter(&config.left_side_filter, raw_left_distance)
                    .map(|d| d + mech.left_sensor_offset_y);

                let right_distance = self
                    .right_filter
                    .filter(&config.right_side_filter, raw_right_distance)
                    .map(|d| d + mech.left_sensor_offset_y);

                let front_distance = raw_front_distance
                    .value()
                    .map(|d| d + mech.front_sensor_offset_x);

                // Where are we left/right within the cell?
                let center_offset = match (left_distance, right_distance) {
                    (Some(left), Some(right)) => {
                        if left + right <= maze.cell_width {
                            Some((right - left) / 2.0)
                        } else if left < right {
                            Some(maze.center_to_wall() - left)
                        } else {
                            Some(right - maze.center_to_wall())
                        }
                    }
                    (None, Some(right)) => Some(right - maze.center_to_wall()),
                    (Some(left), None) => Some(maze.center_to_wall() - left),
                    _ => None,
                };

                let (maybe_x, maybe_y) = if within_east {
                    let y =
                        center_offset.map(|center_offset| cell_center_y + center_offset);

                    let x = front_distance.and_then(|front_distance| {
                        if front_distance
                            < maze.cell_width - maze.wall_width / 2.0 - FRONT_TOLERANCE
                        {
                            Some(cell_center_x + maze.center_to_wall() - front_distance)
                        } else {
                            None
                        }
                    });

                    (x, y)
                } else if within_west {
                    let y =
                        center_offset.map(|center_offset| cell_center_y - center_offset);
                    let x = front_distance.and_then(|front_distance| {
                        if front_distance
                            < maze.cell_width - maze.wall_width / 2.0 - FRONT_TOLERANCE
                        {
                            Some(cell_center_x - maze.center_to_wall() + front_distance)
                        } else {
                            None
                        }
                    });

                    (x, y)
                } else if within_north {
                    let x =
                        center_offset.map(|center_offset| cell_center_x - center_offset);
                    let y = front_distance.and_then(|front_distance| {
                        if front_distance
                            < maze.cell_width - maze.wall_width / 2.0 - FRONT_TOLERANCE
                        {
                            Some(cell_center_y + maze.center_to_wall() - front_distance)
                        } else {
                            None
                        }
                    });

                    (x, y)
                } else if within_south {
                    let x =
                        center_offset.map(|center_offset| cell_center_x + center_offset);
                    let y = front_distance.and_then(|front_distance| {
                        if front_distance
                            < maze.cell_width - maze.wall_width / 2.0 - FRONT_TOLERANCE
                        {
                            Some(cell_center_y - maze.center_to_wall() + front_distance)
                        } else {
                            None
                        }
                    });

                    (x, y)
                } else {
                    (None, None)
                };

                let position = Vector {
                    x: maybe_x.unwrap_or(encoder_orientation.position.x),
                    y: maybe_y.unwrap_or(encoder_orientation.position.y),
                };

                let direction_moved = (position - self.orientation.position).direction();

                let direction_moved_reset = !encoder_orientation
                    .direction
                    .within(direction_moved, DIRECTION_WITHIN)
                    && !encoder_orientation
                        .direction
                        .within(self.last_direction_moved, DIRECTION_WITHIN);

                self.last_direction_moved = direction_moved;

                let direction = if moves_completed > 0
                    || left_distance.map(|left| left < 20.0).unwrap_or(false)
                    || right_distance.map(|right| right < 20.0).unwrap_or(false)
                    || direction_moved_reset
                {
                    path_direction
                } else {
                    encoder_orientation.direction
                };

                let orientation = Orientation {
                    position,
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
                (encoder_orientation, None)
            }
        } else {
            self.left_filter = SideDistanceFilter::new();
            self.right_filter = SideDistanceFilter::new();
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
