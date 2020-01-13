use std::f32;

use serde::Deserialize;
use serde::Serialize;

use micromouse_logic::math::Orientation;
use micromouse_logic::mouse::Mouse;
use micromouse_logic::mouse::MouseConfig;
use micromouse_logic::mouse::MouseDebug;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SimulationDebug {
    pub mouse: MouseDebug,
    pub left_encoder: i32,
    pub right_encoder: i32,
    pub orientation: Orientation,
    pub config: SimulationConfig,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub mouse: MouseConfig,
    pub max_speed: f32,
    pub initial_orientation: Orientation,
    pub millis_per_step: u32,

    /// The max speed a wheel can accelerate by before slipping
    pub max_wheel_accel: f32,
}

impl SimulationConfig {
    pub fn sec_per_step(&self) -> f32 {
        self.millis_per_step as f32 / 1000.0
    }
}

pub struct Simulation {
    mouse: Mouse,
    orientation: Orientation,
    last_left_wheel_speed: f32,
    last_right_wheel_speed: f32,
    left_encoder: i32,
    right_encoder: i32,
    time: u32,
}

impl Simulation {
    pub fn new(config: &SimulationConfig) -> Simulation {
        Simulation {
            mouse: Mouse::new(
                &config.mouse,
                config.initial_orientation,
                0,
                0,
                0,
            ),
            orientation: config.initial_orientation,
            left_encoder: 0,
            right_encoder: 0,
            last_left_wheel_speed: 0.0,
            last_right_wheel_speed: 0.0,
            time: 0,
        }
    }

    pub fn default_config() -> SimulationConfig {
        SimulationConfig::default()
    }

    pub fn update(&mut self, config: &SimulationConfig) -> SimulationDebug {
        // Update the mouse for the current time
        let (raw_left_power, raw_right_power, mouse_debug) = self.mouse.update(
            &config.mouse,
            self.time,
            0,
            self.left_encoder,
            self.right_encoder,
            255,
            255,
            255,
        );

        // Make sure the wheel powers are in range -1.0 to 1.0

        let left_power = if raw_left_power > 1.0 {
            1.0
        } else if raw_left_power < -1.0 {
            -1.0
        } else {
            raw_left_power
        };

        let right_power = if raw_right_power > 1.0 {
            1.0
        } else if raw_right_power < -1.0 {
            -1.0
        } else {
            raw_right_power
        };

        // Update the state for the next run
        let left_wheel_speed = left_power * config.max_speed;
        let right_wheel_speed = right_power * config.max_speed;

        let delta_left_wheel = config.mouse.mechanical.mm_to_ticks(
            left_wheel_speed * (config.millis_per_step as f32 / 1000.0),
        ) as i32;

        let delta_right_wheel = config.mouse.mechanical.mm_to_ticks(
            right_wheel_speed * (config.millis_per_step as f32 / 1000.0),
        ) as i32;

        self.left_encoder += delta_left_wheel;
        self.right_encoder += delta_right_wheel;
        self.time += config.millis_per_step;

        let left_accel = (left_wheel_speed - self.last_left_wheel_speed)
            / config.sec_per_step();
        let right_accel = (right_wheel_speed - self.last_right_wheel_speed)
            / config.sec_per_step();

        let left_ground_speed = if left_accel > config.max_wheel_accel {
            self.last_left_wheel_speed
                + config.max_wheel_accel * config.sec_per_step()
        } else {
            left_wheel_speed
        };

        let right_ground_speed = if right_accel > config.max_wheel_accel {
            self.last_right_wheel_speed
                + config.max_wheel_accel * config.sec_per_step()
        } else {
            right_wheel_speed
        };

        let delta_left_ground = config
            .mouse
            .mechanical
            .mm_to_ticks(left_ground_speed * config.sec_per_step())
            as i32;

        let delta_right_ground = config
            .mouse
            .mechanical
            .mm_to_ticks(right_ground_speed * config.sec_per_step())
            as i32;

        self.orientation.update_from_encoders(
            &config.mouse.mechanical,
            delta_left_ground,
            delta_right_ground,
        );

        // Collect debug info from this run
        let debug = SimulationDebug {
            mouse: mouse_debug,
            left_encoder: self.left_encoder,
            right_encoder: self.right_encoder,
            orientation: self.orientation,
            config: config.clone(),
        };

        debug
    }
}
