use core::f32;

use crate::map::Map;
use crate::path::Path;
use crate::path::PathConfig;

pub struct Config {
    pub mouse: MouseConfig,
    pub path: PathConfig,
}

/**
 *  Various physical parameters about the mouse
 */
#[derive(Copy, Clone, Debug)]
pub struct MouseConfig {
    pub wheel_diameter: f32,
    pub gearbox_ratio: f32,
    pub ticks_per_rev: f32,
    pub wheelbase: f32,
    pub width: f32,
    pub length: f32,
    pub front_offset: f32,
}

impl MouseConfig {
    pub fn ticks_per_mm(&self) -> f32 {
        (self.ticks_per_rev * self.gearbox_ratio) / (self.wheel_diameter * f32::consts::PI)
    }

    pub fn ticks_to_mm(&self, ticks: f32) -> f32 {
        ticks / self.ticks_per_mm()
    }

    pub fn mm_to_ticks(&self, mm: f32) -> f32 {
        mm * self.ticks_per_mm()
    }

    pub fn ticks_per_rad(&self) -> f32 {
        self.mm_to_ticks(self.wheelbase / 2.0)
    }

    pub fn ticks_to_rads(&self, ticks: f32) -> f32 {
        ticks / self.ticks_per_rad()
    }

    pub fn rads_to_ticks(&self, rads: f32) -> f32 {
        rads * self.ticks_per_rad()
    }

    pub fn mm_per_rad(&self) -> f32 {
        self.wheelbase / 2.0
    }

    pub fn mm_to_rads(&self, mm: f32) -> f32 {
        mm / self.mm_per_rad()
    }

    pub fn rads_to_mm(&self, rads: f32) -> f32 {
        rads * self.mm_per_rad()
    }
}

struct Mouse {
    map: Map,
    path: Path,
}

impl Mouse {
    fn update(
        &mut self,
        config: Config,
        time: u32,
        left_encoder: i32,
        right_encoder: i32,
        left_distance: u8,
        right_distance: u8,
        front_distance: u8,
    ) -> (f32, f32) {
        let linear_power = 1.0;

        let orientation = self
            .map
            .update(config.mouse, time, left_encoder, right_encoder);

        let angular_power = self.path.update(config.path, time, orientation.position);

        let left_power = linear_power - angular_power;
        let right_power = linear_power + angular_power;

        (left_power, right_power)
    }
}
