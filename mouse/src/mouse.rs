use core::f32;
use core::f32::consts::FRAC_PI_2;

use crate::config::MouseConfig;
use crate::map::Map;
use crate::map::Orientation;
use crate::map::Vector;
use crate::path::Path;
use crate::path::PathDebug;
use crate::path::Segment;

#[derive(Debug)]
pub struct MouseDebug<'a> {
    pub orientation: Orientation,
    pub path_debug: PathDebug<'a>,
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
        let mut path = Path::new(&config.path, time);

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
            self.path.add_segments(&[
                Segment::Arc(
                    Vector {
                        x: 1000.0,
                        y: 1090.0,
                    },
                    Vector {
                        x: 1090.0,
                        y: 1090.0,
                    },
                    -FRAC_PI_2,
                ),
                Segment::Line(
                    Vector {
                        x: 1000.0,
                        y: 1910.0,
                    },
                    Vector {
                        x: 1000.0,
                        y: 1090.0,
                    },
                ),
                Segment::Arc(
                    Vector {
                        x: 1090.0,
                        y: 2000.0,
                    },
                    Vector {
                        x: 1090.0,
                        y: 1910.0,
                    },
                    -FRAC_PI_2,
                ),
                Segment::Line(
                    Vector {
                        x: 1210.0,
                        y: 2000.0,
                    },
                    Vector {
                        x: 1090.0,
                        y: 2000.0,
                    },
                ),
                Segment::Arc(
                    Vector {
                        x: 1300.0,
                        y: 1910.0,
                    },
                    Vector {
                        x: 1210.0,
                        y: 1910.0,
                    },
                    -FRAC_PI_2,
                ),
                Segment::Line(
                    Vector {
                        x: 1300.0,
                        y: 1590.0,
                    },
                    Vector {
                        x: 1300.0,
                        y: 1910.0,
                    },
                ),
                Segment::Arc(
                    Vector {
                        x: 1390.0,
                        y: 1500.0,
                    },
                    Vector {
                        x: 1390.0,
                        y: 1590.0,
                    },
                    FRAC_PI_2,
                ),
                Segment::Line(
                    Vector {
                        x: 1610.0,
                        y: 1500.0,
                    },
                    Vector {
                        x: 1390.0,
                        y: 1500.0,
                    },
                ),
                Segment::Arc(
                    Vector {
                        x: 1700.0,
                        y: 1590.0,
                    },
                    Vector {
                        x: 1610.0,
                        y: 1590.0,
                    },
                    FRAC_PI_2,
                ),
                Segment::Line(
                    Vector {
                        x: 1700.0,
                        y: 1910.0,
                    },
                    Vector {
                        x: 1700.0,
                        y: 1590.0,
                    },
                ),
                Segment::Arc(
                    Vector {
                        x: 1790.0,
                        y: 2000.0,
                    },
                    Vector {
                        x: 1790.0,
                        y: 1910.0,
                    },
                    -FRAC_PI_2,
                ),
                Segment::Line(
                    Vector {
                        x: 1910.0,
                        y: 2000.0,
                    },
                    Vector {
                        x: 1790.0,
                        y: 2000.0,
                    },
                ),
                Segment::Arc(
                    Vector {
                        x: 2000.0,
                        y: 1910.0,
                    },
                    Vector {
                        x: 1910.0,
                        y: 1910.0,
                    },
                    -FRAC_PI_2,
                ),
                Segment::Line(
                    Vector {
                        x: 2000.0,
                        y: 1090.0,
                    },
                    Vector {
                        x: 2000.0,
                        y: 1910.0,
                    },
                ),
                Segment::Arc(
                    Vector {
                        x: 1910.0,
                        y: 1000.0,
                    },
                    Vector {
                        x: 1910.0,
                        y: 1090.0,
                    },
                    -FRAC_PI_2,
                ),
                Segment::Line(
                    Vector {
                        x: 1090.0,
                        y: 1000.0,
                    },
                    Vector {
                        x: 1910.0,
                        y: 1000.0,
                    },
                ),
            ]);
        }
        let orientation = self
            .map
            .update(&config.mechanical, left_encoder, right_encoder);

        let (angular_power, done, path_debug) =
            self.path.update(&config.path, time, orientation.position);

        self.done = done;

        let linear_power = if done { 0.0 } else { 0.5 };

        let left_power = linear_power - angular_power;
        let right_power = linear_power + angular_power;

        let debug = MouseDebug {
            orientation,
            path_debug,
        };

        (left_power, right_power, debug)
    }
}
