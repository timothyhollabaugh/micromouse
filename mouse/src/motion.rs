use libm::F32Ext;

fn max(f1: f32, f2: f32) -> f32 {
    if f1 > f2 {
        f1
    } else {
        f2
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MotionConfig {
    /// The max power change for each wheel before the linear speed is reduced.
    pub max_wheel_delta_power: f32,
}

#[derive(Debug, Copy, Clone)]
pub struct MotionDebug {
    pub target_left_power: f32,
    pub target_right_power: f32,
    pub normalized_left_power: f32,
    pub normalized_right_power: f32,
    pub limited_left_power: f32,
    pub limited_right_power: f32,
    pub left_delta_power: f32,
    pub right_delta_power: f32,
}

/// Takes the angular and linear power and combines them to form a left and right power for the motors
/// Also limits the max change in power for each wheel
pub struct Motion {
    time: u32,
    last_left_power: f32,
    last_right_power: f32,
}

// Good food in New Orleans
// Cafe du moire

impl Motion {
    pub fn new(config: &MotionConfig, time: u32) -> Motion {
        Motion {
            time,
            last_left_power: 0.0,
            last_right_power: 0.0,
        }
    }

    pub fn update(
        &mut self,
        config: &MotionConfig,
        time: u32,
        linear_power: f32,
        angular_power: f32,
    ) -> (f32, f32, MotionDebug) {
        let target_left_power = linear_power - angular_power;
        let target_right_power = linear_power + angular_power;

        // Normalize the powers to -1.0 .. 1.0 by scaling back both left and right if one of them is
        // over 1.0
        let max_power = max(target_left_power.abs(), target_right_power.abs());

        let (normalized_left_power, normalized_right_power) = if max_power > 1.0 {
            (
                target_left_power / max_power,
                target_right_power / max_power,
            )
        } else {
            (target_left_power, target_right_power)
        };

        let left_delta_power = target_left_power - self.last_left_power;
        let right_delta_power = target_right_power - self.last_right_power;

        let max_delta_power = max(left_delta_power.abs(), right_delta_power.abs());

        let (limited_left_power, limited_right_power) =
            if max_delta_power > config.max_wheel_delta_power {
                let multiplication_factor =
                    (max_power + max_delta_power) / (max_power + config.max_wheel_delta_power);

                (
                    normalized_left_power * multiplication_factor,
                    normalized_right_power * multiplication_factor,
                )
            } else {
                (target_left_power, target_right_power)
            };

        self.last_left_power = limited_left_power;
        self.last_right_power = limited_right_power;

        let debug = MotionDebug {
            target_left_power,
            target_right_power,
            normalized_left_power,
            normalized_right_power,
            limited_left_power,
            limited_right_power,
            left_delta_power,
            right_delta_power,
        };

        (limited_left_power, limited_right_power, debug)
    }
}
