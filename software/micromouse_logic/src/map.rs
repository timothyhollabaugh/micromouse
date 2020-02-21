use core::f32::consts::FRAC_PI_8;
use core::f32::consts::{FRAC_PI_2, FRAC_PI_4};

use libm::F32Ext;

use serde::Deserialize;
use serde::Serialize;

use crate::math::{
    Direction, Orientation, Vector, DIRECTION_0, DIRECTION_3_PI_2, DIRECTION_PI,
    DIRECTION_PI_2,
};

use heapless::ArrayLength;
use heapless::Vec;
use typenum::U1;
use typenum::U8;

use crate::config::MechanicalConfig;
use crate::maze::{
    Maze, MazeConfig, MazeIndex, MazeProjectionResult, Wall, WallDirection,
};
use itertools::Itertools;

pub struct DistanceFilter<N: ArrayLength<f32>> {
    values: Vec<f32, N>,
}

impl<N: ArrayLength<f32>> DistanceFilter<N> {
    pub fn new() -> DistanceFilter<N> {
        DistanceFilter { values: Vec::new() }
    }

    pub fn filter(&mut self, value: f32) -> f32 {
        let len = self.values.len();
        if len >= self.values.capacity() {
            self.values.rotate_left(1);
            self.values[len - 1] = value;
        } else {
            self.values.push(value);
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

    use super::DistanceFilter;
    use typenum::U8;

    #[test]
    fn unfilled() {
        let mut filter = DistanceFilter::<U8>::new();

        assert_close(filter.filter(1.0), 1.0);
        assert_close(filter.filter(2.0), (1.0 + 2.0) / 2.0);
        assert_close(filter.filter(3.0), (1.0 + 2.0 + 3.0) / 3.0);
    }

    #[test]
    fn filled() {
        let mut filter = DistanceFilter::<U8>::new();

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
pub struct MapConfig {
    pub maze: MazeConfig,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MapDebug {
    //pub maze: Maze,
    pub front_result: Option<MazeProjectionResult>,
    pub left_result: Option<MazeProjectionResult>,
    pub right_result: Option<MazeProjectionResult>,
    pub left_distance: Option<f32>,
    pub front_distance: Option<f32>,
    pub right_distance: Option<f32>,
    pub encoder_orientation: Orientation,
    pub maybe_x: Option<f32>,
    pub maybe_y: Option<f32>,
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
fn cleanup_distance_reading<N: ArrayLength<f32>>(
    offset: f32,
    limit: f32,
    tolerance: f32,
    filter: &mut DistanceFilter<N>,
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

    // If we don't see a wall, but should
    let distance = if let (None, Some(result)) = (distance, result) {
        if result.distance < limit {
            Some(limit)
        } else {
            None
        }
    } else {
        distance
    };

    if let Some(d) = distance {
        Some(filter.filter(d))
    } else {
        *filter = DistanceFilter::new();
        None
    }
}

pub struct Map {
    orientation: Orientation,
    delta_position: Vector,
    maze: Maze,
    left_encoder: i32,
    right_encoder: i32,
    left_filter: DistanceFilter<U8>,
    front_filter: DistanceFilter<U1>,
    right_filter: DistanceFilter<U8>,
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
            left_filter: DistanceFilter::new(),
            front_filter: DistanceFilter::new(),
            right_filter: DistanceFilter::new(),
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
            &mut self.front_filter,
            front_distance,
            front_result,
        );

        let left_distance = cleanup_distance_reading(
            mech.left_sensor_offset,
            mech.left_sensor_limit as f32,
            maze_config.cell_width / 2.0,
            &mut self.left_filter,
            left_distance,
            left_result,
        );

        let right_distance = cleanup_distance_reading(
            mech.right_sensor_offset,
            mech.right_sensor_limit as f32,
            maze_config.cell_width / 2.0,
            &mut self.right_filter,
            right_distance,
            right_result,
        );

        let (maybe_x_sensor, maybe_y_sensor) = update_position_from_distances(
            encoder_orientation.direction,
            front_result,
            front_distance,
            left_result,
            left_distance,
            right_result,
            right_distance,
        );

        let maybe_x = maybe_x_sensor.map(|x| {
            x - mech.sensor_center_offset
                * F32Ext::cos(f32::from(self.orientation.direction))
        });

        let maybe_y = maybe_y_sensor.map(|y| {
            y - mech.sensor_center_offset
                * F32Ext::sin(f32::from(self.orientation.direction))
        });

        let position = Vector {
            x: maybe_x.unwrap_or(encoder_orientation.position.x),
            y: maybe_y.unwrap_or(encoder_orientation.position.y),
        };

        let direction = if maybe_x.is_none() && maybe_y.is_none() {
            encoder_orientation.direction
        } else {
            //(position - self.orientation.position).direction()
            encoder_orientation.direction
        };

        let orientation = Orientation {
            position,
            direction,
        };

        let debug = MapDebug {
            //maze: self.maze.clone(),
            front_result,
            left_result,
            right_result,
            left_distance,
            front_distance,
            right_distance,
            encoder_orientation,
            maybe_x,
            maybe_y,
        };

        self.orientation = orientation;

        (self.orientation, debug)
    }
}

fn h_h_direction(
    left_wall: f32,
    right_wall: f32,
    left_distance: f32,
    right_distance: f32,
) -> f32 {
    let mut cos_theta = (left_wall - right_wall) / (left_distance + right_distance);

    if cos_theta >= 1.0 {
        cos_theta = 1.0
    } else if cos_theta <= -1.0 {
        cos_theta = -1.0
    }

    cos_theta
}

#[cfg(test)]
mod test_h_h_direction {
    #[allow(unused_imports)]
    use crate::test::*;

    use crate::map::h_h_direction;
    use crate::math::{Direction, Orientation, Vector, DIRECTION_0, DIRECTION_PI};
    use core::f32::consts::FRAC_PI_8;

    #[test]
    fn facing_left() {
        let cos_theta = h_h_direction(174.0, 6.0, 90.92, 90.92);
        assert_close(cos_theta, 0.92388);
    }

    #[test]
    fn facing_right() {
        let cos_theta = h_h_direction(6.0, 174.0, 90.92, 90.92);
        assert_close(cos_theta, -0.92388);
    }
}

fn v_v_direction(
    left_wall: f32,
    right_wall: f32,
    left_distance: f32,
    right_distance: f32,
) -> f32 {
    let mut sin_theta = (right_wall - left_wall) / (left_distance + right_distance);

    if sin_theta >= 1.0 {
        sin_theta = 1.0
    } else if sin_theta <= -1.0 {
        sin_theta = -1.0
    }

    sin_theta
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
    fn facing_up() {
        let sin_theta = v_v_direction(6.0, 174.0, 90.92, 90.92);
        assert_close(sin_theta, 0.923889);
    }

    #[test]
    fn facing_down() {
        let sin_theta = v_v_direction(174.0, 6.0, 90.92, 90.92);
        assert_close(sin_theta, -0.923889);
    }
}

fn update_position_from_distances(
    direction: Direction,
    front_result: Option<MazeProjectionResult>,
    front_distance: Option<f32>,
    left_result: Option<MazeProjectionResult>,
    left_distance: Option<f32>,
    right_result: Option<MazeProjectionResult>,
    right_distance: Option<f32>,
) -> (Option<f32>, Option<f32>) {
    const WITHIN_ANGLE: f32 = FRAC_PI_4;

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
            && right_result.direction == WallDirection::Horizontal
            && (direction.within(&DIRECTION_0, FRAC_PI_4)
                || direction.within(&DIRECTION_PI, FRAC_PI_4)) =>
        {
            let cos_theta = h_h_direction(
                left_result.hit_point.y,
                right_result.hit_point.y,
                left_distance,
                right_distance,
            );

            (
                Some(front_result.hit_point.x - front_distance * cos_theta),
                Some(left_result.hit_point.y - left_distance * cos_theta),
            )
        }

        // H_H
        (
            (Some(left_result), Some(left_distance)),
            _,
            (Some(right_result), Some(right_distance)),
        ) if left_result.direction == WallDirection::Horizontal
            && right_result.direction == WallDirection::Horizontal
            && (direction.within(&DIRECTION_0, FRAC_PI_4)
                || direction.within(&DIRECTION_PI, FRAC_PI_4)) =>
        {
            let cos_theta = h_h_direction(
                left_result.hit_point.y,
                right_result.hit_point.y,
                left_distance,
                right_distance,
            );

            (
                None,
                Some(left_result.hit_point.y - left_distance * cos_theta),
            )
        }

        // HV_
        (
            (Some(left_result), Some(left_distance)),
            (Some(front_result), Some(front_distance)),
            _,
        ) if left_result.direction == WallDirection::Horizontal
            && front_result.direction == WallDirection::Vertical
            && (direction.within(&DIRECTION_0, FRAC_PI_4)
                || direction.within(&DIRECTION_PI, FRAC_PI_4)) =>
        {
            let cos_theta = F32Ext::cos(f32::from(direction));

            (
                Some(front_result.hit_point.x - front_distance * cos_theta),
                Some(left_result.hit_point.y - left_distance * cos_theta),
            )
        }

        // _VH
        (
            _,
            (Some(front_result), Some(front_distance)),
            (Some(right_result), Some(right_distance)),
        ) if front_result.direction == WallDirection::Vertical
            && right_result.direction == WallDirection::Horizontal
            && (direction.within(&DIRECTION_0, FRAC_PI_4)
                || direction.within(&DIRECTION_PI, FRAC_PI_4)) =>
        {
            let cos_theta = F32Ext::cos(f32::from(direction));

            (
                Some(front_result.hit_point.x - front_distance * cos_theta),
                Some(right_result.hit_point.y + right_distance * cos_theta),
            )
        }

        // VHV
        (
            (Some(left_result), Some(left_distance)),
            (Some(front_result), Some(front_distance)),
            (Some(right_result), Some(right_distance)),
        ) if left_result.direction == WallDirection::Vertical
            && front_result.direction == WallDirection::Horizontal
            && right_result.direction == WallDirection::Vertical
            && (direction.within(&DIRECTION_PI_2, FRAC_PI_4)
                || direction.within(&DIRECTION_3_PI_2, FRAC_PI_4)) =>
        {
            let sin_theta = v_v_direction(
                left_result.hit_point.x,
                right_result.hit_point.x,
                left_distance,
                right_distance,
            );

            (
                Some(left_result.hit_point.x + left_distance * sin_theta),
                Some(front_result.hit_point.y - front_distance * sin_theta),
            )
        }

        // V_V
        (
            (Some(left_result), Some(left_distance)),
            _,
            (Some(right_result), Some(right_distance)),
        ) if left_result.direction == WallDirection::Vertical
            && right_result.direction == WallDirection::Vertical
            && (direction.within(&DIRECTION_PI_2, FRAC_PI_4)
                || direction.within(&DIRECTION_3_PI_2, FRAC_PI_4)) =>
        {
            let sin_theta = v_v_direction(
                left_result.hit_point.x,
                right_result.hit_point.x,
                left_distance,
                right_distance,
            );

            (
                Some(left_result.hit_point.x + left_distance * sin_theta),
                None,
            )
        }

        // VH_
        (
            (Some(left_result), Some(left_distance)),
            (Some(front_result), Some(front_distance)),
            _,
        ) if left_result.direction == WallDirection::Vertical
            && front_result.direction == WallDirection::Horizontal
            && (direction.within(&DIRECTION_PI_2, FRAC_PI_4)
                || direction.within(&DIRECTION_3_PI_2, FRAC_PI_4)) =>
        {
            let sin_theta = F32Ext::sin(f32::from(direction));

            (
                Some(left_result.hit_point.x - left_distance * sin_theta),
                Some(front_result.hit_point.y - front_distance * sin_theta),
            )
        }

        // _HV
        (
            _,
            (Some(front_result), Some(front_distance)),
            (Some(right_result), Some(right_distance)),
        ) if front_result.direction == WallDirection::Vertical
            && right_result.direction == WallDirection::Horizontal
            && (direction.within(&DIRECTION_PI_2, FRAC_PI_4)
                || direction.within(&DIRECTION_3_PI_2, FRAC_PI_4)) =>
        {
            let sin_theta = F32Ext::sin(f32::from(direction));

            (
                Some(right_result.hit_point.x + right_distance * sin_theta),
                Some(front_result.hit_point.y - front_distance * sin_theta),
            )
        }

        _ => (None, None),
    }
}

#[cfg(test)]
mod test_update_orientation_from_distance {
    #[allow(unused_imports)]
    use crate::test;

    use crate::map::update_position_from_distances;
    use crate::math::{
        Direction, Orientation, Vector, DIRECTION_0, DIRECTION_PI, DIRECTION_PI_2,
    };
    use crate::maze::{MazeIndex, MazeProjectionResult, WallDirection, WallIndex};
    use crate::test::{assert_close, assert_close2};
    use core::f32::consts::FRAC_PI_8;

    #[test]
    fn hvh() {
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

        let result_orientation = update_position_from_distances(
            DIRECTION_0,
            Some(front_result),
            Some(front_distance),
            Some(left_result),
            Some(left_distance),
            Some(right_result),
            Some(right_distance),
        );

        assert_eq!(result_orientation, (Some(90.0), Some(80.0)));
    }

    #[test]
    fn h_h() {
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

        let result_orientation = update_position_from_distances(
            DIRECTION_0,
            Some(front_result),
            Some(front_distance),
            Some(left_result),
            Some(left_distance),
            Some(right_result),
            Some(right_distance),
        );

        assert_eq!(result_orientation, (None, Some(80.0)));
    }

    #[test]
    fn vhv() {
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

        let result_orientation = update_position_from_distances(
            DIRECTION_PI_2,
            Some(front_result),
            Some(front_distance),
            Some(left_result),
            Some(left_distance),
            Some(right_result),
            Some(right_distance),
        );

        assert_eq!(result_orientation, (Some(90.0), Some(79.99999)));
    }

    #[test]
    fn v_v() {
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

        let result_orientation = update_position_from_distances(
            DIRECTION_PI_2,
            Some(front_result),
            Some(front_distance),
            Some(left_result),
            Some(left_distance),
            Some(right_result),
            Some(right_distance),
        );

        assert_eq!(result_orientation, (Some(1175.4408), None));
    }
}
