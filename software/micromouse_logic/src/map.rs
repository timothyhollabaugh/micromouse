use core::f32::consts::{FRAC_PI_2, FRAC_PI_4};
use core::f32::consts::{FRAC_PI_6, FRAC_PI_8};

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
    pub left_distance: f32,
    pub front_distance: f32,
    pub right_distance: f32,
    pub encoder_orientation: Orientation,
    pub cell_center: Vector,
    pub sensor_width: f32,
    pub center_offset: f32,
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
    last_buffer_len: usize,
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
            last_buffer_len: 0,
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
        path_direction: Direction,
        buffer_len: usize,
    ) -> (Orientation, MapDebug) {
        let delta_left = left_encoder - self.left_encoder;
        let delta_right = right_encoder - self.right_encoder;

        let encoder_orientation =
            self.orientation
                .update_from_encoders(&mech, delta_left, delta_right);

        let cell_center_x = (encoder_orientation.position.x / maze_config.cell_width)
            .floor()
            * maze_config.cell_width
            + maze_config.cell_width / 2.0;

        let cell_center_y = (encoder_orientation.position.y / maze_config.cell_width)
            .floor()
            * maze_config.cell_width
            + maze_config.cell_width / 2.0;

        let left_distance = left_distance as f32 + mech.left_sensor_offset;
        let right_distance = right_distance as f32 + mech.right_sensor_offset;
        let front_distance =
            front_distance as f32 + mech.sensor_center_offset + mech.front_sensor_offset;

        let sensor_width = left_distance + right_distance;

        let center_to_wall = maze_config.cell_width / 2.0 - maze_config.wall_width / 2.0;

        let center_offset = if sensor_width <= maze_config.cell_width {
            (right_distance - left_distance) / 2.0
        } else if left_distance > right_distance {
            right_distance - center_to_wall
        } else if right_distance > left_distance {
            center_to_wall - left_distance
        } else {
            0.0
        };

        const DIRECTION_WITHIN: f32 = FRAC_PI_8 / 2.0;
        const FRONT_TOLERANCE: f32 = 20.0;

        let (maybe_x, maybe_y) = if path_direction.within(&DIRECTION_0, DIRECTION_WITHIN)
        {
            let y = Some(cell_center_y + center_offset);
            let x = if front_distance
                < maze_config.cell_width - maze_config.wall_width / 2.0 - FRONT_TOLERANCE
            {
                Some(cell_center_x + center_to_wall - front_distance)
            } else {
                None
            };

            (x, y)
        } else if path_direction.within(&DIRECTION_PI, DIRECTION_WITHIN) {
            let y = Some(cell_center_y - center_offset);
            let x = if front_distance
                < maze_config.cell_width - maze_config.wall_width / 2.0 - FRONT_TOLERANCE
            {
                Some(cell_center_x - center_to_wall + front_distance)
            } else {
                None
            };

            (x, y)
        } else if path_direction.within(&DIRECTION_PI_2, DIRECTION_WITHIN) {
            let x = Some(cell_center_x - center_offset);
            let y = if front_distance
                < maze_config.cell_width - maze_config.wall_width / 2.0 - FRONT_TOLERANCE
            {
                Some(cell_center_y + center_to_wall - front_distance)
            } else {
                None
            };

            (x, y)
        } else if path_direction.within(&DIRECTION_3_PI_2, DIRECTION_WITHIN) {
            let x = Some(cell_center_x + center_offset);
            let y = if front_distance
                < maze_config.cell_width - maze_config.wall_width / 2.0 - FRONT_TOLERANCE
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

        let debug = MapDebug {
            //maze: self.maze.clone(),
            left_distance,
            front_distance,
            right_distance,
            encoder_orientation,
            cell_center: Vector {
                x: cell_center_x,
                y: cell_center_y,
            },
            sensor_width,
            center_offset,
            maybe_x,
            maybe_y,
        };

        self.left_encoder = left_encoder;
        self.right_encoder = right_encoder;
        self.orientation = orientation;
        self.last_buffer_len = buffer_len;

        (self.orientation, debug)
    }
}
