#[allow(unused_imports)]
use libm::F32Ext;

use pid_control::Controller;
use pid_control::PIDController;

use crate::config::MechanicalConfig;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PidfConfig {
    pub p: f32,
    pub i: f32,
    pub d: f32,
    pub f: f32,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MotorControlConfig {
    pub left_pidf: PidfConfig,
    pub left_reverse: bool,
    pub right_pidf: PidfConfig,
    pub right_reverse: bool,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MotorControlDebug {
    pub target_left_velocity: f64,
    pub target_right_velocity: f64,
    pub left_velocity: f64,
    pub right_velocity: f64,
    pub left_power: i32,
    pub right_power: i32,
}

/// Takes a linear power and a curvature. The curvature is the inverse of the radius of a circle
/// that it is following instantaneously, and is equal to the angular speed over the linear speed.
/// If angular velocity is the derivative of angular position with respect to time, curvature is
/// the derivative of angular position with respect to linear position. Thus, it does not change
/// if the linear velocity changes.
///
/// This will then calculate the desired speeds for the left and right motors and do pid on them
///
pub struct MotorControl {
    left_pid: PIDController,
    right_pid: PIDController,
    last_time: u32,
    last_left_encoder: i32,
    last_right_encoder: i32,
}

// Good food in New Orleans according to my uncle
// Cafe du moire

impl MotorControl {
    /// Create a new Motion. The time and encoder values are used to calculate the deltas from now
    /// until the update function is called.
    pub fn new(
        config: &MotorControlConfig,
        time: u32,
        left_encoder: i32,
        right_encoder: i32,
    ) -> MotorControl {
        let mut left_pid = PIDController::new(
            config.left_pidf.p as f64,
            config.left_pidf.i as f64,
            config.left_pidf.d as f64,
        );

        left_pid.set_limits(-10000.0, 10000.0);

        let mut right_pid = PIDController::new(
            config.right_pidf.p as f64,
            config.right_pidf.i as f64,
            config.right_pidf.d as f64,
        );

        right_pid.set_limits(-10000.0, 10000.0);

        MotorControl {
            left_pid,
            right_pid,
            last_time: time,
            last_left_encoder: left_encoder,
            last_right_encoder: right_encoder,
        }
    }

    /// Updates
    pub fn update(
        &mut self,
        config: &MotorControlConfig,
        mech: &MechanicalConfig,
        time: u32,
        left_encoder: i32,
        right_encoder: i32,
        target_left_velocity: f32,
        target_right_velocity: f32,
    ) -> (i32, i32, MotorControlDebug) {
        self.left_pid.p_gain = config.left_pidf.p as f64;
        self.left_pid.i_gain = config.left_pidf.i as f64;
        self.left_pid.d_gain = config.left_pidf.d as f64;

        self.right_pid.p_gain = config.right_pidf.p as f64;
        self.right_pid.i_gain = config.right_pidf.i as f64;
        self.right_pid.d_gain = config.right_pidf.d as f64;

        let delta_time = time - self.last_time;

        let target_left_velocity = mech.mm_to_ticks(target_left_velocity) as f64;
        let target_right_velocity = mech.mm_to_ticks(target_right_velocity) as f64;

        let delta_left = left_encoder - self.last_left_encoder;
        let delta_right = right_encoder - self.last_right_encoder;

        let left_velocity = delta_left as f64 / delta_time as f64;
        let right_velocity = delta_right as f64 / delta_time as f64;

        let (left_power, right_power) = if delta_time > 0 {
            self.left_pid.set_target(target_left_velocity);
            self.right_pid.set_target(target_right_velocity);

            let mut left_power = (target_left_velocity * config.left_pidf.f as f64)
                as i32
                + self.left_pid.update(left_velocity, delta_time as f64) as i32;

            if config.left_reverse {
                left_power *= -1;
            }

            let mut right_power = (target_right_velocity * config.right_pidf.f as f64)
                as i32
                + self.right_pid.update(right_velocity, delta_time as f64) as i32;

            if config.right_reverse {
                right_power *= -1;
            }

            (left_power, right_power)
        } else {
            (0, 0)
        };

        let debug = MotorControlDebug {
            target_left_velocity,
            target_right_velocity,
            left_velocity,
            right_velocity,
            left_power,
            right_power,
        };

        self.last_time = time;
        self.last_left_encoder = left_encoder;
        self.last_right_encoder = right_encoder;

        (left_power, right_power, debug)
    }
}
