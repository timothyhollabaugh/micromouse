#[allow(unused_imports)]
use libm::F32Ext;

use serde::Deserialize;
use serde::Serialize;

fn max(f1: f32, f2: f32) -> f32 {
    if f1 > f2 {
        f1
    } else {
        f2
    }
}

/// Find the number that is farthest from 0
fn signed_max(f1: f32, f2: f32) -> f32 {
    if f1.abs() > f2.abs() {
        f1
    } else {
        f2
    }
}

/// Find the number that is closest to 0
fn nearest_zero(f1: f32, f2: f32) -> f32 {
    if f1.abs() < f2.abs() {
        f1
    } else {
        f2
    }
}

/// Find the number that is closest to 0, but keep the sign of the first number
///
/// ```rust
/// use mouse::motion::signed_nearest_zero;
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
fn clip_linear_to_wheel_max(wheel_max: f32, power: f32, angular_ratio: f32) -> f32 {
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

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MotionConfig {
    /// The max power change for each wheel before the linear speed is reduced.
    pub max_delta_power: f32,

    /// The max power to send to the wheel before linear speed is reduced
    pub max_wheel_power: f32,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MotionDebug {
    pub power: f32,
    pub angular_ratio: f32,
    pub power_clipped: f32,
    pub delta_power: f32,
    pub delta_power_clipped: f32,
    pub final_power: f32,
    pub delta_angular_ratio: f32,
    pub delta_angular_ratio_clipped: f32,
    pub final_angular_ratio: f32,
    pub left_power: f32,
    pub right_power: f32,
}

/// Takes the angular and linear power and combines them to form a left and right power for the motors
/// Also limits the max change in power for each wheel
///
/// The linear power is a -1.0 to 1.0 value, with 0 being stopped, 1.0 being full forward, and -1.0
/// being full reverse. The angular ratio is a ratio of units angular power per unit linear power.
/// If the wheel speed is directly proportional to the power, the angular ratio will be inversely
/// proportional to the radius that the mouse is turning at. This keeps the radius of turn constant
/// as the linear power changes, and makes it easier to limit the power change for each wheel.
///
/// If we fully characterized the motors to find what speed each power level and battery
/// voltage gives, we could try to scale the linear power and angular power to be real units
/// and try to correct for non-linearness of the motors. This might in turn make it possible to
/// calculate theoretical values for all the pid terms that come before this module. Without
/// characterizing the motors, all the adjustments would end up in the pid terms determined
/// experimentally.
///
pub struct Motion {
    power: f32,
    angular_ratio: f32,
}

// Good food in New Orleans according to my uncle
// Cafe du moire

impl Motion {
    pub fn new(_config: &MotionConfig, _time: u32) -> Motion {
        Motion {
            power: 0.0,
            angular_ratio: 0.0,
        }
    }

    /// Updates
    pub fn update(
        &mut self,
        config: &MotionConfig,
        _time: u32,
        power: f32,
        angular_ratio: f32,
    ) -> (f32, f32, MotionDebug) {
        // Limit the linear power so that the power for each wheel is in the range -1.0 to 1.0
        let power_clipped = clip_linear_to_wheel_max(config.max_wheel_power, power, angular_ratio);

        // Limit the change in power for each wheel due to linear power change
        let delta_power = power_clipped - self.power;
        let delta_power_clipped =
            clip_linear_to_wheel_max(config.max_delta_power, delta_power, angular_ratio);
        self.power += delta_power_clipped;

        // Limit the change in power for each wheel due to angular ratio change
        let delta_angular_ratio = angular_ratio - self.angular_ratio;
        let delta_angular_ratio_clipped = clip_delta_angular_ratio_to_delta_angular_wheel_max(
            config.max_delta_power,
            self.power,
            delta_angular_ratio,
        );
        self.angular_ratio += delta_angular_ratio_clipped;

        let left_power = self.power * (1.0 - self.angular_ratio);
        let right_power = self.power * (1.0 + self.angular_ratio);

        let debug = MotionDebug {
            power,
            angular_ratio,
            power_clipped,
            delta_power,
            delta_power_clipped,
            final_power: self.power,
            delta_angular_ratio,
            delta_angular_ratio_clipped,
            final_angular_ratio: self.angular_ratio,
            left_power,
            right_power,
        };

        (left_power, right_power, debug)
    }
}
