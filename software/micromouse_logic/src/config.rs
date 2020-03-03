use core::f32;

use serde::Deserialize;
use serde::Serialize;

use crate::slow::map::MapConfig;
use crate::slow::maze::MazeConfig;

use crate::fast::localize::LocalizeConfig;
use crate::slow::motion_plan::MotionPlanConfig;

pub const MAZE: MazeConfig = MazeConfig {
    cell_width: 180.0,
    wall_width: 12.0,
};

pub const LOCALIZE: LocalizeConfig = LocalizeConfig { use_sensors: true };

pub const MAP: MapConfig = MapConfig {
    front_threhold: 200,
    left_threshold: 100,
    right_threshold: 100,
};

pub const MOTION_PLAN: MotionPlanConfig = MotionPlanConfig {};

pub mod sim {
    use crate::fast::motion_control::MotionControlConfig;
    use crate::fast::motor_control::{MotorControlConfig, PidfConfig};
    use crate::fast::path::PathHandlerConfig;
    use crate::fast::turn::TurnHandlerConfig;
    use crate::mouse::MouseConfig;
    use core::f32::consts::FRAC_PI_8;

    pub const PIDF: PidfConfig = PidfConfig {
        p: 0.0,
        i: 0.0,
        d: 0.0,
        f: 1000.0,
    };

    pub const MOTION_CONTROL: MotionControlConfig = MotionControlConfig {
        path: PathHandlerConfig {
            p: 0.1,
            i: 0.0,
            d: 0.0,
            offset_p: 0.02,
            velocity: 0.5,
        },
        turn: TurnHandlerConfig {
            rad_per_sec: 0.1,
            p: 1.0,
            i: 0.0,
            d: 0.0,
            tolerance: FRAC_PI_8 / 2.0,
        },
        motor_control: MotorControlConfig {
            left_pidf: PIDF,
            left_reverse: false,
            right_pidf: PIDF,
            right_reverse: false,
        },
    };

    pub const MOUSE_2020: MouseConfig = MouseConfig {
        mechanical: super::mouse_2020::MECH,
        maze: super::MAZE,
        map: super::MAP,
        motion_plan: super::MOTION_PLAN,
        localize: super::LOCALIZE,
        motion_control: MOTION_CONTROL,
    };

    pub const MOUSE_2019: MouseConfig = MouseConfig {
        mechanical: super::mouse_2019::MECH,
        maze: super::MAZE,
        map: super::MAP,
        motion_plan: super::MOTION_PLAN,
        localize: super::LOCALIZE,
        motion_control: MOTION_CONTROL,
    };
}

pub mod mouse_2020 {
    use crate::config::MechanicalConfig;
    use crate::fast::motion_control::MotionControlConfig;
    use crate::fast::motor_control::{MotorControlConfig, PidfConfig};
    use crate::fast::path::PathHandlerConfig;
    use crate::fast::turn::TurnHandlerConfig;
    use crate::mouse::MouseConfig;
    use core::f32::consts::FRAC_PI_8;

    pub const MECH: MechanicalConfig = MechanicalConfig {
        wheel_diameter: 32.0,
        gearbox_ratio: 75.81,
        ticks_per_rev: 12.0,
        wheelbase: 78.0,
        width: 64.0,
        length: 57.5,
        front_offset: 40.0,

        front_sensor_offset_x: 40.0,
        left_sensor_offset_y: 32.0,
        left_sensor_offset_x: 26.0,
        right_sensor_offset_y: 32.0,
        right_sensor_offset_x: 26.0,

        front_sensor_limit: 200,
        left_sensor_limit: 100,
        right_sensor_limit: 100,
    };

    pub const PIDF: PidfConfig = PidfConfig {
        p: 5000.0,
        i: 0.5,
        d: 25000.0,
        f: 0.0,
    };

    pub const MOUSE: MouseConfig = MouseConfig {
        mechanical: MECH,
        maze: super::MAZE,
        map: super::MAP,
        motion_plan: super::MOTION_PLAN,
        localize: super::LOCALIZE,
        motion_control: MotionControlConfig {
            path: PathHandlerConfig {
                p: 0.07,
                i: 0.0,
                d: 0.0,
                offset_p: 0.01,
                velocity: 0.3,
            },
            turn: TurnHandlerConfig {
                rad_per_sec: 0.05,
                p: 0.10,
                i: 0.0,
                d: 0.0,
                tolerance: 0.02,
            },
            motor_control: MotorControlConfig {
                left_pidf: PIDF,
                left_reverse: false,
                right_pidf: PIDF,
                right_reverse: true,
            },
        },
    };
}

pub mod mouse_2019 {
    use crate::config::MechanicalConfig;
    use crate::fast::motion_control::MotionControlConfig;
    use crate::fast::motor_control::{MotorControlConfig, PidfConfig};
    use crate::fast::path::PathHandlerConfig;
    use crate::fast::turn::TurnHandlerConfig;
    use crate::mouse::MouseConfig;
    use core::f32::consts::FRAC_PI_8;

    pub const MECH: MechanicalConfig = MechanicalConfig {
        wheel_diameter: 32.0,
        gearbox_ratio: 75.81,
        ticks_per_rev: 12.0,
        wheelbase: 74.0,
        width: 64.0,
        length: 90.0,
        front_offset: 48.0,

        front_sensor_offset_x: 48.0,
        left_sensor_offset_y: 32.0,
        left_sensor_offset_x: 30.0,
        right_sensor_offset_y: 32.0,
        right_sensor_offset_x: 30.0,

        front_sensor_limit: 200,
        left_sensor_limit: 150,
        right_sensor_limit: 150,
    };

    pub const PIDF: PidfConfig = PidfConfig {
        p: 5000.0,
        i: 0.5,
        d: 25000.0,
        f: 0.0,
    };

    pub const MOUSE: MouseConfig = MouseConfig {
        mechanical: MECH,
        maze: super::MAZE,
        map: super::MAP,
        motion_plan: super::MOTION_PLAN,
        localize: super::LOCALIZE,
        motion_control: MotionControlConfig {
            path: PathHandlerConfig {
                p: 0.15,
                i: 0.0,
                d: 0.0,
                offset_p: 0.01,
                velocity: 0.2,
            },
            turn: TurnHandlerConfig {
                rad_per_sec: 0.05,
                p: 1.0,
                i: 0.0,
                d: 0.0,
                tolerance: 0.02,
            },
            motor_control: MotorControlConfig {
                left_pidf: PIDF,
                left_reverse: false,
                right_pidf: PIDF,
                right_reverse: false,
            },
        },
    };
}

/**
 *  Various physical parameters about the mouse
 */
#[derive(Copy, Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct MechanicalConfig {
    /// The diameter of the wheels
    pub wheel_diameter: f32,

    /// The gearbox ratio between the encoder and the wheels
    pub gearbox_ratio: f32,

    /// The ticks per revolution of the encoder
    pub ticks_per_rev: f32,

    /// The distance between the centers of the wheels
    pub wheelbase: f32,

    /// The width of the body
    pub width: f32,

    /// The length of the body
    pub length: f32,

    /// The offset from the front of the body to the center of rotation
    pub front_offset: f32,

    /// The distance from the center to the front distance sensor
    pub front_sensor_offset_x: f32,

    /// The distance from the center to the left distance sensor
    pub left_sensor_offset_y: f32,
    pub left_sensor_offset_x: f32,

    /// The distance from the center to the right distance sensor
    pub right_sensor_offset_y: f32,
    pub right_sensor_offset_x: f32,

    pub front_sensor_limit: u8,
    pub left_sensor_limit: u8,
    pub right_sensor_limit: u8,
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
