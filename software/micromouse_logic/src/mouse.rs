use core::f32;

use serde::Deserialize;
use serde::Serialize;

use crate::config::MechanicalConfig;
use crate::localize::{Localize, LocalizeConfig, LocalizeDebug};
use crate::map::Map;
use crate::map::MapConfig;
use crate::map::MapDebug;
use crate::math::Vector;
use crate::math::DIRECTION_0;
use crate::math::DIRECTION_3_PI_2;
use crate::math::DIRECTION_PI;
use crate::math::DIRECTION_PI_2;
use crate::math::{Direction, Orientation};
use crate::maze::MazeConfig;
use crate::motion::Motion;
use crate::motion::MotionConfig;
use crate::motion::MotionDebug;
use crate::path::{path_from_directions, MazeDirection, PathConfig};
use crate::path::{MazeOrientation, MazePosition, PathDebug};
use crate::path::{Path, Segment};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HardwareDebug {
    pub left_encoder: i32,
    pub right_encoder: i32,
    pub left_distance: u8,
    pub front_distance: u8,
    pub right_distance: u8,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MouseDebug {
    pub hardware: HardwareDebug,
    pub orientation: Orientation,
    pub path: PathDebug,
    pub localize: LocalizeDebug,
    pub motion: MotionDebug,
    pub battery: u16,
    pub time: u32,
    pub delta_time: u32,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MouseConfig {
    pub mechanical: MechanicalConfig,
    pub path: PathConfig,
    pub localize: LocalizeConfig,
    pub map: MapConfig,
    pub maze: MazeConfig,
    pub motion: MotionConfig,
}

pub struct Mouse {
    last_time: u32,
    map: Map,
    path: Path,
    motion: Motion,
    path_direction: Direction,
    localize: Localize,
    done: bool,
}

impl Mouse {
    pub fn new(
        config: &MouseConfig,
        orientation: Orientation,
        time: u32,
        left_encoder: i32,
        right_encoder: i32,
    ) -> Mouse {
        let mut path = Path::new(&config.path, time);

        /*
        let directions = [
            MazeDirection::North,
            MazeDirection::West,
            MazeDirection::South,
            MazeDirection::East,
        ];

        let starting_orientation = MazeOrientation {
            position: MazePosition { x: 7, y: 6 },
            direction: MazeDirection::East,
        };

        let path_segments =
            path_from_directions(&config.map.maze, starting_orientation, &directions);

        path.add_segments(&path_segments);
        */

        Mouse {
            last_time: time,
            map: Map::new(),
            localize: Localize::new(orientation, left_encoder, right_encoder),
            path,
            motion: Motion::new(&config.motion, time, left_encoder, right_encoder),
            path_direction: orientation.direction,
            done: true,
        }
    }

    pub fn update(
        &mut self,
        config: &MouseConfig,
        time: u32,
        battery: u16,
        left_encoder: i32,
        right_encoder: i32,
        left_distance: u8,
        front_distance: u8,
        right_distance: u8,
    ) -> (i32, i32, MouseDebug) {
        let delta_time = time - self.last_time;
        if self.done {
            let directions = [
                MazeDirection::East,
                MazeDirection::East,
                MazeDirection::North,
                MazeDirection::North,
                MazeDirection::West,
                MazeDirection::West,
                MazeDirection::West,
                MazeDirection::South,
                MazeDirection::South,
                MazeDirection::East,
            ];

            let starting_orientation = MazeOrientation {
                position: MazePosition { x: 7, y: 6 },
                direction: MazeDirection::East,
            };

            let path =
                path_from_directions(&config.maze, starting_orientation, &directions);

            self.path.add_segments(&path);
        }

        let (orientation, localize_debug) = self.localize.update(
            &config.mechanical,
            &config.maze,
            &config.localize,
            left_encoder,
            right_encoder,
            left_distance,
            front_distance,
            right_distance,
            self.path_direction,
            self.path.buffer_len(),
        );

        let (target_curvature, target_velocity, path_direction, done, path_debug) =
            self.path.update(&config.path, time, orientation);

        self.path_direction = path_direction.unwrap_or(orientation.direction);

        self.done = done;

        let (left_power, right_power, motion_debug) = self.motion.update(
            &config.motion,
            &config.mechanical,
            time,
            left_encoder,
            right_encoder,
            target_velocity,
            target_curvature,
        );

        let hardware_debug = HardwareDebug {
            left_encoder,
            right_encoder,
            left_distance,
            front_distance,
            right_distance,
        };

        let debug = MouseDebug {
            hardware: hardware_debug,
            orientation,
            path: path_debug,
            localize: localize_debug,
            motion: motion_debug,
            battery,
            time,
            delta_time,
        };

        self.last_time = time;

        (left_power, right_power, debug)
    }
}

pub struct TestMouse {}

impl TestMouse {
    pub fn new() -> TestMouse {
        TestMouse {}
    }

    pub fn update(
        &mut self,
        _config: &MouseConfig,
        time: u32,
        _left_encoder: i32,
        _right_encoder: i32,
    ) -> (f32, f32) {
        if time % 10000 <= 5000 {
            (0.0, 0.0)
        } else {
            (1.0, 1.0)
        }
    }
}
