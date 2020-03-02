use heapless::Vec;
use typenum::{Unsigned, U4};

use serde::{Deserialize, Serialize};

use crate::fast::path::PathMotion;
use crate::fast::turn::{TurnHandlerConfig, TurnMotion};
use crate::fast::Orientation;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum Motion {
    Path(PathMotion),
    Turn(TurnMotion),
}

impl Motion {
    pub fn done(
        &self,
        turn_config: &TurnHandlerConfig,
        orientation: Orientation,
    ) -> bool {
        match self {
            Motion::Path(path_motion) => path_motion.done(orientation),
            Motion::Turn(turn_motion) => turn_motion.done(turn_config, orientation),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MotionQueueDebug {
    queue: MotionQueueBuffer,
}

pub type MotionQueueSize = U4;
pub type MotionQueueBuffer = Vec<Motion, MotionQueueSize>;

pub struct MotionQueue {
    queue: MotionQueueBuffer,
}

// heapless::Vec is dump and needs to be a stack
impl MotionQueue {
    pub fn new() -> MotionQueue {
        MotionQueue { queue: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.queue.clear();
    }

    pub fn add_motions(&mut self, motions: &[Motion]) -> Result<usize, usize> {
        for (i, motion) in motions.iter().enumerate() {
            if self.queue.push(*motion).is_err() {
                return Err(i);
            }
        }

        Ok(MotionQueueSize::to_usize() - self.queue.len())
    }

    pub fn pop_completed(
        &mut self,
        turn_config: &TurnHandlerConfig,
        orientation: Orientation,
    ) -> usize {
        let mut i = 0;
        // Go through the buffer and pop off any moves that have been completed
        while let Some(motion) = self.queue.pop() {
            if motion.done(turn_config, orientation) {
                i += 1;
            } else {
                self.queue.push(motion).ok();
                break;
            }
        }

        i
    }

    pub fn motions_remaining(&self) -> usize {
        self.queue.len()
    }

    pub fn next_motion(&self) -> Option<Motion> {
        self.queue.last().cloned()
    }

    pub fn debug(&self) -> MotionQueueDebug {
        MotionQueueDebug {
            queue: self.queue.clone(),
        }
    }
}
