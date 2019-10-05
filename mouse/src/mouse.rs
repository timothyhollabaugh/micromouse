use core::f32;

use crate::config::MouseConfig;
use crate::map::Map;
use crate::map::Orientation;
use crate::path::Path;

pub struct Mouse {
    map: Map,
    path: Path,
}

impl Mouse {
    pub fn new(
        config: &MouseConfig,
        orientation: Orientation,
        time: u32,
        left_encoder: i32,
        right_encoder: i32,
    ) -> Mouse {
        Mouse {
            map: Map::new(orientation, left_encoder, right_encoder),
            path: Path::new(&config.path, time),
        }
    }

    pub fn update(
        &mut self,
        config: &MouseConfig,
        time: u32,
        left_encoder: i32,
        right_encoder: i32,
    ) -> (f32, f32) {
        let linear_power = 0.0;

        let orientation = self
            .map
            .update(&config.mechanical, left_encoder, right_encoder);

        let angular_power = self.path.update(&config.path, time, orientation.position);

        let left_power = linear_power - angular_power;
        let right_power = linear_power + angular_power;

        (left_power, right_power)
    }
}
