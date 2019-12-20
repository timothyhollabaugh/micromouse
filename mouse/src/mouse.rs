use core::f32;

use crate::config::MouseConfig;
use crate::map::Map;
use crate::map::MapDebug;
use crate::map::Orientation;
use crate::map::Vector;
use crate::motion::Motion;
use crate::motion::MotionDebug;
use crate::path;
use crate::path::Path;
use crate::path::PathDebug;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct MouseDebug {
    pub orientation: Orientation,
    pub path: PathDebug,
    pub map: MapDebug,
    pub motion: MotionDebug,
}

pub struct Mouse {
    map: Map,
    path: Path,
    motion: Motion,
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
        let path = Path::new(&config.path, time);

        Mouse {
            map: Map::new(orientation, left_encoder, right_encoder),
            path,
            motion: Motion::new(&config.motion, time),
            done: true,
        }
    }

    pub fn update(
        &mut self,
        config: &MouseConfig,
        time: u32,
        left_encoder: i32,
        right_encoder: i32,
        left_distance: u8,
        front_distance: u8,
        right_distance: u8,
    ) -> (f32, f32, MouseDebug) {
        if self.done {
            self.path
                .add_segments(&path::rounded_rectangle(
                    Vector {
                        x: 1170.0,
                        y: 1350.0,
                    },
                    540.0,
                    180.0,
                    80.0,
                ))
                .ok();
        }

        let (orientation, map_debug) = self.map.update(
            &config.mechanical,
            &config.map.maze,
            left_encoder,
            right_encoder,
            left_distance,
            front_distance,
            right_distance,
        );

        let (angular_power, done, path_debug) = self.path.update(&config.path, time, orientation);

        self.done = done;

        let linear_power = if done { 0.0 } else { 1.0 };

        let (left_power, right_power, motion_debug) =
            self.motion
                .update(&config.motion, time, linear_power, angular_power);

        let debug = MouseDebug {
            orientation,
            path: path_debug,
            map: map_debug,
            motion: motion_debug,
        };

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
