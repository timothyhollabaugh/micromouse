use serde::{Deserialize, Serialize};

use pid_control::{Controller, PIDController};

use super::Direction;
use crate::config::MechanicalConfig;
use crate::fast::{Orientation, DIRECTION_PI_2};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum TurnDirection {
    Clockwise,
    Counterclockwise,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct TurnMotion {
    target: Direction,
    direction: TurnDirection,
}

impl TurnMotion {
    pub fn new(current: Direction, target: Direction) -> TurnMotion {
        let direction = if target - current < DIRECTION_PI_2 {
            TurnDirection::Counterclockwise
        } else {
            TurnDirection::Clockwise
        };

        TurnMotion { target, direction }
    }

    pub fn done(&self, orientation: Orientation) -> bool {
        let centered_direction = orientation.direction.centered_at(self.target);
        match self.direction {
            TurnDirection::Counterclockwise => {
                centered_direction >= f32::from(self.target)
            }
            TurnDirection::Clockwise => centered_direction <= f32::from(self.target),
        }
    }
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TurnHandlerConfig {
    pub p: f32,
    pub i: f32,
    pub d: f32,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TurnHandlerDebug {}

pub struct TurnHandler {
    pid: PIDController,
    time: u32,
}

impl TurnHandler {
    pub fn new(config: &TurnHandlerConfig, time: u32) -> TurnHandler {
        let pid = PIDController::new(config.p as f64, config.i as f64, config.d as f64);
        TurnHandler { pid, time }
    }

    pub fn update(
        &mut self,
        config: &TurnHandlerConfig,
        mech: &MechanicalConfig,
        time: u32,
        orientation: Orientation,
        motion: TurnMotion,
    ) -> (f32, f32, Direction, TurnHandlerDebug) {
        let delta_time = time - self.time;

        self.pid.p_gain = config.p as f64;
        self.pid.i_gain = config.i as f64;
        self.pid.d_gain = config.d as f64;

        let centered_direction = orientation.direction.centered_at(motion.target);

        self.pid.set_target(f32::from(motion.target) as f64);

        let turn_velocity = self
            .pid
            .update(f32::from(centered_direction) as f64, delta_time as f64)
            as f32;

        let left_target = -mech.rads_to_mm(turn_velocity);
        let right_target = mech.rads_to_mm(turn_velocity);

        self.time = time;

        let target_direction = orientation.direction + turn_velocity * delta_time as f32;

        (
            left_target,
            right_target,
            target_direction,
            TurnHandlerDebug {},
        )
    }
}