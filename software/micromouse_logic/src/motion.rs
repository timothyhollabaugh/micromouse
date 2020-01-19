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

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MotionConfig {
    pub p: f32,
    pub i: f32,
    pub d: f32,
    pub f: f32,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MotionDebug {
    pub delta_left: i32,
    pub delta_right: i32,
    pub target_curvature: f32,
    pub current_curvature: f32,
    pub power_curvature: f32,
    pub linear_power: f32,
    pub angular_power: f32,
    pub left_power: f32,
    pub right_power: f32,
}

/// Takes a linear power and a curvature. The curvature is the inverse of the radius of a circle
/// that it is following instantaneously, and is equal to the angular speed over the linear speed.
/// If angular velocity is the derivative of angular position with respect to time, curvature is
/// the derivative of angular position with respect to linear position. Thus, it does not change
/// if the linear velocity changes.
///
/// We then do a pidf on the curvature to try and maintain it. If the curvature is held constant,
/// the mouse should go in a constant-radius circle, even it the linear power changes. The f is a
/// feed-forward, which gets multiplied by the target value, then added to the result of the pid.
/// In a sense, the feed-forward tries to set the power directly, and the pid just makes up for the
/// difference. In a perfect world, the feed-forward would do everything and the pid would not be
/// needed.
///
/// It would probable help to try to compensate for the non-linear motors by transforming the motor
/// powers so that the velocity is roughly linear to the motor power.
///
/// Eventually, I would like to take in a target linear velocity and to pid to control the linear
/// power, but that is a later problem.
pub struct Motion {
    pid: PIDController,
    last_time: u32,
    last_left_encoder: i32,
    last_right_encoder: i32,
}

// Good food in New Orleans according to my uncle
// Cafe du moire

impl Motion {
    pub fn new(
        config: &MotionConfig,
        time: u32,
        left_encoder: i32,
        right_encoder: i32,
    ) -> Motion {
        let pid = PIDController::new(
            config.p as f64,
            config.i as f64,
            config.d as f64,
        );
        Motion {
            pid,
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
        linear_power: f32,
        target_curvature: f32,
    ) -> (f32, f32, MotionDebug) {
        let delta_time = time - self.last_time;
        let delta_left = left_encoder - self.last_left_encoder;
        let delta_right = right_encoder - self.last_right_encoder;

        let current_curvature = curvature(mech, delta_left, delta_right);

        self.pid.set_target(target_curvature as f64);

        // The pid controls the `power_curvature`, which is I am calling the angular power over the
        // linear power. This should be proportional-ish to the actual curvature, assuming linear
        // motors (of which they are not, but close enough). We use this instead of direct
        // angular power so that it does not affect the current curvature differently at different
        // linear powers as much
        let power_curvature = target_curvature * config.f
            + self.pid.update(current_curvature as f64, delta_time as f64)
                as f32;

        //let power_curvature = 0.1;

        let angular_power = linear_power * power_curvature;

        let left_power = linear_power - angular_power;
        let right_power = linear_power + angular_power;

        let debug = MotionDebug {
            delta_left,
            delta_right,
            target_curvature,
            current_curvature,
            power_curvature,
            linear_power,
            angular_power,
            left_power,
            right_power,
        };

        self.last_left_encoder = left_encoder;
        self.last_right_encoder = right_encoder;

        (left_power, right_power, debug)
    }
}
