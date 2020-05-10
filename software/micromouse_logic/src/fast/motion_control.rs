use serde::{Deserialize, Serialize};

use crate::config::MechanicalConfig;
use crate::fast::motion_queue::Motion;
use crate::fast::motor_control::{MotorControl, MotorControlConfig, MotorControlDebug};
use crate::fast::path::{PathHandler, PathHandlerConfig, PathHandlerDebug, PathMotion};
use crate::fast::turn::{TurnHandler, TurnHandlerConfig, TurnHandlerDebug};
use crate::fast::{Direction, Orientation};

pub enum MotionHandler {
    Turn(TurnHandler),
    Path(PathHandler),
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum MotionHandlerDebug {
    Turn(TurnHandlerDebug),
    Path(PathHandlerDebug),
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MotionControlDebug {
    pub handler: Option<MotionHandlerDebug>,
    pub motor_control: MotorControlDebug,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MotionControlConfig {
    pub turn: TurnHandlerConfig,
    pub path: PathHandlerConfig,
    pub motor_control: MotorControlConfig,
    pub stop_distance: f32,
}

pub struct MotionControl {
    handler: Option<MotionHandler>,
    motor_control: MotorControl,
    last_orientation: Orientation,
}

impl MotionControl {
    pub fn new(
        config: &MotionControlConfig,
        time: u32,
        left_encoder: i32,
        right_encoder: i32,
        orientation: Orientation,
    ) -> MotionControl {
        MotionControl {
            handler: None,
            motor_control: MotorControl::new(
                &config.motor_control,
                time,
                left_encoder,
                right_encoder,
            ),
            last_orientation: orientation,
        }
    }

    pub fn update(
        &mut self,
        config: &MotionControlConfig,
        mech: &MechanicalConfig,
        time: u32,
        left_encoder: i32,
        right_encoder: i32,
        motion: Option<Motion>,
        orientation: Orientation,
    ) -> (i32, i32, MotionControlDebug) {
        let handler = self.handler.take();

        let motion = if let Some(motion) = motion {
            self.last_orientation = orientation;
            motion
        } else {
            Motion::Path(PathMotion::line(
                self.last_orientation.position,
                self.last_orientation.position
                    + config.stop_distance
                        * self.last_orientation.direction.into_unit_vector(),
            ))
        };

        let (left_target, right_target, handler_debug) = match motion {
            Motion::Path(motion) => {
                let mut handler = if let Some(MotionHandler::Path(handler)) = handler {
                    handler
                } else {
                    PathHandler::new(&config.path, time)
                };

                let (left, right, debug) =
                    handler.update(&config.path, mech, time, orientation, motion);

                self.handler = Some(MotionHandler::Path(handler));

                (left, right, Some(MotionHandlerDebug::Path(debug)))
            }
            Motion::Turn(motion) => {
                let mut handler = if let Some(MotionHandler::Turn(handler)) = handler {
                    handler
                } else {
                    TurnHandler::new(&config.turn, time)
                };

                let (left, right, debug) =
                    handler.update(&config.turn, mech, time, orientation, motion);

                self.handler = Some(MotionHandler::Turn(handler));

                (left, right, Some(MotionHandlerDebug::Turn(debug)))
            }
        };

        let (left_power, right_power, motor_debug) = self.motor_control.update(
            &config.motor_control,
            mech,
            time,
            left_encoder,
            right_encoder,
            left_target,
            right_target,
        );

        let debug = MotionControlDebug {
            handler: handler_debug,
            motor_control: motor_debug,
        };

        (left_power, right_power, debug)
    }
}
