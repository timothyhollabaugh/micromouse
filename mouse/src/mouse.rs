use core::f32;

use crate::config::MouseConfig;
use crate::map::Map;
use crate::map::Orientation;
use crate::map::Vector;
use crate::path;
use crate::path::Path;
use crate::path::PathDebug;

#[derive(Debug, Clone)]
pub struct MouseDebug {
    pub orientation: Orientation,
    pub path_debug: PathDebug,
}

pub struct Mouse {
    map: Map,
    path: Path,
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
            done: true,
        }
    }

    pub fn update(
        &mut self,
        config: &MouseConfig,
        time: u32,
        left_encoder: i32,
        right_encoder: i32,
    ) -> (f32, f32, MouseDebug) {
        if self.done {
            self.path
                .add_segments(&path::rounded_rectangle(
                    Vector {
                        x: 1000.0,
                        y: 1000.0,
                    },
                    700.0,
                    400.0,
                    100.0,
                ))
                .ok();
        }
        let orientation = self
            .map
            .update(&config.mechanical, left_encoder, right_encoder);

        let (angular_power, done, path_debug) = self.path.update(&config.path, time, orientation);

        self.done = done;

        let linear_power = if done { 0.0 } else { 1.0 };

        let left_power = linear_power - angular_power;
        let right_power = linear_power + angular_power;

        let debug = MouseDebug {
            orientation,
            path_debug,
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
