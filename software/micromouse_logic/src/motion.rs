#[allow(unused_imports)]
use libm::F32Ext;

use pid_control::Controller;
use pid_control::PIDController;

use crate::config::MechanicalConfig;
use serde::Deserialize;
use serde::Serialize;

/// Find the number that is closest to 0
pub fn nearest_zero(f1: f32, f2: f32) -> f32 {
    if f1.abs() < f2.abs() {
        f1
    } else {
        f2
    }
}

/// Find the number that is closest to 0, but keep the sign of the first number
///
/// ```rust
/// use micromouse_logic::motion::signed_nearest_zero;
///
/// assert_eq!(signed_nearest_zero(1.0, 0.5), 0.5);
/// assert_eq!(signed_nearest_zero(-1.0, -0.5), -0.5);
/// assert_eq!(signed_nearest_zero(1.0, -0.5), 0.5);
/// assert_eq!(signed_nearest_zero(-1.0, 0.5), -0.5);
///
/// assert_eq!(signed_nearest_zero(0.5, 1.0), 0.5);
/// assert_eq!(signed_nearest_zero(-0.5, -1.0), -0.5);
/// assert_eq!(signed_nearest_zero(0.5, -1.0), 0.5);
/// assert_eq!(signed_nearest_zero(-0.5, 1.0), -0.5);
/// ```
pub fn signed_nearest_zero(f1: f32, f2: f32) -> f32 {
    if f1.abs() < f2.abs() {
        f1
    } else {
        match (f1 > 0.0, f2 > 0.0) {
            (true, true) | (false, false) => f2,
            (true, false) | (false, true) => -f2,
        }
    }
}

fn curvature_to_left_right(
    config: &MechanicalConfig,
    velocity: f32,
    curvature: f32,
) -> (f32, f32) {
    let rotations_per_ms = velocity * curvature;
    let angular_mm_per_ms = rotations_per_ms * config.wheelbase / 2.0;
    let left = velocity - angular_mm_per_ms;
    let right = velocity + angular_mm_per_ms;
    (left, right)
}

#[cfg(test)]
mod curvature_to_left_right_test {
    #[allow(unused_imports)]
    use crate::test::*;

    use crate::config::MechanicalConfig;
    use crate::motion::curvature_to_left_right;

    const CONFIG: MechanicalConfig = crate::config::MOUSE_2019_MECH;

    #[test]
    fn test_curvature_to_left_right_circle() {
        let (left, right) = curvature_to_left_right(&CONFIG, 0.5, 1.0 / 90.0);
        assert_close(left, 0.294444);
        assert_close(right, 0.705556);
    }

    #[test]
    fn test_curvature_to_left_right_straight() {
        let (left, right) = curvature_to_left_right(&CONFIG, 0.5, 0.0);
        assert_close(left, 0.5);
        assert_close(right, 0.5);
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PidfConfig {
    pub p: f32,
    pub i: f32,
    pub d: f32,
    pub f: f32,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MotionConfig {
    pub left_pidf: PidfConfig,
    pub right_pidf: PidfConfig,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MotionDebug {
    pub target_curvature: f32,
    pub target_velocity: f32,
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
pub struct Motion {
    left_pid: PIDController,
    right_pid: PIDController,
    last_time: u32,
    last_left_encoder: i32,
    last_right_encoder: i32,
}

// Good food in New Orleans according to my uncle
// Cafe du moire

impl Motion {
    /// Create a new Motion. The time and encoder values are used to calculate the deltas from now
    /// until the update function is called.
    pub fn new(
        config: &MotionConfig,
        time: u32,
        left_encoder: i32,
        right_encoder: i32,
    ) -> Motion {
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

        Motion {
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
        config: &MotionConfig,
        mech: &MechanicalConfig,
        time: u32,
        left_encoder: i32,
        right_encoder: i32,
        target_velocity: f32,
        target_curvature: f32,
    ) -> (i32, i32, MotionDebug) {
        self.left_pid.p_gain = config.left_pidf.p as f64;
        self.left_pid.i_gain = config.left_pidf.i as f64;
        self.left_pid.d_gain = config.left_pidf.d as f64;

        self.right_pid.p_gain = config.right_pidf.p as f64;
        self.right_pid.i_gain = config.right_pidf.i as f64;
        self.right_pid.d_gain = config.right_pidf.d as f64;

        let delta_time = time - self.last_time;

        let (target_left_velocity, target_right_velocity) =
            curvature_to_left_right(mech, target_velocity, target_curvature);

        let target_left_velocity = mech.mm_to_ticks(target_left_velocity) as f64;
        let target_right_velocity = mech.mm_to_ticks(target_right_velocity) as f64;

        let delta_left = left_encoder - self.last_left_encoder;
        let delta_right = right_encoder - self.last_right_encoder;

        let left_velocity = delta_left as f64 / delta_time as f64;
        let right_velocity = delta_right as f64 / delta_time as f64;

        let (left_power, right_power) = if delta_time > 0 {
            self.left_pid.set_target(target_left_velocity);
            self.right_pid.set_target(target_right_velocity);

            let left_power = (target_left_velocity * config.left_pidf.f as f64) as i32
                + self.left_pid.update(left_velocity, delta_time as f64) as i32;

            let right_power = (target_right_velocity * config.right_pidf.f as f64) as i32
                + self.right_pid.update(right_velocity, delta_time as f64) as i32;

            (left_power, right_power)
        } else {
            (0, 0)
        };

        let debug = MotionDebug {
            target_curvature,
            target_velocity,
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
