use core::f32;

use serde::Deserialize;
use serde::Serialize;

use crate::map::MapConfig;
use crate::maze::MazeConfig;
use crate::motion::MotionConfig;
use crate::mouse::MouseConfig;
use crate::path::PathConfig;

pub const MOUSE_MAZE_MAP: MapConfig = MapConfig {
    maze: MazeConfig {
        cell_width: 180.0,
        wall_width: 12.0,
    },
};

pub const MOUSE_SIM_PATH: PathConfig = PathConfig {
    p: 10.0,
    i: 0.0,
    d: 0.0,
    offset_p: 0.002,
};

pub const MOUSE_2020_MOTION: MotionConfig = MotionConfig {
    max_delta_power: 1.0,
    max_wheel_power: 0.6,
};

pub const MOUSE_2020_MECH: MechanicalConfig = MechanicalConfig {
    wheel_diameter: 29.5,
    gearbox_ratio: 30.0,
    ticks_per_rev: 12.0,
    wheelbase: 80.0,
    width: 64.0,
    length: 57.5,
    front_offset: 40.0,
};

pub const MOUSE_2020_MECH2: MechanicalConfig = MechanicalConfig {
    wheel_diameter: 32.0,
    gearbox_ratio: 29.86,
    ticks_per_rev: 12.0,
    wheelbase: 85.0,
    width: 64.0,
    length: 57.5,
    front_offset: 40.0,
};

pub const MOUSE_2020_PATH: PathConfig = PathConfig {
    p: 1.0,
    i: 0.0,
    d: 0.0,
    offset_p: 0.005,
};

pub const MOUSE_2020: MouseConfig = MouseConfig {
    mechanical: MOUSE_2020_MECH,
    path: MOUSE_2020_PATH,
    map: MOUSE_MAZE_MAP,
    motion: MOUSE_2020_MOTION,
    linear_power: 1.0,
};

pub const MOUSE_SIM_2020: MouseConfig = MouseConfig {
    mechanical: MOUSE_2020_MECH,
    path: MOUSE_SIM_PATH,
    map: MOUSE_MAZE_MAP,
    motion: MOUSE_2020_MOTION,
    linear_power: 1.0,
};

pub const MOUSE_2019_MECH: MechanicalConfig = MechanicalConfig {
    wheel_diameter: 32.0,
    gearbox_ratio: 75.81,
    ticks_per_rev: 12.0,
    wheelbase: 74.0,
    width: 64.0,
    length: 90.0,
    front_offset: 48.0,
};

pub const MOUSE_2019_PATH: PathConfig = PathConfig {
    p: 0.5,
    i: 0.0,
    d: 0.0,
    offset_p: 0.000,
};

pub const MOUSE_2019_MOTION: MotionConfig = MotionConfig {
    max_delta_power: 1.0,
    max_wheel_power: 0.6,
};

pub const MOUSE_2019: MouseConfig = MouseConfig {
    mechanical: MOUSE_2019_MECH,
    path: MOUSE_2019_PATH,
    map: MOUSE_MAZE_MAP,
    motion: MOUSE_2019_MOTION,
    linear_power: 1.0,
};

pub const MOUSE_SIM_2019: MouseConfig = MouseConfig {
    mechanical: MOUSE_2019_MECH,
    path: MOUSE_SIM_PATH,
    map: MOUSE_MAZE_MAP,
    motion: MOUSE_2019_MOTION,
    linear_power: 1.0,
};

/**
 *  Various physical parameters about the mouse
 */
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MechanicalConfig {
    pub wheel_diameter: f32,
    pub gearbox_ratio: f32,
    pub ticks_per_rev: f32,
    pub wheelbase: f32,
    pub width: f32,
    pub length: f32,
    pub front_offset: f32,
}

impl MechanicalConfig {
    pub fn ticks_per_mm(&self) -> f32 {
        (self.ticks_per_rev * self.gearbox_ratio)
            / (self.wheel_diameter * f32::consts::PI)
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
