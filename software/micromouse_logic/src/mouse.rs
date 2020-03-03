use core::f32;

use serde::{Deserialize, Serialize};

use crate::config::MechanicalConfig;

use crate::fast::localize::{Localize, LocalizeConfig, LocalizeDebug};
use crate::fast::motion_queue::{Motion, MotionQueue, MotionQueueDebug};
use crate::fast::{Direction, Orientation};

use crate::fast::motion_control::{
    MotionControl, MotionControlConfig, MotionControlDebug,
};
use crate::slow::map::{Map, MapConfig};
use crate::slow::maze::MazeConfig;
use crate::slow::motion_plan::{motion_plan, MotionPlanConfig};
use crate::slow::navigate::TwelvePartitionNavigate;
use crate::slow::{MazeDirection, MazeOrientation, SlowDebug};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HardwareDebug {
    pub left_encoder: i32,
    pub right_encoder: i32,
    pub left_distance: u8,
    pub front_distance: u8,
    pub right_distance: u8,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MouseDebug {
    pub hardware: HardwareDebug,
    pub orientation: Orientation,
    pub maze_orientation: MazeOrientation,
    pub localize: LocalizeDebug,
    pub motion_control: MotionControlDebug,
    pub motion_queue: MotionQueueDebug,
    pub slow: Option<SlowDebug>,
    pub battery: u16,
    pub time: u32,
    pub delta_time: u32,
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MouseConfig {
    pub mechanical: MechanicalConfig,
    pub localize: LocalizeConfig,
    pub map: MapConfig,
    pub motion_plan: MotionPlanConfig,
    pub maze: MazeConfig,
    pub motion_control: MotionControlConfig,
}

pub struct Mouse {
    last_time: u32,
    map: Map,
    navigate: TwelvePartitionNavigate,
    target_direction: Direction,
    localize: Localize,
    motion_queue: MotionQueue,
    motion_control: MotionControl,
    moves_completed: usize,
}

impl Mouse {
    pub fn new(
        config: &MouseConfig,
        orientation: Orientation,
        time: u32,
        left_encoder: i32,
        right_encoder: i32,
    ) -> Mouse {
        Mouse {
            last_time: time,
            map: Map::new(),
            navigate: TwelvePartitionNavigate::new(),
            localize: Localize::new(orientation, left_encoder, right_encoder),
            motion_control: MotionControl::new(
                &config.motion_control,
                time,
                left_encoder,
                right_encoder,
            ),
            target_direction: orientation.direction,
            motion_queue: MotionQueue::new(),
            moves_completed: 0,
        }
    }

    pub fn update(
        &mut self,
        config: &MouseConfig,
        time: u32,
        battery: u16,
        left_encoder: i32,
        right_encoder: i32,
        left_distance: u8,
        front_distance: u8,
        right_distance: u8,
    ) -> (i32, i32, MouseDebug) {
        let delta_time = time - self.last_time;

        let (orientation, localize_debug) = self.localize.update(
            &config.mechanical,
            &config.maze,
            &config.localize,
            left_encoder,
            right_encoder,
            left_distance,
            front_distance,
            right_distance,
            self.motion_queue.next_motion(),
            self.moves_completed,
        );

        let next_motion_going_forward = match self.motion_queue.next_motion() {
            Some(Motion::Path(path_motion)) => {
                match orientation.to_maze_orientation(&config.maze).direction {
                    MazeDirection::North => path_motion.end().y > orientation.position.y,
                    MazeDirection::South => path_motion.end().y < orientation.position.y,
                    MazeDirection::East => path_motion.end().x > orientation.position.x,
                    MazeDirection::West => path_motion.end().x < orientation.position.x,
                }
            }

            _ => false,
        };

        self.moves_completed = if front_distance < 40 && next_motion_going_forward {
            let len = self.motion_queue.motions_remaining();
            self.motion_queue.clear();
            len
        } else {
            self.motion_queue
                .pop_completed(&config.motion_control.turn, orientation)
        };

        let slow_debug = if self.motion_queue.motions_remaining() == 0 {
            let (move_options, map_debug) = self.map.update(
                &config.mechanical,
                &config.maze,
                &config.map,
                left_distance,
                front_distance,
                right_distance,
            );

            let (next_direction, navigate_debug) = self
                .navigate
                .navigate(orientation.to_maze_orientation(&config.maze), move_options);

            let path = motion_plan(
                &config.motion_plan,
                &config.maze,
                orientation,
                &[next_direction],
            );

            self.motion_queue.add_motions(&path).ok();

            Some(SlowDebug {
                map: map_debug,
                move_options,
                navigate: navigate_debug,
                next_direction,
            })
        } else {
            None
        };

        let (left_power, right_power, target_direction, motion_debug) =
            self.motion_control.update(
                &config.motion_control,
                &config.mechanical,
                time,
                left_encoder,
                right_encoder,
                self.motion_queue.next_motion(),
                orientation,
            );

        let hardware_debug = HardwareDebug {
            left_encoder,
            right_encoder,
            left_distance,
            front_distance,
            right_distance,
        };

        let debug = MouseDebug {
            hardware: hardware_debug,
            orientation,
            maze_orientation: orientation.to_maze_orientation(&config.maze),
            localize: localize_debug,
            motion_control: motion_debug,
            motion_queue: self.motion_queue.debug(),
            slow: slow_debug,
            battery,
            time,
            delta_time,
        };

        self.last_time = time;
        self.target_direction = target_direction;

        (left_power, right_power, debug)
    }
}

pub struct TestMouse {}

impl TestMouse {
    pub fn new() -> TestMouse {
        TestMouse {}
    }

    pub fn update(
        &mut self,
        _config: &MouseConfig,
        time: u32,
        _left_encoder: i32,
        _right_encoder: i32,
    ) -> (f32, f32) {
        if time % 10000 <= 5000 {
            (0.0, 0.0)
        } else {
            (1.0, 1.0)
        }
    }
}
