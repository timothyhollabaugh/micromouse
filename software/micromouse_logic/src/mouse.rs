use core::f32;

use serde::Deserialize;
use serde::Serialize;

use crate::config::MechanicalConfig;
use crate::map::Map;
use crate::map::MapConfig;
use crate::map::MapDebug;
use crate::math::Orientation;
use crate::math::Vector;
use crate::math::DIRECTION_0;
use crate::math::DIRECTION_3_PI_2;
use crate::math::DIRECTION_PI;
use crate::math::DIRECTION_PI_2;
use crate::motion::Motion;
use crate::motion::MotionConfig;
use crate::motion::MotionDebug;
use crate::path::PathConfig;
use crate::path::PathDebug;
use crate::path::{Path, Segment};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MouseDebug {
    pub orientation: Orientation,
    pub path: PathDebug,
    pub map: MapDebug,
    pub motion: MotionDebug,
    pub battery: u16,
    pub time: u32,
    pub delta_time: u32,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MouseConfig {
    pub mechanical: MechanicalConfig,
    pub path: PathConfig,
    pub map: MapConfig,
    pub motion: MotionConfig,
}

pub struct Mouse {
    last_time: u32,
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
            last_time: time,
            map: Map::new(orientation, left_encoder, right_encoder),
            path,
            motion: Motion::new(&config.motion, time, left_encoder, right_encoder),
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
            let start = Vector {
                x: 6.5 * 180.0,
                y: 6.5 * 180.0,
            };

            let width = 3.0 * 180.0;
            let height = 3.0 * 180.0;
            let radius = 180.0;

            /*
            self.path
                .add_segments(&[Segment::line(
                    start,
                    Vector {
                        x: start.x + width,
                        y: start.y,
                    },
                )])
                .ok();
                */

            self.path
                .add_segments(&[
                    Segment::corner(start, DIRECTION_3_PI_2, DIRECTION_0, radius),
                    Segment::corner(
                        Vector {
                            x: start.x,
                            y: start.y + 2.0 * 180.0,
                        },
                        DIRECTION_PI,
                        DIRECTION_3_PI_2,
                        radius,
                    ),
                    Segment::line(
                        Vector {
                            x: start.x + width - radius,
                            y: start.y + 2.0 * 180.0,
                        },
                        Vector {
                            x: start.x + radius,
                            y: start.y + 2.0 * 180.0,
                        },
                    ),
                    Segment::corner(
                        Vector {
                            x: start.x + width,
                            y: start.y + 2.0 * 180.0,
                        },
                        DIRECTION_PI_2,
                        DIRECTION_PI,
                        radius,
                    ),
                    Segment::corner(
                        Vector {
                            x: start.x + width,
                            y: start.y,
                        },
                        DIRECTION_0,
                        DIRECTION_PI_2,
                        radius,
                    ),
                    Segment::line(
                        Vector {
                            x: start.x + radius,
                            y: start.y,
                        },
                        Vector {
                            x: start.x + width - radius,
                            y: start.y,
                        },
                    ),
                ])
                .ok();

            /*
            self.path
                .add_segments(&[
                    Segment::corner(start, DIRECTION_3_PI_2, DIRECTION_0, radius),
                    Segment::line(
                        Vector {
                            x: start.x,
                            y: start.y + height - radius,
                        },
                        Vector {
                            x: start.x,
                            y: start.y + radius,
                        },
                    ),
                    Segment::corner(
                        Vector {
                            x: start.x,
                            y: start.y + height,
                        },
                        DIRECTION_PI,
                        DIRECTION_3_PI_2,
                        radius,
                    ),
                    Segment::line(
                        Vector {
                            x: start.x + width - radius,
                            y: start.y + height,
                        },
                        Vector {
                            x: start.x + radius,
                            y: start.y + height,
                        },
                    ),
                    Segment::corner(
                        Vector {
                            x: start.x + width,
                            y: start.y + height,
                        },
                        DIRECTION_PI_2,
                        DIRECTION_PI,
                        radius,
                    ),
                    Segment::line(
                        Vector {
                            x: start.x + width,
                            y: start.y + radius,
                        },
                        Vector {
                            x: start.x + width,
                            y: start.y + height - radius,
                        },
                    ),
                    Segment::corner(
                        Vector {
                            x: start.x + width,
                            y: start.y,
                        },
                        DIRECTION_0,
                        DIRECTION_PI_2,
                        radius,
                    ),
                    Segment::line(
                        Vector {
                            x: start.x + radius,
                            y: start.y,
                        },
                        Vector {
                            x: start.x + width - radius,
                            y: start.y,
                        },
                    ),
                ])
                .ok();
                */
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

        let (target_curvature, target_velocity, done, path_debug) =
            self.path.update(&config.path, time, orientation);

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

        let debug = MouseDebug {
            orientation,
            path: path_debug,
            map: map_debug,
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
