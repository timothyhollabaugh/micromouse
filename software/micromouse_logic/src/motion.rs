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

/// Clips the linear power `power` such that both wheels are within their max powers
///
/// This also works for delta powers because the derivative is the same as the original
fn clip_linear_to_wheel_max(
    wheel_max: f32,
    power: f32,
    angular_ratio: f32,
) -> f32 {
    // when the angular ratio is 1.0, the left wheel will be stopped, so it does not care what
    // the linear power is
    let max_for_left = if angular_ratio == 1.0 {
        core::f32::INFINITY
    } else {
        wheel_max / (1.0 - angular_ratio)
    };

    // same for the right wheel at -1.0
    let max_for_right = if angular_ratio == -1.0 {
        core::f32::INFINITY
    } else {
        wheel_max / (1.0 + angular_ratio)
    };

    let max = nearest_zero(max_for_left, max_for_right);
    let clipped = signed_nearest_zero(power, max);
    clipped
}

/// Clips the change in angular ratio to keep the change in wheel powers within their max
fn clip_delta_angular_ratio_to_delta_angular_wheel_max(
    delta_wheel_max: f32,
    power: f32,
    delta_angular_ratio: f32,
) -> f32 {
    if power == 0.0 {
        0.0
    } else {
        let max = delta_wheel_max / power;
        let clipped = signed_nearest_zero(delta_angular_ratio, max);
        clipped
    }
}

/// Calculate the instantaneous curvature
fn curvature(
    config: &MechanicalConfig,
    delta_left: i32,
    delta_right: i32,
) -> f32 {
    if (delta_right == 0 && delta_left == 0) || delta_right == delta_left {
        return 0.0;
    }

    let delta_left_mm = config.ticks_to_mm(delta_left as f32);
    let delta_right_mm = config.ticks_to_mm(delta_right as f32);

    let r = (config.wheelbase as f32 / 2.0) * (delta_left_mm + delta_right_mm)
        / (delta_right_mm - delta_left_mm);

    1.0 / r
}

#[cfg(test)]
mod curvature_test {
    #[allow(unused_imports)]
    use crate::test::*;

    use super::curvature;
    use crate::config::MechanicalConfig;

    const CONFIG: MechanicalConfig = crate::config::MOUSE_2019_MECH;

    #[test]
    fn test_curvature_1m_radius() {
        assert_close(curvature(&CONFIG, 56625, 57090), 0.000110518115)
    }

    #[test]
    fn test_curvature_stopped() {
        assert_close(curvature(&CONFIG, 0, 0), 0.0)
    }

    #[test]
    fn test_curvature_strait() {
        assert_close(curvature(&CONFIG, 10, 10), 0.0)
    }

    #[test]
    fn test_curvature_spin() {
        assert_close(curvature(&CONFIG, 10, -10), core::f32::NEG_INFINITY)
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
pub struct MotionConfig {
    pub left_p: f32,
    pub left_i: f32,
    pub left_d: f32,
    pub left_f: f32,
    pub right_p: f32,
    pub right_i: f32,
    pub right_d: f32,
    pub right_f: f32,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MotionDebug {
    pub target_curvature: f32,
    pub target_velocity: f32,
    pub left_velocity: f32,
    pub right_velocity: f32,
    pub left_power: f32,
    pub right_power: f32,
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
        let left_pid = PIDController::new(
            config.left_p as f64,
            config.left_i as f64,
            config.left_d as f64,
        );

        let right_pid = PIDController::new(
            config.right_p as f64,
            config.right_i as f64,
            config.right_d as f64,
        );

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
    ) -> (f32, f32, MotionDebug) {
        self.left_pid.p_gain = config.left_p as f64;
        self.left_pid.i_gain = config.left_i as f64;
        self.left_pid.d_gain = config.left_d as f64;

        self.right_pid.p_gain = config.right_p as f64;
        self.right_pid.i_gain = config.right_i as f64;
        self.right_pid.d_gain = config.right_d as f64;

        let (target_left_velocity, target_right_velocity) =
            curvature_to_left_right(mech, target_velocity, target_curvature);

        let delta_time = time - self.last_time;
        let delta_left = left_encoder - self.last_left_encoder;
        let delta_right = right_encoder - self.last_right_encoder;

        let left_velocity =
            mech.ticks_to_mm(delta_left as f32) / delta_time as f32;
        let right_velocity =
            mech.ticks_to_mm(delta_right as f32) / delta_time as f32;

        let (left_power, right_power) = if delta_time > 0 {
            self.left_pid.set_target(target_left_velocity as f64);
            self.right_pid.set_target(target_right_velocity as f64);

            let left_power = target_left_velocity * config.left_f
                + self
                    .left_pid
                    .update(left_velocity as f64, delta_time as f64)
                    as f32;
            let right_power = target_right_velocity * config.right_f
                + self
                    .right_pid
                    .update(right_velocity as f64, delta_time as f64)
                    as f32;

            (left_power, right_power)
        } else {
            (0.0, 0.0)
        };

        let debug = MotionDebug {
            target_curvature,
            target_velocity,
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
